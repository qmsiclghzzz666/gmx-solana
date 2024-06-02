use anchor_lang::prelude::*;
use gmx_core::{Pool as GmxCorePool, PoolKind};

use crate::{
    states::{Action, DataStore, Market, MarketChangeEvent, MarketMeta, Pool, Roles, Seed},
    utils::internal,
    DataStoreError,
};

/// Number of pools.
pub const NUM_POOLS: u8 = 13;

/// Number of clocks.
pub const NUM_CLOCKS: u8 = 3;

/// Initialize the account for [`Market`].
pub fn initialize_market(
    ctx: Context<InitializeMarket>,
    market_token_mint: Pubkey,
    index_token_mint: Pubkey,
    long_token_mint: Pubkey,
    short_token_mint: Pubkey,
) -> Result<()> {
    let market = &mut ctx.accounts.market;
    market.init(
        ctx.bumps.market,
        ctx.accounts.store.key(),
        market_token_mint,
        index_token_mint,
        long_token_mint,
        short_token_mint,
        NUM_POOLS,
        NUM_CLOCKS,
    )?;
    emit!(MarketChangeEvent {
        address: market.key(),
        action: Action::Init,
        market: (**market).clone(),
    });
    Ok(())
}

#[derive(Accounts)]
#[instruction(market_token: Pubkey)]
pub struct InitializeMarket<'info> {
    #[account(mut)]
    authority: Signer<'info>,
    only_market_keeper: Account<'info, Roles>,
    store: Account<'info, DataStore>,
    #[account(
        init,
        payer = authority,
        space = 8 + Market::init_space(NUM_POOLS, NUM_CLOCKS),
        seeds = [
            Market::SEED,
            store.key().as_ref(),
            market_token.as_ref(),
        ],
        bump,
    )]
    market: Account<'info, Market>,
    system_program: Program<'info, System>,
}

impl<'info> internal::Authentication<'info> for InitializeMarket<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &Account<'info, DataStore> {
        &self.store
    }

    fn roles(&self) -> &Account<'info, Roles> {
        &self.only_market_keeper
    }
}

/// Remove market.
pub fn remove_market(ctx: Context<RemoveMarket>) -> Result<()> {
    let market = &mut ctx.accounts.market;
    emit!(MarketChangeEvent {
        address: market.key(),
        action: Action::Remove,
        market: (**market).clone(),
    });
    Ok(())
}

#[derive(Accounts)]
pub struct RemoveMarket<'info> {
    #[account(mut)]
    authority: Signer<'info>,
    only_market_keeper: Account<'info, Roles>,
    store: Account<'info, DataStore>,
    #[account(
        mut,
        has_one = store,
        seeds = [Market::SEED, store.key().as_ref(), market.meta.market_token_mint.as_ref()],
        bump = market.bump,
        close = authority,
    )]
    market: Account<'info, Market>,
}

impl<'info> internal::Authentication<'info> for RemoveMarket<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &Account<'info, DataStore> {
        &self.store
    }

    fn roles(&self) -> &Account<'info, Roles> {
        &self.only_market_keeper
    }
}

/// Apply delta to market pool.
pub fn apply_delta_to_market_pool(
    ctx: Context<ApplyDeltaToMarketPool>,
    pool: PoolKind,
    is_long_token: bool,
    delta: i128,
) -> Result<()> {
    let market = &mut ctx.accounts.market;
    market
        .with_pool_mut(pool, |pool| {
            if is_long_token {
                pool.apply_delta_to_long_amount(&delta)
                    .map_err(|_| DataStoreError::Computation)?;
            } else {
                pool.apply_delta_to_short_amount(&delta)
                    .map_err(|_| DataStoreError::Computation)?;
            }
            Result::Ok(())
        })
        .ok_or(DataStoreError::UnsupportedPoolKind)??;
    Ok(())
}

#[derive(Accounts)]
pub struct ApplyDeltaToMarketPool<'info> {
    pub authority: Signer<'info>,
    pub store: Account<'info, DataStore>,
    pub only_controller: Account<'info, Roles>,
    #[account(
        mut,
        has_one = store,
        seeds = [
            Market::SEED,
            store.key().as_ref(),
            market.meta.market_token_mint.as_ref(),
        ],
        bump = market.bump,
    )]
    pub(crate) market: Account<'info, Market>,
}

impl<'info> internal::Authentication<'info> for ApplyDeltaToMarketPool<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &Account<'info, DataStore> {
        &self.store
    }

    fn roles(&self) -> &Account<'info, Roles> {
        &self.only_controller
    }
}

/// Get the given pool info of the market.
pub fn get_pool(ctx: Context<GetPool>, pool: PoolKind) -> Result<Option<Pool>> {
    Ok(ctx.accounts.market.pool(pool))
}

#[derive(Accounts)]
pub struct GetPool<'info> {
    pub(crate) market: Account<'info, Market>,
}

/// Get the market token mint of the market.
pub fn get_market_token_mint(ctx: Context<GetMarketTokenMint>) -> Result<Pubkey> {
    Ok(ctx.accounts.market.meta.market_token_mint)
}

#[derive(Accounts)]
pub struct GetMarketTokenMint<'info> {
    pub(crate) market: Account<'info, Market>,
}

/// Get the meta of the market.
pub fn get_market_meta(ctx: Context<GetMarketMeta>) -> Result<MarketMeta> {
    Ok(ctx.accounts.market.meta.clone())
}

#[derive(Accounts)]
pub struct GetMarketMeta<'info> {
    pub(crate) market: Account<'info, Market>,
}
