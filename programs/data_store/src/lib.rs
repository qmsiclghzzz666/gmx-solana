use anchor_lang::prelude::*;
use gmx_solana_utils::to_seed;
use role_store::{Authenticate, RoleStore};

/// Defined keys used in data store.
pub mod keys;

/// Instructions.
pub mod instructions;

/// States.
pub mod states;

use self::instructions::*;

declare_id!("8hJ2dGQ2Ccr5G6iEqQQEoBApRSXt7Jn8Qyf9Qf3eLBX2");

#[program]
pub mod data_store {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, key: String) -> Result<()> {
        ctx.accounts
            .data_store
            .init(ctx.accounts.role_store.key(), &key, ctx.bumps.data_store);
        Ok(())
    }

    #[access_control(Authenticate::only_controller(&ctx))]
    pub fn initialize_token_config(
        ctx: Context<InitializeTokenConfig>,
        key: String,
        price_feed: Pubkey,
        token_decimals: u8,
        precision: u8,
    ) -> Result<()> {
        instructions::initialize_token_config(ctx, key, price_feed, token_decimals, precision)
    }

    #[access_control(Authenticate::only_controller(&ctx))]
    pub fn update_token_config(
        ctx: Context<UpdateTokenConfig>,
        key: String,
        price_feed: Option<Pubkey>,
        token_decimals: Option<u8>,
        precision: Option<u8>,
    ) -> Result<()> {
        instructions::update_token_config(ctx, key, price_feed, token_decimals, precision)
    }
}

#[derive(Accounts)]
#[instruction(key: String)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    pub role_store: Account<'info, RoleStore>,
    #[account(
        init,
        payer = authority,
        space = 8 + DataStore::INIT_SPACE,
        seeds = [DataStore::SEED, &role_store.key().to_bytes(), &to_seed(&key)],
        bump,
    )]
    pub data_store: Account<'info, DataStore>,
    pub system_program: Program<'info, System>,
}

#[account]
#[derive(InitSpace)]
pub struct DataStore {
    role_store: Pubkey,
    #[max_len(32)]
    key: Vec<u8>,
    bump: u8,
}

impl DataStore {
    /// Seed.
    pub const SEED: &'static [u8] = b"data_store";

    fn init(&mut self, role_store: Pubkey, key: &str, bump: u8) {
        self.role_store = role_store;
        self.key = to_seed(key).into();
        self.bump = bump;
    }

    /// Get the role store key.
    pub fn role_store(&self) -> &Pubkey {
        &self.role_store
    }
}

#[error_code]
pub enum DataStoreError {
    #[msg("Mismatched role store")]
    MismatchedRoleStore,
    #[msg("Invalid pda")]
    InvalidPDA,
}
