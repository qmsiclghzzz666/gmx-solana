use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use gmx_core::{LiquidityMarketExt, PositionImpactMarketExt};

use crate::{
    states::{
        ops::ValidateMarketBalances,
        revertible::{
            swap_market::{SwapDirection, SwapMarkets},
            Revertible, RevertibleLiquidityMarket,
        },
        Deposit, HasMarketMeta, Market, MarketMeta, Oracle, Seed, Store, ValidateOracleTime,
    },
    utils::internal,
    DataStoreError, GmxCoreError,
};

use super::utils::swap::unchecked_swap_with_params;

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
) -> Result<bool> {
    ctx.accounts.validate_oracle()?;
    match ctx.accounts.execute2(ctx.remaining_accounts) {
        Ok(()) => Ok(true),
        Err(err) => {
            // TODO: catch and throw missing oracle price error.
            msg!("Execute deposit error: {}", err);
            Ok(false)
        }
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
    fn oracle_updated_after(&self) -> Result<Option<i64>> {
        Ok(Some(self.deposit.fixed.updated_at))
    }

    fn oracle_updated_before(&self) -> Result<Option<i64>> {
        let ts = self
            .store
            .load()?
            .request_expiration_at(self.deposit.fixed.updated_at)?;
        Ok(Some(ts))
    }

    fn oracle_updated_after_slot(&self) -> Result<Option<u64>> {
        Ok(Some(self.deposit.fixed.updated_at_slot))
    }
}

impl<'info> ExecuteDeposit<'info> {
    fn validate_oracle(&self) -> Result<()> {
        self.oracle.validate_time(self)
    }

    fn validate_market(&self) -> Result<()> {
        self.market.load()?.validate(&self.store.key())
    }

    fn execute2(&mut self, remaining_accounts: &'info [AccountInfo<'info>]) -> Result<()> {
        self.validate_market()?;

        // Prepare the execution context.
        let current_market = self.market.key();
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
            .unpack_markets_for_swap(&current_market, remaining_accounts)?;
        let mut swap_markets = SwapMarkets::new(&loaders, Some(&current_market))?;

        // Distribute position impact.
        {
            let report = market
                .distribute_position_impact()
                .map_err(GmxCoreError::from)?
                .execute()
                .map_err(GmxCoreError::from)?;
            msg!("Deposit pre-execute: {:?}", report);
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
                .map_err(GmxCoreError::from)?;
            market.validate_market_balances(0, 0)?;
            msg!("Deposit executed: {:?}", report);
        }

        // Commit the changes.
        market.commit();
        swap_markets.commit();

        // Set amounts to zero to make sure it can be removed without transferring out any tokens.
        self.deposit.fixed.tokens.params.initial_long_token_amount = 0;
        self.deposit.fixed.tokens.params.initial_short_token_amount = 0;
        Ok(())
    }

    fn validate(&self) -> Result<()> {
        self.oracle.validate_time(self)?;
        self.market.load()?.validate(&self.store.key())?;
        Ok(())
    }

    fn pre_execute(&mut self) -> Result<()> {
        let report = self
            .market
            .load_mut()?
            .as_market(&mut self.market_token_mint)
            .distribute_position_impact()
            .map_err(GmxCoreError::from)?
            .execute()
            .map_err(GmxCoreError::from)?;
        msg!("{:?}", report);
        Ok(())
    }

    fn execute(&mut self, remaining_accounts: &'info [AccountInfo<'info>]) -> Result<()> {
        let meta = *self.market.load()?.meta();
        let (long_amount, short_amount) = self.perform_swaps(&meta, remaining_accounts)?;
        self.perform_deposit(&meta, long_amount, short_amount)?;
        // Set amounts to zero to make sure it can be removed without transferring out any tokens.
        self.deposit.fixed.tokens.params.initial_long_token_amount = 0;
        self.deposit.fixed.tokens.params.initial_short_token_amount = 0;
        Ok(())
    }

    fn perform_swaps(
        &mut self,
        meta: &MarketMeta,
        remaining_accounts: &'info [AccountInfo<'info>],
    ) -> Result<(u64, u64)> {
        // // Exit must be called to update the external state.
        // self.market.exit(&crate::ID)?;
        // CHECK: `exit` has been called above, and `reload` will be called after.
        let (long_swap_out, short_swap_out) = unchecked_swap_with_params(
            &self.oracle,
            &self.deposit.dynamic.swap_params,
            remaining_accounts,
            (meta.long_token_mint, meta.short_token_mint),
            (
                self.deposit.fixed.tokens.initial_long_token,
                self.deposit.fixed.tokens.initial_short_token,
            ),
            (
                self.deposit.fixed.tokens.params.initial_long_token_amount,
                self.deposit.fixed.tokens.params.initial_short_token_amount,
            ),
        )?;
        // // Call `reload` to make sure the state is up-to-date.
        // self.market.reload()?;
        Ok((
            long_swap_out.transfer_to_market(&self.market, true)?,
            short_swap_out.transfer_to_market(&self.market, true)?,
        ))
    }

    fn perform_deposit(
        &mut self,
        meta: &MarketMeta,
        long_amount: u64,
        short_amount: u64,
    ) -> Result<()> {
        let index_token_price = self
            .oracle
            .primary
            .get(&meta.index_token_mint)
            .ok_or(error!(DataStoreError::InvalidArgument))?
            .max
            .to_unit_price();
        let long_token_price = self
            .oracle
            .primary
            .get(&meta.long_token_mint)
            .ok_or(error!(DataStoreError::InvalidArgument))?
            .max
            .to_unit_price();
        let short_token_price = self
            .oracle
            .primary
            .get(&meta.short_token_mint)
            .ok_or(error!(DataStoreError::InvalidArgument))?
            .max
            .to_unit_price();
        let report = {
            let mut market = self.market.load_mut()?;
            let mut market = market
                .as_market(&mut self.market_token_mint)
                .enable_transfer(self.token_program.to_account_info(), &self.store)
                .with_receiver(self.receiver.to_account_info());
            let report = market
                .deposit(
                    long_amount.into(),
                    short_amount.into(),
                    gmx_core::action::Prices {
                        index_token_price,
                        long_token_price,
                        short_token_price,
                    },
                )
                .map_err(GmxCoreError::from)?
                .execute()
                .map_err(|err| {
                    msg!(&err.to_string());
                    GmxCoreError::from(err)
                })?;
            market.validate_market_balances(0, 0)?;
            report
        };
        msg!("{:?}", report);
        Ok(())
    }
}
