use anchor_lang::prelude::*;
use gmsol_store::states::Store;
use gmsol_utils::InitSpace;

use crate::states::Controller;

#[derive(Accounts)]
pub struct InitializeController<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub store: AccountLoader<'info, Store>,
    #[account(
        init,
        payer = payer,
        space = 8 + Controller::INIT_SPACE,
        seeds =[
            crate::constants::CONTROLLER_SEED,
            store.key().as_ref(),
        ],
        bump,
    )]
    pub controller: AccountLoader<'info, Controller>,
    pub system_program: Program<'info, System>,
}

/// Initialize a [`Controller`] Account.
pub fn initialize_controller(ctx: Context<InitializeController>) -> Result<()> {
    ctx.accounts
        .controller
        .load_init()?
        .init(ctx.accounts.store.key(), ctx.bumps.controller);
    Ok(())
}
