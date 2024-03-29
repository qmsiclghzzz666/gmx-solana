use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};
use data_store::{
    constants,
    cpi::accounts::{CheckRole, MarketVaultTransferOut, RemoveWithdrawal},
    program::DataStore,
    states::Withdrawal,
    utils::Authentication,
};

use crate::ExchangeError;

#[derive(Accounts)]
pub struct CancelWithdrawal<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    /// CHECK: used and checked by CPI.
    pub store: UncheckedAccount<'info>,
    /// CHECK: used and checked by CPI.
    pub only_controller: UncheckedAccount<'info>,
    pub data_store_program: Program<'info, DataStore>,
    /// The withdrawal to cancel.
    ///
    /// ## Notes
    /// - Only the user who created the withdrawal can receive the funds,
    /// which is checked by [`remove_withdrawal`](data_store::instructions::remove_withdrawal)
    /// through CPI.
    #[account(
        mut,
        constraint = withdrawal.tokens.market_token == market_token.mint @ ExchangeError::InvalidWithdrawalToCancel,
    )]
    pub withdrawal: Account<'info, Withdrawal>,
    /// CHECK: only used to receive lamports.
    #[account(mut)]
    pub user: UncheckedAccount<'info>,
    /// Token account for receiving the market tokens.
    #[account(mut, token::authority = user)]
    pub market_token: Account<'info, TokenAccount>,
    /// The vault saving the market tokens for withdrawal.
    #[account(mut,
        token::mint = market_token.mint,
        seeds = [
            constants::MARKET_VAULT_SEED,
            store.key().as_ref(),
            market_token_withdrawal_vault.mint.as_ref(),
            &[],
        ],
        bump,
        seeds::program = data_store_program.key(),
    )]
    pub market_token_withdrawal_vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

/// Cancel Withdrawal.
pub fn cancel_withdrawal(ctx: Context<CancelWithdrawal>, execution_fee: u64) -> Result<()> {
    let market_token_amount = ctx.accounts.withdrawal.tokens.market_token_amount;
    // FIXME: it seems that we don't have to check this?
    // require!(
    //     market_token_amount != 0,
    //     ExchangeError::EmptyWithdrawalAmount,
    // );
    let refund = ctx
        .accounts
        .withdrawal
        .get_lamports()
        .checked_sub(execution_fee.min(super::MAX_WITHDRAWAL_EXECUTION_FEE))
        .ok_or(ExchangeError::NotEnoughExecutionFee)?;
    data_store::cpi::remove_withdrawal(ctx.accounts.remove_withdrawal_ctx(), refund)?;

    if market_token_amount != 0 {
        data_store::cpi::market_vault_transfer_out(
            ctx.accounts.market_vault_transfer_out_ctx(),
            market_token_amount,
        )?;
    }
    Ok(())
}

impl<'info> Authentication<'info> for CancelWithdrawal<'info> {
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

impl<'info> CancelWithdrawal<'info> {
    fn remove_withdrawal_ctx(&self) -> CpiContext<'_, '_, '_, 'info, RemoveWithdrawal<'info>> {
        CpiContext::new(
            self.data_store_program.to_account_info(),
            RemoveWithdrawal {
                authority: self.authority.to_account_info(),
                store: self.store.to_account_info(),
                only_controller: self.only_controller.to_account_info(),
                withdrawal: self.withdrawal.to_account_info(),
                user: self.user.to_account_info(),
                system_program: self.system_program.to_account_info(),
            },
        )
    }

    fn market_vault_transfer_out_ctx(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, MarketVaultTransferOut<'info>> {
        CpiContext::new(
            self.data_store_program.to_account_info(),
            MarketVaultTransferOut {
                authority: self.authority.to_account_info(),
                only_controller: self.only_controller.to_account_info(),
                store: self.store.to_account_info(),
                market_vault: self.market_token_withdrawal_vault.to_account_info(),
                to: self.market_token.to_account_info(),
                token_program: self.token_program.to_account_info(),
            },
        )
    }
}
