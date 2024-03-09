use anchor_lang::prelude::*;
use data_store::states::DataStore;
use gmx_solana_utils::to_seed;

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
