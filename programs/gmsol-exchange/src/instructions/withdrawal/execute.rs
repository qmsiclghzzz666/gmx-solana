use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use gmsol_store::{
    constants::EVENT_AUTHORITY_SEED,
    cpi::accounts::MarketTransferOut,
    program::GmsolStore,
    states::{PriceProvider, Withdrawal},
    utils::{Authentication, WithOracle, WithOracleExt},
};

use crate::{utils::ControllerSeeds, ExchangeError};

use super::utils::CancelWithdrawalUtils;

#[derive(Accounts)]
pub struct ExecuteWithdrawal<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    /// CHECK: only used as signing PDA.
    #[account(
        seeds = [
            crate::constants::CONTROLLER_SEED,
            store.key().as_ref(),
        ],
        bump,
    )]
    pub controller: UncheckedAccount<'info>,
    /// CHECK: only used to invoke CPI.
    pub store: UncheckedAccount<'info>,
    /// CHECK: check by CPI.
    pub token_map: UncheckedAccount<'info>,
    pub price_provider: Interface<'info, PriceProvider>,
    #[account(mut)]
    pub oracle: Account<'info, gmsol_store::states::Oracle>,
    /// Withdrawal to execute.
    ///
    /// ## Notes
    /// - `user` is checked on the removal CPI of the withdrawal.
    #[account(
        mut,
        constraint = withdrawal.fixed.tokens.market_token == market_token_mint.key() @ ExchangeError::InvalidWithdrawalToExecute,
        constraint = withdrawal.fixed.receivers.final_long_token_receiver == final_long_token_receiver.key() @ ExchangeError::InvalidWithdrawalToExecute,
        constraint = withdrawal.fixed.receivers.final_short_token_receiver == final_short_token_receiver.key() @ ExchangeError::InvalidWithdrawalToExecute,
        constraint = withdrawal.fixed.market_token_account == market_token_account.key() @ ExchangeError::InvalidWithdrawalToExecute,
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
    /// CHECK: check by `try_removable` method and CPI.
    #[account(mut)]
    pub market_token_account: UncheckedAccount<'info>,
    #[account(mut)]
    pub final_long_token_receiver: Account<'info, TokenAccount>,
    #[account(mut)]
    pub final_short_token_receiver: Account<'info, TokenAccount>,
    #[account(mut)]
    pub final_long_token_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub final_short_token_vault: Account<'info, TokenAccount>,
    /// CHECK: Only the event authority can invoke self-CPI
    #[account(seeds = [EVENT_AUTHORITY_SEED], bump, seeds::program = data_store_program.key())]
    pub event_authority: UncheckedAccount<'info>,
    pub data_store_program: Program<'info, GmsolStore>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

/// Execute the withdrawal.
pub fn execute_withdrawal<'info>(
    ctx: Context<'_, '_, 'info, 'info, ExecuteWithdrawal<'info>>,
    execution_fee: u64,
    cancel_on_execution_error: bool,
) -> Result<()> {
    let store = ctx.accounts.store.key();
    let controller = ControllerSeeds::find(&store);
    let withdrawal = &ctx.accounts.withdrawal;

    // TODO: check for the pre-condition of the execution.
    let (final_long_amount, final_long_market, final_short_amount, final_short_market) =
        ctx.accounts.with_oracle_prices(
            withdrawal.dynamic.tokens_with_feed.tokens.clone(),
            ctx.remaining_accounts,
            &controller.as_seeds(),
            |accounts, remaining_accounts| {
                let store = accounts.store.key;
                let swap = &accounts.withdrawal.dynamic.swap;
                let final_long_market = swap
                    .find_last_market(store, true, remaining_accounts)
                    .unwrap_or(accounts.market.to_account_info());
                let final_short_market = swap
                    .find_last_market(store, false, remaining_accounts)
                    .unwrap_or(accounts.market.to_account_info());
                let (final_long_amount, final_short_amount) = gmsol_store::cpi::execute_withdrawal(
                    accounts
                        .execute_withdrawal_ctx()
                        .with_signer(&[&controller.as_seeds()])
                        .with_remaining_accounts(remaining_accounts.to_vec()),
                    !cancel_on_execution_error,
                )?
                .get();
                accounts.withdrawal.reload()?;
                Ok((
                    final_long_amount,
                    final_long_market,
                    final_short_amount,
                    final_short_market,
                ))
            },
        )?;

    let mut reason = "execution failed";
    // Transfer out final tokens.
    if final_long_amount != 0 {
        // Must have been validated during the execution.
        gmsol_store::cpi::market_transfer_out(
            ctx.accounts
                .market_transfer_out_ctx(true, final_long_market)
                .with_signer(&[&controller.as_seeds()]),
            final_long_amount,
        )?;
        reason = "executed";
    }
    if final_short_amount != 0 {
        // Must have been validated during the execution.
        gmsol_store::cpi::market_transfer_out(
            ctx.accounts
                .market_transfer_out_ctx(false, final_short_market)
                .with_signer(&[&controller.as_seeds()]),
            final_short_amount,
        )?;
        reason = "executed";
    }

    ctx.accounts.cancel_utils(reason).execute(
        ctx.accounts.authority.to_account_info(),
        &controller,
        execution_fee,
    )?;

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
}

impl<'info> WithOracle<'info> for ExecuteWithdrawal<'info> {
    fn price_provider(&self) -> AccountInfo<'info> {
        self.price_provider.to_account_info()
    }

    fn oracle(&self) -> AccountInfo<'info> {
        self.oracle.to_account_info()
    }

    fn token_map(&self) -> AccountInfo<'info> {
        self.token_map.to_account_info()
    }

    fn controller(&self) -> AccountInfo<'info> {
        self.controller.to_account_info()
    }
}

impl<'info> ExecuteWithdrawal<'info> {
    fn cancel_utils<'a>(&'a self, reason: &'a str) -> CancelWithdrawalUtils<'a, 'info> {
        CancelWithdrawalUtils {
            event_authority: self.event_authority.to_account_info(),
            data_store_program: self.data_store_program.to_account_info(),
            token_program: self.token_program.to_account_info(),
            system_program: self.system_program.to_account_info(),
            controller: self.controller.to_account_info(),
            store: self.store.to_account_info(),
            user: self.user.to_account_info(),
            withdrawal: &self.withdrawal,
            market_token_account: self.market_token_account.to_account_info(),
            market_token_vault: self.market_token_withdrawal_vault.to_account_info(),
            reason,
        }
    }

    fn execute_withdrawal_ctx(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, gmsol_store::cpi::accounts::ExecuteWithdrawal<'info>> {
        CpiContext::new(
            self.data_store_program.to_account_info(),
            gmsol_store::cpi::accounts::ExecuteWithdrawal {
                authority: self.controller.to_account_info(),
                store: self.store.to_account_info(),
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

    fn market_transfer_out_ctx(
        &self,
        is_long: bool,
        market: AccountInfo<'info>,
    ) -> CpiContext<'_, '_, '_, 'info, MarketTransferOut<'info>> {
        let (to, vault) = if is_long {
            (
                self.final_long_token_receiver.to_account_info(),
                self.final_long_token_vault.to_account_info(),
            )
        } else {
            (
                self.final_short_token_receiver.to_account_info(),
                self.final_short_token_vault.to_account_info(),
            )
        };

        CpiContext::new(
            self.data_store_program.to_account_info(),
            MarketTransferOut {
                authority: self.controller.to_account_info(),
                store: self.store.to_account_info(),
                market,
                to,
                vault,
                token_program: self.token_program.to_account_info(),
            },
        )
    }
}
