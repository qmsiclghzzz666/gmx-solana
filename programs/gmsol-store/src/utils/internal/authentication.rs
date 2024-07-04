use anchor_lang::{prelude::*, Bumps};

use crate::{
    states::{RoleKey, Store},
    StoreError,
};

/// Accounts that can be used for authentication.
pub(crate) trait Authentication<'info> {
    /// Get the authority to check.
    fn authority(&self) -> &Signer<'info>;

    /// Get the data store account.
    fn store(&self) -> &AccountLoader<'info, Store>;
}

/// Provides access control utils for [`Authentication`]s.
pub(crate) trait Authenticate<'info>: Authentication<'info> + Bumps + Sized {
    /// Verify that context matches.
    fn verify(_ctx: &Context<Self>) -> Result<()> {
        Ok(())
    }

    /// Check that the `authority` has the given `role`.
    fn only(ctx: &Context<Self>, role: &str) -> Result<()> {
        Self::verify(ctx)?;
        require!(
            ctx.accounts
                .store()
                .load()?
                .has_role(ctx.accounts.authority().key, role)?,
            StoreError::PermissionDenied
        );
        Ok(())
    }

    /// Check that the `authority` is an admin.
    fn only_admin(ctx: &Context<Self>) -> Result<()> {
        Self::verify(ctx)?;
        require!(
            ctx.accounts
                .store()
                .load()?
                .is_authority(ctx.accounts.authority().key),
            StoreError::NotAnAdmin
        );
        Ok(())
    }

    /// Check that the `authority` has the [`CONTROLLER`](`RoleKey::CONTROLLER`) role.
    fn only_controller(ctx: &Context<Self>) -> Result<()> {
        Self::only(ctx, RoleKey::CONTROLLER)
    }

    /// Check that the `authority` has the [`MARKET_KEEPER`](`RoleKey::MARKET_KEEPER`) role.
    fn only_market_keeper(ctx: &Context<Self>) -> Result<()> {
        Self::only(ctx, RoleKey::MARKET_KEEPER)
    }

    /// Check that the `authority` has the [`ORDER_KEEPER`](`RoleKey::ORDER_KEEPER`) role.
    fn only_order_keeper(ctx: &Context<Self>) -> Result<()> {
        Self::only(ctx, RoleKey::ORDER_KEEPER)
    }
}

impl<'info, T> Authenticate<'info> for T where T: Authentication<'info> + Bumps + Sized {}
