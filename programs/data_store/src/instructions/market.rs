use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, MintTo, Token, TokenAccount, Transfer};
use gmx_core::PoolKind;

use crate::{
    constants,
    states::{Action, DataStore, Market, MarketChangeEvent, MarketMeta, Pool, Pools, Roles, Seed},
    utils::internal,
    DataStoreError,
};

/// Number of pools.
pub const NUM_POOLS: u8 = 3;

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
        market_token_mint,
        index_token_mint,
        long_token_mint,
        short_token_mint,
        NUM_POOLS,
    );
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
        space = 8 + MarketMeta::INIT_SPACE + Pools::init_space(NUM_POOLS),
        seeds = [
            Market::SEED,
            store.key().as_ref(),
            &Market::create_key_seed(&market_token),
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
        seeds = [Market::SEED, store.key().as_ref(), &market.expected_key_seed()],
        bump = market.meta.bump,
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
                pool.apply_delta_to_long_token_amount(delta)?;
            } else {
                pool.apply_delta_to_short_token_amount(delta)?;
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
        seeds = [Market::SEED, store.key().as_ref(), &market.expected_key_seed()],
        bump = market.meta.bump,
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

/// Initialize a new market token.
#[allow(unused_variables)]
pub fn initialize_market_token(
    ctx: Context<InitializeMarketToken>,
    index_token_mint: Pubkey,
    long_token_mint: Pubkey,
    short_token_mint: Pubkey,
) -> Result<()> {
    Ok(())
}

#[derive(Accounts)]
#[instruction(index_token_mint: Pubkey, long_token_mint: Pubkey, short_token_mint: Pubkey)]
pub struct InitializeMarketToken<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    pub only_market_keeper: Account<'info, Roles>,
    pub store: Account<'info, DataStore>,
    #[account(
        init,
        payer = authority,
        mint::decimals = constants::MARKET_TOKEN_DECIMALS,
        mint::authority = market_sign,
        seeds = [
            constants::MAREKT_TOKEN_MINT_SEED,
            store.key().as_ref(),
            index_token_mint.as_ref(),
            long_token_mint.key().as_ref(),
            short_token_mint.key().as_ref(),
        ],
        bump,
    )]
    pub market_token_mint: Account<'info, Mint>,
    /// CHECK: only used as a signing PDA.
    #[account(seeds = [constants::MARKET_SIGN_SEED], bump)]
    pub market_sign: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

impl<'info> internal::Authentication<'info> for InitializeMarketToken<'info> {
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

/// Mint the given amount of market tokens to the destination account.
pub fn mint_market_token_to(ctx: Context<MintMarketTokenTo>, amount: u64) -> Result<()> {
    anchor_spl::token::mint_to(
        ctx.accounts
            .mint_to_ctx()
            .with_signer(&[&[constants::MARKET_SIGN_SEED, &[ctx.bumps.market_sign]]]),
        amount,
    )
}

#[derive(Accounts)]
pub struct MintMarketTokenTo<'info> {
    pub authority: Signer<'info>,
    pub only_controller: Account<'info, Roles>,
    pub store: Account<'info, DataStore>,
    // We don't have to check the mint is really a market token,
    // since the mint authority must be derived from `MARKET_SIGN`.
    #[account(mut)]
    pub market_token_mint: Account<'info, Mint>,
    #[account(mut, token::mint = market_token_mint)]
    pub to: Account<'info, TokenAccount>,
    /// CHECK: only used as a signing PDA.
    #[account(seeds = [constants::MARKET_SIGN_SEED], bump)]
    pub market_sign: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
}

impl<'info> internal::Authentication<'info> for MintMarketTokenTo<'info> {
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

impl<'info> MintMarketTokenTo<'info> {
    fn mint_to_ctx(&self) -> CpiContext<'_, '_, '_, 'info, MintTo<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            MintTo {
                mint: self.market_token_mint.to_account_info(),
                to: self.to.to_account_info(),
                authority: self.market_sign.to_account_info(),
            },
        )
    }
}

/// Initialize a vault of the given token for a market.
/// The address is derived from token mint addresses (the `market_token_mint` seed is optional).
#[allow(unused_variables)]
pub fn initialize_market_vault(
    ctx: Context<InitializeMarketVault>,
    market_token_mint: Option<Pubkey>,
) -> Result<()> {
    Ok(())
}

#[derive(Accounts)]
#[instruction(market_token_mint: Option<Pubkey>)]
pub struct InitializeMarketVault<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    pub only_market_keeper: Account<'info, Roles>,
    pub store: Account<'info, DataStore>,
    pub mint: Account<'info, Mint>,
    #[account(
        init,
        payer = authority,
        token::mint = mint,
        token::authority = market_sign,
        seeds = [
            constants::MARKET_VAULT_SEED,
            store.key().as_ref(),
            mint.key().as_ref(),
            market_token_mint.as_ref().map(|key| key.as_ref()).unwrap_or(&[]),
        ],
        bump,
    )]
    pub vault: Account<'info, TokenAccount>,
    /// CHECK: only used as a signing PDA.
    #[account(seeds = [constants::MARKET_SIGN_SEED], bump)]
    pub market_sign: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

impl<'info> internal::Authentication<'info> for InitializeMarketVault<'info> {
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

/// Transfer the given amount of tokens out to the destination account.
pub fn market_vault_transfer_out(ctx: Context<MarketVaultTransferOut>, amount: u64) -> Result<()> {
    anchor_spl::token::transfer(
        ctx.accounts
            .transfer_ctx()
            .with_signer(&[&[constants::MARKET_SIGN_SEED, &[ctx.bumps.market_sign]]]),
        amount,
    )
}

#[derive(Accounts)]
pub struct MarketVaultTransferOut<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    pub only_controller: Account<'info, Roles>,
    pub store: Account<'info, DataStore>,
    // We don't have to check the vault is really a market token,
    // since the owner must be derived from `MARKET_SIGN`.
    #[account(mut)]
    pub market_vault: Account<'info, TokenAccount>,
    #[account(mut, token::mint = market_vault.mint)]
    pub to: Account<'info, TokenAccount>,
    /// CHECK: only used as a signing PDA.
    #[account(seeds = [constants::MARKET_SIGN_SEED], bump)]
    pub market_sign: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
}

impl<'info> internal::Authentication<'info> for MarketVaultTransferOut<'info> {
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

impl<'info> MarketVaultTransferOut<'info> {
    fn transfer_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.to_account_info(),
            Transfer {
                from: self.market_vault.to_account_info(),
                to: self.to.to_account_info(),
                authority: self.market_sign.to_account_info(),
            },
        )
    }
}
