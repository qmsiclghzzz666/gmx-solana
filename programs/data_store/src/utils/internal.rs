use anchor_lang::{prelude::*, Bumps};

use crate::{
    states::{DataStore, RoleKey, Roles, Seed},
    DataStoreError,
};

/// Accounts that can be used for authentication.
pub(crate) trait Authentication<'info> {
    /// Get the authority to check.
    fn authority(&self) -> &Signer<'info>;

    /// Get the data store account.
    fn store(&self) -> &Account<'info, DataStore>;

    /// Get the roles account.
    fn roles(&self) -> &Account<'info, Roles>;
}

/// Provides access control utils for [`Authentication`]s.
pub(crate) trait Authenticate<'info>: Authentication<'info> + Bumps + Sized {
    /// Verify that context matches.
    fn verify(ctx: &Context<Self>) -> Result<()> {
        let authority = ctx.accounts.authority().key();
        let store_key = ctx.accounts.store().key();
        let roles = ctx.accounts.roles();
        require_eq!(roles.authority, authority, DataStoreError::PermissionDenied);
        require_eq!(roles.store, store_key, DataStoreError::PermissionDenied);
        let expected = Pubkey::create_program_address(
            &[
                Roles::SEED,
                store_key.as_ref(),
                authority.as_ref(),
                &[roles.bump],
            ],
            &crate::ID,
        )
        .map_err(|_| DataStoreError::InvalidPDA)?;
        require_eq!(roles.key(), expected, DataStoreError::PermissionDenied);
        Ok(())
    }

    /// Check that the `authority` has the given `role`.
    fn only(ctx: &Context<Self>, role: &str) -> Result<()> {
        Self::verify(ctx)?;
        require!(
            ctx.accounts.store().has_role(ctx.accounts.roles(), role)?,
            DataStoreError::PermissionDenied
        );
        Ok(())
    }

    /// Check that the `authority` is an admin.
    fn only_admin(ctx: &Context<Self>) -> Result<()> {
        Self::verify(ctx)?;
        require!(ctx.accounts.roles().is_admin(), DataStoreError::NotAnAdmin);
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
