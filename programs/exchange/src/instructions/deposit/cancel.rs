use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};
use data_store::{constants::EVENT_AUTHORITY_SEED, program::DataStore, states::Deposit};

use crate::utils::ControllerSeeds;

use super::utils::CancelDepositUtils;

#[derive(Accounts)]
pub struct CancelDeposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
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
    /// The deposit to cancel.
    ///
    /// ## Notes
    /// - Only the user who created the deposit can receive the funds,
    /// which is checked by [`remove_deposit`](data_store::instructions::remove_deposit)
    /// through CPI, who also checks whether the `store` matches.
    #[account(
        mut,
        constraint = deposit.fixed.senders.user == user.key(),
    )]
    pub deposit: Account<'info, Deposit>,
    /// The token account for receiving the initial long tokens.
    #[account(mut, token::authority = user)]
    pub initial_long_token: Option<Account<'info, TokenAccount>>,
    /// The token account for receiving the initial short tokens.
    #[account(mut, token::authority = user)]
    pub initial_short_token: Option<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub long_token_deposit_vault: Option<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub short_token_deposit_vault: Option<Account<'info, TokenAccount>>,
    /// CHECK: check by cancel utils.
    #[account(mut)]
    pub initial_long_market: Option<UncheckedAccount<'info>>,
    /// CHECK: check by cancen utils.
    #[account(mut)]
    pub initial_short_market: Option<UncheckedAccount<'info>>,
    /// CHECK: Only the event authority can invoke self-CPI
    #[account(seeds = [EVENT_AUTHORITY_SEED], bump, seeds::program = data_store_program.key())]
    pub event_authority: UncheckedAccount<'info>,
    pub data_store_program: Program<'info, DataStore>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

/// Cancel a deposit.
pub fn cancel_deposit(ctx: Context<CancelDeposit>) -> Result<()> {
    // We will attach the controller seeds even it may not be provided.
    let controller = ControllerSeeds::find(ctx.accounts.store.key);
    ctx.accounts
        .cancel_utils()
        .execute(ctx.accounts.user.to_account_info(), &controller, 0)?;
    // TODO: emit deposit removed event.
    Ok(())
}

impl<'info> CancelDeposit<'info> {
    fn cancel_utils(&self) -> CancelDepositUtils<'_, 'info> {
        CancelDepositUtils {
            event_authority: self.event_authority.to_account_info(),
            data_store_program: self.data_store_program.to_account_info(),
            token_program: self.token_program.to_account_info(),
            system_program: self.system_program.to_account_info(),
            store: self.store.to_account_info(),
            controller: self.controller.to_account_info(),
            user: self.user.to_account_info(),
            deposit: &self.deposit,
            initial_long_token_transfer: super::utils::TransferIn::new(
                self.initial_long_token.as_ref(),
                self.long_token_deposit_vault.as_ref(),
                self.initial_long_market.as_ref(),
            ),
            initial_short_token_transfer: super::utils::TransferIn::new(
                self.initial_short_token.as_ref(),
                self.short_token_deposit_vault.as_ref(),
                self.initial_short_market.as_ref(),
            ),
            reason: "canceled by the user",
        }
    }
}
