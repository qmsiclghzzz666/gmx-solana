use anchor_lang::prelude::*;
use data_store::{
    cpi::accounts::CheckRole,
    states::{DataStore, Oracle, Roles},
    utils::Authentication,
};

use crate::OracleError;

/// Clear all prices of the oracle account.
pub fn clear_all_prices(ctx: Context<ClearAllPrices>) -> Result<()> {
    data_store::cpi::clear_all_prices(ctx.accounts.clear_all_prices_ctx())
}

#[derive(Accounts)]
pub struct ClearAllPrices<'info> {
    pub authority: Signer<'info>,
    pub only_controller: Account<'info, Roles>,
    pub store: Account<'info, DataStore>,
    #[account(mut)]
    pub oracle: Account<'info, Oracle>,
    pub data_store_program: Program<'info, data_store::program::DataStore>,
}

impl<'info> ClearAllPrices<'info> {
    fn clear_all_prices_ctx(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, data_store::cpi::accounts::ClearAllPrices<'info>> {
        CpiContext::new(
            self.data_store_program.to_account_info(),
            data_store::cpi::accounts::ClearAllPrices {
                authority: self.authority.to_account_info(),
                store: self.store.to_account_info(),
                only_controller: self.only_controller.to_account_info(),
                oracle: self.oracle.to_account_info(),
            },
        )
    }
}

impl<'info> Authentication<'info> for ClearAllPrices<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn check_role_ctx(&self) -> CpiContext<'_, '_, '_, 'info, CheckRole<'info>> {
        CpiContext::new(
            self.data_store_program.to_account_info(),
            CheckRole {
                store: self.store.to_account_info(),
                roles: self.only_controller.to_account_info(),
            },
        )
    }

    fn on_error(&self) -> Result<()> {
        Err(OracleError::PermissionDenied.into())
    }
}
