use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};
use data_store::{
    cpi::accounts::RemoveDeposit,
    program::DataStore,
    states::Deposit,
    utils::{Authenticate, Authentication},
};

use crate::{utils::ControllerSeeds, ExchangeError};

pub(crate) fn only_controller_or_deposit_creator(ctx: &Context<CancelDeposit>) -> Result<()> {
    if ctx.accounts.user.is_signer {
        // The creator is signed for the cancellation.
        Ok(())
    } else {
        // `check_role` CPI will only pass when `authority` is a signer.
        Authenticate::only_controller(ctx)
    }
}

#[derive(Accounts)]
pub struct CancelDeposit<'info> {
    /// CHECK: check by access control.
    #[account(mut)]
    pub authority: UncheckedAccount<'info>,
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
    /// through CPI, who also checks whether the `store` matches.
    #[account(mut)]
    pub deposit: Account<'info, Deposit>,
    /// CHECK: check by access control.
    #[account(mut)]
    pub user: UncheckedAccount<'info>,
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
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

/// Cancel a deposit.
pub fn cancel_deposit(ctx: Context<CancelDeposit>, execution_fee: u64) -> Result<()> {
    // We will attach the controller seeds even it may not be provided.
    let controller = ControllerSeeds::find(ctx.accounts.store.key);
    let refund = ctx
        .accounts
        .deposit
        .get_lamports()
        .checked_sub(execution_fee.min(crate::MAX_DEPOSIT_EXECUTION_FEE))
        .ok_or(ExchangeError::NotEnoughExecutionFee)?;
    data_store::cpi::remove_deposit(
        ctx.accounts
            .remove_deposit_ctx()
            .with_signer(&[&controller.as_seeds()]),
        refund,
    )?;

    // TODO: emit deposit removed event.
    Ok(())
}

impl<'info> Authentication<'info> for CancelDeposit<'info> {
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
        self.only_controller.to_account_info()
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
                initial_long_token: self
                    .initial_long_token
                    .as_ref()
                    .map(|a| a.to_account_info()),
                initial_short_token: self
                    .initial_short_token
                    .as_ref()
                    .map(|a| a.to_account_info()),
                long_token_deposit_vault: self
                    .long_token_deposit_vault
                    .as_ref()
                    .map(|a| a.to_account_info()),
                short_token_deposit_vault: self
                    .short_token_deposit_vault
                    .as_ref()
                    .map(|a| a.to_account_info()),
                system_program: self.system_program.to_account_info(),
                token_program: self.token_program.to_account_info(),
            },
        )
    }
}
