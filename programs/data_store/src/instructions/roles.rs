use anchor_lang::prelude::*;

use crate::{
    states::{DataStore, Roles, Seed},
    DataStoreError,
};

/// Initialize a new roles account.
pub fn initialize_roles(ctx: Context<InitializeRoles>) -> Result<()> {
    ctx.accounts.roles.init(
        ctx.accounts.authority.key(),
        ctx.accounts.store.key(),
        ctx.bumps.roles,
    );
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

/// Verify that the `authority` has the given role in the given `store`.
#[allow(unused_variables)]
pub fn check_role(ctx: Context<CheckRole>, user: Pubkey, role: String) -> Result<bool> {
    ctx.accounts.store.has_role(&ctx.accounts.roles, &role)
}

#[derive(Accounts)]
#[instruction(authority: Pubkey)]
pub struct CheckRole<'info> {
    pub store: Account<'info, DataStore>,
    #[account(
        has_one = store @ DataStoreError::PermissionDenied,
        has_one = authority @ DataStoreError::PermissionDenied,
        seeds = [Roles::SEED, store.key().as_ref(), authority.key().as_ref()],
        bump = roles.bump,
    )]
    pub roles: Account<'info, Roles>,
}

/// Verify that the `user` is an admin of the given `store`.
#[allow(unused_variables)]
pub fn check_admin(ctx: Context<CheckRole>, user: Pubkey) -> Result<bool> {
    Ok(ctx.accounts.roles.is_admin())
}
