use anchor_lang::prelude::*;
use gmx_solana_utils::to_seed;
use role_store::RoleStore;

use crate::states::{DataStore, DataStoreInitEvent, Roles, Seed};

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
    #[account(
        init,
        payer = authority,
        space = 8 + Roles::INIT_SPACE,
        seeds = [Roles::SEED, data_store.key().as_ref(), authority.key().as_ref()],
        bump,
    )]
    pub roles: Account<'info, Roles>,
    pub system_program: Program<'info, System>,
}

pub fn initialize(ctx: Context<Initialize>, key: String) -> Result<()> {
    let data_store = &mut ctx.accounts.data_store;
    data_store.init(
        &mut ctx.accounts.roles,
        ctx.bumps.roles,
        ctx.accounts.role_store.key(),
        &key,
        ctx.bumps.data_store,
    )?;
    emit!(DataStoreInitEvent {
        key,
        address: ctx.accounts.data_store.key(),
        role_store: ctx.accounts.role_store.key(),
    });
    Ok(())
}
