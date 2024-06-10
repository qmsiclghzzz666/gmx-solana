use anchor_lang::prelude::*;
use gmx_solana_utils::to_seed;

use crate::{
    states::{DataStoreInitEvent, InitSpace, Seed, Store, TokenMapHeader},
    utils::internal,
};

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

#[derive(Accounts)]
pub struct SetTokenMap<'info> {
    pub authority: Signer<'info>,
    #[account(mut)]
    pub store: AccountLoader<'info, Store>,
    #[account(has_one = store)]
    pub token_map: AccountLoader<'info, TokenMapHeader>,
}

/// Set token map.
///
/// ## Check
/// - Only MARKET_KEEPER can perform this action.
pub fn unchecked_set_token_map(ctx: Context<SetTokenMap>) -> Result<()> {
    ctx.accounts.store.load_mut()?.token_map = ctx.accounts.token_map.key();
    Ok(())
}

impl<'info> internal::Authentication<'info> for SetTokenMap<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &AccountLoader<'info, Store> {
        &self.store
    }
}

#[derive(Accounts)]
pub struct ReadStore<'info> {
    pub store: AccountLoader<'info, Store>,
}

/// Get the token map address of the store.
pub fn get_token_map(ctx: Context<ReadStore>) -> Result<Option<Pubkey>> {
    Ok(ctx.accounts.store.load()?.token_map().copied())
}
