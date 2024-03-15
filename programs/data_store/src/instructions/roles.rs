use anchor_lang::prelude::*;

use crate::states::{DataStore, Roles, Seed};

/// Initialize a new roles account.
pub fn initialize_roles(ctx: Context<InitializeRoles>) -> Result<()> {
    ctx.accounts.roles.init(ctx.bumps.roles);
    Ok(())
}

#[derive(Accounts)]
pub struct InitializeRoles<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    pub store: Account<'info, DataStore>,
    #[account(
        init,
        payer = authority,
        space = 8 + Roles::INIT_SPACE,
        seeds = [Roles::SEED, store.key().as_ref(), authority.key().as_ref()],
        bump,
    )]
    pub roles: Account<'info, Roles>,
    pub system_program: Program<'info, System>,
}
