use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};
use gmsol_store::{constants::EVENT_AUTHORITY_SEED, program::GmsolStore, states::Withdrawal};

use crate::{states::Controller, utils::ControllerSeeds};

use super::utils::CancelWithdrawalUtils;

#[derive(Accounts)]
pub struct CancelWithdrawal<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    /// Controller.
    #[account(
        has_one = store,
        seeds = [
            crate::constants::CONTROLLER_SEED,
            store.key().as_ref(),
        ],
        bump = controller.load()?.bump,
    )]
    pub controller: AccountLoader<'info, Controller>,
    /// CHECK: used and checked by CPI.
    pub store: UncheckedAccount<'info>,
    /// The withdrawal to cancel.
    ///
    /// ## Notes
    /// - Only the user who created the withdrawal can receive the funds,
    ///   which is checked by [`remove_withdrawal`](gmsol_store::gmsol_store::remove_withdrawal)
    ///   through CPI.
    #[account(mut)]
    pub withdrawal: Account<'info, Withdrawal>,
    /// Token account for receiving the market tokens.
    #[account(mut, token::authority = user)]
    pub market_token: Account<'info, TokenAccount>,
    /// CHECK: check by CPI.
    #[account(mut)]
    pub market_token_withdrawal_vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    /// CHECK: Only the event authority can invoke self-CPI
    #[account(seeds = [EVENT_AUTHORITY_SEED], bump, seeds::program = data_store_program.key())]
    pub event_authority: UncheckedAccount<'info>,
    pub data_store_program: Program<'info, GmsolStore>,
}

/// Cancel Withdrawal.
pub fn cancel_withdrawal(ctx: Context<CancelWithdrawal>) -> Result<()> {
    let controller = ControllerSeeds::find(ctx.accounts.store.key);
    ctx.accounts
        .cancel_utils()
        .execute(ctx.accounts.user.to_account_info(), &controller, 0)?;
    Ok(())
}

impl<'info> CancelWithdrawal<'info> {
    fn cancel_utils<'a>(&'a self) -> CancelWithdrawalUtils<'a, 'info> {
        CancelWithdrawalUtils {
            event_authority: self.event_authority.to_account_info(),
            data_store_program: self.data_store_program.to_account_info(),
            token_program: self.token_program.to_account_info(),
            system_program: self.system_program.to_account_info(),
            controller: self.controller.to_account_info(),
            store: self.store.to_account_info(),
            user: self.user.to_account_info(),
            withdrawal: &self.withdrawal,
            market_token_account: self.market_token.to_account_info(),
            market_token_vault: self.market_token_withdrawal_vault.to_account_info(),
            reason: "canceled by the user",
        }
    }
}
