use anchor_lang::prelude::*;
use data_store::utils::Authentication;

use crate::cpi::{
    self,
    accounts::{ClearAllPrices, SetPricesFromPriceFeed},
};

/// Accounts that with oracle context.
pub trait WithOracle<'info>: Authentication<'info> {
    /// Get the oracle program.
    fn oracle_program(&self) -> AccountInfo<'info>;

    /// Get the chainlink program.
    fn chainlink_program(&self) -> AccountInfo<'info>;

    /// Get the oracle account.
    fn oracle(&self) -> AccountInfo<'info>;

    /// Get the token config map account.
    fn token_config_map(&self) -> AccountInfo<'info>;
}

/// Extension trait for [`WithOracle`].
pub trait WithOracleExt<'info>: WithOracle<'info> {
    /// Get the CPI context for set prices.
    fn set_prices_from_price_feed_ctx(
        &self,
        remaining_accounts: Vec<AccountInfo<'info>>,
    ) -> CpiContext<'_, '_, '_, 'info, SetPricesFromPriceFeed<'info>> {
        let check_role = self.check_role_ctx();
        CpiContext::new(
            self.oracle_program(),
            SetPricesFromPriceFeed {
                authority: self.authority().to_account_info(),
                only_controller: check_role.accounts.roles,
                store: check_role.accounts.store,
                token_config_map: self.token_config_map(),
                oracle: self.oracle(),
                chainlink_program: self.chainlink_program(),
                data_store_program: check_role.program,
            },
        )
        .with_remaining_accounts(remaining_accounts)
    }

    /// Get the CPI context for clear all prices.
    fn clear_all_prices_ctx(&self) -> CpiContext<'_, '_, '_, 'info, ClearAllPrices<'info>> {
        let check_role = self.check_role_ctx();
        CpiContext::new(
            self.oracle_program(),
            ClearAllPrices {
                authority: self.authority().to_account_info(),
                only_controller: check_role.accounts.roles,
                store: check_role.accounts.store,
                oracle: self.oracle(),
                data_store_program: check_role.program,
            },
        )
    }

    /// Run the given function inside the scope with oracle prices.
    fn with_oracle_prices<T>(
        &mut self,
        tokens: Vec<Pubkey>,
        remaining_accounts: Vec<AccountInfo<'info>>,
        f: impl FnOnce(&mut Self) -> Result<T>,
    ) -> Result<T> {
        cpi::set_prices_from_price_feed(
            self.set_prices_from_price_feed_ctx(remaining_accounts),
            tokens,
        )?;
        let output = f(self)?;
        cpi::clear_all_prices(self.clear_all_prices_ctx())?;
        Ok(output)
    }
}

impl<'info, T> WithOracleExt<'info> for T where T: WithOracle<'info> {}
