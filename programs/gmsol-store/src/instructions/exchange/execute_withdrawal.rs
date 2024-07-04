use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use gmsol_model::{LiquidityMarketExt, PositionImpactMarketExt};

use crate::{
    constants,
    states::{
        ops::ValidateMarketBalances,
        revertible::{
            swap_market::{SwapDirection, SwapMarkets},
            Revertible, RevertibleLiquidityMarket,
        },
        HasMarketMeta, Market, Oracle, Seed, Store, ValidateOracleTime, Withdrawal,
    },
    utils::internal,
    StoreError, ModelError, StoreResult,
};

#[derive(Accounts)]
pub struct ExecuteWithdrawal<'info> {
    pub authority: Signer<'info>,
    pub store: AccountLoader<'info, Store>,
    #[account(has_one = store)]
    pub oracle: Account<'info, Oracle>,
    #[account(
        mut,
        constraint = withdrawal.fixed.store == store.key(),
        constraint = withdrawal.fixed.market == market.key(),
        constraint = withdrawal.fixed.tokens.market_token == market_token_mint.key(),
        constraint = withdrawal.fixed.receivers.final_long_token_receiver == final_long_token_receiver.key(),
        constraint = withdrawal.fixed.receivers.final_short_token_receiver == final_short_token_receiver.key(),
        seeds = [
            Withdrawal::SEED,
            store.key().as_ref(),
            withdrawal.fixed.user.as_ref(),
            &withdrawal.fixed.nonce,
        ],
        bump = withdrawal.fixed.bump,
    )]
    pub withdrawal: Account<'info, Withdrawal>,
    #[account(mut, has_one = store)]
    pub market: AccountLoader<'info, Market>,
    #[account(mut)]
    pub market_token_mint: Account<'info, Mint>,
    #[account(
        mut,
        token::mint = market_token_mint,
        seeds = [
            constants::MARKET_VAULT_SEED,
            store.key().as_ref(),
            market_token_withdrawal_vault.mint.as_ref(),
            &[],
        ],
        bump,
    )]
    pub market_token_withdrawal_vault: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        token::mint = final_long_token_receiver.mint,
        seeds = [
            constants::MARKET_VAULT_SEED,
            store.key().as_ref(),
            final_long_token_vault.mint.as_ref(),
            &[],
        ],
        bump,
    )]
    pub final_long_token_vault: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        token::mint = final_short_token_receiver.mint,
        seeds = [
            constants::MARKET_VAULT_SEED,
            store.key().as_ref(),
            final_short_token_vault.mint.as_ref(),
            &[],
        ],
        bump,
    )]
    pub final_short_token_vault: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub final_long_token_receiver: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub final_short_token_receiver: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
}

/// Execute a withdrawal.
pub fn execute_withdrawal<'info>(
    ctx: Context<'_, '_, 'info, 'info, ExecuteWithdrawal<'info>>,
    throw_on_execution_error: bool,
) -> Result<(u64, u64)> {
    match ctx.accounts.validate_oracle() {
        Ok(()) => {}
        Err(StoreError::OracleTimestampsAreLargerThanRequired) if !throw_on_execution_error => {
            msg!(
                "Withdrawal expired at {}",
                ctx.accounts
                    .oracle_updated_before()
                    .ok()
                    .flatten()
                    .expect("must have an expiration time"),
            );
            return Ok((0, 0));
        }
        Err(err) => {
            return Err(error!(err));
        }
    }
    match ctx.accounts.execute2(ctx.remaining_accounts) {
        Ok(res) => Ok(res),
        Err(err) if !throw_on_execution_error => {
            // TODO: catch and throw missing oracle price error.
            msg!("Execute withdrawal error: {}", err);
            Ok((0, 0))
        }
        Err(err) => Err(err),
    }
}

impl<'info> internal::Authentication<'info> for ExecuteWithdrawal<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &AccountLoader<'info, Store> {
        &self.store
    }
}

impl<'info> ValidateOracleTime for ExecuteWithdrawal<'info> {
    fn oracle_updated_after(&self) -> StoreResult<Option<i64>> {
        Ok(Some(self.withdrawal.fixed.updated_at))
    }

    fn oracle_updated_before(&self) -> StoreResult<Option<i64>> {
        let ts = self
            .store
            .load()
            .map_err(|_| StoreError::LoadAccountError)?
            .request_expiration_at(self.withdrawal.fixed.updated_at)?;
        Ok(Some(ts))
    }

    fn oracle_updated_after_slot(&self) -> StoreResult<Option<u64>> {
        Ok(Some(self.withdrawal.fixed.updated_at_slot))
    }
}

impl<'info> ExecuteWithdrawal<'info> {
    fn validate_oracle(&self) -> StoreResult<()> {
        self.oracle.validate_time(self)
    }

    fn validate_market(&self) -> Result<()> {
        self.market.load()?.validate(&self.store.key())
    }

    fn execute2(&mut self, remaining_accounts: &'info [AccountInfo<'info>]) -> Result<(u64, u64)> {
        self.validate_market()?;

        // Prepare the execution context.
        let current_market_token = self.market_token_mint.key();
        let mut market = RevertibleLiquidityMarket::new(
            &self.market,
            &mut self.market_token_mint,
            self.token_program.to_account_info(),
            &self.store,
        )?
        .enable_burn(self.market_token_withdrawal_vault.to_account_info());
        let loaders = self
            .withdrawal
            .dynamic
            .swap
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
            msg!("[Withdrawal] pre-execute: {:?}", report);
        }

        // Perform the withdrawal.
        let (long_amount, short_amount) = {
            let prices = self.oracle.market_prices(&market)?;
            let report = market
                .withdraw(
                    self.withdrawal.fixed.tokens.market_token_amount.into(),
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
            market.validate_market_balances(long_amount, short_amount)?;
            msg!("[Withdrawal] executed: {:?}", report);
            (long_amount, short_amount)
        };

        // Perform the swap.
        let (final_long_amount, final_short_amount) = {
            let meta = *market.market_meta();
            swap_markets.revertible_swap(
                SwapDirection::From(&mut market),
                &self.oracle,
                &self.withdrawal.dynamic.swap,
                (
                    self.withdrawal.fixed.tokens.final_long_token,
                    self.withdrawal.fixed.tokens.final_short_token,
                ),
                (Some(meta.long_token_mint), Some(meta.short_token_mint)),
                (long_amount, short_amount),
            )?
        };

        // Commit the changes.
        market.commit();
        swap_markets.commit();

        self.withdrawal.fixed.tokens.market_token_amount = 0;

        Ok((final_long_amount, final_short_amount))
    }
}
