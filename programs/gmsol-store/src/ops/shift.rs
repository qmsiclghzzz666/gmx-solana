use anchor_lang::prelude::*;
use anchor_spl::token::Mint;
use gmsol_model::{Bank, LiquidityMarketMutExt, PositionImpactMarketMutExt};
use typed_builder::TypedBuilder;

use crate::{
    states::{
        common::action::Action,
        revertible::{Revertible, RevertibleLiquidityMarket},
        HasMarketMeta, Market, Oracle, Shift, Store, ValidateMarketBalances, ValidateOracleTime,
    },
    CoreError, ModelError, StoreError, StoreResult,
};

/// Execute a shift.
#[derive(TypedBuilder)]
pub struct ExecuteShiftOp<'a, 'info> {
    store: &'a AccountLoader<'info, Store>,
    oracle: &'a Oracle,
    shift: &'a AccountLoader<'info, Shift>,
    from_market: &'a AccountLoader<'info, Market>,
    from_market_token_mint: &'a mut Account<'info, Mint>,
    from_market_token_vault: AccountInfo<'info>,
    to_market: &'a AccountLoader<'info, Market>,
    to_market_token_mint: &'a mut Account<'info, Mint>,
    to_market_token_account: AccountInfo<'info>,
    throw_on_execution_error: bool,
    token_program: AccountInfo<'info>,
}

impl<'a, 'info> ExecuteShiftOp<'a, 'info> {
    pub(crate) fn execute(mut self) -> Result<bool> {
        let throw_on_execution_error = self.throw_on_execution_error;

        match self.validate_oracle() {
            Ok(()) => {}
            Err(StoreError::OracleTimestampsAreLargerThanRequired) if !throw_on_execution_error => {
                msg!(
                    "shift expired at {}",
                    self.oracle_updated_before()
                        .ok()
                        .flatten()
                        .expect("must have an expiration time"),
                );
                return Ok(false);
            }
            Err(err) => {
                return Err(error!(err));
            }
        }
        match self.do_execute() {
            Ok(()) => Ok(true),
            Err(err) if !throw_on_execution_error => {
                msg!("Execute shift error: {}", err);
                Ok(false)
            }
            Err(err) => Err(err),
        }
    }

    fn validate_oracle(&self) -> StoreResult<()> {
        self.oracle.validate_time(self)
    }

    fn validate_markets(&self) -> Result<()> {
        require!(
            self.from_market.key() != self.to_market.key(),
            CoreError::Internal
        );
        let from_market = self.from_market.load()?;
        let to_market = self.to_market.load()?;
        from_market.validate(&self.store.key())?;
        to_market.validate(&self.store.key())?;

        // Currently we only support the shift between markets with
        // with the same long tokens and short tokens.
        //
        // It should be possible to allow shift between markets with the compatible tokens in the future,
        // for example, allowing shifting from BTC[WSOL-USDC] to SOL[USDC-WSOL].

        require_eq!(
            from_market.meta().long_token_mint,
            to_market.meta().long_token_mint,
            CoreError::TokenMintMismatched,
        );

        require_eq!(
            from_market.meta().short_token_mint,
            to_market.meta().short_token_mint,
            CoreError::TokenMintMismatched,
        );

        Ok(())
    }

    #[inline(never)]
    fn do_execute(&mut self) -> Result<()> {
        self.validate_markets()?;

        let mut from_market = RevertibleLiquidityMarket::new(
            self.from_market,
            self.from_market_token_mint,
            self.token_program.to_account_info(),
            self.store,
        )?
        .enable_burn(self.from_market_token_vault.clone())
        .boxed();

        let mut to_market = RevertibleLiquidityMarket::new(
            self.to_market,
            self.to_market_token_mint,
            self.token_program.to_account_info(),
            self.store,
        )?
        .enable_mint(self.to_market_token_account.clone())
        .boxed();

        // Distribute position impact for the from market.
        {
            let report = from_market
                .distribute_position_impact()
                .map_err(ModelError::from)?
                .execute()
                .map_err(ModelError::from)?;
            msg!("[Shift-Withdrawal] pre-execute: {:?}", report);
        }

        // Perform the shift-withdrawal.
        let (long_amount, short_amount) = {
            let prices = self
                .oracle
                .market_prices(&*from_market)
                .expect("all the required prices must have been provided");
            let report = from_market
                .withdraw(
                    self.shift.load()?.params.from_market_token_amount().into(),
                    prices,
                )
                .and_then(|w| w.execute())
                .map_err(ModelError::from)?;
            let (long_amount, short_amount) = (
                (*report.long_token_output())
                    .try_into()
                    .map_err(|_| StoreError::AmountOverflow)?,
                (*report.short_token_output())
                    .try_into()
                    .map_err(|_| StoreError::AmountOverflow)?,
            );
            // Validate current market.
            from_market.validate_market_balances(long_amount, short_amount)?;
            msg!("[Shift-Withdrawal] executed: {:?}", report);
            (long_amount, short_amount)
        };

        // Transfer tokens from the `from_market` to `to_market`.
        // The vaults are assumed to be shared.
        {
            let long_token = from_market.market_meta().long_token_mint;
            let short_token = to_market.market_meta().short_token_mint;

            from_market
                .record_transferred_out_by_token(&long_token, &long_amount)
                .map_err(ModelError::from)?;
            to_market
                .record_transferred_in_by_token(&long_token, &long_amount)
                .map_err(ModelError::from)?;

            from_market
                .record_transferred_out_by_token(&short_token, &short_amount)
                .map_err(ModelError::from)?;
            to_market
                .record_transferred_in_by_token(&short_token, &short_amount)
                .map_err(ModelError::from)?;
        }

        // TODO: validate first deposit.

        // Distribute position impact for the to market.
        {
            let report = to_market
                .distribute_position_impact()
                .map_err(ModelError::from)?
                .execute()
                .map_err(ModelError::from)?;
            msg!("[Shift-Deposit] pre-execute: {:?}", report);
        }

        // Perform the shift-deposit.
        {
            let prices = self
                .oracle
                .market_prices(&*to_market)
                .expect("all requried prices must have been provided");
            let report = to_market
                .deposit(long_amount.into(), short_amount.into(), prices)
                .and_then(|d| d.execute())
                .map_err(ModelError::from)?;
            to_market.validate_market_balances(0, 0)?;

            let minted: u64 = (*report.minted())
                .try_into()
                .map_err(|_| error!(StoreError::AmountOverflow))?;

            require_gte!(
                minted,
                self.shift.load()?.params.min_to_market_token_amount(),
                CoreError::InsufficientOutputAmount
            );

            msg!("[Shift-Deposit] executed: {:?}", report);
        }

        // Commit the changes.
        from_market.commit();
        to_market.commit();

        Ok(())
    }
}

impl<'a, 'info> ValidateOracleTime for ExecuteShiftOp<'a, 'info> {
    fn oracle_updated_after(&self) -> StoreResult<Option<i64>> {
        Ok(Some(
            self.shift
                .load()
                .map_err(|_| StoreError::LoadAccountError)?
                .header()
                .updated_at,
        ))
    }

    fn oracle_updated_before(&self) -> StoreResult<Option<i64>> {
        let ts = self
            .store
            .load()
            .map_err(|_| StoreError::LoadAccountError)?
            .request_expiration_at(
                self.shift
                    .load()
                    .map_err(|_| StoreError::LoadAccountError)?
                    .header()
                    .updated_at,
            )?;
        Ok(Some(ts))
    }

    fn oracle_updated_after_slot(&self) -> StoreResult<Option<u64>> {
        Ok(Some(
            self.shift
                .load()
                .map_err(|_| StoreError::LoadAccountError)?
                .header()
                .updated_at_slot,
        ))
    }
}
