use anchor_lang::prelude::*;
use gmx_solana_utils::to_seed;

use crate::states::{DataStoreInitEvent, InitSpace, Seed, Store};

#[derive(Accounts)]
#[instruction(key: String)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init,
        payer = authority,
        space = 8 + Store::INIT_SPACE,
        seeds = [Store::SEED, &to_seed(&key)],
        bump,
    )]
    pub data_store: AccountLoader<'info, Store>,
    pub system_program: Program<'info, System>,
}

pub fn initialize(ctx: Context<Initialize>, key: String) -> Result<()> {
    let mut store = ctx.accounts.data_store.load_init()?;
    store.init(ctx.accounts.authority.key(), &key, ctx.bumps.data_store)?;
    emit!(DataStoreInitEvent {
        key,
        address: ctx.accounts.data_store.key(),
    });
    Ok(())
}
