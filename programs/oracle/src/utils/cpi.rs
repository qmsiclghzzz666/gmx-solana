use anchor_lang::prelude::*;
use data_store::{
    cpi::accounts::{ClearAllPrices, SetPricesFromPriceFeed},
    utils::Authentication,
};

/// Accounts that with oracle context.
pub trait WithOracle<'info>: Authentication<'info> {
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
        feeds: Vec<AccountInfo<'info>>,
    ) -> CpiContext<'_, '_, '_, 'info, SetPricesFromPriceFeed<'info>> {
        let check_role = self.check_role_ctx();
        CpiContext::new(
            check_role.program,
            SetPricesFromPriceFeed {
                authority: self.authority().to_account_info(),
                only_controller: check_role.accounts.roles,
                store: check_role.accounts.store,
                token_config_map: self.token_config_map(),
                oracle: self.oracle(),
                chainlink_program: self.chainlink_program(),
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
                only_controller: check_role.accounts.roles,
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
        data_store::cpi::set_prices_from_price_feed(
            self.set_prices_from_price_feed_ctx(feeds),
            tokens,
        )?;
        let output = f(self, remaining_accounts)?;
        data_store::cpi::clear_all_prices(self.clear_all_prices_ctx())?;
        Ok(output)
    }
}

impl<'info, T> WithOracleExt<'info> for T where T: WithOracle<'info> {}
