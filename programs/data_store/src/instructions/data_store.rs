use anchor_lang::prelude::*;
use gmx_solana_utils::to_seed;

use crate::states::{Store, DataStoreInitEvent, Roles, Seed};

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
    pub data_store: Account<'info, Store>,
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
    let roles = &mut ctx.accounts.roles;
    roles.init(
        ctx.accounts.authority.key(),
        ctx.accounts.data_store.key(),
        ctx.bumps.roles,
    );
    let data_store = &mut ctx.accounts.data_store;
    data_store.init(roles, &key, ctx.bumps.data_store)?;
    emit!(DataStoreInitEvent {
        key,
        address: ctx.accounts.data_store.key(),
    });
    Ok(())
}
