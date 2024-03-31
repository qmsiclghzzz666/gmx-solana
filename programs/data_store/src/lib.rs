use anchor_lang::prelude::*;

/// Instructions.
pub mod instructions;

/// States.
pub mod states;

/// Constants.
pub mod constants;

/// Utils.
pub mod utils;

pub use self::states::Data;

use self::{
    instructions::*,
    states::{
        deposit::TokenParams as DepositTokenParams,
        market::{MarketMeta, Pool},
        token_config::TokenConfig,
        withdrawal::TokenParams as WithdrawalTokenParams,
    },
    utils::internal,
};
use gmx_solana_utils::price::Price;

declare_id!("EjfyBCoSMd6rjkUNz1SFfD7DBYbvAuaxhQs8phcu7Eb6");

#[program]
pub mod data_store {
    use super::*;

    // Data Store.
    pub fn initialize(ctx: Context<Initialize>, key: String) -> Result<()> {
        instructions::initialize(ctx, key)
    }

    // Roles.
    pub fn initialize_roles(ctx: Context<InitializeRoles>) -> Result<()> {
        instructions::initialize_roles(ctx)
    }

    pub fn check_admin(ctx: Context<CheckRole>, authority: Pubkey) -> Result<bool> {
        instructions::check_admin(ctx, authority)
    }

    pub fn check_role(ctx: Context<CheckRole>, authority: Pubkey, role: String) -> Result<bool> {
        instructions::check_role(ctx, authority, role)
    }

    #[access_control(internal::Authenticate::only_admin(&ctx))]
    pub fn enable_role(ctx: Context<EnableRole>, role: String) -> Result<()> {
        instructions::enable_role(ctx, role)
    }

    #[access_control(internal::Authenticate::only_admin(&ctx))]
    pub fn disable_role(ctx: Context<DisableRole>, role: String) -> Result<()> {
        instructions::disable_role(ctx, role)
    }

    #[access_control(internal::Authenticate::only_admin(&ctx))]
    pub fn grant_role(ctx: Context<GrantRole>, user: Pubkey, role: String) -> Result<()> {
        instructions::grant_role(ctx, user, role)
    }

    #[access_control(internal::Authenticate::only_admin(&ctx))]
    pub fn revoke_role(ctx: Context<RevokeRole>, user: Pubkey, role: String) -> Result<()> {
        instructions::revoke_role(ctx, user, role)
    }

    #[access_control(internal::Authenticate::only_admin(&ctx))]
    pub fn add_admin(ctx: Context<AddAdmin>, user: Pubkey) -> Result<()> {
        instructions::add_admin(ctx, user)
    }

    #[access_control(internal::Authenticate::only_admin(&ctx))]
    pub fn remove_admin(ctx: Context<RemoveAdmin>, user: Pubkey) -> Result<()> {
        instructions::remove_admin(ctx, user)
    }

    // Token Config.
    #[access_control(internal::Authenticate::only_controller(&ctx))]
    pub fn initialize_token_config_map(
        ctx: Context<InitializeTokenConfigMap>,
        len: u16,
    ) -> Result<()> {
        instructions::initialize_token_config_map(ctx, len)
    }

    #[access_control(internal::Authenticate::only_controller(&ctx))]
    pub fn insert_token_config(
        ctx: Context<InsertTokenConfig>,
        price_feed: Pubkey,
        heartbeat_duration: u32,
        precision: u8,
    ) -> Result<()> {
        instructions::insert_token_config(ctx, price_feed, heartbeat_duration, precision)
    }

    #[access_control(internal::Authenticate::only_controller(&ctx))]
    pub fn toggle_token_config(
        ctx: Context<ToggleTokenConfig>,
        token: Pubkey,
        enable: bool,
    ) -> Result<()> {
        instructions::toggle_token_config(ctx, token, enable)
    }

    pub fn get_token_config(
        ctx: Context<GetTokenConfig>,
        store: Pubkey,
        token: Pubkey,
    ) -> Result<Option<TokenConfig>> {
        instructions::get_token_config(ctx, store, token)
    }

    #[access_control(internal::Authenticate::only_controller(&ctx))]
    pub fn extend_token_config_map(ctx: Context<ExtendTokenConfigMap>, len: u16) -> Result<()> {
        instructions::extend_token_config_map(ctx, len)
    }

    // Market.
    #[access_control(internal::Authenticate::only_market_keeper(&ctx))]
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

    #[access_control(internal::Authenticate::only_market_keeper(&ctx))]
    pub fn remove_market(ctx: Context<RemoveMarket>) -> Result<()> {
        instructions::remove_market(ctx)
    }

    #[access_control(internal::Authenticate::only_controller(&ctx))]
    pub fn apply_delta_to_market_pool(
        ctx: Context<ApplyDeltaToMarketPool>,
        pool: u8,
        is_long_token: bool,
        delta: i128,
    ) -> Result<()> {
        instructions::apply_delta_to_market_pool(
            ctx,
            pool.try_into()
                .map_err(|_| DataStoreError::InvalidArgument)?,
            is_long_token,
            delta,
        )
    }

    pub fn get_pool(ctx: Context<GetPool>, pool: u8) -> Result<Option<Pool>> {
        instructions::get_pool(
            ctx,
            pool.try_into()
                .map_err(|_| DataStoreError::InvalidArgument)?,
        )
    }

    pub fn get_market_token_mint(ctx: Context<GetMarketTokenMint>) -> Result<Pubkey> {
        instructions::get_market_token_mint(ctx)
    }

    pub fn get_market_meta(ctx: Context<GetMarketMeta>) -> Result<MarketMeta> {
        instructions::get_market_meta(ctx)
    }

    // Token.
    #[access_control(internal::Authenticate::only_market_keeper(&ctx))]
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

    #[access_control(internal::Authenticate::only_controller(&ctx))]
    pub fn mint_market_token_to(ctx: Context<MintMarketTokenTo>, amount: u64) -> Result<()> {
        instructions::mint_market_token_to(ctx, amount)
    }

    #[access_control(internal::Authenticate::only_controller(&ctx))]
    pub fn burn_market_token_from(ctx: Context<BurnMarketTokenFrom>, amount: u64) -> Result<()> {
        instructions::burn_market_token_from(ctx, amount)
    }

    #[access_control(internal::Authenticate::only_market_keeper(&ctx))]
    pub fn initialize_market_vault(
        ctx: Context<InitializeMarketVault>,
        market_token_mint: Option<Pubkey>,
    ) -> Result<()> {
        instructions::initialize_market_vault(ctx, market_token_mint)
    }

    #[access_control(internal::Authenticate::only_controller(&ctx))]
    pub fn market_vault_transfer_out(
        ctx: Context<MarketVaultTransferOut>,
        amount: u64,
    ) -> Result<()> {
        instructions::market_vault_transfer_out(ctx, amount)
    }

    // Oracle.
    #[access_control(internal::Authenticate::only_controller(&ctx))]
    pub fn initialize_oracle(ctx: Context<InitializeOracle>, index: u8) -> Result<()> {
        instructions::initialize_oracle(ctx, index)
    }

    #[access_control(internal::Authenticate::only_controller(&ctx))]
    pub fn clear_all_prices(ctx: Context<ClearAllPrices>) -> Result<()> {
        instructions::clear_all_prices(ctx)
    }

    #[access_control(internal::Authenticate::only_controller(&ctx))]
    pub fn set_price(ctx: Context<SetPrice>, token: Pubkey, price: Price) -> Result<()> {
        instructions::set_price(ctx, token, price)
    }

    // Nonce.
    #[access_control(internal::Authenticate::only_controller(&ctx))]
    pub fn initialize_nonce(ctx: Context<InitializeNonce>) -> Result<()> {
        instructions::initialize_nonce(ctx)
    }

    #[access_control(internal::Authenticate::only_controller(&ctx))]
    pub fn increment_nonce(ctx: Context<IncrementNonce>) -> Result<[u8; 32]> {
        instructions::increment_nonce(ctx)
    }

    pub fn get_nonce_bytes(ctx: Context<GetNonceBytes>) -> Result<[u8; 32]> {
        instructions::get_nonce_bytes(ctx)
    }

    // Deposit.
    #[access_control(internal::Authenticate::only_controller(&ctx))]
    pub fn initialize_deposit(
        ctx: Context<InitializeDeposit>,
        nonce: [u8; 32],
        ui_fee_receiver: Pubkey,
        tokens: DepositTokenParams,
    ) -> Result<()> {
        instructions::initialize_deposit(ctx, nonce, ui_fee_receiver, tokens)
    }

    #[access_control(internal::Authenticate::only_controller(&ctx))]
    pub fn remove_deposit(ctx: Context<RemoveDeposit>, refund: u64) -> Result<()> {
        instructions::remove_deposit(ctx, refund)
    }

    // Withdrawal.
    #[access_control(internal::Authenticate::only_controller(&ctx))]
    pub fn initialize_withdrawal(
        ctx: Context<InitializeWithdrawal>,
        nonce: [u8; 32],
        tokens: WithdrawalTokenParams,
        tokens_with_feed: Vec<(Pubkey, Pubkey)>,
        market_token_amount: u64,
        ui_fee_receiver: Pubkey,
    ) -> Result<()> {
        instructions::initialize_withdrawal(
            ctx,
            nonce,
            tokens,
            tokens_with_feed,
            market_token_amount,
            ui_fee_receiver,
        )
    }

    #[access_control(internal::Authenticate::only_controller(&ctx))]
    pub fn remove_withdrawal(ctx: Context<RemoveWithdrawal>, refund: u64) -> Result<()> {
        instructions::remove_withdrawal(ctx, refund)
    }
}

#[error_code]
pub enum DataStoreError {
    // Common.
    #[msg("Invalid pda")]
    InvalidPDA,
    #[msg("Invalid key")]
    InvalidKey,
    #[msg("Exceed max length limit")]
    ExceedMaxLengthLimit,
    #[msg("Exceed max string length limit")]
    ExceedMaxStringLengthLimit,
    #[msg("Invalid argument")]
    InvalidArgument,
    #[msg("Lamports not enough")]
    LamportsNotEnough,
    #[msg("Required resource not found")]
    RequiredResourceNotFound,
    // Roles.
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
    #[msg("Invalid roles account")]
    InvalidRoles,
    #[msg("Permission denied")]
    PermissionDenied,
    // Oracle.
    #[msg("Price of the given token already set")]
    PriceAlreadySet,
    // Market.
    #[msg("Computation error")]
    Computation,
    #[msg("Unsupported pool kind")]
    UnsupportedPoolKind,
}
