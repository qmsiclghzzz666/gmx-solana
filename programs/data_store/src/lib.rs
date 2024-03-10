use anchor_lang::prelude::*;
use role_store::Authenticate;

/// Instructions.
pub mod instructions;

/// States.
pub mod states;

pub use self::states::Data;

use self::instructions::*;

declare_id!("8hJ2dGQ2Ccr5G6iEqQQEoBApRSXt7Jn8Qyf9Qf3eLBX2");

#[program]
pub mod data_store {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, key: String) -> Result<()> {
        instructions::initialize(ctx, key)
    }

    #[access_control(Authenticate::only_controller(&ctx))]
    pub fn initialize_token_config(
        ctx: Context<InitializeTokenConfig>,
        key: String,
        price_feed: Pubkey,
        heartbeat_duration: u32,
        token_decimals: u8,
        precision: u8,
    ) -> Result<()> {
        instructions::initialize_token_config(
            ctx,
            key,
            price_feed,
            heartbeat_duration,
            token_decimals,
            precision,
        )
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

#[error_code]
pub enum DataStoreError {
    #[msg("Mismatched role store")]
    MismatchedRoleStore,
    #[msg("Invalid pda")]
    InvalidPDA,
    #[msg("Invalid key")]
    InvalidKey,
}
