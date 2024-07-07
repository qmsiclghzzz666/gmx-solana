use anchor_lang::prelude::*;
use gmsol_model::PoolKind;

use crate::{
    states::{Market, Store},
    utils::internal,
    StoreError,
};

/// Convert non-pure pool to a pure pool.
#[derive(Accounts)]
pub struct TurnIntoPurePool<'info> {
    authority: Signer<'info>,
    store: AccountLoader<'info, Store>,
    #[account(mut, has_one = store)]
    market: AccountLoader<'info, Market>,
}

/// Convert non-pure pool to a pure pool.
///
/// ## CHECK
/// - Only MARKET_KEEPER can change the pure flag of a pool.
pub fn unchecked_turn_into_pure_pool(ctx: Context<TurnIntoPurePool>, kind: PoolKind) -> Result<()> {
    let mut market = ctx.accounts.market.load_mut()?;
    let mint = market.meta.market_token_mint;
    let pool = market
        .pool_mut(kind)
        .ok_or(error!(StoreError::RequiredResourceNotFound))?;
    require!(!pool.is_pure(), StoreError::InvalidArgument);
    msg!("{}: turning pool `{}` to pure", mint, kind);
    pool.set_is_pure(true);
    pool.merge_if_pure()?;
    Ok(())
}

impl<'info> internal::Authentication<'info> for TurnIntoPurePool<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &AccountLoader<'info, Store> {
        &self.store
    }
}
