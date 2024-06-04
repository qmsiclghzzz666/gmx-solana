use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};
use gmx_core::{Pool as GmxCorePool, PoolKind};

use crate::{
    constants,
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

#[derive(Accounts)]
pub struct MarketTransferIn<'info> {
    pub authority: Signer<'info>,
    pub store: Account<'info, DataStore>,
    #[account(
        seeds = [Roles::SEED, store.key().as_ref(), authority.key().as_ref()],
        bump = only_controller.bump,
    )]
    pub only_controller: Account<'info, Roles>,
    pub from_authority: Signer<'info>,
    #[account(mut, has_one = store)]
    pub market: Account<'info, Market>,
    #[account(mut, token::mint = vault.mint, constraint = from.key() != vault.key())]
    pub from: Account<'info, TokenAccount>,
    #[account(
        mut,
        token::authority = store,
        seeds = [
            constants::MARKET_VAULT_SEED,
            store.key().as_ref(),
            vault.mint.as_ref(),
            &[],
        ],
        bump,
    )]
    pub vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

/// Transfer some tokens into the market.
pub fn market_transfer_in(ctx: Context<MarketTransferIn>, amount: u64) -> Result<()> {
    use anchor_spl::token;

    if amount != 0 {
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.from.to_account_info(),
                    to: ctx.accounts.vault.to_account_info(),
                    authority: ctx.accounts.from_authority.to_account_info(),
                },
            ),
            amount,
        )?;
        let token = &ctx.accounts.vault.mint;
        ctx.accounts
            .market
            .record_transferred_in_by_token(token, amount)?;
    }

    Ok(())
}

impl<'info> internal::Authentication<'info> for MarketTransferIn<'info> {
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

#[derive(Accounts)]
pub struct MarketTransferOut<'info> {
    pub authority: Signer<'info>,
    pub store: Account<'info, DataStore>,
    #[account(
        seeds = [Roles::SEED, store.key().as_ref(), authority.key().as_ref()],
        bump = only_controller.bump,
    )]
    pub only_controller: Account<'info, Roles>,
    #[account(mut, has_one = store)]
    pub market: Account<'info, Market>,
    #[account(mut, token::mint = vault.mint, constraint = to.key() != vault.key())]
    pub to: Account<'info, TokenAccount>,
    #[account(
        mut,
        token::authority = store,
        seeds = [
            constants::MARKET_VAULT_SEED,
            store.key().as_ref(),
            vault.mint.as_ref(),
            &[],
        ],
        bump,
    )]
    pub vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

/// Transfer some tokens out of the market.
pub fn market_transfer_out(ctx: Context<MarketTransferOut>, amount: u64) -> Result<()> {
    use crate::utils::internal::TransferUtils;

    if amount != 0 {
        TransferUtils::new(
            ctx.accounts.token_program.to_account_info(),
            &ctx.accounts.store,
            None,
        )
        .transfer_out(
            ctx.accounts.vault.to_account_info(),
            ctx.accounts.to.to_account_info(),
            amount,
        )?;
        let token = &ctx.accounts.vault.mint;
        ctx.accounts
            .market
            .record_transferred_out_by_token(token, amount)?;
    }

    Ok(())
}

impl<'info> internal::Authentication<'info> for MarketTransferOut<'info> {
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
