use anchor_lang::prelude::*;
use gmx_solana_utils::to_seed;
use role_store::{Authorization, Role};

use crate::states::{DataStore, TokenConfig};

#[derive(Accounts)]
#[instruction(key: String)]
pub struct InitializeTokenConfig<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    pub store: Account<'info, DataStore>,
    pub only_controller: Account<'info, Role>,
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
    _key: String,
    price_feed: Pubkey,
    token_decimals: u8,
    precision: u8,
) -> Result<()> {
    ctx.accounts.token_config.init(
        ctx.bumps.token_config,
        price_feed,
        token_decimals,
        precision,
    );
    Ok(())
}

impl<'info> Authorization<'info> for InitializeTokenConfig<'info> {
    fn role_store(&self) -> Pubkey {
        self.store.role_store
    }

    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn role(&self) -> &Account<'info, Role> {
        &self.only_controller
    }
}

#[derive(Accounts)]
#[instruction(key: String)]
pub struct UpdateTokenConfig<'info> {
    pub authority: Signer<'info>,
    pub store: Account<'info, DataStore>,
    pub only_controller: Account<'info, Role>,
    #[account(
        mut,
        seeds = [TokenConfig::SEED, store.key().as_ref(), &to_seed(&key)],
        bump = token_config.bump,
    )]
    pub token_config: Account<'info, TokenConfig>,
}

impl<'info> Authorization<'info> for UpdateTokenConfig<'info> {
    fn role_store(&self) -> Pubkey {
        self.store.role_store
    }

    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn role(&self) -> &Account<'info, Role> {
        &self.only_controller
    }
}

pub fn update_token_config(
    ctx: Context<UpdateTokenConfig>,
    _key: String,
    price_feed: Option<Pubkey>,
    token_decimals: Option<u8>,
    precision: Option<u8>,
) -> Result<()> {
    ctx.accounts
        .token_config
        .update(price_feed, token_decimals, precision);
    Ok(())
}
