//! # The GMSOL Store Program
//!
//! ## Store Account
//!
//! A [`Store`](states::Store) Account serves as both an authority and a global configuration
//! storage.
//!
//! ### Instructions for Store Accounts
//! - [`initialize`](gmsol_store::initialize): Create a new [`Store`](states::Store) account.
//! - [`transfer_store_authority`]: Transfer the authority of the given store to a new authority.
//!
//! ## Role-based Permission Management
//!
//! The complete role-based permission table for each GMSOL deployment is directly stored in the
//! [`Store`](states::Store) Account of that deployment. The current permission structure in GMSOL includes:
//! - (Unique) Administrator: The administrator's address is directly stored in the `authority` field
//!   of the [`Store`](states::Store) Account. Only this address can modify the permission table.
//! - Custom Roles: The custom role table and member table are stored in the `role` field of the
//!   [`Store`](states::Store) account as a [`RoleStore`](states::RoleStore) structure.
//!
//! ### Instructions for Permission Management
//! - [`enable_role`]: Insert or enable a role for the given store.
//! - [`disable_role`]: Disable an existing role for the given store.
//! - [`grant_role`]: Grant a role to the given user in the given store.
//! - [`revoke_role`]: Revoke a role from the given user in the given store.
//! - [`check_admin`](gmsol_store::check_admin): Check that the signer is the admin of the given store,
//!   throw error if the check fails.
//! - [`check_role`](gmsol_store::check_role): Check that the signer has the given role in the given store,
//!   throw error if the check fails.
//! - [`has_admin`](gmsol_store::has_admin): Return whether the given address is the admin of the given store,
//!   or not.
//! - [`has_role`](gmsol_store::has_role): Return whether the given address has the given role in the given store,
//!   or not.
//!
//! ## Oracle Price Management
//!
//! ### Instructions for [`TokenConfig`](states::TokenConfig) and token maps.
//! - [`initialize_token_map`](gmsol_store::initialize_token_map): Initialize a new token map account.
//!   This is a permissionless instruction.
//! - [`set_token_map`]: Set the token map address used in the given store.
//! - [`push_to_token_map`]: Push a new token config for an existing token to the given token map.
//! - [`push_to_token_map_synthetic`]: Push a new token config for a "synthetic"
//!   token to the given token map.
//! - [`toggle_token_config`]: Enable or disable a token config of the given token map.
//! - [`set_expected_provider`]: Set the expected provider for the given token.
//! - [`set_feed_config`]: Set the feed config of the given provider for the given token.
//! - [`is_token_config_enabled`](gmsol_store::is_token_config_enabled): Check if the config for the given token is enabled.
//! - [`token_expected_provider`](gmsol_store::token_expected_provider): Get the expected provider set for the given token.
//! - [`token_feed`](gmsol_store::token_feed): Get the feed address of the given provider set for the given token.
//! - [`token_timestamp_adjustment`](gmsol_store::token_timestamp_adjustment): Get the timestamp adjustment of the given
//!   provider for the give token.
//! - [`token_name`](gmsol_store::token_name): Get the name of the given token.
//! - [`token_decimals`](gmsol_store::token_decimals): Get the token decimals of the given token.
//! - [`token_precision`](gmsol_store::token_precision): Get the price precision of the given token.
//!
//! ### Instructions for [`Oracle`](states::Oracle) accounts
//! - [`initialize_oracle`]: Initialize a new [`Oracle`](states::Oracle) account.
//! - [`clear_all_prices`](gmsol_store::clear_all_prices): Clear the prices of the given oracle account.
//! - [`set_price`](gmsol_store::set_price): Set a price for the given token in the given oracle account.
//! - [`set_prices_from_price_feed`](gmsol_store::set_prices_from_price_feed): Validate and set prices parsed from the
//!   provided price feed accounts.
//!
//! ## Market Management
//! The instructions related to market management are as follows:
//!
//! ### Instructions for [`Market`](states::Market) management
//! - [`initialize_market`]: Initialize a [`Market`](states::Market) account.
//! - [`remove_market`]: Close the given [`Market`](states::Market) account.
//! - [`toggle_market`]: Enable or diable the given market.
//! - [`market_transfer_in`]: Transfer tokens into the market and record in its balance.
//! - [`market_transfer_out`]: Transfer tokens out from the market and record in its balance.
//! - [`update_market_config`]: Update an item in the market config.
//! - [`update_market_config_with_buffer`]: Update the market config with the given
//!   [`MarketConfigBuffer`](states::MarketConfigBuffer) account.
//! - [`get_validated_market_meta`](gmsol_store::get_validated_market_meta): Validate the market and
//!   return its [meta](states::MarketMeta).
//! - [`get_market_config`](gmsol_store::get_market_config): Read an item from the market config by the key.
//! - [`get_market_meta`](gmsol_store::get_market_meta): Get the [meta](states::MarketMeta) of the market
//!   without validation.
//!
//! ### Instructions for [`MarketConfigBuffer`](states::MarketConfigBuffer) accounts
//! - [`initialize_market_config_buffer`](gmsol_store::initialize_market_config_buffer): Initialize a market config buffer account.
//! - [`set_market_config_buffer_authority`](gmsol_store::set_market_config_buffer_authority): Replace the authority of the market
//!   config buffer account with the new one.
//! - [`close_market_config_buffer`](gmsol_store::close_market_config_buffer): Close the given market config buffer account.
//! - [`push_to_market_config_buffer`](gmsol_store::push_to_market_config_buffer): Push config items to the given market config
//!   buffer account.
//!
//! ### Instructions for market tokens
//! - [`mint_market_token_to`]: Mint the given amount of market tokens to the destination
//!   account.
//! - [`burn_market_token_from`]: Burn the given amount of market tokens from the given account.
//!
//! ### Instructions for market vaults
//! - [`initialize_market_vault`]: Initialize the market vault for the given token.
//! - [`market_vault_transfer_out`]: Transfer the given amount of tokens out to the destination
//!   account.

/// Instructions.
pub mod instructions;

/// States.
pub mod states;

/// Operations.
pub mod ops;

/// Constants.
pub mod constants;

/// Utils.
pub mod utils;

/// Events.
pub mod events;

use self::{
    instructions::*,
    ops::{
        deposit::CreateDepositParams,
        glv::{CreateGlvDepositParams, CreateGlvWithdrawalParams},
        order::{CreateOrderParams, PositionCutKind},
        shift::CreateShiftParams,
        withdrawal::CreateWithdrawalParams,
    },
    states::{
        common::action::{Close, Create},
        market::{config::EntryArgs, status::MarketStatus, MarketMeta},
        order::UpdateOrderParams,
        token_config::TokenConfigBuilder,
        FactorKey, PriceProviderKind,
    },
    utils::internal,
};
use anchor_lang::prelude::*;
use gmsol_model::price::Prices;
use gmsol_utils::price::Price;

#[cfg_attr(test, macro_use)]
extern crate static_assertions;

declare_id!("gmX4GEZycT14vqJ3yDoCA5jW53vBaSQpQDYNDXtkWt1");

#[program]
/// Instructions definitions of the GMSOL Store Program.
pub mod gmsol_store {

    use super::*;

    // Data Store.
    /// Create a new [`Store`](states::Store) account.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](Initialize).*
    ///
    /// # Arguments
    /// - `key`: The name of the store, also used as seed to derive
    /// the address of the store account. The length of the `key`
    /// cannot exceed [`MAX_LEN`](states::Store::MAX_LEN).
    /// - `authority`: The authority (admin) address that will be set
    /// after the Store is created. If not provided,
    /// [`payer`](Initialize::payer) will be used as the default
    /// authority address.
    ///
    /// # Errors
    /// - The [`payer`](Initialize::payer) must a signer.
    /// - The [`store`](Initialize::store) must not be initialized.
    /// - The address of the [`store`](Initialize::store) must be the PDA
    ///   derived from the store account seed [`SEED`](states::Store::SEED)
    ///   and the SHA-256 encoded `key` parameter.
    pub fn initialize(
        ctx: Context<Initialize>,
        key: String,
        authority: Option<Pubkey>,
    ) -> Result<()> {
        instructions::initialize(ctx, key, authority)
    }

    /// Transfer the authority of the given store to a new authority.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](TransferStoreAuthority).*
    ///
    /// # Arguments
    /// - `new_authority`: The new authority to be set for the store account.
    ///
    /// # Errors
    /// - The [`authority`](TransferStoreAuthority::authority) must be a signer
    /// and be the `ADMIN` of the store.
    /// - The [`store`](TransferStoreAuthority::store) must have been initialized
    /// and owned by the store program.
    #[access_control(internal::Authenticate::only_admin(&ctx))]
    pub fn transfer_store_authority(
        ctx: Context<TransferStoreAuthority>,
        new_authority: Pubkey,
    ) -> Result<()> {
        instructions::unchecked_transfer_store_authority(ctx, new_authority)
    }

    /// Set the receiver address.
    pub fn set_receiver(ctx: Context<SetReceiver>) -> Result<()> {
        instructions::set_receiver(ctx)
    }

    #[access_control(internal::Authenticate::only_market_keeper(&ctx))]
    pub fn set_token_map(ctx: Context<SetTokenMap>) -> Result<()> {
        instructions::unchecked_set_token_map(ctx)
    }

    pub fn get_token_map(ctx: Context<ReadStore>) -> Result<Option<Pubkey>> {
        instructions::get_token_map(ctx)
    }

    // Roles.
    /// Check that the signer is the admin of the given store, throw error if
    /// the check fails.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](CheckRole).*
    ///
    /// # Errors
    /// - The [`authority`](CheckRole::authority) must be a signer and be
    /// the `ADMIN` of the store.
    /// - The [`store`](CheckRole::store) must have been initialized
    /// and owned by the store program.
    pub fn check_admin(ctx: Context<CheckRole>) -> Result<bool> {
        instructions::check_admin(ctx)
    }

    /// Check that the signer has the given role in the given store, throw
    /// error if the check fails.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](CheckRole).*
    ///
    /// # Arguments
    /// - `role`: The name of the role to be checked.
    ///
    /// # Errors
    /// - The [`authority`](CheckRole::authority) must be a signer and
    /// must be a member with the `role` role in the store.
    /// - The [`store`](CheckRole::store) must have been initialized
    /// and owned by the store program.
    /// - The `role` must exist and be enabled in the store.
    pub fn check_role(ctx: Context<CheckRole>, role: String) -> Result<bool> {
        instructions::check_role(ctx, role)
    }

    /// Return whether the given address is the admin of the given store, or not.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](HasRole).*
    ///
    /// # Arguments
    /// - `authority`: The address to check for admin privileges.
    ///
    /// # Errors
    /// - The [`store`](HasRole::store) must have been initialized
    /// and owned by the store program.
    pub fn has_admin(ctx: Context<HasRole>, authority: Pubkey) -> Result<bool> {
        instructions::has_admin(ctx, authority)
    }

    /// Return whether the given address has the given role in the given store, or not.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](HasRole).*
    ///
    /// # Arguments
    /// - `authority`: The address to check for the role.
    /// - `role`: The role to be checked.
    ///
    /// # Errors
    /// - The [`store`](HasRole::store) must have been initialized
    /// and owned by the store program.
    /// - The `role` must exist and be enabled in the store.
    pub fn has_role(ctx: Context<HasRole>, authority: Pubkey, role: String) -> Result<bool> {
        instructions::has_role(ctx, authority, role)
    }

    /// Insert or enable a role for the given store.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](EnableRole).*
    ///
    /// # Arguments
    /// - `role`: The name of the role to be added/enabled. The length cannot exceed
    /// [`MAX_ROLE_NAME_LEN`](states::roles::MAX_ROLE_NAME_LEN).
    ///
    /// # Errors
    /// - The [`authority`](EnableRole::authority) must be a signer and be
    /// the `ADMIN` of the store.
    /// - The [`store`](EnableRole::store) must have been initialized
    /// and owned by the store program.
    #[access_control(internal::Authenticate::only_admin(&ctx))]
    pub fn enable_role(ctx: Context<EnableRole>, role: String) -> Result<()> {
        instructions::unchecked_enable_role(ctx, role)
    }

    /// Disable an existing role for the given store.
    /// It has no effect if this role does not exist in the store.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](DisableRole).*
    ///
    /// # Arguments
    /// - `role`: The name of the role to be disabled.
    ///
    /// # Errors
    /// - The [`authority`](DisableRole::authority) must be a signer and be
    /// the `ADMIN` of the store.
    /// - The [`store`](DisableRole::store) must have been initialized
    /// and owned by the store program.
    #[access_control(internal::Authenticate::only_admin(&ctx))]
    pub fn disable_role(ctx: Context<DisableRole>, role: String) -> Result<()> {
        instructions::unchecked_disable_role(ctx, role)
    }

    /// Grant a role to the given user in the given store.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](GrantRole).*
    ///
    /// # Arguments
    /// - `user`: The user to whom the role is to be granted.
    /// - `role`: The role to be granted to the user.
    ///
    /// # Errors
    /// - The [`authority`](GrantRole::authority) must be a signer and
    /// be the `ADMIN` of the store.
    /// - The [`store`](GrantRole::store) must have been initialized
    /// and owned by the store program.
    /// - The `role` must exist and be enabled in the store.
    #[access_control(internal::Authenticate::only_admin(&ctx))]
    pub fn grant_role(ctx: Context<GrantRole>, user: Pubkey, role: String) -> Result<()> {
        instructions::unchecked_grant_role(ctx, user, role)
    }

    /// Revoke a role from the given user in the given store.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](RevokeRole).*
    ///
    /// # Arguments
    /// - `user`: The user to whom the role is to be revoked.
    /// - `role`: The role to be revoked from the user.
    ///
    /// # Errors
    /// - The [`authority`](RevokeRole::authority) must be a signer and be
    /// the `ADMIN` of the store.
    /// - The [`store`](RevokeRole::store) must have been initialized
    /// and owned by the store program.
    /// - The `user` must exist in the member table.
    /// - The `role` must exist and be enabled in the store.
    #[access_control(internal::Authenticate::only_admin(&ctx))]
    pub fn revoke_role(ctx: Context<RevokeRole>, user: Pubkey, role: String) -> Result<()> {
        instructions::unchecked_revoke_role(ctx, user, role)
    }

    // Config.
    #[access_control(internal::Authenticate::only_controller(&ctx))]
    pub fn insert_amount(ctx: Context<InsertAmount>, key: String, amount: u64) -> Result<()> {
        instructions::insert_amount(ctx, &key, amount)
    }

    #[access_control(internal::Authenticate::only_controller(&ctx))]
    pub fn insert_factor(ctx: Context<InsertFactor>, key: String, factor: u128) -> Result<()> {
        instructions::insert_factor(ctx, &key, factor)
    }

    #[access_control(internal::Authenticate::only_controller(&ctx))]
    pub fn insert_address(ctx: Context<InsertAddress>, key: String, address: Pubkey) -> Result<()> {
        instructions::insert_address(ctx, &key, address)
    }

    #[access_control(internal::Authenticate::only_market_keeper(&ctx))]
    pub fn insert_gt_minting_cost_referred_discount(
        ctx: Context<InsertFactor>,
        factor: u128,
    ) -> Result<()> {
        let key = FactorKey::GtMintingCostReferredDiscount;
        instructions::insert_factor(ctx, &key.to_string(), factor)
    }

    /// Enable or diable the given feature.
    #[access_control(internal::Authenticate::only_market_keeper(&ctx))]
    pub fn toggle_feature(
        ctx: Context<ToggleFeature>,
        domain: String,
        action: String,
        enable: bool,
    ) -> Result<()> {
        let domain = domain
            .parse()
            .map_err(|_| error!(CoreError::InvalidArgument))?;
        let action = action
            .parse()
            .map_err(|_| error!(CoreError::InvalidArgument))?;
        instructions::unchecked_toggle_feature(ctx, domain, action, enable)
    }

    // Token Config.
    /// Initialize a new token map account with its store set to [`store`](InitializeTokenMap::store).
    ///
    /// Anyone can initialize a token map account without any permissions, but after initialization, only
    /// addresses authorized by the store can modify this token map.
    ///
    /// # Accounts
    /// [*See the documentation for the accounts*](InitializeTokenMap).
    ///
    /// # Errors
    /// - The [`payer`](InitializeTokenMap::payer) must be a signer.
    /// - The [`store`](InitializeTokenMap::store) must be an initialized [`Store`](states::Store)
    /// account owned by the store program.
    /// - The [`token_map`](InitializeTokenMap::token_map) must be a uninitialized account.
    pub fn initialize_token_map(ctx: Context<InitializeTokenMap>) -> Result<()> {
        instructions::initialize_token_map(ctx)
    }

    /// Push a new token config to the given token map.
    ///
    /// This instruction is used to add or update the token config for an existing token,
    /// where its `token_decimals` will naturally be set to the decimals of this token.
    ///
    /// # Accounts
    /// [*See the documentation for the accounts*](PushToTokenMap).
    ///
    /// # Arguments
    /// - `name`: The name of token.
    /// - `builder`: Builder for the token config.
    /// - `enable`: Whether the token config should be enabled/disabled after the push.
    /// - `new`: Enforce insert if new = true, and an error will be returned if the config
    /// for the given token already exists.
    ///
    /// # Errors
    /// - The [`authority`](PushToTokenMap::authority) must be a signer and a MARKET_KEEPER
    /// of the given store.
    /// - The [`store`](PushToTokenMap::store) must be an initialized [`Store`](states::Store)
    /// account owned by the store program. And it must be the owner of the token map.
    /// - The [`token_map`](PushToTokenMap::token_map) must be an initialized token map account
    /// owned by the store program.
    /// - The [`token`](PushToTokenMap::token) must be an initialized token mint account owned
    /// by the SPL token program.
    #[access_control(internal::Authenticate::only_market_keeper(&ctx))]
    pub fn push_to_token_map(
        ctx: Context<PushToTokenMap>,
        name: String,
        builder: TokenConfigBuilder,
        enable: bool,
        new: bool,
    ) -> Result<()> {
        instructions::unchecked_push_to_token_map(ctx, &name, builder, enable, new)
    }

    /// Push a new synthetic token config to the given token map.
    ///
    /// This instruction can set or update the token config for a non-existent token.
    /// Its token decimals are determined by the corresponding argument.
    ///
    /// # Accounts
    /// [*See the documentation for the accounts*](PushToTokenMapSynthetic).
    ///
    /// # Arguments
    /// - `name`: The name of synthetic token.
    /// - `token`: The address of the synthetic token.
    /// - `token_decimals`: The token decimals to use for the synthetic token.
    /// - `builder`: Builder for the token config.
    /// - `enable`: Whether the token config should be enabled/disabled after the push.
    /// - `new`: Enforce insert if new = true, and an error will be returned if the config
    /// for the given token already exists.
    ///
    /// # Errors
    /// - The [`authority`](PushToTokenMapSynthetic::authority) must be a signer and a MARKET_KEEPER
    /// of the given store.
    /// - The [`store`](PushToTokenMapSynthetic::store) must be an initialized [`Store`](states::Store)
    /// account owned by the store program. And it must be the owner of the token map.
    /// - The [`token_map`](PushToTokenMapSynthetic::token_map) must be an initialized token map account
    /// owned by the store program.
    #[access_control(internal::Authenticate::only_market_keeper(&ctx))]
    pub fn push_to_token_map_synthetic(
        ctx: Context<PushToTokenMapSynthetic>,
        name: String,
        token: Pubkey,
        token_decimals: u8,
        builder: TokenConfigBuilder,
        enable: bool,
        new: bool,
    ) -> Result<()> {
        instructions::unchecked_push_to_token_map_synthetic(
            ctx,
            &name,
            token,
            token_decimals,
            builder,
            enable,
            new,
        )
    }

    pub fn is_token_config_enabled(ctx: Context<ReadTokenMap>, token: Pubkey) -> Result<bool> {
        instructions::is_token_config_enabled(ctx, &token)
    }

    pub fn token_expected_provider(ctx: Context<ReadTokenMap>, token: Pubkey) -> Result<u8> {
        instructions::token_expected_provider(ctx, &token).map(|kind| kind as u8)
    }

    pub fn token_feed(ctx: Context<ReadTokenMap>, token: Pubkey, provider: u8) -> Result<Pubkey> {
        instructions::token_feed(
            ctx,
            &token,
            &PriceProviderKind::try_from(provider)
                .map_err(|_| CoreError::InvalidProviderKindIndex)?,
        )
    }

    pub fn token_timestamp_adjustment(
        ctx: Context<ReadTokenMap>,
        token: Pubkey,
        provider: u8,
    ) -> Result<u32> {
        instructions::token_timestamp_adjustment(
            ctx,
            &token,
            &PriceProviderKind::try_from(provider)
                .map_err(|_| CoreError::InvalidProviderKindIndex)?,
        )
    }

    pub fn token_name(ctx: Context<ReadTokenMap>, token: Pubkey) -> Result<String> {
        instructions::token_name(ctx, &token)
    }

    pub fn token_decimals(ctx: Context<ReadTokenMap>, token: Pubkey) -> Result<u8> {
        instructions::token_decimals(ctx, &token)
    }

    pub fn token_precision(ctx: Context<ReadTokenMap>, token: Pubkey) -> Result<u8> {
        instructions::token_precision(ctx, &token)
    }

    /// Enable of disable the config for the given token.
    ///
    /// # Accounts
    /// [*See the documentation for the accounts*](ToggleTokenConfig).
    ///
    /// # Arguments
    /// - `token`: The token whose config will be updated.
    /// - `enable`: Enable or diable the config.
    ///
    /// # Errors
    /// - The [`authority`](ToggleTokenConfig::authority) must be a signer
    /// and a MARKET_KEEPER of the give store.
    /// - The [`store`](ToggleTokenConfig::store) must be an initialized [`Store`](states::Store)
    /// account owned by the store program. And it must be the owner of the token map.
    /// - The [`token_map`](ToggleTokenConfig::token_map) must be an initialized token map account
    /// owned by the store program.
    /// - The given `token` must exist in the token map.
    #[access_control(internal::Authenticate::only_market_keeper(&ctx))]
    pub fn toggle_token_config(
        ctx: Context<ToggleTokenConfig>,
        token: Pubkey,
        enable: bool,
    ) -> Result<()> {
        instructions::unchecked_toggle_token_config(ctx, token, enable)
    }

    /// Set the expected provider for the given token.
    ///
    /// # Accounts
    /// [*See the documentation for the accounts*](SetExpectedProvider).
    ///
    /// # Arguments
    /// - `token`: The token whose config will be updated.
    /// - `provider`: The provider index to be set as the expected provider
    /// for the token. See also [`PriceProviderKind`].
    ///
    /// # Errors
    /// - The [`authority`](SetExpectedProvider::authority) must be a signer
    /// and a MARKET_KEEPER of the give store.
    /// - The [`store`](SetExpectedProvider::store) must be an initialized [`Store`](states::Store)
    /// account owned by the store program. And it must be the owner of the token map.
    /// - The [`token_map`](SetExpectedProvider::token_map) must be an initialized token map account
    /// owned by the store program.
    /// - The given `token` must exist in the token map.
    /// - The index of the provider must be valid.
    #[access_control(internal::Authenticate::only_market_keeper(&ctx))]
    pub fn set_expected_provider(
        ctx: Context<SetExpectedProvider>,
        token: Pubkey,
        provider: u8,
    ) -> Result<()> {
        instructions::unchecked_set_expected_provider(
            ctx,
            token,
            PriceProviderKind::try_from(provider)
                .map_err(|_| CoreError::InvalidProviderKindIndex)?,
        )
    }

    /// Set the feed config of the given provider for the given token.
    ///
    /// # Accounts
    /// [*See the documentation for the accounts*](SetFeedConfig).
    ///
    /// # Arguments
    /// - `token`: The token whose config will be updated.
    /// - `provider`: The index of the provider whose feed config will be updated.
    /// - `feed`: The new feed address.
    /// - `timestamp_adjustment`: The new timestamp adjustment seconds.
    ///
    /// # Errors
    /// - The [`authority`](SetFeedConfig::authority) must be a signer
    /// and a MARKET_KEEPER of the give store.
    /// - The [`store`](SetFeedConfig::store) must be an initialized [`Store`](states::Store)
    /// account owned by the store program. And it must be the owner of the token map.
    /// - The [`token_map`](SetFeedConfig::token_map) must be an initialized token map account
    /// owned by the store program.
    /// - The given `token` must exist in the token map.
    /// - The index of the provider must be valid.
    #[access_control(internal::Authenticate::only_market_keeper(&ctx))]
    pub fn set_feed_config(
        ctx: Context<SetFeedConfig>,
        token: Pubkey,
        provider: u8,
        feed: Pubkey,
        timestamp_adjustment: u32,
    ) -> Result<()> {
        instructions::unchecked_set_feed_config(
            ctx,
            token,
            &PriceProviderKind::try_from(provider)
                .map_err(|_| CoreError::InvalidProviderKindIndex)?,
            feed,
            timestamp_adjustment,
        )
    }

    // Market.
    /// Initialize a [`Market`](states::Market) account.
    ///
    /// # Accounts
    /// [*See the documentation for the accounts.*](InitializeMarket)
    ///
    /// # Arguments
    /// - `market_token_mint`: The address of the corresponding market token.
    /// - `index_token_mint`: The address of the index token.
    /// - `long_token_mint`: The address of the long token.
    /// - `short_token_mint`: The address of the short token.
    /// - `name`: The name of the market.
    /// - `enable`: Whether to enable the market after initialization.
    ///
    /// # Errors
    /// - *TODO*
    #[access_control(internal::Authenticate::only_market_keeper(&ctx))]
    pub fn initialize_market(
        ctx: Context<InitializeMarket>,
        index_token_mint: Pubkey,
        name: String,
        enable: bool,
    ) -> Result<()> {
        instructions::unchecked_initialize_market(ctx, index_token_mint, &name, enable)
    }

    /// Close a [`Market`](states::Market) account.
    ///
    /// # Accounts
    /// [*See the documentation for the accounts.*](RemoveMarket)
    ///
    /// # Errors
    /// - *TODO*
    #[access_control(internal::Authenticate::only_market_keeper(&ctx))]
    pub fn remove_market(ctx: Context<RemoveMarket>) -> Result<()> {
        instructions::unchecked_remove_market(ctx)
    }

    /// Validate the market and returns its [meta](states::MarketMeta).
    ///
    /// # Accounts
    /// [*See the documentation for the accounts.*](GetValidatedMarketMeta)
    ///
    /// # Errors
    /// - *TODO*
    pub fn get_validated_market_meta(ctx: Context<GetValidatedMarketMeta>) -> Result<MarketMeta> {
        instructions::get_validated_market_meta(ctx)
    }

    /// Transfer tokens into the market and record in its balance.
    ///
    /// # Accounts
    /// [*See the documentation for the accounts.*](MarketTransferIn)
    ///
    /// # Arguments
    /// - `amount`: The amount to transfer in.
    ///
    /// # Errors
    /// - *TODO*
    #[access_control(internal::Authenticate::only_market_keeper(&ctx))]
    pub fn market_transfer_in(ctx: Context<MarketTransferIn>, amount: u64) -> Result<()> {
        instructions::unchecked_market_transfer_in(ctx, amount)
    }

    /// Get the [meta](states::MarketMeta) of the market without validation.
    ///
    /// # Errors
    /// [*See the documentation for the accounts.*](ReadMarket)
    ///
    pub fn get_market_meta(ctx: Context<ReadMarket>) -> Result<MarketMeta> {
        instructions::get_market_meta(ctx)
    }

    /// Read an item from the market config by the key.
    ///
    /// # Accounts
    /// [*See the documentation for the accounts.*](ReadMarket)
    ///
    /// # Arguments
    /// - `key`: The key of the config item.
    pub fn get_market_config(ctx: Context<ReadMarket>, key: String) -> Result<u128> {
        instructions::get_market_config(ctx, &key)
    }

    /// Read current market status.
    ///
    /// # Accounts
    /// [*See the documentation for the accounts.*](ReadMarket)
    ///
    /// # Arguments
    /// - `prices`: The unit prices of tokens.
    /// - `maximize_pnl`: Whether to maximize the PnL.
    /// - `maximize_pool_value`: Whether to maximize the pool value.
    pub fn get_market_status(
        ctx: Context<ReadMarket>,
        prices: Prices<u128>,
        maximize_pnl: bool,
        maximize_pool_value: bool,
    ) -> Result<MarketStatus> {
        instructions::get_market_status(ctx, &prices, maximize_pnl, maximize_pool_value)
    }

    /// Get current market token price.
    ///
    /// # Accounts
    /// [*See the documentation for the accounts.*](ReadMarket)
    ///
    /// # Arguments
    /// - `prices`: The unit prices of tokens.
    /// - `maximize_pnl`: Whether to maximize the PnL.
    /// - `maximize_pool_value`: Whether to maximize the pool value.
    pub fn get_market_token_price(
        ctx: Context<ReadMarketWithToken>,
        prices: Prices<u128>,
        pnl_factor: String,
        maximize: bool,
    ) -> Result<u128> {
        instructions::get_market_token_price(
            ctx,
            &prices,
            pnl_factor
                .parse()
                .map_err(|_| error!(CoreError::InvalidArgument))?,
            maximize,
        )
    }

    /// Update an item in the market config.
    ///
    /// # Accounts
    /// [*See the documentation for the accounts.*](UpdateMarketConfig)
    ///
    /// # Arguments
    /// - `key`: The key of the config item.
    /// - `value`: The value to update the config item to.
    ///
    /// # Errors
    /// - *TODO*
    #[access_control(internal::Authenticate::only_market_keeper(&ctx))]
    pub fn update_market_config(
        ctx: Context<UpdateMarketConfig>,
        key: String,
        value: u128,
    ) -> Result<()> {
        instructions::unchecked_update_market_config(ctx, &key, value)
    }

    /// Update the market config with the given
    /// [`MarketConfigBuffer`](states::MarketConfigBuffer) account.
    ///
    /// # Accounts
    /// [*See the documentation for the accounts.*](UpdateMarketConfigWithBuffer)
    ///
    /// # Errors
    /// - *TODO*
    #[access_control(internal::Authenticate::only_market_keeper(&ctx))]
    pub fn update_market_config_with_buffer(
        ctx: Context<UpdateMarketConfigWithBuffer>,
    ) -> Result<()> {
        instructions::unchecked_update_market_config_with_buffer(ctx)
    }

    /// Initialize a market config buffer account.
    ///
    /// # Accounts
    /// [*See the documentation for the accounts.*](InitializeMarketConfigBuffer)
    ///
    /// # Arguments
    /// - `expire_after_secs`: The expiration time of the buffer in seconds.
    ///
    /// # Errors
    /// - *TODO*
    pub fn initialize_market_config_buffer(
        ctx: Context<InitializeMarketConfigBuffer>,
        expire_after_secs: u32,
    ) -> Result<()> {
        instructions::initialize_market_config_buffer(ctx, expire_after_secs)
    }

    /// Replace the authority of the market config buffer account
    /// with the new one.
    ///
    /// # Accounts
    /// [*See the documentation for the accounts.*](SetMarketConfigBufferAuthority)
    ///
    /// # Arguments
    /// - `new_authority`: The new authority.
    ///
    /// # Errors
    /// - *TODO*
    pub fn set_market_config_buffer_authority(
        ctx: Context<SetMarketConfigBufferAuthority>,
        new_authority: Pubkey,
    ) -> Result<()> {
        instructions::set_market_config_buffer_authority(ctx, new_authority)
    }

    /// Close the given market config buffer account.
    ///
    /// # Accounts
    /// [*See the documentation for the accounts.*](CloseMarketConfigBuffer)
    ///
    /// # Errors
    /// - *TODO*
    pub fn close_market_config_buffer(ctx: Context<CloseMarketConfigBuffer>) -> Result<()> {
        instructions::close_market_config_buffer(ctx)
    }

    /// Push config items to the given market config buffer account.
    ///
    /// # Accounts
    /// [*See the documentation for the accounts.*](PushToMarketConfigBuffer)
    ///
    /// # Arguments
    /// - `new_configs`: The list of new config items.
    ///
    /// # Errors
    /// - *TODO*
    pub fn push_to_market_config_buffer(
        ctx: Context<PushToMarketConfigBuffer>,
        new_configs: Vec<EntryArgs>,
    ) -> Result<()> {
        instructions::push_to_market_config_buffer(ctx, new_configs)
    }

    /// Enable or diable the given market.
    ///
    /// # Accounts
    /// [*See the documentation for the accounts.*](ToggleMarket)
    ///
    /// # Arguments
    /// - `enable`: Whether to enable or disable the market.
    ///
    /// # Errors
    /// - *TODO*
    #[access_control(internal::Authenticate::only_market_keeper(&ctx))]
    pub fn toggle_market(ctx: Context<ToggleMarket>, enable: bool) -> Result<()> {
        instructions::unchecked_toggle_market(ctx, enable)
    }

    /// Enable or diable GT minting for the given market.
    ///
    /// # Accounts
    /// [*See the documentation for the accounts.*](ToggleGTMinting)
    ///
    /// # Arguments
    /// - `enable`: Whether to enable or disable GT minting for the given market.
    ///
    /// # Errors
    /// - *TODO*
    #[access_control(internal::Authenticate::only_market_keeper(&ctx))]
    pub fn toggle_gt_minting(ctx: Context<ToggleGTMinting>, enable: bool) -> Result<()> {
        instructions::unchecked_toggle_gt_minting(ctx, enable)
    }

    /// Claim fees from the given market. The claimed amount remains in the market balance,
    /// and requires a subsequent transfer.
    ///
    /// # Accounts
    /// [*See the documentation for the accounts.*](ClaimFeesFromMarket)
    ///
    /// # Return
    /// - Returns the claimed amount.
    ///
    /// # Errors
    /// - The [`authority`](ClaimFeesFromMarket) must be a signer and be the receiver
    /// in the given store.
    /// - The [`store`](ClaimFeesFromMarket) must be an initialized [`Store`](crate::states::Store)
    /// account owned by this store program.
    /// - The [`market`](ClaimFeesFromMarket) must be an initialized [`Market`](crate::states::Market)
    /// account owned by this store program, whose the store must be the given one.
    /// - The `token` must be one of the collateral token.
    /// - Token accounts must be matched.
    /// - The market balance validation must pass after the claim.
    pub fn claim_fees_from_market(ctx: Context<ClaimFeesFromMarket>) -> Result<u64> {
        let claimed = instructions::claim_fees_from_market(ctx)?;
        Ok(claimed)
    }

    /// Mint the given amount of market tokens to the destination account.
    ///
    /// # Accounts
    /// [*See the documentation for the accounts.*](MintMarketTokenTo)
    ///
    /// # Arguments
    /// - `amount`: The amount to mint.
    ///
    /// # Errors
    /// - *TODO*
    #[access_control(internal::Authenticate::only_controller(&ctx))]
    pub fn mint_market_token_to(ctx: Context<MintMarketTokenTo>, amount: u64) -> Result<()> {
        instructions::unchecked_mint_market_token_to(ctx, amount)
    }

    /// Burn the given amount of market tokens from the given account.
    ///
    /// # Accounts
    /// [*See the documentation for the accounts.*](BurnMarketTokenFrom)
    ///
    /// # Arguments
    /// - `amount`: The amount to burn.
    ///
    /// # Errors
    /// - *TODO*
    #[access_control(internal::Authenticate::only_controller(&ctx))]
    pub fn burn_market_token_from(ctx: Context<BurnMarketTokenFrom>, amount: u64) -> Result<()> {
        instructions::unchecked_burn_market_token_from(ctx, amount)
    }

    /// Initialize the market vault for the given token.
    ///
    /// # Accounts
    /// [*See the documentation for the accounts.*](InitializeMarketVault)
    ///
    /// # Arguments
    /// - `market_token_mint`: (*deprecated*) The market that owns this vault.
    ///
    /// # Errors
    /// - *TODO*
    #[access_control(internal::Authenticate::only_market_keeper(&ctx))]
    pub fn initialize_market_vault(
        ctx: Context<InitializeMarketVault>,
        market_token_mint: Option<Pubkey>,
    ) -> Result<()> {
        instructions::unchecked_initialize_market_vault(ctx, market_token_mint)
    }

    /// Transfer the given amount of tokens out to the destination account.
    ///
    /// # Accounts
    /// [*See the documentation for the accounts.*](MarketVaultTransferOut)
    ///
    /// # Arguments
    /// - `amount`: The amount to transfer.
    ///
    /// # Errors
    /// - *TODO*
    #[access_control(internal::Authenticate::only_controller(&ctx))]
    pub fn market_vault_transfer_out(
        ctx: Context<MarketVaultTransferOut>,
        amount: u64,
    ) -> Result<()> {
        instructions::unchecked_market_vault_transfer_out(ctx, amount)
    }

    #[access_control(internal::Authenticate::only_order_keeper(&ctx))]
    pub fn use_claimable_account(
        ctx: Context<UseClaimableAccount>,
        timestamp: i64,
        amount: u64,
    ) -> Result<()> {
        instructions::unchecked_use_claimable_account(ctx, timestamp, amount)
    }

    #[access_control(internal::Authenticate::only_order_keeper(&ctx))]
    pub fn close_empty_claimable_account(
        ctx: Context<CloseEmptyClaimableAccount>,
        timestamp: i64,
    ) -> Result<()> {
        instructions::unchecked_close_empty_claimable_account(ctx, timestamp)
    }

    /// Prepare an associated token account.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](PrepareAssociatedTokenAccount).*
    ///
    /// # Errors
    /// - The [`payer`](PrepareAssociatedTokenAccount::payer) must be a signer.
    /// - The [`mint`](PrepareAssociatedTokenAccount::mint) must be a
    /// [`Mint`](anchor_spl::token::Mint) account.
    /// - The [`account`] must be an associated token account with mint = `mint`
    /// and owner = `owner`. It can be uninitialized.
    pub fn prepare_associated_token_account(
        ctx: Context<PrepareAssociatedTokenAccount>,
    ) -> Result<()> {
        instructions::prepare_associated_token_account(ctx)
    }

    // Oracle.
    /// Initailize a new oracle account for the given store with the given index.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](InitializeOracle).*
    ///
    /// # Arguments
    /// - `index`: The oracle index to use.
    ///
    /// # Errors
    /// - The [`authority`](InitializeOracle::authority) must be a signer and a MARKET_KEEPER of the store.
    /// - The [`store`](InitializeOracle::store) must be an initialized [`Store`](states::Store)
    /// account owned by the store program. And it must be the owner of the token map.
    /// - The [`oralce`](InitializeOracle::oracle) account must be uninitialized and its address must be the PDA
    /// derived from the oracle account seed [`SEED`](states::Oracle::SEED), the `store` address and
    /// the `index`.
    #[access_control(internal::Authenticate::only_market_keeper(&ctx))]
    pub fn initialize_oracle(ctx: Context<InitializeOracle>, index: u8) -> Result<()> {
        instructions::unchecked_initialize_oracle(ctx, index)
    }

    #[access_control(internal::Authenticate::only_controller(&ctx))]
    pub fn clear_all_prices(ctx: Context<ClearAllPrices>) -> Result<()> {
        instructions::clear_all_prices(ctx)
    }

    #[access_control(internal::Authenticate::only_controller(&ctx))]
    pub fn set_price(ctx: Context<SetPrice>, token: Pubkey, price: Price) -> Result<()> {
        instructions::set_price(ctx, token, price)
    }

    #[access_control(internal::Authenticate::only_controller(&ctx))]
    pub fn set_prices_from_price_feed<'info>(
        ctx: Context<'_, '_, 'info, 'info, SetPricesFromPriceFeed<'info>>,
        tokens: Vec<Pubkey>,
    ) -> Result<()> {
        instructions::set_prices_from_price_feed(ctx, tokens)
    }

    // Position.
    #[cfg(not(feature = "no-bug-fix"))]
    #[access_control(internal::Authenticate::only_market_keeper(&ctx))]
    pub fn turn_into_pure_pool(ctx: Context<TurnPureFlag>, kind: u8) -> Result<()> {
        instructions::unchecked_turn_into_pure_pool(
            ctx,
            kind.try_into()
                .map_err(|_| error!(CoreError::InvalidArgument))?,
        )
    }

    #[cfg(not(feature = "no-bug-fix"))]
    #[access_control(internal::Authenticate::only_market_keeper(&ctx))]
    pub fn turn_into_impure_pool(ctx: Context<TurnPureFlag>, kind: u8) -> Result<()> {
        instructions::unchecked_turn_into_impure_pool(
            ctx,
            kind.try_into()
                .map_err(|_| error!(CoreError::InvalidArgument))?,
        )
    }

    // Exchange.
    pub fn prepare_deposit_escrow(
        ctx: Context<PrepareDepositEscrow>,
        nonce: [u8; 32],
    ) -> Result<()> {
        instructions::prepare_deposit_escrow(ctx, nonce)
    }

    pub fn create_deposit<'info>(
        ctx: Context<'_, '_, 'info, 'info, CreateDeposit<'info>>,
        nonce: [u8; 32],
        params: CreateDepositParams,
    ) -> Result<()> {
        instructions::create_deposit(ctx, nonce, &params)
    }

    pub fn close_deposit(ctx: Context<CloseDeposit>, reason: String) -> Result<()> {
        instructions::close_deposit(ctx, &reason)
    }

    #[access_control(internal::Authenticate::only_order_keeper(&ctx))]
    pub fn execute_deposit<'info>(
        ctx: Context<'_, '_, 'info, 'info, ExecuteDeposit<'info>>,
        execution_fee: u64,
        throw_on_execution_error: bool,
    ) -> Result<()> {
        instructions::unchecked_execute_deposit(ctx, execution_fee, throw_on_execution_error)
    }

    pub fn prepare_withdrawal_escrow(
        ctx: Context<PrepareWithdrawalEscrow>,
        nonce: [u8; 32],
    ) -> Result<()> {
        instructions::prepare_withdrawal_escrow(ctx, nonce)
    }

    pub fn create_withdrawal<'info>(
        ctx: Context<'_, '_, 'info, 'info, CreateWithdrawal<'info>>,
        nonce: [u8; 32],
        params: CreateWithdrawalParams,
    ) -> Result<()> {
        instructions::create_withdrawal(ctx, nonce, &params)
    }

    pub fn close_withdrawal(ctx: Context<CloseWithdrawal>, reason: String) -> Result<()> {
        instructions::close_withdrawal(ctx, &reason)
    }

    #[access_control(internal::Authenticate::only_order_keeper(&ctx))]
    pub fn execute_withdrawal<'info>(
        ctx: Context<'_, '_, 'info, 'info, ExecuteWithdrawal<'info>>,
        execution_fee: u64,
        throw_on_execution_error: bool,
    ) -> Result<()> {
        instructions::unchecked_execute_withdrawal(ctx, execution_fee, throw_on_execution_error)
    }

    pub fn prepare_swap_order_escrow(
        ctx: Context<PrepareSwapOrderEscrow>,
        nonce: [u8; 32],
    ) -> Result<()> {
        instructions::prepare_swap_order_escrow(ctx, nonce)
    }

    pub fn prepare_increase_order_escrow(
        ctx: Context<PrepareIncreaseOrderEscrow>,
        nonce: [u8; 32],
    ) -> Result<()> {
        instructions::prepare_increase_order_escrow(ctx, nonce)
    }

    pub fn prepare_decrease_order_escrow(
        ctx: Context<PrepareDecreaseOrderEscrow>,
        nonce: [u8; 32],
    ) -> Result<()> {
        instructions::prepare_decrease_order_escrow(ctx, nonce)
    }

    pub fn prepare_position(
        ctx: Context<PreparePosition>,
        params: CreateOrderParams,
    ) -> Result<()> {
        instructions::prepare_position(ctx, &params)
    }

    pub fn create_order<'info>(
        ctx: Context<'_, '_, 'info, 'info, CreateOrder<'info>>,
        nonce: [u8; 32],
        params: CreateOrderParams,
    ) -> Result<()> {
        instructions::create_order(ctx, &nonce, &params)
    }

    pub fn close_order(ctx: Context<CloseOrder>, reason: String) -> Result<()> {
        instructions::close_order(ctx, &reason)
    }

    pub fn prepare_trade_event_buffer(
        ctx: Context<PrepareTradeEventBuffer>,
        index: u8,
    ) -> Result<()> {
        instructions::prepare_trade_event_buffer(ctx, index)
    }

    pub fn update_order(ctx: Context<UpdateOrder>, params: UpdateOrderParams) -> Result<()> {
        instructions::update_order(ctx, &params)
    }

    #[access_control(internal::Authenticate::only_order_keeper(&ctx))]
    pub fn execute_order<'info>(
        ctx: Context<'_, '_, 'info, 'info, ExecuteOrder<'info>>,
        recent_timestamp: i64,
        execution_fee: u64,
        throw_on_execution_error: bool,
    ) -> Result<()> {
        instructions::unchecked_execute_order(
            ctx,
            recent_timestamp,
            execution_fee,
            throw_on_execution_error,
        )
    }

    #[access_control(internal::Authenticate::only_order_keeper(&ctx))]
    pub fn execute_decrease_order<'info>(
        ctx: Context<'_, '_, 'info, 'info, ExecuteDecreaseOrder<'info>>,
        recent_timestamp: i64,
        execution_fee: u64,
        throw_on_execution_error: bool,
    ) -> Result<()> {
        instructions::unchecked_execute_decrease_order(
            ctx,
            recent_timestamp,
            execution_fee,
            throw_on_execution_error,
        )
    }

    #[access_control(internal::Authenticate::only_order_keeper(&ctx))]
    pub fn liquidate<'info>(
        ctx: Context<'_, '_, 'info, 'info, PositionCut<'info>>,
        nonce: [u8; 32],
        recent_timestamp: i64,
        execution_fee: u64,
    ) -> Result<()> {
        instructions::unchecked_process_position_cut(
            ctx,
            &nonce,
            recent_timestamp,
            PositionCutKind::Liquidate,
            execution_fee,
        )
    }

    /// Update the ADL state for the market.
    ///
    /// # Accounts.
    /// *[See the documentation for the accounts.](UpdateAdlState).*
    ///
    /// # Arguments
    /// - `is_long`: The market side to update for.
    ///
    /// # Errors
    /// - The [`authority`](UpdateAdlState::authority) must be a signer and a
    /// CONTROLLER of the store.
    /// - The [`store`](UpdateAdlState::store) must be an initialized [`Store`](states::Store)
    /// account owned by the store program.
    /// - The [`oracle`](UpdateAdlState::oracle) must be an initialized [`Oracle`](states::Oracle)
    /// account owned by the store program, and it must be owned by the store.
    /// - The [`market`](UpdateAdlState::market) must be enabled and owned by the store.
    #[access_control(internal::Authenticate::only_order_keeper(&ctx))]
    pub fn update_adl_state<'info>(
        ctx: Context<'_, '_, 'info, 'info, UpdateAdlState<'info>>,
        is_long: bool,
    ) -> Result<()> {
        instructions::unchecked_update_adl_state(ctx, is_long)
    }

    #[access_control(internal::Authenticate::only_order_keeper(&ctx))]
    pub fn auto_deleverage<'info>(
        ctx: Context<'_, '_, 'info, 'info, PositionCut<'info>>,
        nonce: [u8; 32],
        recent_timestamp: i64,
        size_delta_in_usd: u128,
        execution_fee: u64,
    ) -> Result<()> {
        instructions::unchecked_process_position_cut(
            ctx,
            &nonce,
            recent_timestamp,
            PositionCutKind::AutoDeleverage(size_delta_in_usd),
            execution_fee,
        )
    }

    pub fn prepare_shift_escrow(ctx: Context<PrepareShiftEscorw>, nonce: [u8; 32]) -> Result<()> {
        instructions::prepare_shift_escrow(ctx, nonce)
    }

    pub fn create_shift<'info>(
        ctx: Context<'_, '_, 'info, 'info, CreateShift<'info>>,
        nonce: [u8; 32],
        params: CreateShiftParams,
    ) -> Result<()> {
        instructions::create_shift(ctx, &nonce, &params)
    }

    /// Execute Shift.
    #[access_control(internal::Authenticate::only_order_keeper(&ctx))]
    pub fn execute_shift<'info>(
        ctx: Context<'_, '_, 'info, 'info, ExecuteShift<'info>>,
        execution_lamports: u64,
        throw_on_execution_error: bool,
    ) -> Result<()> {
        instructions::unchecked_execute_shift(ctx, execution_lamports, throw_on_execution_error)
    }

    pub fn close_shift(ctx: Context<CloseShift>, reason: String) -> Result<()> {
        instructions::close_shift(ctx, &reason)
    }

    /// Initialize GT Mint.
    #[access_control(internal::Authenticate::only_market_keeper(&ctx))]
    pub fn initialize_gt(
        ctx: Context<InitializeGt>,
        decimals: u8,
        initial_minting_cost: u128,
        grow_factor: u128,
        grow_step: u64,
        ranks: Vec<u64>,
    ) -> Result<()> {
        instructions::unchecked_initialize_gt(
            ctx,
            decimals,
            initial_minting_cost,
            grow_factor,
            grow_step,
            &ranks,
        )
    }

    /// Set order fee discount factors.
    #[access_control(internal::Authenticate::only_market_keeper(&ctx))]
    pub fn gt_set_order_fee_discount_factors(
        ctx: Context<ConfigurateGt>,
        factors: Vec<u128>,
    ) -> Result<()> {
        instructions::unchecked_gt_set_order_fee_discount_factors(ctx, &factors)
    }

    /// Set referral reward factors.
    #[access_control(internal::Authenticate::only_gt_controller(&ctx))]
    pub fn gt_set_referral_reward_factors(
        ctx: Context<ConfigurateGt>,
        factors: Vec<u128>,
    ) -> Result<()> {
        instructions::unchecked_gt_set_referral_reward_factors(ctx, &factors)
    }

    /// Set esGT receiver factor.
    #[access_control(internal::Authenticate::only_gt_controller(&ctx))]
    pub fn gt_set_es_receiver_factor(ctx: Context<ConfigurateGt>, factor: u128) -> Result<()> {
        instructions::unchecked_gt_set_es_receiver_factor(ctx, factor)
    }

    /// Set GT exchange time window (in seconds).
    #[access_control(internal::Authenticate::only_gt_controller(&ctx))]
    pub fn gt_set_exchange_time_window(ctx: Context<ConfigurateGt>, window: u32) -> Result<()> {
        instructions::unchecked_gt_set_exchange_time_window(ctx, window)
    }

    /// Set GT esGT vault receiver.
    #[access_control(internal::Authenticate::only_gt_controller(&ctx))]
    pub fn gt_set_receiver(ctx: Context<ConfigurateGt>, receiver: Pubkey) -> Result<()> {
        instructions::unchecked_gt_set_receiver(ctx, &receiver)
    }

    /// Prepare GT Exchange Vault.
    pub fn prepare_gt_exchange_vault(
        ctx: Context<PrepareGtExchangeVault>,
        time_window_index: i64,
        time_window: u32,
    ) -> Result<()> {
        instructions::prepare_gt_exchange_vault(ctx, time_window_index, time_window)
    }

    /// Confirm GT exchange vault.
    #[access_control(internal::Authenticate::only_gt_controller(&ctx))]
    pub fn confirm_gt_exchange_vault(ctx: Context<ConfirmGtExchangeVault>) -> Result<()> {
        instructions::unchecked_confirm_gt_exchange_vault(ctx)
    }

    /// Request a GT exchange.
    pub fn request_gt_exchange(ctx: Context<RequestGtExchange>, amount: u64) -> Result<()> {
        instructions::request_gt_exchange(ctx, amount)
    }

    /// Close a confirmed GT exchange.
    #[access_control(internal::Authenticate::only_gt_controller(&ctx))]
    pub fn close_gt_exchange(ctx: Context<CloseGtExchange>) -> Result<()> {
        instructions::unchecked_close_gt_exchange(ctx)
    }

    /// Claim esGT.
    pub fn claim_es_gt(ctx: Context<ClaimEsGt>) -> Result<()> {
        instructions::claim_es_gt(ctx)
    }

    /// Request GT vesting.
    pub fn request_gt_vesting(ctx: Context<RequestGtVesting>, amount: u64) -> Result<()> {
        instructions::request_gt_vesting(ctx, amount)
    }

    /// Update GT vesting.
    pub fn update_gt_vesting(ctx: Context<UpdateGtVesting>) -> Result<()> {
        instructions::update_gt_vesting(ctx)
    }

    /// Close GT vesting account.
    pub fn close_gt_vesting(ctx: Context<CloseGtVesting>) -> Result<()> {
        instructions::close_gt_vesting(ctx)
    }

    /// Claim esGT vault by vesting.
    pub fn claim_es_gt_vault_by_vesting(
        ctx: Context<ClaimEsGtVaultByVesting>,
        amount: u64,
    ) -> Result<()> {
        instructions::claim_es_gt_vault_by_vesting(ctx, amount)
    }

    /// Prepare User Account.
    pub fn prepare_user(ctx: Context<PrepareUser>) -> Result<()> {
        instructions::prepare_user(ctx)
    }

    /// Initialize referral code.
    pub fn initialize_referral_code(
        ctx: Context<InitializeReferralCode>,
        code: [u8; 8],
    ) -> Result<()> {
        instructions::initialize_referral_code(ctx, code)
    }

    /// Set referrer.
    pub fn set_referrer(ctx: Context<SetReferrer>, code: [u8; 8]) -> Result<()> {
        instructions::set_referrer(ctx, code)
    }

    /// Transfer referral code.
    pub fn transfer_referral_code(ctx: Context<TransferReferralCode>) -> Result<()> {
        instructions::transfer_referral_code(ctx)
    }

    // GLV instructions.
    /// Initialize GLV.
    #[access_control(internal::Authenticate::only_market_keeper(&ctx))]
    pub fn initialize_glv<'info>(
        ctx: Context<'_, '_, 'info, 'info, InitializeGlv<'info>>,
        index: u8,
        length: u16,
    ) -> Result<()> {
        instructions::unchecked_initialize_glv(ctx, index, length as usize)
    }

    /// GLV update market config.
    #[access_control(internal::Authenticate::only_market_keeper(&ctx))]
    pub fn update_glv_market_config(
        ctx: Context<UpdateGlvMarketConfig>,
        max_amount: Option<u64>,
        max_value: Option<u128>,
    ) -> Result<()> {
        instructions::unchecked_update_glv_market_config(ctx, max_amount, max_value)
    }

    /// Create GLV deposit.
    pub fn create_glv_deposit<'info>(
        ctx: Context<'_, '_, 'info, 'info, CreateGlvDeposit<'info>>,
        nonce: [u8; 32],
        params: CreateGlvDepositParams,
    ) -> Result<()> {
        instructions::create_glv_deposit(ctx, &nonce, &params)
    }

    /// Close GLV deposit.
    pub fn close_glv_deposit(ctx: Context<CloseGlvDeposit>, reason: String) -> Result<()> {
        instructions::close_glv_deposit(ctx, &reason)
    }

    #[access_control(internal::Authenticate::only_order_keeper(&ctx))]
    pub fn execute_glv_deposit<'info>(
        ctx: Context<'_, '_, 'info, 'info, ExecuteGlvDeposit<'info>>,
        execution_lamports: u64,
        throw_on_execution_error: bool,
    ) -> Result<()> {
        instructions::unchecked_execute_glv_deposit(
            ctx,
            execution_lamports,
            throw_on_execution_error,
        )
    }

    /// Create GLV withdrawal.
    pub fn create_glv_withdrawal<'info>(
        ctx: Context<'_, '_, 'info, 'info, CreateGlvWithdrawal<'info>>,
        nonce: [u8; 32],
        params: CreateGlvWithdrawalParams,
    ) -> Result<()> {
        instructions::create_glv_withdrawal(ctx, &nonce, &params)
    }

    /// Close GLV withdrawal.
    pub fn close_glv_withdrawal(ctx: Context<CloseGlvWithdrawal>, reason: String) -> Result<()> {
        instructions::close_glv_withdrawal(ctx, &reason)
    }

    /// Execute GLV withdrawal.
    #[access_control(internal::Authenticate::only_order_keeper(&ctx))]
    pub fn execute_glv_withdrawal<'info>(
        ctx: Context<'_, '_, 'info, 'info, ExecuteGlvWithdrawal<'info>>,
        execution_lamports: u64,
        throw_on_execution_error: bool,
    ) -> Result<()> {
        instructions::unchecked_execute_glv_withdrawal(
            ctx,
            execution_lamports,
            throw_on_execution_error,
        )
    }

    /// Create GLV shift.
    #[access_control(internal::Authenticate::only_order_keeper(&ctx))]
    pub fn create_glv_shift<'info>(
        mut ctx: Context<'_, '_, 'info, 'info, CreateGlvShift<'info>>,
        nonce: [u8; 32],
        params: CreateShiftParams,
    ) -> Result<()> {
        CreateGlvShift::create(&mut ctx, &nonce, &params)
    }

    /// Close a GLV shift.
    #[access_control(internal::Authenticate::only_order_keeper(&ctx))]
    pub fn close_glv_shift<'info>(
        ctx: Context<'_, '_, 'info, 'info, CloseGlvShift<'info>>,
        reason: String,
    ) -> Result<()> {
        CloseGlvShift::close(&ctx, &reason)
    }

    /// Execute GLV shift.
    #[access_control(internal::Authenticate::only_order_keeper(&ctx))]
    pub fn execute_glv_shift<'info>(
        ctx: Context<'_, '_, 'info, 'info, ExecuteGlvShift<'info>>,
        execution_lamports: u64,
        throw_on_execution_error: bool,
    ) -> Result<()> {
        instructions::unchecked_execute_glv_shift(ctx, execution_lamports, throw_on_execution_error)
    }
}

/// Result type with [`CoreError`] as error type.
pub type CoreResult<T> = std::result::Result<T, CoreError>;

#[error_code]
pub enum CoreError {
    /// Internal error.
    #[msg("internal error")]
    Internal,
    /// Not an Admin.
    #[msg("not an admin")]
    NotAnAdmin,
    /// Permission denied.
    #[msg("permission denied")]
    PermissionDenied,
    /// Feature disabled.
    #[msg("feature disabled")]
    FeatureDisabled,
    /// Model Error.
    #[msg("model")]
    Model,
    /// Invalid Argument.
    #[msg("invalid argument")]
    InvalidArgument,
    /// Preconditions are not met.
    #[msg("preconditions are not met")]
    PreconditionsAreNotMet,
    /// Not found.
    #[msg("not found")]
    NotFound,
    /// Exceed max length limit.
    #[msg("exceed max length limit")]
    ExceedMaxLengthLimit,
    /// Not enough space.
    #[msg("not enough space")]
    NotEnoughSpace,
    /// Token amount overflow.
    #[msg("token amount overflow")]
    TokenAmountOverflow,
    /// Value overflow.
    #[msg("value overflow")]
    ValueOverflow,
    /// Unknown Action State.
    #[msg("unknown action state")]
    UnknownActionState,
    /// Load account error.
    #[msg("load zero-copy account error")]
    LoadAccountError,
    /// Token account is not provided.
    #[msg("required token account is not provided")]
    TokenAccountNotProvided,
    /// Token mint is not provided.
    #[msg("required token mint is not provided")]
    TokenMintNotProvided,
    /// Market account is not provided.
    #[msg("market account is not provided")]
    MarketAccountIsNotProvided,
    /// Store Mismatched.
    #[msg("store mismatched")]
    StoreMismatched,
    /// Owner mismatched.
    #[msg("owner mismatched")]
    OwnerMismatched,
    /// Market mismatched.
    #[msg("market mismatched")]
    MarketMismatched,
    /// Market token mint mismatched.
    #[msg("market token mint mismatched")]
    MarketTokenMintMismatched,
    /// Mint account not provided.
    #[msg("mint account not provided")]
    MintAccountNotProvided,
    /// Market token account mismatched.
    #[msg("market token account mismatched")]
    MarketTokenAccountMismatched,
    /// Token mint mismatched.
    #[msg("token mint mismatched")]
    TokenMintMismatched,
    /// Token account mismatched.
    #[msg("token account mismatched")]
    TokenAccountMismatched,
    /// Not an ATA for the given token.
    #[msg("not an ATA for the given token")]
    NotAnATA,
    /// Not enough token amounts.
    #[msg("not enough token amount")]
    NotEnoughTokenAmount,
    /// Token amount exceeds limit.
    #[msg("token amount exceeds limit")]
    TokenAmountExceedsLimit,
    /// Unknown or disabled token.
    #[msg("unknown or disabled token")]
    UnknownOrDisabledToken,
    /// Not enough execution fee.
    #[msg("not enough execution fee")]
    NotEnoughExecutionFee,
    /// Invalid Swap Path length.
    #[msg("invalid swap path length")]
    InvalidSwapPathLength,
    /// Not enough swap markets in the path.
    #[msg("not enough swap markets in the path")]
    NotEnoughSwapMarkets,
    /// Invalid Swap Path.
    #[msg("invalid swap path")]
    InvalidSwapPath,
    /// Insufficient output amounts.
    #[msg("insufficient output amounts")]
    InsufficientOutputAmount,
    /* Errors for Store */
    /// Invalid Store Config Key.
    #[msg("invalid store config key")]
    InvalidStoreConfigKey,
    /* Errors for Oracle */
    /// Invalid Provider Kind Index.
    #[msg("invalid provider kind index")]
    InvalidProviderKindIndex,
    /// Not enough token feeds.
    #[msg("not enough token feeds")]
    NotEnoughTokenFeeds,
    /// Oracle timestamps are larger than required.
    #[msg("oracle timestamps are larger than required")]
    OracleTimestampsAreLargerThanRequired,
    /// Oracle timestamps are smaller than required.
    #[msg("oracle timestamps are smaller than required")]
    OracleTimestampsAreSmallerThanRequired,
    /// Invalid Oracle timestamps range.
    #[msg("invalid oracle timestamps range")]
    InvalidOracleTimestampsRange,
    /// Max oracle timestamps range exceeded.
    #[msg("max oracle timestamps range exceeded")]
    MaxOracleTimestampsRangeExceeded,
    /// Oracle not updated.
    #[msg("oracle not updated")]
    OracleNotUpdated,
    /// Max price age exceeded.
    #[msg("max price age exceeded")]
    MaxPriceAgeExceeded,
    /// Invalid Oracle slot.
    #[msg("invalid oracle slot")]
    InvalidOracleSlot,
    /// Missing oracle price.
    #[msg("missing oracle price")]
    MissingOraclePrice,
    /// Invalid Price feed price.
    #[msg("invalid price feed price")]
    InvalidPriceFeedPrice,
    /// Invalid price feed account.
    #[msg("invalid price feed account")]
    InvalidPriceFeedAccount,
    /// Price feed is not updated.
    #[msg("price feed is not updated")]
    PriceFeedNotUpdated,
    /// Prices are already set.
    #[msg("prices are already set")]
    PricesAreAlreadySet,
    /// Price is already set.
    #[msg("price is already set")]
    PriceIsAlreadySet,
    /// Token config is diabled.
    #[msg("token config is disabled")]
    TokenConfigDisabled,
    /* Errors for Deposit */
    /// Empty Deposit.
    #[msg("empty deposit")]
    EmptyDeposit,
    /// Invalid owner for the first deposit.
    #[msg("invalid owner for the first deposit")]
    InvalidOwnerForFirstDeposit,
    /// Not enough market token amount for the first deposit.
    #[msg("not enough market token amount for the first deposit")]
    NotEnoughMarketTokenAmountForFirstDeposit,
    /// Not enough GLV token amount for the first deposit.
    #[msg("not enough GLV token amount for the first deposit")]
    NotEnoughGlvTokenAmountForFirstDeposit,
    /* Errors for Withdrawal */
    /// Empty Withdrawal.
    #[msg("emtpy withdrawal")]
    EmptyWithdrawal,
    /* Errors for Orders */
    /// Empty Order.
    #[msg("emtpy order")]
    EmptyOrder,
    /// Invalid min output amount for limit swap.
    #[msg("invalid min output amount for limit swap order")]
    InvalidMinOutputAmount,
    /// Invalid trigger price.
    #[msg("invalid trigger price")]
    InvalidTriggerPrice,
    /// Invalid position.
    #[msg("invalid position")]
    InvalidPosition,
    /// Invalid position kind.
    #[msg("invalid position kind")]
    InvalidPositionKind,
    /// Position mismatched.
    #[msg("position mismatched")]
    PositionMismatched,
    /// Position is not required.
    #[msg("position is not required")]
    PositionItNotRequired,
    /// Position is required.
    #[msg("position is required")]
    PositionIsRequired,
    /// Order kind is not allowed.
    #[msg("the order kind is not allowed by this instruction")]
    OrderKindNotAllowed,
    /// Unknown Order Kind.
    #[msg("unknown order kind")]
    UnknownOrderKind,
    /// Unknown Order Side.
    #[msg("unknown order side")]
    UnknownOrderSide,
    /// Missing initial collateral token.
    #[msg("missing initial collateral token")]
    MissingInitialCollateralToken,
    /// Missing final output token.
    #[msg("missing final output token")]
    MissingFinalOutputToken,
    /// Missing pool tokens.
    #[msg("missing pool tokens")]
    MissingPoolTokens,
    /// Invalid Trade ID.
    #[msg("invalid trade ID")]
    InvalidTradeID,
    /// Invalid Trade delta size.
    #[msg("invalid trade delta size")]
    InvalidTradeDeltaSize,
    /// Invalid Trade delta tokens.
    #[msg("invalid trade delta tokens")]
    InvalidTradeDeltaTokens,
    /// Invalid Borrowing Factor.
    #[msg("invalid borrowing factor")]
    InvalidBorrowingFactor,
    /// Invalid funding factors.
    #[msg("invalid funding factors")]
    InvalidFundingFactors,
    /// No delegated authority is set.
    #[msg("no delegated authority is set")]
    NoDelegatedAuthorityIsSet,
    /// Claimable collateral for holding cannot be in output tokens.
    #[msg("claimable collateral for holding cannot be in output tokens")]
    ClaimableCollateralForHoldingCannotBeInOutputTokens,
    /// ADL is not enabled.
    #[msg("ADL is not enabled")]
    AdlNotEnabled,
    /// ADL is not required.
    #[msg("ADL is not required")]
    AdlNotRequired,
    /// Invalid ADL.
    #[msg("invalid ADL")]
    InvalidAdl,
    /// The output token and the secondary output token are the same,
    /// but the token amounts are not merged togather.
    #[msg("same output tokens not merged")]
    SameOutputTokensNotMerged,
    /* Errors for Shift */
    /// Empty Shift.
    #[msg("emtpy shift")]
    EmptyShift,
    /// Invalid Shift Markets
    #[msg("invalid shift markets")]
    InvalidShiftMarkets,
    /* Errors for GT and User accounts */
    /// GT State has been initialized.
    #[msg("GT State has been initialized")]
    GTStateHasBeenInitialized,
    /// Invalid GT config.
    #[msg("invalid GT config")]
    InvalidGTConfig,
    /// Invalid GT discount.
    #[msg("invalid GT discount")]
    InvalidGTDiscount,
    /// User account has been initialized.
    #[msg("user account has been initialized")]
    UserAccountHasBeenInitialized,
    /* Errors for referral */
    /// Referral Code has been set.
    #[msg("referral code has been set")]
    ReferralCodeHasBeenSet,
    /// Referrer has been set.
    #[msg("referrer has been set")]
    ReferrerHasBeenSet,
    /// Invalid User Account.
    #[msg("invalid user account")]
    InvalidUserAccount,
    /// Referral Code Mismatched.
    #[msg("referral code mismatched")]
    ReferralCodeMismatched,
    /// Self-referral is not allowed.
    #[msg("self-referral is not allowed")]
    SelfReferral,
    /// Mutual-referral is not allowed.
    #[msg("mutual-referral is not allowed")]
    MutualReferral,
    /* Errors for market */
    /// Invalid market config key.
    #[msg("invalid market config key")]
    InvalidMarketConfigKey,
    /// Invalid collateral token.
    #[msg("invalid collateral token")]
    InvalidCollateralToken,
    /// Disabled market.
    #[msg("disabled market")]
    DisabledMarket,
    /* Errors for GLV */
    /// Failed to calculate GLV value for market.
    #[msg("failed to calculate GLV value for this market")]
    FailedToCalculateGlvValueForMarket,
    /// Failed to calculate GLV amount to mint.
    #[msg("failed to calculate GLV amount to mint")]
    FailedToCalculateGlvAmountToMint,
    /// Failed to calculate market token amount to burn.
    FailedTOCalculateMarketTokenAmountToBurn,
    /// Exceed max market token balance amount of GLV.
    #[msg("GLV max market token balance amount exceeded")]
    ExceedMaxGlvMarketTokenBalanceAmount,
    /// Exceed max market token balance value of GLV.
    #[msg("GLV max market token balance value exceeded")]
    ExceedMaxGlvMarketTokenBalanceValue,
    /// Empty GLV withdrawal.
    #[msg("Empty GLV withdrawal")]
    EmptyGlvWithdrawal,
}

impl CoreError {
    pub(crate) const fn unknown_action_state(_kind: u8) -> Self {
        Self::UnknownActionState
    }

    pub(crate) const fn unknown_order_kind(_kind: u8) -> Self {
        Self::UnknownOrderKind
    }

    pub(crate) const fn unknown_order_side(_kind: u8) -> Self {
        Self::UnknownOrderSide
    }

    pub(crate) const fn invalid_position_kind(_kind: u8) -> Self {
        Self::InvalidPositionKind
    }
}
