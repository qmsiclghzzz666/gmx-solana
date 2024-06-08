use anchor_lang::prelude::*;
use anchor_spl::token::Mint;

use crate::{
    states::{PriceProviderKind, Seed, Store, TokenConfig, TokenConfigBuilder, TokenConfigMap},
    utils::internal,
};

#[derive(Accounts)]
#[instruction(len: u16)]
pub struct InitializeTokenConfigMap<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    pub store: AccountLoader<'info, Store>,
    #[account(
        init,
        space = 8 + TokenConfigMap::init_space(len as usize),
        payer = authority,
        seeds = [TokenConfigMap::SEED, store.key().as_ref()],
        bump,
    )]
    pub map: Account<'info, TokenConfigMap>,
    pub system_program: Program<'info, System>,
}

/// Initialize token config map with the given length.
pub fn initialize_token_config_map(
    ctx: Context<InitializeTokenConfigMap>,
    _len: u16,
) -> Result<()> {
    ctx.accounts
        .map
        .init(ctx.bumps.map, ctx.accounts.store.key());
    Ok(())
}

impl<'info> internal::Authentication<'info> for InitializeTokenConfigMap<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &AccountLoader<'info, Store> {
        &self.store
    }
}

#[derive(Accounts)]
pub struct InsertTokenConfig<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    pub store: AccountLoader<'info, Store>,
    #[account(
        mut,
        has_one = store,
        seeds = [TokenConfigMap::SEED, store.key().as_ref()],
        bump = map.bump,
        realloc = 8 + TokenConfigMap::init_space(map.length_after_insert(&token.key())),
        realloc::zero = false,
        realloc::payer = authority,
    )]
    pub map: Account<'info, TokenConfigMap>,
    pub token: Account<'info, Mint>,
    pub system_program: Program<'info, System>,
}

/// Insert or update the config of the given token.
pub fn insert_token_config(
    ctx: Context<InsertTokenConfig>,
    builder: TokenConfigBuilder,
    enable: bool,
) -> Result<()> {
    let token = &ctx.accounts.token;
    ctx.accounts.map.checked_insert(
        token.key(),
        TokenConfig::new(false, token.decimals, builder, enable),
    )
}

impl<'info> internal::Authentication<'info> for InsertTokenConfig<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &AccountLoader<'info, Store> {
        &self.store
    }
}

#[derive(Accounts)]
#[instruction(token: Pubkey)]
pub struct InsertSyntheticTokenConfig<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    pub store: AccountLoader<'info, Store>,
    #[account(
        mut,
        has_one = store,
        seeds = [TokenConfigMap::SEED, store.key().as_ref()],
        bump = map.bump,
        realloc = 8 + TokenConfigMap::init_space(map.length_after_insert(&token)),
        realloc::zero = false,
        realloc::payer = authority,
    )]
    pub map: Account<'info, TokenConfigMap>,
    pub system_program: Program<'info, System>,
}

/// Insert or update the config of the given synthetic token.
pub fn insert_synthetic_token_config(
    ctx: Context<InsertSyntheticTokenConfig>,
    token: Pubkey,
    decimals: u8,
    builder: TokenConfigBuilder,
    enable: bool,
) -> Result<()> {
    ctx.accounts.map.checked_insert(
        token.key(),
        TokenConfig::new(true, decimals, builder, enable),
    )
}

impl<'info> internal::Authentication<'info> for InsertSyntheticTokenConfig<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &AccountLoader<'info, Store> {
        &self.store
    }
}

#[derive(Accounts)]
pub struct ToggleTokenConfig<'info> {
    pub authority: Signer<'info>,
    pub store: AccountLoader<'info, Store>,
    #[account(
        mut,
        has_one = store,
        seeds = [TokenConfigMap::SEED, store.key().as_ref()],
        bump = map.bump,
    )]
    pub map: Account<'info, TokenConfigMap>,
}

/// Toggle the config for the given token.
pub fn toggle_token_config(
    ctx: Context<ToggleTokenConfig>,
    token: Pubkey,
    enable: bool,
) -> Result<()> {
    ctx.accounts.map.toggle_token_config(&token, enable)
}

impl<'info> internal::Authentication<'info> for ToggleTokenConfig<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &AccountLoader<'info, Store> {
        &self.store
    }
}

#[derive(Accounts)]
pub struct SetExpectedProvider<'info> {
    pub authority: Signer<'info>,
    pub store: AccountLoader<'info, Store>,
    #[account(
        mut,
        has_one = store,
        seeds = [TokenConfigMap::SEED, store.key().as_ref()],
        bump = map.bump,
    )]
    pub map: Account<'info, TokenConfigMap>,
}

/// Set the expected provider for the given token.
pub fn set_expected_provider(
    ctx: Context<SetExpectedProvider>,
    token: Pubkey,
    provider: PriceProviderKind,
) -> Result<()> {
    ctx.accounts.map.set_expected_provider(&token, provider)
}

impl<'info> internal::Authentication<'info> for SetExpectedProvider<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &AccountLoader<'info, Store> {
        &self.store
    }
}

#[derive(Accounts)]
#[instruction(store: Pubkey)]
pub struct GetTokenConfig<'info> {
    #[account(
        has_one = store,
        seeds = [TokenConfigMap::SEED, store.as_ref()],
        bump = map.bump,
    )]
    pub map: Account<'info, TokenConfigMap>,
}

/// Get token config of the given token.
pub fn get_token_config(
    ctx: Context<GetTokenConfig>,
    _store: Pubkey,
    token: Pubkey,
) -> Result<Option<TokenConfig>> {
    Ok(ctx.accounts.map.as_map().get(&token).cloned())
}

#[derive(Accounts)]
#[instruction(len: u16)]
pub struct ExtendTokenConfigMap<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    pub store: AccountLoader<'info, Store>,
    #[account(
        mut,
        has_one = store,
        seeds = [TokenConfigMap::SEED, store.key().as_ref()],
        bump = map.bump,
        realloc = 8 + TokenConfigMap::init_space(map.as_map().len() + len as usize),
        realloc::payer = authority,
        realloc::zero = false,
    )]
    pub map: Account<'info, TokenConfigMap>,
    pub system_program: Program<'info, System>,
}

/// Extend the length of the map with the given length.
pub fn extend_token_config_map(_ctx: Context<ExtendTokenConfigMap>, _len: u16) -> Result<()> {
    Ok(())
}

impl<'info> internal::Authentication<'info> for ExtendTokenConfigMap<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &AccountLoader<'info, Store> {
        &self.store
    }
}

#[derive(Accounts)]
pub struct InsertTokenConfigAmount<'info> {
    pub authority: Signer<'info>,
    pub store: AccountLoader<'info, Store>,
    #[account(
        mut,
        has_one = store,
        seeds = [TokenConfigMap::SEED, store.key().as_ref()],
        bump = map.bump,
    )]
    pub map: Account<'info, TokenConfigMap>,
}

/// Insert amount of the given key for the token.
pub fn insert_token_config_amount(
    ctx: Context<InsertTokenConfigAmount>,
    token: &Pubkey,
    key: &str,
    amount: u64,
) -> Result<()> {
    ctx.accounts.map.insert_amount(token, key, amount)
}

impl<'info> internal::Authentication<'info> for InsertTokenConfigAmount<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &AccountLoader<'info, Store> {
        &self.store
    }
}
