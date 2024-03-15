use anchor_lang::prelude::*;
use data_store::{
    cpi::accounts::CheckRole,
    states::{DataStore, Roles},
    utils::Authentication,
};
use gmx_solana_utils::to_seed;

use crate::{states::Oracle, OracleError};

#[derive(Accounts)]
#[instruction(key: String)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    pub store: Account<'info, DataStore>,
    #[account(
        init,
        payer = authority,
        space = 8 + Oracle::INIT_SPACE,
        seeds = [Oracle::SEED, store.key().as_ref(), &to_seed(&key)],
        bump,
    )]
    pub oracle: Account<'info, Oracle>,
    pub system_program: Program<'info, System>,
}

pub fn initialize(ctx: Context<Initialize>, key: String) -> Result<()> {
    // FIXME: Is it still correct if we not clear here?
    ctx.accounts.oracle.primary.clear();
    ctx.accounts.oracle.bump = ctx.bumps.oracle;
    ctx.accounts.oracle.data_store = ctx.accounts.store.key();
    msg!("new oracle initialized with key: {}", key);
    Ok(())
}

#[derive(Accounts)]
pub struct ClearAllPrices<'info> {
    pub authority: Signer<'info>,
    pub store: Account<'info, DataStore>,
    pub only_controller: Account<'info, Roles>,
    #[account(mut)]
    pub oracle: Account<'info, Oracle>,
    pub data_store_program: Program<'info, data_store::program::DataStore>,
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

pub fn clear_all_prices(ctx: Context<ClearAllPrices>) -> Result<()> {
    ctx.accounts.oracle.primary.clear();
    Ok(())
}
