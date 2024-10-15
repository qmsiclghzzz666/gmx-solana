use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount};
use gmsol_model::{Bank, LiquidityMarketMutExt, PositionImpactMarketMutExt};
use typed_builder::TypedBuilder;

use crate::{
    states::{
        common::action::{Action, ActionExt},
        market::{
            revertible::{Revertible, RevertibleLiquidityMarket},
            utils::ValidateMarketBalances,
        },
        HasMarketMeta, Market, NonceBytes, Oracle, Shift, Store, ValidateOracleTime,
    },
    CoreError, CoreResult, ModelError,
};

/// Create Shift Params.
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CreateShiftParams {
    /// Execution fee in lamports.
    pub execution_lamports: u64,
    /// From market token amount.
    pub from_market_token_amount: u64,
    /// The minimum acceptable to market token amount to receive.
    pub min_to_market_token_amount: u64,
}

/// Operation for creating a shift.
#[derive(TypedBuilder)]
pub struct CreateShiftOperation<'a, 'info> {
    store: &'a AccountLoader<'info, Store>,
    owner: AccountInfo<'info>,
    shift: &'a AccountLoader<'info, Shift>,
    from_market: &'a AccountLoader<'info, Market>,
    from_market_token_account: &'a Account<'info, TokenAccount>,
    to_market: &'a AccountLoader<'info, Market>,
    to_market_token_account: &'a Account<'info, TokenAccount>,
    nonce: &'a NonceBytes,
    bump: u8,
    params: &'a CreateShiftParams,
}

impl<'a, 'info> CreateShiftOperation<'a, 'info> {
    pub(crate) fn execute(self) -> Result<()> {
        self.validate_markets()?;
        self.validate_params()?;

        let id = self.from_market.load_mut()?.state_mut().next_shift_id()?;

        let mut shift = self.shift.load_init()?;

        // Initialize the header.
        shift.header.init(
            id,
            self.store.key(),
            self.from_market.key(),
            self.owner.key(),
            *self.nonce,
            self.bump,
            self.params.execution_lamports,
        )?;

        // Initialize tokens.
        shift
            .tokens
            .from_market_token
            .init(self.from_market_token_account);
        shift
            .tokens
            .to_market_token
            .init(self.to_market_token_account);
        {
            let market = self.from_market.load()?;
            shift.tokens.long_token = market.meta().long_token_mint;
            shift.tokens.short_token = market.meta().short_token_mint;
        }

        // Initialize params.
        shift.params.from_market_token_amount = self.params.from_market_token_amount;
        shift.params.min_to_market_token_amount = self.params.min_to_market_token_amount;

        Ok(())
    }

    fn validate_markets(&self) -> Result<()> {
        require!(
            self.from_market.key() != self.to_market.key(),
            CoreError::InvalidShiftMarkets,
        );

        let from_market = self.from_market.load()?;
        let to_market = self.to_market.load()?;

        let store = &self.store.key();
        from_market.validate(store)?;
        to_market.validate(store)?;

        from_market.validate_shiftable(&to_market)?;

        require_eq!(
            from_market.meta().market_token_mint,
            self.from_market_token_account.mint,
            CoreError::MarketTokenMintMismatched,
        );

        require_eq!(
            to_market.meta().market_token_mint,
            self.to_market_token_account.mint,
            CoreError::MarketTokenMintMismatched,
        );
        Ok(())
    }

    fn validate_params(&self) -> Result<()> {
        let params = &self.params;

        require!(params.from_market_token_amount != 0, CoreError::EmptyShift);
        require_gte!(
            self.from_market_token_account.amount,
            params.from_market_token_amount,
            CoreError::NotEnoughTokenAmount
        );

        ActionExt::validate_balance(self.shift, params.execution_lamports)?;
        Ok(())
    }
}

/// Operation for executing a shift.
#[derive(TypedBuilder)]
pub struct ExecuteShiftOperation<'a, 'info> {
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

impl<'a, 'info> ExecuteShiftOperation<'a, 'info> {
    pub(crate) fn execute(mut self) -> Result<bool> {
        let throw_on_execution_error = self.throw_on_execution_error;

        match self.validate_oracle() {
            Ok(()) => {}
            Err(CoreError::OracleTimestampsAreLargerThanRequired) if !throw_on_execution_error => {
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
        match self.perfrom_shift() {
            Ok(()) => Ok(true),
            Err(err) if !throw_on_execution_error => {
                msg!("Execute shift error: {}", err);
                Ok(false)
            }
            Err(err) => Err(err),
        }
    }

    fn validate_oracle(&self) -> CoreResult<()> {
        self.oracle.validate_time(self)
    }

    fn validate_markets_and_shift(&self) -> Result<()> {
        require!(
            self.from_market.key() != self.to_market.key(),
            CoreError::Internal
        );

        let from_market = self.from_market.load()?;
        let to_market = self.to_market.load()?;

        from_market.validate(&self.store.key())?;
        to_market.validate(&self.store.key())?;

        from_market.validate_shiftable(&to_market)?;

        self.shift
            .load()?
            .validate_for_execution(&self.to_market_token_mint.to_account_info(), &to_market)?;

        Ok(())
    }

    #[inline(never)]
    fn perfrom_shift(&mut self) -> Result<()> {
        self.validate_markets_and_shift()?;

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
                    .map_err(|_| CoreError::TokenAmountOverflow)?,
                (*report.short_token_output())
                    .try_into()
                    .map_err(|_| CoreError::TokenAmountOverflow)?,
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
                .map_err(|_| error!(CoreError::TokenAmountOverflow))?;

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

impl<'a, 'info> ValidateOracleTime for ExecuteShiftOperation<'a, 'info> {
    fn oracle_updated_after(&self) -> CoreResult<Option<i64>> {
        Ok(Some(
            self.shift
                .load()
                .map_err(|_| CoreError::LoadAccountError)?
                .header()
                .updated_at,
        ))
    }

    fn oracle_updated_before(&self) -> CoreResult<Option<i64>> {
        let ts = self
            .store
            .load()
            .map_err(|_| CoreError::LoadAccountError)?
            .request_expiration_at(
                self.shift
                    .load()
                    .map_err(|_| CoreError::LoadAccountError)?
                    .header()
                    .updated_at,
            )?;
        Ok(Some(ts))
    }

    fn oracle_updated_after_slot(&self) -> CoreResult<Option<u64>> {
        Ok(Some(
            self.shift
                .load()
                .map_err(|_| CoreError::LoadAccountError)?
                .header()
                .updated_at_slot,
        ))
    }
}
