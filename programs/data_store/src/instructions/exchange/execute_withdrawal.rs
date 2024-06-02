use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use gmx_core::MarketExt;

use crate::{
    constants,
    states::{Config, DataStore, Market, Oracle, Roles, Seed, ValidateOracleTime, Withdrawal},
    utils::internal::{self, TransferUtils},
    DataStoreError, GmxCoreError,
};

use super::utils::swap::unchecked_swap_with_params;

#[derive(Accounts)]
pub struct ExecuteWithdrawal<'info> {
    pub authority: Signer<'info>,
    pub store: Account<'info, DataStore>,
    pub only_order_keeper: Account<'info, Roles>,
    #[account(
        has_one = store,
        seeds = [Config::SEED, store.key().as_ref()],
        bump = config.bump,
    )]
    config: Account<'info, Config>,
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
    pub market: Account<'info, Market>,
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
    pub market_token_withdrawal_vault: Account<'info, TokenAccount>,
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
    pub final_long_token_vault: Account<'info, TokenAccount>,
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
    pub final_short_token_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub final_long_token_receiver: Account<'info, TokenAccount>,
    #[account(mut)]
    pub final_short_token_receiver: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

/// Execute a withdrawal.
pub fn execute_withdrawal<'info>(
    ctx: Context<'_, '_, 'info, 'info, ExecuteWithdrawal<'info>>,
) -> Result<()> {
    ctx.accounts.validate()?;
    ctx.accounts.pre_execute()?;
    ctx.accounts.execute(ctx.remaining_accounts)?;
    Ok(())
}

impl<'info> internal::Authentication<'info> for ExecuteWithdrawal<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &Account<'info, DataStore> {
        &self.store
    }

    fn roles(&self) -> &Account<'info, Roles> {
        &self.only_order_keeper
    }
}

impl<'info> ValidateOracleTime for ExecuteWithdrawal<'info> {
    fn oracle_updated_after(&self) -> Result<Option<i64>> {
        Ok(Some(self.withdrawal.fixed.updated_at))
    }

    fn oracle_updated_before(&self) -> Result<Option<i64>> {
        let ts = self
            .config
            .request_expiration_at(self.withdrawal.fixed.updated_at)?;
        Ok(Some(ts))
    }

    fn oracle_updated_after_slot(&self) -> Result<Option<u64>> {
        Ok(Some(self.withdrawal.fixed.updated_at_slot))
    }
}

impl<'info> ExecuteWithdrawal<'info> {
    fn validate(&self) -> Result<()> {
        self.oracle.validate_time(self)?;
        Ok(())
    }

    fn pre_execute(&mut self) -> Result<()> {
        let report = self
            .market
            .as_market(&mut self.market_token_mint)
            .distribute_position_impact()
            .map_err(GmxCoreError::from)?
            .execute()
            .map_err(GmxCoreError::from)?;
        msg!("{:?}", report);
        Ok(())
    }

    fn execute(&mut self, remaining_accounts: &'info [AccountInfo<'info>]) -> Result<()> {
        let (long_amount, short_amount) = self.perform_withdrawal()?;
        let (final_long_amount, final_short_amount) =
            self.perform_swaps(remaining_accounts, long_amount, short_amount)?;
        require_gte!(
            final_long_amount,
            self.withdrawal.fixed.tokens.params.min_long_token_amount,
            DataStoreError::OutputAmountTooSmall
        );
        require_gte!(
            final_short_amount,
            self.withdrawal.fixed.tokens.params.min_short_token_amount,
            DataStoreError::OutputAmountTooSmall
        );
        self.transfer_out(true, final_long_amount)?;
        self.transfer_out(false, final_short_amount)?;
        self.withdrawal.fixed.tokens.market_token_amount = 0;
        Ok(())
    }

    fn perform_withdrawal(&mut self) -> Result<(u64, u64)> {
        let meta = &self.market.meta;
        let index_token_price = self
            .oracle
            .primary
            .get(&meta.index_token_mint)
            .ok_or(DataStoreError::RequiredResourceNotFound)?
            .max
            .to_unit_price();
        let long_token_price = self
            .oracle
            .primary
            .get(&meta.long_token_mint)
            .ok_or(DataStoreError::RequiredResourceNotFound)?
            .max
            .to_unit_price();
        let short_token_price = self
            .oracle
            .primary
            .get(&meta.short_token_mint)
            .ok_or(DataStoreError::RequiredResourceNotFound)?
            .max
            .to_unit_price();
        let report = self
            .market
            .as_market(&mut self.market_token_mint)
            .enable_transfer(self.token_program.to_account_info(), &self.store)
            .with_vault(self.market_token_withdrawal_vault.to_account_info())
            .withdraw(
                self.withdrawal.fixed.tokens.market_token_amount.into(),
                gmx_core::action::Prices {
                    index_token_price,
                    long_token_price,
                    short_token_price,
                },
            )
            .map_err(GmxCoreError::from)?
            .execute()
            .map_err(GmxCoreError::from)?;
        msg!("{:?}", report);
        Ok((
            (*report.long_token_output())
                .try_into()
                .map_err(|_| DataStoreError::AmountOverflow)?,
            (*report.short_token_output())
                .try_into()
                .map_err(|_| DataStoreError::AmountOverflow)?,
        ))
    }

    fn perform_swaps(
        &mut self,
        remaining_accounts: &'info [AccountInfo<'info>],
        long_amount: u64,
        short_amount: u64,
    ) -> Result<(u64, u64)> {
        let meta = &self.market.meta;
        // Call exit and reload to make sure the data are written to the storage.
        // In case that there are markets also appear in the swap paths.
        self.market.exit(&crate::ID)?;
        // CHECK: `exit` and `reload` have been called on the modified market account before and after the swap.
        let res = unchecked_swap_with_params(
            &self.oracle,
            &self.withdrawal.dynamic.swap,
            remaining_accounts,
            (
                self.withdrawal.fixed.tokens.final_long_token,
                self.withdrawal.fixed.tokens.final_short_token,
            ),
            (Some(meta.long_token_mint), Some(meta.short_token_mint)),
            (long_amount, short_amount),
        )?;
        // Call `reload` to make sure the state is up-to-date.
        self.market.reload()?;
        Ok(res)
    }

    fn transfer_out(&self, is_long_token: bool, amount: u64) -> Result<()> {
        let (from, to) = if is_long_token {
            (
                &self.final_long_token_vault,
                &self.final_long_token_receiver,
            )
        } else {
            (
                &self.final_short_token_vault,
                &self.final_short_token_receiver,
            )
        };
        TransferUtils::new(self.token_program.to_account_info(), &self.store, None).transfer_out(
            from.to_account_info(),
            to.to_account_info(),
            amount,
        )
    }
}
