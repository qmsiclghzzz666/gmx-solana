use anchor_lang::{prelude::*, Bumps};

use crate::{
    cpi::accounts::{CheckRole, ClearAllPrices, SetPricesFromPriceFeed},
    states::RoleKey,
};

/// Accounts that can be used for authentication.
pub trait Authentication<'info>: Bumps + Sized {
    /// Get the authority to check.
    ///
    /// ## Notes
    /// - `authority` should be a signer.
    fn authority(&self) -> AccountInfo<'info>;

    /// Get data store program.
    fn data_store_program(&self) -> AccountInfo<'info>;

    /// Get data store.
    fn store(&self) -> AccountInfo<'info>;

    /// Get the cpi context for checking role or admin permission.
    fn check_role_ctx(&self) -> CpiContext<'_, '_, '_, 'info, CheckRole<'info>> {
        CpiContext::new(
            self.data_store_program(),
            CheckRole {
                authority: self.authority(),
                store: self.store(),
            },
        )
    }

    /// Callback on authentication error.
    fn on_error(&self) -> Result<()>;
}

/// Provides access control utils for [`Authentication`]s.
pub trait Authenticate<'info>: Authentication<'info> {
    /// Check that the `authority` has the given `role`.
    fn only(ctx: &Context<Self>, role: &str) -> Result<()> {
        let has_role =
            crate::cpi::check_role(ctx.accounts.check_role_ctx(), role.to_string())?.get();
        if has_role {
            Ok(())
        } else {
            ctx.accounts.on_error()
        }
    }

    /// Check that the `authority` is an admin.
    fn only_admin(ctx: &Context<Self>) -> Result<()> {
        let is_admin = crate::cpi::check_admin(ctx.accounts.check_role_ctx())?.get();
        if is_admin {
            Ok(())
        } else {
            ctx.accounts.on_error()
        }
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

impl<'info, T> Authenticate<'info> for T where T: Authentication<'info> {}

/// Accounts that with oracle context.
pub trait WithOracle<'info>: Authentication<'info> {
    /// Get the price provider.
    fn price_provider(&self) -> AccountInfo<'info>;

    /// Get the oracle account.
    fn oracle(&self) -> AccountInfo<'info>;

    /// Get the token config map account.
    fn token_config_map(&self) -> AccountInfo<'info>;

    /// Get config account.
    fn config(&self) -> AccountInfo<'info>;
}

/// Extension trait for [`WithOracle`].
pub trait WithOracleExt<'info>: WithOracle<'info> {
    /// Get the CPI context for set prices.
    fn set_prices_from_price_feed_ctx(
        &self,
        feeds: Vec<AccountInfo<'info>>,
    ) -> CpiContext<'_, '_, '_, 'info, SetPricesFromPriceFeed<'info>> {
        let check_role = self.check_role_ctx();
        CpiContext::new(
            check_role.program,
            SetPricesFromPriceFeed {
                authority: self.authority().to_account_info(),
                store: check_role.accounts.store,
                config: self.config(),
                token_config_map: self.token_config_map(),
                oracle: self.oracle(),
                price_provider: self.price_provider(),
            },
        )
        .with_remaining_accounts(feeds)
    }

    /// Get the CPI context for clear all prices.
    fn clear_all_prices_ctx(&self) -> CpiContext<'_, '_, '_, 'info, ClearAllPrices<'info>> {
        let check_role = self.check_role_ctx();
        CpiContext::new(
            check_role.program,
            ClearAllPrices {
                authority: self.authority().to_account_info(),
                store: check_role.accounts.store,
                oracle: self.oracle(),
            },
        )
    }

    /// Run the given function inside the scope with oracle prices.
    fn with_oracle_prices<T>(
        &mut self,
        tokens: Vec<Pubkey>,
        remaining_accounts: &'info [AccountInfo<'info>],
        f: impl FnOnce(&mut Self, &'info [AccountInfo<'info>]) -> Result<T>,
    ) -> Result<T> {
        require_gte!(
            remaining_accounts.len(),
            tokens.len(),
            ErrorCode::AccountNotEnoughKeys
        );
        let feeds = remaining_accounts[..tokens.len()].to_vec();
        let remaining_accounts = &remaining_accounts[tokens.len()..];
        crate::cpi::set_prices_from_price_feed(self.set_prices_from_price_feed_ctx(feeds), tokens)?;
        let output = f(self, remaining_accounts)?;
        crate::cpi::clear_all_prices(self.clear_all_prices_ctx())?;
        Ok(output)
    }
}

impl<'info, T> WithOracleExt<'info> for T where T: WithOracle<'info> {}
