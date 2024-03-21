use anchor_lang::prelude::*;
use data_store::{cpi::accounts::CheckRole, program::DataStore, utils::Authentication};

use crate::ExchangeError;

/// Execute a deposit.
pub fn execute_deposit(_ctx: Context<ExecuteDeposit>) -> Result<()> {
    Ok(())
}

#[derive(Accounts)]
pub struct ExecuteDeposit<'info> {
    pub authority: Signer<'info>,
    /// CHECK: used and checked by CPI.
    pub only_order_keeper: UncheckedAccount<'info>,
    /// CHECK: used and checked by CPI.
    pub store: UncheckedAccount<'info>,
    pub data_store_program: Program<'info, DataStore>,
    /// CHECK: used and checked by CPI.
    #[account(mut)]
    pub deposit: UncheckedAccount<'info>,
}

impl<'info> Authentication<'info> for ExecuteDeposit<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn check_role_ctx(&self) -> CpiContext<'_, '_, '_, 'info, CheckRole<'info>> {
        todo!()
    }

    fn on_error(&self) -> Result<()> {
        Err(error!(ExchangeError::PermissionDenied))
    }
}
