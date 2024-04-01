use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};
use data_store::{
    constants,
    cpi::accounts::{CheckRole, MarketVaultTransferOut, RemoveDeposit},
    program::DataStore,
    states::Deposit,
    utils::Authentication,
};

use crate::ExchangeError;

/// Cancel a deposit.
pub fn cancel_deposit(ctx: Context<CancelDeposit>, execution_fee: u64) -> Result<()> {
    let initial_long_amount = ctx
        .accounts
        .deposit
        .fixed
        .tokens
        .params
        .initial_long_token_amount;
    let initial_short_amount = ctx
        .accounts
        .deposit
        .fixed
        .tokens
        .params
        .initial_short_token_amount;
    // FIXME: it seems that we don't have to check this?
    // require!(
    //     initial_long_amount != 0 || initial_short_amount != 0,
    //     ExchangeError::EmptyDepositAmounts
    // );
    let refund = ctx
        .accounts
        .deposit
        .get_lamports()
        .checked_sub(execution_fee.min(crate::MAX_DEPOSIT_EXECUTION_FEE))
        .ok_or(ExchangeError::NotEnoughExecutionFee)?;
    data_store::cpi::remove_deposit(ctx.accounts.remove_deposit_ctx(), refund)?;

    if initial_long_amount != 0 {
        data_store::cpi::market_vault_transfer_out(
            ctx.accounts.market_vault_transfer_out_ctx(true),
            initial_long_amount,
        )?;
    }

    if initial_short_amount != 0 {
        data_store::cpi::market_vault_transfer_out(
            ctx.accounts.market_vault_transfer_out_ctx(false),
            initial_short_amount,
        )?;
    }

    // TODO: emit deposit removed event.
    Ok(())
}

#[derive(Accounts)]
pub struct CancelDeposit<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    /// CHECK: only used to invoke CPI.
    pub store: UncheckedAccount<'info>,
    /// CHECK: only used to invoke CPI.
    pub only_controller: UncheckedAccount<'info>,
    pub data_store_program: Program<'info, DataStore>,
    /// The deposit to cancel.
    ///
    /// ## Notes
    /// - Only the user who created the deposit can receive the funds,
    /// which is checked by [`remove_deposit`](data_store::instructions::remove_deposit)
    /// through CPI.
    #[account(
        mut,
        constraint = deposit.fixed.tokens.initial_long_token == initial_long_token.mint @ ExchangeError::InvalidDepositToCancel,
        constraint = deposit.fixed.tokens.initial_short_token == initial_short_token.mint @ ExchangeError::InvalidDepositToCancel,
    )]
    pub deposit: Account<'info, Deposit>,
    /// CHECK: only used to receive lamports.
    #[account(mut)]
    pub user: UncheckedAccount<'info>,
    /// The token account for receiving the initial long tokens.
    #[account(mut, token::authority = user)]
    pub initial_long_token: Account<'info, TokenAccount>,
    /// The token account for receiving the initial short tokens.
    #[account(mut, token::authority = user)]
    pub initial_short_token: Account<'info, TokenAccount>,
    #[account(
        mut,
        token::mint = initial_long_token.mint,
        seeds = [
            constants::MARKET_VAULT_SEED,
            store.key().as_ref(),
            long_token_deposit_vault.mint.as_ref(),
            &[],
        ],
        bump,
        seeds::program = data_store_program.key(),
    )]
    pub long_token_deposit_vault: Account<'info, TokenAccount>,
    #[account(
        mut,
        token::mint = initial_short_token.mint,
        seeds = [
            constants::MARKET_VAULT_SEED,
            store.key().as_ref(),
            short_token_deposit_vault.mint.as_ref(),
            &[],
        ],
        bump,
        seeds::program = data_store_program.key(),
    )]
    pub short_token_deposit_vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> Authentication<'info> for CancelDeposit<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn check_role_ctx(&self) -> CpiContext<'_, '_, '_, 'info, CheckRole<'info>> {
        CpiContext::new(
            self.data_store_program.to_account_info(),
            CheckRole {
                store: self.store.to_account_info(),
                roles: self.only_controller.to_account_info(),
            },
        )
    }

    fn on_error(&self) -> Result<()> {
        Err(error!(ExchangeError::PermissionDenied))
    }
}

impl<'info> CancelDeposit<'info> {
    fn remove_deposit_ctx(&self) -> CpiContext<'_, '_, '_, 'info, RemoveDeposit<'info>> {
        CpiContext::new(
            self.data_store_program.to_account_info(),
            RemoveDeposit {
                authority: self.authority.to_account_info(),
                only_controller: self.only_controller.to_account_info(),
                store: self.store.to_account_info(),
                deposit: self.deposit.to_account_info(),
                user: self.user.to_account_info(),
                system_program: self.system_program.to_account_info(),
            },
        )
    }

    fn market_vault_transfer_out_ctx(
        &self,
        is_long_token: bool,
    ) -> CpiContext<'_, '_, '_, 'info, MarketVaultTransferOut<'info>> {
        CpiContext::new(
            self.data_store_program.to_account_info(),
            MarketVaultTransferOut {
                authority: self.authority.to_account_info(),
                only_controller: self.only_controller.to_account_info(),
                store: self.store.to_account_info(),
                market_vault: if is_long_token {
                    self.long_token_deposit_vault.to_account_info()
                } else {
                    self.short_token_deposit_vault.to_account_info()
                },
                to: if is_long_token {
                    self.initial_long_token.to_account_info()
                } else {
                    self.initial_short_token.to_account_info()
                },
                token_program: self.token_program.to_account_info(),
            },
        )
    }
}
