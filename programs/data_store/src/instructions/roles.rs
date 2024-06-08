use anchor_lang::prelude::*;

use crate::{states::Store, utils::internal};

#[derive(Accounts)]
pub struct CheckRole<'info> {
    pub authority: Signer<'info>,
    pub store: AccountLoader<'info, Store>,
}

/// Verify that the `authority` has the given role in the given `store`.
pub fn check_role(ctx: Context<CheckRole>, role: String) -> Result<bool> {
    ctx.accounts
        .store
        .load()?
        .has_role(ctx.accounts.authority.key, &role)
}

/// Verify that the `user` is an admin of the given `store`.
pub fn check_admin(ctx: Context<CheckRole>) -> Result<bool> {
    Ok(ctx
        .accounts
        .store
        .load()?
        .is_authority(ctx.accounts.authority.key))
}

#[derive(Accounts)]
pub struct HasRole<'info> {
    pub store: AccountLoader<'info, Store>,
}

/// Verify that the `authority` has the given role in the given `store` without signing.
pub fn has_role(ctx: Context<HasRole>, authority: Pubkey, role: String) -> Result<bool> {
    ctx.accounts.store.load()?.has_role(&authority, &role)
}

/// Verify that the `user` is an admin of the given `store` without signing.
pub fn has_admin(ctx: Context<HasRole>, authority: Pubkey) -> Result<bool> {
    Ok(ctx.accounts.store.load()?.is_authority(&authority))
}

#[derive(Accounts)]
pub struct EnableRole<'info> {
    pub authority: Signer<'info>,
    #[account(mut)]
    pub store: AccountLoader<'info, Store>,
}

/// Enable the given role in the data store.
pub fn enable_role(ctx: Context<EnableRole>, role: String) -> Result<()> {
    ctx.accounts.store.load_mut()?.enable_role(&role)
}

impl<'info> internal::Authentication<'info> for EnableRole<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &AccountLoader<'info, Store> {
        &self.store
    }
}

#[derive(Accounts)]
pub struct DisableRole<'info> {
    pub authority: Signer<'info>,
    #[account(mut)]
    pub store: AccountLoader<'info, Store>,
}

/// Disable the given role in the data store.
pub fn disable_role(ctx: Context<DisableRole>, role: String) -> Result<()> {
    ctx.accounts.store.load_mut()?.disable_role(&role)
}

impl<'info> internal::Authentication<'info> for DisableRole<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &AccountLoader<'info, Store> {
        &self.store
    }
}

#[derive(Accounts)]
pub struct GrantRole<'info> {
    pub authority: Signer<'info>,
    #[account(mut)]
    pub store: AccountLoader<'info, Store>,
}

/// Grant a role to the user.
pub fn grant_role(ctx: Context<GrantRole>, user: Pubkey, role: String) -> Result<()> {
    ctx.accounts.store.load_mut()?.grant(&user, &role)
}

impl<'info> internal::Authentication<'info> for GrantRole<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &AccountLoader<'info, Store> {
        &self.store
    }
}

#[derive(Accounts)]
pub struct RevokeRole<'info> {
    pub authority: Signer<'info>,
    #[account(mut)]
    pub store: AccountLoader<'info, Store>,
}

/// Revoke a role to the user.
pub fn revoke_role(ctx: Context<RevokeRole>, user: Pubkey, role: String) -> Result<()> {
    ctx.accounts.store.load_mut()?.revoke(&user, &role)
}

impl<'info> internal::Authentication<'info> for RevokeRole<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &AccountLoader<'info, Store> {
        &self.store
    }
}
