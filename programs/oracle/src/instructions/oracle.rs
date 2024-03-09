use anchor_lang::prelude::*;
use data_store::states::DataStore;
use gmx_solana_utils::to_seed;
use role_store::{Authorization, Role};

use crate::states::Oracle;

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
    ctx.accounts.oracle.role_store = *ctx.accounts.store.role_store();
    ctx.accounts.oracle.data_store = ctx.accounts.store.key();
    msg!("new oracle initialized with key: {}", key);
    Ok(())
}

#[derive(Accounts)]
pub struct ClearAllPrices<'info> {
    pub authority: Signer<'info>,
    pub only_controller: Account<'info, Role>,
    #[account(mut)]
    pub oracle: Account<'info, Oracle>,
}

impl<'info> Authorization<'info> for ClearAllPrices<'info> {
    fn role_store(&self) -> Pubkey {
        self.oracle.role_store
    }

    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn role(&self) -> &Account<'info, Role> {
        &self.only_controller
    }
}

pub fn clear_all_prices(ctx: Context<ClearAllPrices>) -> Result<()> {
    ctx.accounts.oracle.primary.clear();
    Ok(())
}
