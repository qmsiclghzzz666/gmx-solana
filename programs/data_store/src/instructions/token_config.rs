use anchor_lang::prelude::*;
use anchor_spl::token::Mint;
use gmx_solana_utils::to_seed;

use crate::{
    states::{
        DataStore, Roles, Seed, TokenConfig, TokenConfig2, TokenConfigChangeEvent, TokenConfigMap,
    },
    utils::internal,
};

#[derive(Accounts)]
#[instruction(key: String)]
pub struct InitializeTokenConfig<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    pub store: Account<'info, DataStore>,
    pub only_controller: Account<'info, Roles>,
    #[account(
        init,
        payer = authority,
        space = 8 + TokenConfig::INIT_SPACE,
        seeds = [TokenConfig::SEED, store.key().as_ref(), &to_seed(&key)],
        bump,
    )]
    pub token_config: Account<'info, TokenConfig>,
    pub system_program: Program<'info, System>,
}

pub fn initialize_token_config(
    ctx: Context<InitializeTokenConfig>,
    key: String,
    price_feed: Pubkey,
    heartbeat_duration: u32,
    token_decimals: u8,
    precision: u8,
) -> Result<()> {
    ctx.accounts.token_config.init(
        ctx.bumps.token_config,
        price_feed,
        heartbeat_duration,
        token_decimals,
        precision,
    );
    emit!(TokenConfigChangeEvent {
        key,
        address: ctx.accounts.token_config.key(),
        init: true,
        config: (*ctx.accounts.token_config).clone(),
    });
    Ok(())
}

impl<'info> internal::Authentication<'info> for InitializeTokenConfig<'info> {
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
#[instruction(key: String)]
pub struct UpdateTokenConfig<'info> {
    pub authority: Signer<'info>,
    pub store: Account<'info, DataStore>,
    pub only_controller: Account<'info, Roles>,
    #[account(
        mut,
        seeds = [TokenConfig::SEED, store.key().as_ref(), &to_seed(&key)],
        bump = token_config.bump,
    )]
    pub token_config: Account<'info, TokenConfig>,
}

impl<'info> internal::Authentication<'info> for UpdateTokenConfig<'info> {
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

pub fn update_token_config(
    ctx: Context<UpdateTokenConfig>,
    key: String,
    price_feed: Option<Pubkey>,
    token_decimals: Option<u8>,
    precision: Option<u8>,
) -> Result<()> {
    ctx.accounts
        .token_config
        .update(price_feed, token_decimals, precision);
    emit!(TokenConfigChangeEvent {
        key,
        address: ctx.accounts.token_config.key(),
        init: false,
        config: (*ctx.accounts.token_config).clone(),
    });
    Ok(())
}

#[derive(Accounts)]
#[instruction(len: u16)]
pub struct InitializeTokenConfigMap<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    pub only_controller: Account<'info, Roles>,
    pub store: Account<'info, DataStore>,
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
    ctx.accounts.map.init(ctx.bumps.map);
    Ok(())
}

impl<'info> internal::Authentication<'info> for InitializeTokenConfigMap<'info> {
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
pub struct InsertTokenConfig<'info> {
    pub authority: Signer<'info>,
    pub only_controller: Account<'info, Roles>,
    pub store: Account<'info, DataStore>,
    #[account(
        mut,
        seeds = [TokenConfigMap::SEED, store.key().as_ref()],
        bump = map.bump,
    )]
    pub map: Account<'info, TokenConfigMap>,
    pub token: Account<'info, Mint>,
}

/// Insert or update the config of the given token.
pub fn insert_token_config(
    ctx: Context<InsertTokenConfig>,
    price_feed: Pubkey,
    heartbeat_duration: u32,
    precision: u8,
) -> Result<()> {
    let token = &ctx.accounts.token;
    ctx.accounts.map.as_map_mut().insert(
        token.key(),
        TokenConfig2 {
            price_feed,
            heartbeat_duration,
            precision,
            token_decimals: token.decimals,
        },
    );
    Ok(())
}

impl<'info> internal::Authentication<'info> for InsertTokenConfig<'info> {
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
#[instruction(store: Pubkey)]
pub struct GetTokenConfig<'info> {
    #[account(
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
) -> Result<Option<TokenConfig2>> {
    Ok(ctx.accounts.map.as_map().get(&token).cloned())
}

#[derive(Accounts)]
#[instruction(len: u16)]
pub struct ExtendTokenConfigMap<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    pub only_controller: Account<'info, Roles>,
    pub store: Account<'info, DataStore>,
    #[account(
        mut,
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

    fn store(&self) -> &Account<'info, DataStore> {
        &self.store
    }

    fn roles(&self) -> &Account<'info, Roles> {
        &self.only_controller
    }
}
