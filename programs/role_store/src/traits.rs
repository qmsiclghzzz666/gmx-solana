use anchor_lang::{prelude::*, Bumps};

use crate::{Role, RoleStoreError, MAX_ROLE_LEN};

/// Authorization.
pub trait Authorization<'info> {
    /// Get the address of role store.
    fn role_store(&self) -> Pubkey;

    /// Get the checked authority account.
    fn authority(&self) -> &Signer<'info>;

    /// Get the role to check.
    fn role(&self) -> &Account<'info, Role>;
}

/// Provides access control methods for [`Authorization`].
pub trait Authenticate<'info>: Authorization<'info> + Bumps + Sized {
    /// Check if the authorization is valid.
    fn valid(ctx: &Context<Self>) -> Result<()> {
        require_eq!(
            ctx.accounts.role_store(),
            ctx.accounts.role().store,
            RoleStoreError::MismatchedStore
        );
        require_eq!(
            *ctx.accounts.authority().key,
            ctx.accounts.role().authority,
            RoleStoreError::Unauthorized
        );
        Ok(())
    }

    /// Check if the authorization is valid and the role matches the given.
    fn only_role(ctx: &Context<Self>, role: &str) -> Result<()> {
        Self::valid(ctx)?;
        require!(role.len() <= MAX_ROLE_LEN, RoleStoreError::RoleNameTooLarge);
        require_eq!(
            role,
            &ctx.accounts.role().role,
            RoleStoreError::PermissionDenied
        );
        Ok(())
    }

    /// Check if the authorization is valid and the role is [`CONTROLLER`](Role::CONTROLLER).
    fn only_controller(ctx: &Context<Self>) -> Result<()> {
        Self::only_role(ctx, Role::CONTROLLER)
    }

    /// Check if the authorization is valid and the role is [`ROLE_ADMIN`](Role::ROLE_ADMIN).
    fn only_role_admin(ctx: &Context<Self>) -> Result<()> {
        Self::only_role(ctx, Role::ROLE_ADMIN)
    }
}

impl<'info, T: Authorization<'info> + Bumps> Authenticate<'info> for T {}
