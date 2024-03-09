use anchor_lang::prelude::*;
use gmx_solana_utils::to_seed;
use role_store::RoleStore;

use crate::states::{DataStore, DataStoreInitEvent};

#[derive(Accounts)]
#[instruction(key: String)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    pub role_store: Account<'info, RoleStore>,
    #[account(
        init,
        payer = authority,
        space = 8 + DataStore::INIT_SPACE,
        seeds = [DataStore::SEED, &role_store.key().to_bytes(), &to_seed(&key)],
        bump,
    )]
    pub data_store: Account<'info, DataStore>,
    pub system_program: Program<'info, System>,
}

pub fn initialize(ctx: Context<Initialize>, key: String) -> Result<()> {
    ctx.accounts
        .data_store
        .init(ctx.accounts.role_store.key(), &key, ctx.bumps.data_store);
    emit!(DataStoreInitEvent {
        key,
        address: ctx.accounts.data_store.key(),
        role_store: ctx.accounts.role_store.key(),
    });
    Ok(())
}
