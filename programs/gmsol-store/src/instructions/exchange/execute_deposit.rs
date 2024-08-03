use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use gmsol_model::{LiquidityMarketExt, PositionImpactMarketMutExt};

use crate::{
    states::{
        ops::ValidateMarketBalances,
        revertible::{
            swap_market::{SwapDirection, SwapMarkets},
            Revertible, RevertibleLiquidityMarket,
        },
        Deposit, HasMarketMeta, Market, Oracle, Seed, Store, ValidateOracleTime,
    },
    utils::internal,
    ModelError, StoreError, StoreResult,
};

#[derive(Accounts)]
pub struct ExecuteDeposit<'info> {
    pub authority: Signer<'info>,
    pub store: AccountLoader<'info, Store>,
    #[account(has_one = store)]
    pub oracle: Account<'info, Oracle>,
    #[account(
        // The `mut` flag must be present, since we are mutating the deposit.
        // It may not throw any errors sometimes if we forget to mark the account as mutable,
        // so be careful.
        mut,
        constraint = deposit.fixed.store == store.key(),
        constraint = deposit.fixed.receivers.receiver == receiver.key(),
        constraint = deposit.fixed.tokens.market_token == market_token_mint.key(),
        constraint = deposit.fixed.market == market.key(),
        seeds = [
            Deposit::SEED,
            store.key().as_ref(),
            deposit.fixed.senders.user.key().as_ref(),
            &deposit.fixed.nonce,
        ],
        bump = deposit.fixed.bump,
    )]
    pub deposit: Account<'info, Deposit>,
    #[account(mut, has_one = store)]
    pub market: AccountLoader<'info, Market>,
    #[account(mut)]
    pub market_token_mint: Account<'info, Mint>,
    #[account(mut)]
    pub receiver: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

/// Execute a deposit.
pub fn execute_deposit<'info>(
    ctx: Context<'_, '_, 'info, 'info, ExecuteDeposit<'info>>,
    throw_on_execution_error: bool,
) -> Result<bool> {
    match ctx.accounts.validate_oracle() {
        Ok(()) => {}
        Err(StoreError::OracleTimestampsAreLargerThanRequired) if !throw_on_execution_error => {
            msg!(
                "Deposit expired at {}",
                ctx.accounts
                    .oracle_updated_before()
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
    match ctx.accounts.execute(ctx.remaining_accounts) {
        Ok(()) => Ok(true),
        Err(err) if !throw_on_execution_error => {
            // TODO: catch and throw missing oracle price error.
            msg!("Execute deposit error: {}", err);
            Ok(false)
        }
        Err(err) => Err(err),
    }
}

impl<'info> internal::Authentication<'info> for ExecuteDeposit<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &AccountLoader<'info, Store> {
        &self.store
    }
}

impl<'info> ValidateOracleTime for ExecuteDeposit<'info> {
    fn oracle_updated_after(&self) -> StoreResult<Option<i64>> {
        Ok(Some(self.deposit.fixed.updated_at))
    }

    fn oracle_updated_before(&self) -> StoreResult<Option<i64>> {
        let ts = self
            .store
            .load()
            .map_err(|_| StoreError::LoadAccountError)?
            .request_expiration_at(self.deposit.fixed.updated_at)?;
        Ok(Some(ts))
    }

    fn oracle_updated_after_slot(&self) -> StoreResult<Option<u64>> {
        Ok(Some(self.deposit.fixed.updated_at_slot))
    }
}

impl<'info> ExecuteDeposit<'info> {
    fn validate_oracle(&self) -> StoreResult<()> {
        self.oracle.validate_time(self)
    }

    fn validate_market(&self) -> Result<()> {
        self.market.load()?.validate(&self.store.key())
    }

    fn execute(&mut self, remaining_accounts: &'info [AccountInfo<'info>]) -> Result<()> {
        self.validate_market()?;

        // Prepare the execution context.
        let current_market_token = self.market_token_mint.key();
        let mut market = RevertibleLiquidityMarket::new(
            &self.market,
            &mut self.market_token_mint,
            self.token_program.to_account_info(),
            &self.store,
        )?
        .enable_mint(self.receiver.to_account_info());
        let loaders = self
            .deposit
            .dynamic
            .swap_params
            .unpack_markets_for_swap(&current_market_token, remaining_accounts)?;
        let mut swap_markets =
            SwapMarkets::new(&self.store.key(), &loaders, Some(&current_market_token))?;

        // Distribute position impact.
        {
            let report = market
                .distribute_position_impact()
                .map_err(ModelError::from)?
                .execute()
                .map_err(ModelError::from)?;
            msg!("[Deposit] pre-execute: {:?}", report);
        }

        // Swap tokens into the target market.
        let (long_token_amount, short_token_amount) = {
            let meta = market.market_meta();
            let expected_token_outs = (meta.long_token_mint, meta.short_token_mint);
            swap_markets.revertible_swap(
                SwapDirection::Into(&mut market),
                &self.oracle,
                &self.deposit.dynamic.swap_params,
                expected_token_outs,
                (
                    self.deposit.fixed.tokens.initial_long_token,
                    self.deposit.fixed.tokens.initial_short_token,
                ),
                (
                    self.deposit.fixed.tokens.params.initial_long_token_amount,
                    self.deposit.fixed.tokens.params.initial_short_token_amount,
                ),
            )?
        };

        // Perform the deposit.
        {
            let prices = self.oracle.market_prices(&market)?;
            let report = market
                .deposit(long_token_amount.into(), short_token_amount.into(), prices)
                .and_then(|d| d.execute())
                .map_err(ModelError::from)?;
            market.validate_market_balances(0, 0)?;

            self.deposit.validate_min_market_tokens(
                (*report.minted())
                    .try_into()
                    .map_err(|_| error!(StoreError::AmountOverflow))?,
            )?;

            msg!("[Deposit] executed: {:?}", report);
        }

        // Commit the changes.
        market.commit();
        swap_markets.commit();

        // Set amounts to zero to make sure it can be removed without transferring out any tokens.
        self.deposit.fixed.tokens.params.initial_long_token_amount = 0;
        self.deposit.fixed.tokens.params.initial_short_token_amount = 0;
        Ok(())
    }
}
