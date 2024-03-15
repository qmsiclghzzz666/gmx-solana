use anchor_lang::prelude::*;
use role_store::Authenticate;

/// Instructions.
pub mod instructions;

/// States.
pub mod states;

/// Constants.
pub mod constants;

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

    #[access_control(Authenticate::only_market_keeper(&ctx))]
    pub fn initialize_market(
        ctx: Context<InitializeMarket>,
        market_token_mint: Pubkey,
        index_token_mint: Pubkey,
        long_token_mint: Pubkey,
        short_token_mint: Pubkey,
    ) -> Result<()> {
        instructions::initialize_market(
            ctx,
            market_token_mint,
            index_token_mint,
            long_token_mint,
            short_token_mint,
        )
    }

    #[access_control(Authenticate::only_market_keeper(&ctx))]
    pub fn remove_market(ctx: Context<RemoveMarket>) -> Result<()> {
        instructions::remove_market(ctx)
    }

    #[access_control(Authenticate::only_market_keeper(&ctx))]
    pub fn initialize_market_token(
        ctx: Context<InitializeMarketToken>,
        index_token_mint: Pubkey,
        long_token_mint: Pubkey,
        short_token_mint: Pubkey,
    ) -> Result<()> {
        instructions::initialize_market_token(
            ctx,
            index_token_mint,
            long_token_mint,
            short_token_mint,
        )
    }

    #[access_control(Authenticate::only_controller(&ctx))]
    pub fn mint_market_token_to(ctx: Context<MintMarketTokenTo>, amount: u64) -> Result<()> {
        instructions::mint_market_token_to(ctx, amount)
    }

    #[access_control(Authenticate::only_market_keeper(&ctx))]
    pub fn initialize_market_vault(
        ctx: Context<InitializeMarketVault>,
        market_token_mint: Option<Pubkey>,
    ) -> Result<()> {
        instructions::initialize_market_vault(ctx, market_token_mint)
    }

    #[access_control(Authenticate::only_controller(&ctx))]
    pub fn market_vault_transfer_out(
        ctx: Context<MarketVaultTransferOut>,
        amount: u64,
    ) -> Result<()> {
        instructions::market_vault_transfer_out(ctx, amount)
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
    #[msg("Exceed max length limit")]
    ExceedMaxLengthLimit,
    #[msg("Exceed max string length limit")]
    ExceedMaxStringLengthLimit,
    #[msg("Too many admins")]
    TooManyAdmins,
    #[msg("At least one admin")]
    AtLeastOneAdmin,
    #[msg("Invalid data store")]
    InvalidDataStore,
    #[msg("Already be an admin")]
    AlreadyBeAnAdmin,
    #[msg("Not an admin")]
    NotAnAdmin,
    #[msg("Invalid role")]
    InvalidRole,
}
