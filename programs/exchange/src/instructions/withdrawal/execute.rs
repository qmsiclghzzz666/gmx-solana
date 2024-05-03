use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use data_store::{
    cpi::accounts::RemoveWithdrawal,
    program::DataStore,
    states::{PriceProvider, Withdrawal},
    utils::{Authentication, WithOracle, WithOracleExt},
};

use crate::ExchangeError;

#[derive(Accounts)]
pub struct ExecuteWithdrawal<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    /// CHECK: only used to invoke CPI.
    pub store: UncheckedAccount<'info>,
    /// CHECK: only used to invoke CPI.
    pub only_order_keeper: UncheckedAccount<'info>,
    pub data_store_program: Program<'info, DataStore>,
    pub price_provider: Interface<'info, PriceProvider>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    #[account(mut)]
    pub oracle: Account<'info, data_store::states::Oracle>,
    /// CHECK: check by CPI.
    pub token_config_map: UncheckedAccount<'info>,
    /// Withdrawal to execute.
    ///
    /// ## Notes
    /// - `user` is checked on the removal CPI of the withdrawal.
    #[account(
        mut,
        constraint = withdrawal.fixed.tokens.market_token == market_token_mint.key() @ ExchangeError::InvalidWIthdrawalToExecute,
        constraint = withdrawal.fixed.receivers.final_long_token_receiver == final_long_token_receiver.key() @ ExchangeError::InvalidWIthdrawalToExecute,
        constraint = withdrawal.fixed.receivers.final_short_token_receiver == final_short_token_receiver.key() @ ExchangeError::InvalidWIthdrawalToExecute,
    )]
    pub withdrawal: Account<'info, Withdrawal>,
    /// CHECK: only used to invoke CPI and should be checked by it.
    #[account(mut)]
    pub market: UncheckedAccount<'info>,
    /// CHECK: only used to receive lamports.
    #[account(mut)]
    pub user: UncheckedAccount<'info>,
    #[account(mut, constraint = market_token_mint.key() == withdrawal.fixed.tokens.market_token)]
    pub market_token_mint: Account<'info, Mint>,
    #[account(mut, token::mint = market_token_mint)]
    pub market_token_withdrawal_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub final_long_token_receiver: Account<'info, TokenAccount>,
    #[account(mut)]
    pub final_short_token_receiver: Account<'info, TokenAccount>,
    #[account(mut)]
    pub final_long_token_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub final_short_token_vault: Account<'info, TokenAccount>,
}

/// Execute the withdrawal.
pub fn execute_withdrawal<'info>(
    ctx: Context<'_, '_, 'info, 'info, ExecuteWithdrawal<'info>>,
    execution_fee: u64,
) -> Result<()> {
    let withdrawal = &ctx.accounts.withdrawal;
    let refund = withdrawal
        .get_lamports()
        .checked_sub(execution_fee.min(super::MAX_WITHDRAWAL_EXECUTION_FEE))
        .ok_or(ExchangeError::NotEnoughExecutionFee)?;
    ctx.accounts.with_oracle_prices(
        withdrawal.dynamic.tokens_with_feed.tokens.clone(),
        ctx.remaining_accounts,
        |accounts, remaining_accounts| {
            data_store::cpi::execute_withdrawal(
                accounts
                    .execute_withdrawal_ctx()
                    .with_remaining_accounts(remaining_accounts.to_vec()),
            )
        },
    )?;
    data_store::cpi::remove_withdrawal(ctx.accounts.remove_withdrawal_ctx(), refund)?;
    Ok(())
}

impl<'info> Authentication<'info> for ExecuteWithdrawal<'info> {
    fn authority(&self) -> AccountInfo<'info> {
        self.authority.to_account_info()
    }

    fn on_error(&self) -> Result<()> {
        Err(error!(ExchangeError::PermissionDenied))
    }

    fn data_store_program(&self) -> AccountInfo<'info> {
        self.data_store_program.to_account_info()
    }

    fn store(&self) -> AccountInfo<'info> {
        self.store.to_account_info()
    }

    fn roles(&self) -> AccountInfo<'info> {
        self.only_order_keeper.to_account_info()
    }
}

impl<'info> WithOracle<'info> for ExecuteWithdrawal<'info> {
    fn price_provider(&self) -> AccountInfo<'info> {
        self.price_provider.to_account_info()
    }

    fn oracle(&self) -> AccountInfo<'info> {
        self.oracle.to_account_info()
    }

    fn token_config_map(&self) -> AccountInfo<'info> {
        self.token_config_map.to_account_info()
    }
}

impl<'info> ExecuteWithdrawal<'info> {
    fn remove_withdrawal_ctx(&self) -> CpiContext<'_, '_, '_, 'info, RemoveWithdrawal<'info>> {
        CpiContext::new(
            self.data_store_program.to_account_info(),
            RemoveWithdrawal {
                authority: self.authority.to_account_info(),
                store: self.store.to_account_info(),
                only_controller: self.only_order_keeper.to_account_info(),
                withdrawal: self.withdrawal.to_account_info(),
                user: self.user.to_account_info(),
                system_program: self.system_program.to_account_info(),
            },
        )
    }

    fn execute_withdrawal_ctx(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, data_store::cpi::accounts::ExecuteWithdrawal<'info>> {
        CpiContext::new(
            self.data_store_program.to_account_info(),
            data_store::cpi::accounts::ExecuteWithdrawal {
                authority: self.authority.to_account_info(),
                store: self.store.to_account_info(),
                only_order_keeper: self.only_order_keeper.to_account_info(),
                oracle: self.oracle.to_account_info(),
                withdrawal: self.withdrawal.to_account_info(),
                market: self.market.to_account_info(),
                market_token_mint: self.market_token_mint.to_account_info(),
                market_token_withdrawal_vault: self.market_token_withdrawal_vault.to_account_info(),
                final_long_token_vault: self.final_long_token_vault.to_account_info(),
                final_short_token_vault: self.final_short_token_vault.to_account_info(),
                final_long_token_receiver: self.final_long_token_receiver.to_account_info(),
                final_short_token_receiver: self.final_short_token_receiver.to_account_info(),
                token_program: self.token_program.to_account_info(),
            },
        )
    }
}
