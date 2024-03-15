use anchor_lang::{prelude::*, Bumps};

use crate::cpi::accounts::CheckRole;

/// Accounts that can be used for authentication.
pub trait Authentication<'info> {
    /// Get the authority to check.
    fn authority(&self) -> Pubkey;

    /// Get the cpi context for checking role or admin permission.
    fn check_role_ctx(&self) -> CpiContext<'_, '_, '_, 'info, CheckRole<'info>>;

    /// Callback on authentication error.
    fn on_error(&self) -> Result<()>;
}

/// Provides access control utils for [`Authentication`]s.
pub trait Authenticate<'info>: Authentication<'info> + Bumps + Sized {
    /// Check that the `authority` has the given `role`.
    fn only(ctx: &Context<Self>, role: &str) -> Result<()> {
        let has_role = crate::cpi::check_role(
            ctx.accounts.check_role_ctx(),
            ctx.accounts.authority(),
            role.to_string(),
        )?
        .get();
        if has_role {
            Ok(())
        } else {
            ctx.accounts.on_error()
        }
    }

    /// Check that the `authority` is an admin.
    fn only_admin(ctx: &Context<Self>) -> Result<()> {
        let is_admin =
            crate::cpi::check_admin(ctx.accounts.check_role_ctx(), ctx.accounts.authority())?.get();
        if is_admin {
            Ok(())
        } else {
            ctx.accounts.on_error()
        }
    }
}

impl<'info, T> Authenticate<'info> for T where T: Authentication<'info> + Bumps + Sized {}
