use anchor_lang::prelude::*;
use gmx_solana_utils::to_seed;

use crate::{
    states::{DataStore, Roles, Seed, TokenConfig, TokenConfigChangeEvent},
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
