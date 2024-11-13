//! # The GMSOL Store Program
//!
//! ## Store
//!
//! A [`Store`](states::Store) Account serves as both an authority and a global configuration
//! storage.
//!
//! #### Instructions for Store Accounts
//! - [`initialize`](gmsol_store::initialize): Create a new [`Store`](states::Store) account.
//! - [`transfer_store_authority`]: Transfer the authority of the given store to a new authority.
//! - [`set_receiver`](gmsol_store::set_receiver): Set the claimable fee receiver address.
//! - [`set_token_map`]: Set the token map account to use.
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
//! #### Instructions for Permission Management
//! - [`check_admin`](gmsol_store::check_admin): Check that the signer is the admin of the given store,
//!   throw error if the check fails.
//! - [`check_role`](gmsol_store::check_role): Check that the signer has the given role in the given store,
//!   throw error if the check fails.
//! - [`has_admin`](gmsol_store::has_admin): Return whether the given address is the admin of the given store,
//!   or not.
//! - [`has_role`](gmsol_store::has_role): Return whether the given address has the given role in the given store,
//!   or not.
//! - [`enable_role`]: Insert or enable a role for the given store.
//! - [`disable_role`]: Disable an existing role for the given store.
//! - [`grant_role`]: Grant a role to the given user in the given store.
//! - [`revoke_role`]: Revoke a role from the given user in the given store.
//!
//! #### Instructions for Config Management
//! - [`insert_amount`]: Insert an amount to the global config.
//! - [`insert_factor`]: Insert a factor to the global config.
//! - [`insert_address`]: Insert an address to the global config.
//! - [`insert_gt_minting_cost_referred_discount`]:
//!   Insert GT miniting cost referred discount factor to the global config.
//!
//! #### Instructions for Feature Management
//! - [`toggle_feature`]: Enable or diable the given feature.
//!
//! ## Token Config and Oracle Management
//!
//! #### Instructions for managing [`TokenConfig`](states::TokenConfig) and token maps.
//! - [`initialize_token_map`](gmsol_store::initialize_token_map): Initialize a new token map account.
//!   This is a permissionless instruction.
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
//! #### Instructions for [`Oracle`](states::Oracle) accounts
//! - [`initialize_oracle`](gmsol_store::initialize_oracle): Initialize a new [`Oracle`](states::Oracle) account.
//! - [`clear_all_prices`]: Clear the prices of the given oracle account.
//! - [`set_prices_from_price_feed`](gmsol_store::set_prices_from_price_feed): Validate and set prices parsed from the
//!   provided price feed accounts.
//! - [`initialize_price_feed`]: Initialize a custom price feed.
//! - [`update_price_feed_with_chainlink`]: Update a custom Chainlink price feed with Chainlink Data Streams report.
//!
//! ## Market Management
//! The instructions related to market management are as follows:
//!
//! #### Instructions for [`Market`](states::Market) management
//! - [`initialize_market`]: Initialize a [`Market`](states::Market) account.
//! - [`toggle_market`]: Enable or diable the given market.
//! - [`market_transfer_in`]: Transfer tokens into the market and record the amount in its balance.
//! - [`update_market_config`]: Update an item in the market config.
//! - [`update_market_config_with_buffer`]: Update the market config with the given
//!   [`MarketConfigBuffer`](states::market::config::MarketConfigBuffer) account.
//! - [`get_market_status`](gmsol_store::get_market_status): Calculate the market status with the given prices.
//! - [`get_market_token_price`](gmsol_store::get_market_token_price): Calculate the market token price the given prices.
//! - [`toggle_gt_minting`]: Enable or diable GT minting for the given market.
//!
//! #### Instructions for [`MarketConfigBuffer`](states::market::config::MarketConfigBuffer) accounts
//! - [`initialize_market_config_buffer`](gmsol_store::initialize_market_config_buffer): Initialize a market config buffer account.
//! - [`set_market_config_buffer_authority`](gmsol_store::set_market_config_buffer_authority): Replace the authority of the market
//!   config buffer account with the new one.
//! - [`close_market_config_buffer`](gmsol_store::close_market_config_buffer): Close the given market config buffer account.
//! - [`push_to_market_config_buffer`](gmsol_store::push_to_market_config_buffer): Push config items to the given market config
//!   buffer account.
//!
//! #### Instructions for token accounts
//! - [`initialize_market_vault`]: Initialize the market vault for the given token.
//! - [`use_claimable_account`]: Prepare a claimable account to receive tokens during the order execution.
//! - [`close_empty_claimable_account`]: Close a empty claimble account.
//! - [`prepare_associated_token_account`](gmsol_store::prepare_associated_token_account): Prepare an ATA.
//!
//! ## Exchange
//! The instructions for providing functionalities as an exchange are as follows:
//!
//! #### Instructions for [`Deposit`](states::Deposit)
//! - [`create_deposit`]: Create a deposit by the owner.
//! - [`execute_deposit`](gmsol_store::execute_deposit()): Execute a deposit by keepers.
//! - [`close_deposit`]: Close a deposit, either by the owner or by keepers.
//!
//! #### Instructions for [`Withdrawal`](states::Withdrawal)
//! - [`create_withdrawal`]: Create a withdrawal by the owner.
//! - [`execute_withdrawal`](gmsol_store::execute_withdrawal()): Execute a withdrawal by keepers.
//! - [`close_withdrawal`]: Close a withdrawal, either by the owner or by keepers.
//!
//! #### Instructions for [`Shift`](states::Shift)
//! - [`create_shift`]: Create a shift by the owner.
//! - [`execute_shift`](gmsol_store::execute_shift()): Execute a shift by keepers.
//! - [`close_shift`]: Close a shift, either by the owner or by keepers.
//!
//! #### Instructions for [`Order`](states::Order) and [`Position`](states::Position)
//! - [`prepare_position`](gmsol_store::prepare_position): Prepare the position account for orders.
//! - [`prepare_trade_event_buffer`](gmsol_store::prepare_trade_event_buffer): Prepare trade event buffer.
//! - [`create_order`]: Create an order by the owner.
//! - [`update_order`](gmsol_store::update_order): Update an order by the owner.
//! - [`execute_increase_or_swap_order`](gmsol_store::execute_increase_or_swap_order()): Execute an order by keepers.
//! - [`execute_decrease_order`]: Execute a decrease order by keepers.
//! - [`close_order`]: Close an order, either by the owner or by keepers.
//! - [`liquidate`]: Perform a liquidation by keepers.
//! - [`auto_deleverage`]: Perform an ADL by keepers.
//! - [`update_adl_state`]: Update the ADL state of the market.
//!
//! ## GLV Model
//! The instructions for providing functionalities for GLV are as follows:
//!
//! #### Instructions for [`Glv`](states::Glv).
//! - [`initialize_glv`]: Initialize a GLV.
//! - [`update_glv_market_config`]: Update GLV market config.
//!
//! #### Instructions for [`GlvDeposit`](states::GlvDeposit)
//! - [`create_glv_deposit`]: Create a GLV deposit by the owner.
//! - [`execute_glv_deposit`]: Execute a GLV deposit by keepers.
//! - [`close_glv_deposit`]: Close a GLV deposit, either by the owner or by keepers.
//!
//! #### Instructions for [`GlvWithdrawal`](states::glv::GlvWithdrawal)
//! - [`create_glv_withdrawal`]: Create a GLV withdrawal by the owner.
//! - [`execute_glv_withdrawal`]: Execute a GLV withdrawal by keepers.
//! - [`close_glv_withdrawal`]: Close a GLV withdrawal, either by the owner or by keepers.
//!
//! #### Instructions for [`GlvShift`](states::glv::GlvShift)
//! - [`create_glv_shift`]: Create a GLV shift by keepers.
//! - [`execute_glv_shift`]: Execute a GLV shift by keepers.
//! - [`close_glv_shift`]: Close a shift by keepers.
//!
//! ## User and Referral
//! The instructions for user accounts and referrals are as follows:
//! - [`prepare_user`](gmsol_store::prepare_user): Prepare a user account.
//! - [`initialize_referral_code`](gmsol_store::initialize_referral_code): Initialize and set a referral code.
//! - [`set_referrer`](gmsol_store::set_referrer): Set the referrer.
//! - [`transfer_referral_code`](gmsol_store::transfer_referral_code): Transfer the referral code to others.
//!
//! ## GT Model
//! The instructions for GT Model are as follows:
//! - [`initialize_gt`]: Initialize the GT state.
//! - [`gt_set_order_fee_discount_factors`]: Set order fee discount factors.
//! - [`gt_set_referral_reward_factors`]: Set referral reward factors.
//! - [`gt_set_es_receiver_factor`]: Set esGT receiver factor.
//! - [`gt_set_exchange_time_window`]: Set GT exchange time window.
//! - [`gt_set_receiver`]: Set esGT vault receiver.
//! - [`prepare_gt_exchange_vault`](gmsol_store::prepare_gt_exchange_vault): Prepare current GT exchange vault.
//! - [`confirm_gt_exchange_vault`]: Confirm GT exchange vault.
//! - [`request_gt_exchange`](gmsol_store::request_gt_exchange): Request a GT exchange.
//! - [`close_gt_exchange`]: Close a confirmed GT exchange.
//! - [`claim_es_gt`](gmsol_store::claim_es_gt): Claim esGT.
//! - [`request_gt_vesting`](gmsol_store::request_gt_vesting): Request GT vesting.
//! - [`update_gt_vesting`](gmsol_store::update_gt_vesting): Update GT vesting state.
//! - [`close_gt_vesting`](gmsol_store::close_gt_vesting): Close an empty GT vesting.
//! - [`claim_es_gt_vault_via_vesting`](gmsol_store::claim_es_gt_vault_via_vesting): Claim esGT vault via vesting.

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
        market::{config::EntryArgs, status::MarketStatus},
        order::UpdateOrderParams,
        token_config::TokenConfigBuilder,
        FactorKey, PriceProviderKind,
    },
    utils::internal,
};
use anchor_lang::prelude::*;
use gmsol_model::price::Prices;

#[cfg_attr(test, macro_use)]
extern crate static_assertions;

declare_id!("gmX4GEZycT14vqJ3yDoCA5jW53vBaSQpQDYNDXtkWt1");

#[program]
/// Instructions definitions of the GMSOL Store Program.
pub mod gmsol_store {

    use super::*;

    // ===========================================
    //                 Data Store
    // ===========================================

    /// Create a new [`Store`](states::Store) account.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](Initialize).*
    ///
    /// # Arguments
    /// - `key`: The name of the store, used as a seed to derive the store account's address.
    ///   The length must not exceed [`MAX_LEN`](states::Store::MAX_LEN).
    /// - `authority`: The authority (admin) address to set for the new Store. If not provided,
    ///   the [`payer`](Initialize::payer) will be used as the authority.
    ///
    /// # Errors
    /// - The `key` must be empty unless the `multi-store` feature is enabled
    /// - The [`payer`](Initialize::payer) must be a signer
    /// - The [`store`](Initialize::store) must not be initialized
    /// - The [`store`](Initialize::store) address must match the PDA derived from
    ///   [`SEED`](states::Store::SEED) and the SHA-256 hash of `key`
    pub fn initialize(
        ctx: Context<Initialize>,
        key: String,
        authority: Option<Pubkey>,
    ) -> Result<()> {
        instructions::initialize(ctx, key, authority)
    }

    /// Transfer the authority (admin) of the given store to a new address.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](TransferStoreAuthority).*
    ///
    /// # Arguments
    /// - `new_authority`: The new authority address that will become the admin of the store.
    ///
    /// # Errors
    /// - The [`authority`](TransferStoreAuthority::authority) must be a signer and the current
    ///   admin of the store.
    /// - The [`store`](TransferStoreAuthority::store) must be an initialized store account
    ///   owned by the store program.
    /// - The `new_authority` cannot be the same as the current authority.
    #[access_control(internal::Authenticate::only_admin(&ctx))]
    pub fn transfer_store_authority(
        ctx: Context<TransferStoreAuthority>,
        new_authority: Pubkey,
    ) -> Result<()> {
        instructions::unchecked_transfer_store_authority(ctx, new_authority)
    }

    /// Set the receiver address for claiming fees.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](SetReceiver).*
    ///
    /// # Errors
    /// - The [`authority`](SetReceiver::authority) must be a signer and the current admin
    ///   of the given store.
    /// - The [`store`](SetReceiver::store) must be an initialized store account owned by
    ///   the store program.
    /// - The new [`receiver`](SetReceiver::receiver) account provided cannot be the same as
    ///   the current receiver.
    pub fn set_receiver(ctx: Context<SetReceiver>) -> Result<()> {
        instructions::set_receiver(ctx)
    }

    /// Set the token map address for the store.
    ///
    /// This instruction allows a MARKET_KEEPER to update which token map account the store uses.
    /// The token map account contains token configurations and price feed configurations.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](SetTokenMap).*
    ///
    /// # Errors
    /// - The [`authority`](SetTokenMap::authority) must be a signer and have the MARKET_KEEPER
    ///   role in the store.
    /// - The [`store`](SetTokenMap::store) must be an initialized store account owned by the
    ///   store program.
    /// - The [`token_map`](SetTokenMap::token_map) must be an initialized token map account
    ///   and owned by the given store.
    /// - The new token map address cannot be the same as the current one.
    #[access_control(internal::Authenticate::only_market_keeper(&ctx))]
    pub fn set_token_map(ctx: Context<SetTokenMap>) -> Result<()> {
        instructions::unchecked_set_token_map(ctx)
    }

    // ===========================================
    //      Role-based Permission Management
    // ===========================================

    /// Return whether the signer is the admin of the given store.
    ///
    /// This instruction verifies if the signer has administrator privileges for the given store
    /// and returns a boolean result.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](CheckRole).*
    ///
    /// # Returns
    /// Returns `true` if the signer is the admin, `false` otherwise.
    ///
    /// # Errors
    /// - The [`authority`](CheckRole::authority) must be a signer.
    /// - The [`store`](CheckRole::store) must be an initialized store account owned by
    ///   the store program.
    pub fn check_admin(ctx: Context<CheckRole>) -> Result<bool> {
        instructions::check_admin(ctx)
    }

    /// Check that the authority has the given role in the given store.
    ///
    /// This instruction verifies if the authority has the specified role in the given store
    /// and returns a boolean result.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](CheckRole).*
    ///
    /// # Arguments
    /// - `role`: The name of the role to check for the authority.
    ///
    /// # Returns
    /// Returns `true` if the authority has the role, `false` otherwise.
    ///
    /// # Errors
    /// - The [`authority`](CheckRole::authority) must be a signer.
    /// - The [`store`](CheckRole::store) must be an initialized store account owned by
    ///   the store program.
    /// - The `role` must exist and be enabled in the store's role configuration.
    pub fn check_role(ctx: Context<CheckRole>, role: String) -> Result<bool> {
        instructions::check_role(ctx, role)
    }

    /// Return whether the given address is the administrator of the given store.
    ///
    /// This instruction checks if the provided address has administrator privileges for the given store
    /// and returns a boolean result.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](HasRole).*
    ///
    /// # Arguments
    /// - `authority`: The address to check for administrator privileges.
    ///
    /// # Returns
    /// Returns `true` if the address is the administrator, `false` otherwise.
    ///
    /// # Errors
    /// - The [`store`](HasRole::store) must be an initialized store account owned by
    ///   the store program.
    pub fn has_admin(ctx: Context<HasRole>, authority: Pubkey) -> Result<bool> {
        instructions::has_admin(ctx, authority)
    }

    /// Return whether the given address has the given role in the given store.
    ///
    /// This instruction checks if the provided address has the specified role in the given store
    /// and returns a boolean result.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](HasRole).*
    ///
    /// # Arguments
    /// - `authority`: The address to check for role membership.
    /// - `role`: The name of the role to check for the authority.
    ///
    /// # Returns
    /// Returns `true` if the address has the specified role, `false` otherwise.
    ///
    /// # Errors
    /// - The [`store`](HasRole::store) must be an initialized store account owned by
    ///   the store program.
    /// - The `role` must exist and be enabled in the store's role configuration.
    pub fn has_role(ctx: Context<HasRole>, authority: Pubkey, role: String) -> Result<bool> {
        instructions::has_role(ctx, authority, role)
    }

    /// Insert or enable a role for the given store.
    ///
    /// This instruction adds a new role or enables an existing disabled role in the store's role configuration.
    /// If the role already exists and is enabled, this instruction has no effect.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](EnableRole).*
    ///
    /// # Arguments
    /// - `role`: The name of the role to be added/enabled. The length cannot exceed
    ///   [`MAX_ROLE_NAME_LEN`](states::roles::MAX_ROLE_NAME_LEN).
    ///
    /// # Errors
    /// - The [`authority`](EnableRole::authority) must be a signer and be the `ADMIN` of the store.
    /// - The [`store`](EnableRole::store) must be an initialized store account owned by the store program.
    /// - The `role` name length must not exceed [`MAX_ROLE_NAME_LEN`](states::roles::MAX_ROLE_NAME_LEN).
    #[access_control(internal::Authenticate::only_admin(&ctx))]
    pub fn enable_role(ctx: Context<EnableRole>, role: String) -> Result<()> {
        instructions::unchecked_enable_role(ctx, role)
    }

    /// Disable an existing role for the given store.
    ///
    /// This instruction disables an existing role in the store's role configuration.
    /// If the role does not exist in the store, this instruction has no effect.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](DisableRole).*
    ///
    /// # Arguments
    /// - `role`: The name of the role to be disabled.
    ///
    /// # Errors
    /// - The [`authority`](DisableRole::authority) must be a signer and be the `ADMIN` of the store.
    /// - The [`store`](DisableRole::store) must be an initialized store account owned by the store program.
    #[access_control(internal::Authenticate::only_admin(&ctx))]
    pub fn disable_role(ctx: Context<DisableRole>, role: String) -> Result<()> {
        instructions::unchecked_disable_role(ctx, role)
    }

    /// Grant a role to the given user in the given store.
    ///
    /// This instruction grants a role to a user in the store's role configuration. If the user already
    /// has the role, this instruction has no effect.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](GrantRole).*
    ///
    /// # Arguments
    /// - `user`: The address of the user to whom the role should be granted.
    /// - `role`: The name of the role to be granted. Must be an enabled role in the store.
    ///
    /// # Errors
    /// - The [`authority`](GrantRole::authority) must be a signer and be the `ADMIN` of the store.
    /// - The [`store`](GrantRole::store) must be an initialized store account owned by the store program.
    /// - The `role` must exist and be enabled in the store's role configuration.
    #[access_control(internal::Authenticate::only_admin(&ctx))]
    pub fn grant_role(ctx: Context<GrantRole>, user: Pubkey, role: String) -> Result<()> {
        instructions::unchecked_grant_role(ctx, user, role)
    }

    /// Revoke a role from the given user in the given store.
    ///
    /// This instruction revokes a role from a user in the store's role configuration. If the user does
    /// not have the role, this instruction has no effect.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](RevokeRole).*
    ///
    /// # Arguments
    /// - `user`: The address of the user from whom the role should be revoked.
    /// - `role`: The name of the role to be revoked. Must be an enabled role in the store.
    ///
    /// # Errors
    /// - The [`authority`](RevokeRole::authority) must be a signer and be the `ADMIN` of the store.
    /// - The [`store`](RevokeRole::store) must be an initialized store account owned by the store program.
    /// - The `role` must exist and be enabled in the store's role configuration.
    /// - The `user` must exist in the store's member table.
    #[access_control(internal::Authenticate::only_admin(&ctx))]
    pub fn revoke_role(ctx: Context<RevokeRole>, user: Pubkey, role: String) -> Result<()> {
        instructions::unchecked_revoke_role(ctx, user, role)
    }

    // ===========================================
    //              Config Management
    // ===========================================

    /// Insert an amount value into the store's global configuration.
    ///
    /// This instruction allows a CONFIG_KEEPER to set or update an amount value in the store's
    /// configuration. The key must be one of the predefined amount keys.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](InsertConfig).*
    ///
    /// # Arguments
    /// - `key`: The configuration key to update. Must be a valid amount key defined in
    ///   [`AmountKey`](crate::states::AmountKey).
    /// - `amount`: The amount value to store for this configuration key.
    ///
    /// # Errors
    /// - The [`authority`](InsertConfig::authority) must be a signer and have the CONFIG_KEEPER role
    ///   in the store.
    /// - The provided `key` must be defined in [`AmountKey`](crate::states::AmountKey).
    /// - The store must be initialized and owned by this program.
    #[access_control(internal::Authenticate::only_config_keeper(&ctx))]
    pub fn insert_amount(ctx: Context<InsertConfig>, key: String, amount: u64) -> Result<()> {
        instructions::unchecked_insert_amount(ctx, &key, amount)
    }

    /// Insert a factor value into the store's global configuration.
    /// This instruction allows a CONFIG_KEEPER to set or update a factor value in the store's
    /// configuration. The key must be one of the predefined factor keys.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](InsertConfig).*
    ///
    /// # Arguments
    /// - `key`: The configuration key to update. Must be a valid factor key defined in
    ///   [`FactorKey`](crate::states::FactorKey).
    /// - `factor`: The factor value to store for this configuration key.
    ///
    /// # Errors
    /// - The [`authority`](InsertConfig::authority) must be a signer and have the CONFIG_KEEPER role
    ///   in the store.
    /// - The provided `key` must be defined in [`FactorKey`](crate::states::FactorKey).
    /// - The store must be initialized and owned by this program.
    #[access_control(internal::Authenticate::only_config_keeper(&ctx))]
    pub fn insert_factor(ctx: Context<InsertConfig>, key: String, factor: u128) -> Result<()> {
        instructions::unchecked_insert_factor(ctx, &key, factor)
    }

    /// Insert an address value into the store's global configuration.
    ///
    /// This instruction allows a CONFIG_KEEPER to set or update an address value in the store's
    /// configuration. The key must be one of the predefined address keys.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](InsertConfig).*
    ///
    /// # Arguments
    /// - `key`: The configuration key to update. Must be a valid address key defined in
    ///   [`AddressKey`](crate::states::AddressKey).
    /// - `address`: The address value to store for this configuration key.
    ///
    /// # Errors
    /// - The [`authority`](InsertConfig::authority) must be a signer and have the CONFIG_KEEPER role
    ///   in the store.
    /// - The provided `key` must be defined in [`AddressKey`](crate::states::AddressKey).
    /// - The store must be initialized and owned by this program.
    #[access_control(internal::Authenticate::only_config_keeper(&ctx))]
    pub fn insert_address(ctx: Context<InsertConfig>, key: String, address: Pubkey) -> Result<()> {
        instructions::unchecked_insert_address(ctx, &key, address)
    }

    /// Insert GT minting cost referred discount factor to the global config.
    ///
    /// This instruction allows a MARKET_KEEPER to set or update the GT minting cost referred
    /// discount factor in the store's configuration. This factor determines the discount
    /// applied to GT minting costs for referred users.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](InsertConfig).*
    ///
    /// # Arguments
    /// - `factor`: The discount factor value to set.
    ///
    /// # Errors
    /// - The [`authority`](InsertConfig::authority) must be a signer and have the
    ///   MARKET_KEEPER role in the store.
    /// - The store must be initialized and owned by this program.
    ///
    /// # Notes
    /// - While [`insert_factor`] can also modify this value, it requires CONFIG_KEEPER
    ///   permissions instead of MARKET_KEEPER permissions required by this instruction.
    /// - The factor is stored under the [`FactorKey::GtMintingCostReferredDiscount`] key.
    #[access_control(internal::Authenticate::only_market_keeper(&ctx))]
    pub fn insert_gt_minting_cost_referred_discount(
        ctx: Context<InsertConfig>,
        factor: u128,
    ) -> Result<()> {
        let key = FactorKey::GtMintingCostReferredDiscount;
        instructions::unchecked_insert_factor(ctx, &key.to_string(), factor)
    }

    // ===========================================
    //             Feature Management
    // ===========================================

    /// Enable or disable a feature in the store.
    ///
    /// This instruction allows a FEATURE_KEEPER to toggle specific features on or off by providing
    /// a domain and action combination. Features are used to control which functionality is available
    /// in the store.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](ToggleFeature).*
    ///
    /// # Arguments
    /// - `domain`: The domain part of the feature flag, must be a valid domain defined in
    ///   [`DomainDisabledFlag`](crate::states::feature::DomainDisabledFlag).
    /// - `action`: The action part of the feature flag, must be a valid action defined in
    ///   [`ActionDisabledFlag`](crate::states::feature::ActionDisabledFlag).
    /// - `enable`: If true, enables the feature. If false, disables it.
    ///
    /// # Errors
    /// - The [`authority`](ToggleFeature::authority) must be a signer and have the
    ///   FEATURE_KEEPER role in the store.
    /// - The `domain` must be a valid domain defined in [`DomainDisabledFlag`](crate::states::feature::DomainDisabledFlag).
    /// - The `action` must be a valid action defined in [`ActionDisabledFlag`](crate::states::feature::ActionDisabledFlag).
    #[access_control(internal::Authenticate::only_feature_keeper(&ctx))]
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

    // ===========================================
    //           Token Config Management
    // ===========================================

    /// Initialize a new token map account with its store set to [`store`](InitializeTokenMap::store).
    ///
    /// Anyone can initialize a token map account without any permissions, but after initialization, only
    /// addresses authorized by the store can modify this token map.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](InitializeTokenMap)*
    ///
    /// # Errors
    /// - The [`payer`](InitializeTokenMap::payer) must be a signer.
    /// - The [`store`](InitializeTokenMap::store) must be an initialized [`Store`](states::Store)
    ///   account owned by the store program.
    /// - The [`token_map`](InitializeTokenMap::token_map) must be an uninitialized account.
    pub fn initialize_token_map(ctx: Context<InitializeTokenMap>) -> Result<()> {
        instructions::initialize_token_map(ctx)
    }

    /// Push a new token config to the given token map.
    ///
    /// This instruction is used to add or update the token config for an existing token.
    /// The token's decimals will be automatically set based on the token mint account.
    ///
    /// # Accounts
    /// [*See the documentation for the accounts*](PushToTokenMap).
    ///
    /// # Arguments
    /// - `name`: The token identifier (e.g. "WSOL", "WBTC")
    /// - `builder`: Configuration builder containing token parameters
    /// - `enable`: If true, enables this token config after pushing. If false, disables it.
    /// - `new`: If true, requires this to be a new token config. An error will be returned if
    ///   a config already exists for this token. If false, allows updating existing configs.
    ///
    /// # Errors
    /// - The [`authority`](PushToTokenMap::authority) must be a signer with the MARKET_KEEPER role
    /// - The [`store`](PushToTokenMap::store) must be an initialized [`Store`](states::Store).
    ///   account owned by the store program and must own the token map.
    /// - The [`token_map`](PushToTokenMap::token_map) must be initialized and owned by the program
    /// - The [`token`](PushToTokenMap::token) must be a valid SPL token mint account.
    /// - If `new` is true, the token must not already have a config in the map.
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
    /// This instruction allows adding or updating token configurations for synthetic tokens that don't have
    /// an actual SPL token mint account. Unlike regular tokens where decimals are read from the mint,
    /// synthetic tokens specify their decimals directly through the `token_decimals` parameter.
    ///
    /// # Accounts
    /// [*See the documentation for the accounts*](PushToTokenMapSynthetic).
    ///
    /// # Arguments
    /// - `name`: The identifier for the synthetic token (e.g. "BTC-PERP")
    /// - `token`: The public key to use as the synthetic token's address
    /// - `token_decimals`: Number of decimals for the synthetic token's amounts
    /// - `builder`: Configuration builder containing token parameters
    /// - `enable`: If true, enables this token config after pushing. If false, disables it.
    /// - `new`: If true, requires this to be a new token config. An error will be returned if
    ///   a config already exists for this token. If false, allows updating existing configs.
    ///
    /// # Errors
    /// - The [`authority`](PushToTokenMapSynthetic::authority) must be a signer with the MARKET_KEEPER role.
    /// - The [`store`](PushToTokenMapSynthetic::store) must be an initialized [`Store`](states::Store)
    ///   account owned by the store program and must own the token map.
    /// - The [`token_map`](PushToTokenMapSynthetic::token_map) must be initialized and owned by the program.
    /// - If updating an existing config, the `token_decimals` must match the original value.
    /// - If `new` is true, the token must not already have a config in the map.
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

    /// Enable or disable the config for the given token.
    ///
    /// # Accounts
    /// [*See the documentation for the accounts*](ToggleTokenConfig).
    ///
    /// # Arguments
    /// - `token`: The token whose config will be updated.
    /// - `enable`: Enable or disable the config.
    ///
    /// # Errors
    /// - The [`authority`](ToggleTokenConfig::authority) must be a signer
    ///   and a MARKET_KEEPER in the given store.
    /// - The [`store`](ToggleTokenConfig::store) must be an initialized [`Store`](states::Store)
    ///   account owned by the store program and must be the owner of the token map.
    /// - The [`token_map`](ToggleTokenConfig::token_map) must be an initialized token map account
    ///   owned by the store program.
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
    ///   for the token. Must be a valid [`PriceProviderKind`] value.
    ///
    /// # Errors
    /// - The [`authority`](SetExpectedProvider::authority) must be a signer
    ///   and have the MARKET_KEEPER role in the given store.
    /// - The [`store`](SetExpectedProvider::store) must be an initialized [`Store`](states::Store)
    ///   account owned by the store program and must be the owner of the token map.
    /// - The [`token_map`](SetExpectedProvider::token_map) must be an initialized token map account
    ///   owned by the store program.
    /// - The given `token` must exist in the token map.
    /// - The `provider` index must correspond to a valid [`PriceProviderKind`].
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
    ///   Must be a valid [`PriceProviderKind`] value.
    /// - `feed`: The new feed address.
    /// - `timestamp_adjustment`: The new timestamp adjustment in seconds.
    ///
    /// # Errors
    /// - The [`authority`](SetFeedConfig::authority) must be a signer
    ///   and a MARKET_KEEPER in the given store.
    /// - The [`store`](SetFeedConfig::store) must be an initialized [`Store`](states::Store)
    ///   account owned by the store program and must be the owner of the token map.
    /// - The [`token_map`](SetFeedConfig::token_map) must be an initialized token map account
    ///   owned by the store program.
    /// - The given `token` must exist in the token map.
    /// - The `provider` index must correspond to a valid [`PriceProviderKind`].
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

    /// Return whether the token config is enabled.
    ///
    /// # Accounts
    /// [*See the documentation for the accounts*](ReadTokenMap).
    ///
    /// # Arguments
    /// - `token`: The address of the token to query for.
    ///
    /// # Errors
    /// - The [`token_map`](ReadTokenMap::token_map) must be an initialized token map account
    ///   owned by the store program.
    /// - The given `token` must exist in the token map.
    ///
    /// # Returns
    /// Returns `true` if the token config is enabled, `false` otherwise.
    pub fn is_token_config_enabled(ctx: Context<ReadTokenMap>, token: Pubkey) -> Result<bool> {
        instructions::is_token_config_enabled(ctx, &token)
    }

    /// Get the expected provider of the given token.
    ///
    /// # Accounts
    /// [*See the documentation for the accounts*](ReadTokenMap).
    ///
    /// # Arguments
    /// - `token`: The address of the token to query for.
    ///
    /// # Errors
    /// - The [`token_map`](ReadTokenMap::token_map) must be an initialized token map account
    ///   owned by the store program.
    /// - The given `token` must exist in the token map.
    ///
    /// # Returns
    /// Returns the expected provider kind as a u8 index. See [`PriceProviderKind`] for valid indices.
    pub fn token_expected_provider(ctx: Context<ReadTokenMap>, token: Pubkey) -> Result<u8> {
        instructions::token_expected_provider(ctx, &token).map(|kind| kind as u8)
    }

    /// Get the configured feed of the given token for the provider.
    ///
    /// # Accounts
    /// [*See the documentation for the accounts*](ReadTokenMap).
    ///
    /// # Arguments
    /// - `token`: The address of the token to query for.
    /// - `provider`: The index of provider to query for. Must be a valid index defined in
    ///   [`PriceProviderKind`].
    ///
    /// # Errors
    /// - The [`token_map`](ReadTokenMap::token_map) must be an initialized token map account
    ///   owned by the store program.
    /// - The given `token` must exist in the token map.
    /// - The `provider` must be a valid index defined in [`PriceProviderKind`], otherwise
    ///   returns [`CoreError::InvalidProviderKindIndex`].
    ///
    /// # Returns
    /// Returns the configured feed address for the given token and provider.
    pub fn token_feed(ctx: Context<ReadTokenMap>, token: Pubkey, provider: u8) -> Result<Pubkey> {
        instructions::token_feed(
            ctx,
            &token,
            &PriceProviderKind::try_from(provider)
                .map_err(|_| CoreError::InvalidProviderKindIndex)?,
        )
    }

    /// Get the configured timestamp adjustment of the given token for the provider.
    ///
    /// # Accounts
    /// [*See the documentation for the accounts*](ReadTokenMap).
    ///
    /// # Arguments
    /// - `token`: The address of the token to query for.
    /// - `provider`: The index of provider to query for. Must be a valid index defined in
    ///   [`PriceProviderKind`].
    ///
    /// # Errors
    /// - The [`token_map`](ReadTokenMap::token_map) must be an initialized token map account
    ///   owned by the store program.
    /// - The given `token` must exist in the token map.
    /// - The `provider` must be a valid index defined in [`PriceProviderKind`], otherwise
    ///   returns [`CoreError::InvalidProviderKindIndex`].
    ///
    /// # Returns
    /// Returns the configured timestamp adjustment for the given token and provider.
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

    /// Get the name of the token.
    ///
    /// # Accounts
    /// [*See the documentation for the accounts*](ReadTokenMap).
    ///
    /// # Arguments
    /// - `token`: The address of the token to query for.
    ///
    /// # Errors
    /// - The [`token_map`](ReadTokenMap::token_map) must be an initialized token map account
    ///   owned by the store program.
    /// - The given `token` must exist in the token map.
    ///
    /// # Returns
    /// Returns the configured name string for the given token.
    pub fn token_name(ctx: Context<ReadTokenMap>, token: Pubkey) -> Result<String> {
        instructions::token_name(ctx, &token)
    }

    /// Get the decimals of the token.
    ///
    /// # Accounts
    /// [*See the documentation for the accounts*](ReadTokenMap).
    ///
    /// # Arguments
    /// - `token`: The address of the token to query for.
    ///
    /// # Errors
    /// - The [`token_map`](ReadTokenMap::token_map) must be an initialized token map account
    ///   owned by the store program.
    /// - The given `token` must exist in the token map.
    ///
    /// # Returns
    /// Returns the configured number of decimals for the given token.
    pub fn token_decimals(ctx: Context<ReadTokenMap>, token: Pubkey) -> Result<u8> {
        instructions::token_decimals(ctx, &token)
    }

    /// Get the price precision of the token.
    ///
    /// # Accounts
    /// [*See the documentation for the accounts*](ReadTokenMap).
    ///
    /// # Arguments
    /// - `token`: The address of the token to query for.
    ///
    /// # Errors
    /// - The [`token_map`](ReadTokenMap::token_map) must be an initialized token map account
    ///   owned by the store program.
    /// - The given `token` must exist in the token map.
    ///
    /// # Returns
    /// Returns the configured price precision for the given token.
    pub fn token_precision(ctx: Context<ReadTokenMap>, token: Pubkey) -> Result<u8> {
        instructions::token_precision(ctx, &token)
    }

    // ===========================================
    //              Oracle Management
    // ===========================================

    /// Initialize a new oracle account for the given store.
    ///
    /// This instruction creates a new oracle account that will be owned by the store. The oracle
    /// account is used to store price data for tokens configured in the store's token map.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](InitializeOracle)*
    ///
    /// # Errors
    /// - The [`store`](InitializeOracle::store) must be an initialized [`Store`](states::Store)
    ///   account owned by the store program and must be the owner of the token map.
    /// - The [`oracle`](InitializeOracle::oracle) account must be uninitialized.
    pub fn initialize_oracle(ctx: Context<InitializeOracle>) -> Result<()> {
        instructions::initialize_oracle(ctx)
    }

    /// Clear all prices from the given oracle.
    ///
    /// This instruction removes all stored price data from the oracle account. This can be useful
    /// when needing to reset price data or when decommissioning an oracle.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](ClearAllPrices)*
    ///
    /// # Errors
    /// - The [`authority`](ClearAllPrices::authority) must be a signer and have the ORACLE_CONTROLLER
    ///   role in the given store.
    /// - The [`store`](ClearAllPrices::store) must be an initialized store account owned by the
    ///   store program.
    /// - The [`oracle`](ClearAllPrices::oracle) must be an initialized oracle account owned by
    ///   the given store.
    #[access_control(internal::Authenticate::only_oracle_controller(&ctx))]
    pub fn clear_all_prices(ctx: Context<ClearAllPrices>) -> Result<()> {
        instructions::unchecked_clear_all_prices(ctx)
    }

    /// Set prices from the provided price feeds.
    ///
    /// This instruction updates token prices in the oracle account using data from configured price feeds.
    /// For each token provided, it reads the current price from the corresponding price feed account and
    /// stores it in the oracle.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](SetPricesFromPriceFeed)*
    ///
    /// # Arguments
    /// - `tokens`: The list of token mint addresses to update prices for. Each token must be configured
    ///   in the token map with a valid price feed.
    ///
    /// # Errors
    /// - The [`authority`](SetPricesFromPriceFeed::authority) must be a signer and have the
    ///   ORACLE_CONTROLLER role in the given store.
    /// - The [`store`](SetPricesFromPriceFeed::store) must be an initialized store account owned by
    ///   the store program.
    /// - The [`oracle`](SetPricesFromPriceFeed::oracle) must be an initialized oracle account owned
    ///   by the given store.
    /// - The [`token_map`](SetPricesFromPriceFeed::token_map) must be an initialized token map account
    ///   that is owned and authorized by the store.
    /// - The number of tokens provided cannot exceed [`MAX_TOKENS`](crate::states::oracle::price_map::PriceMap::MAX_TOKENS).
    /// - Each token in `tokens` must be configured and enabled in the token map.
    /// - For each token, there must be a valid corresponding price feed account included in the remaining accounts.
    #[access_control(internal::Authenticate::only_oracle_controller(&ctx))]
    pub fn set_prices_from_price_feed<'info>(
        ctx: Context<'_, '_, 'info, 'info, SetPricesFromPriceFeed<'info>>,
        tokens: Vec<Pubkey>,
    ) -> Result<()> {
        instructions::set_prices_from_price_feed(ctx, tokens)
    }

    /// Initialize a custom price feed account.
    ///
    /// Creates a new price feed account that can be used to provide custom price data for a token.
    /// The price feed is owned by the store and can only be updated by ORDER_KEEPERs.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](InitializePriceFeed)*
    ///
    /// # Arguments
    /// - `index`: The oracle index this price feed will be associated with.
    /// - `provider`: The price provider kind index that will be used for this feed. Must be a valid
    ///   index from [`PriceProviderKind`] that supports custom price feeds.
    /// - `token`: The mint address of the token this price feed will provide prices for.
    /// - `feed_id`: The unique identifier for this price feed, used to derive its PDA address.
    ///
    /// # Errors
    /// - The [`authority`](InitializePriceFeed::authority) must be a signer and have the ORDER_KEEPER
    ///   role in the store.
    /// - The [`store`](InitializePriceFeed::store) must be an initialized store account owned by
    ///   the store program.
    /// - The [`price_feed`](InitializePriceFeed::price_feed) must be uninitialized and its address
    ///   must match the PDA derived from the store, index, and feed_id.
    /// - The `provider` index must correspond to a valid [`PriceProviderKind`] that supports
    ///   custom price feeds.
    #[access_control(internal::Authenticate::only_order_keeper(&ctx))]
    pub fn initialize_price_feed(
        ctx: Context<InitializePriceFeed>,
        index: u8,
        provider: u8,
        token: Pubkey,
        feed_id: Pubkey,
    ) -> Result<()> {
        let provider = PriceProviderKind::try_from(provider)
            .map_err(|_| error!(CoreError::InvalidProviderKindIndex))?;
        instructions::unchecked_initialize_price_feed(ctx, index, provider, &token, &feed_id)
    }

    /// Update a custom price feed account with Chainlink report.
    ///
    /// Updates the price data in a custom price feed account using a signed price report from
    /// Chainlink Data Streams. The price feed must be configured to use the Chainlink Data Streams
    /// provider. The Chainlink program must be explicitly trusted by the store.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](UpdatePriceFeedWithChainlink)*
    ///
    /// # Arguments
    /// - `signed_report`: A signed price report from Chainlink Data Streams containing the latest
    ///   price data.
    ///
    /// # Errors
    /// - The [`authority`](UpdatePriceFeedWithChainlink::authority) must be a signer and have the
    ///   ORDER_KEEPER role in the store.
    /// - The [`store`](UpdatePriceFeedWithChainlink::store) must be an initialized store account
    /// - The [`verifier_account`](UpdatePriceFeedWithChainlink::verifier_account) must be a valid
    ///   Chainlink verifier account.
    /// - The [`price_feed`](UpdatePriceFeedWithChainlink::price_feed) must be initialized, owned by
    ///   the store, and authorized for the authority.
    /// - The [`chainlink`](UpdatePriceFeedWithChainlink::chainlink) program ID must be trusted in the
    ///   definition of the [`ChainlinkDataStreamsInterface`](chainlink-datastreams::ChainlinkDataStreamsInterface).
    /// - The price feed must be configured to use [`ChainlinkDataStreams`](PriceProviderKind::ChainlinkDataStreams)
    ///   as its provider.
    /// - The `signed_report` must be:
    ///   - Decodable as a valid Chainlink price report
    ///   - Verifiable by the Chainlink Verifier Program
    ///   - Contain valid data for creating a [`PriceFeedPrice`](states::oracle::PriceFeedPrice)
    /// - The current slot and timestamp must be >= the feed's last update.
    /// - The price data timestamp must be >= the feed's last price timestamp
    /// - The price data must meet all validity requirements (see the `update` method of [`PriceFeed`](states::oracle::PriceFeed)).
    #[access_control(internal::Authenticate::only_order_keeper(&ctx))]
    pub fn update_price_feed_with_chainlink(
        ctx: Context<UpdatePriceFeedWithChainlink>,
        signed_report: Vec<u8>,
    ) -> Result<()> {
        instructions::unchecked_update_price_feed_with_chainlink(ctx, signed_report)
    }

    // ===========================================
    //              Market Management
    // ===========================================

    /// Initialize a [`Market`](states::Market) account.
    ///
    /// # Accounts
    /// [*See the documentation for the accounts.*](InitializeMarket)
    ///
    /// # Arguments
    /// - `index_token_mint`: The address of the index token that this market tracks.
    /// - `name`: The name of the market.
    /// - `enable`: Whether to enable the market after initialization.
    ///
    /// # Errors
    /// - The [`authority`](InitializeMarket::authority) must be a signer and have the MARKET_KEEPER role
    ///   in the store.
    /// - The [`store`](InitializeMarket::store) must be initialized.
    /// - The [`market_token_mint`](InitializeMarket::market_token_mint) must be uninitialized
    ///   and a PDA derived from the expected seeds.
    /// - The [`market`](InitializeMarket::market) must be uninitialized and a PDA derived from
    ///   the expected seeds.
    /// - The [`token_map`](InitializeMarket::token_map) must be initialized and owned by the store.
    /// - The [`long_token_vault`](InitializeMarket::long_token_vault) and
    ///   [`short_token_vault`](InitializeMarket::short_token_vault) must be initialized
    ///   and valid market vault accounts of the store for their respective tokens.
    /// - The long and short token mints must be valid Mint accounts.
    #[access_control(internal::Authenticate::only_market_keeper(&ctx))]
    pub fn initialize_market(
        ctx: Context<InitializeMarket>,
        index_token_mint: Pubkey,
        name: String,
        enable: bool,
    ) -> Result<()> {
        instructions::unchecked_initialize_market(ctx, index_token_mint, &name, enable)
    }

    /// Enable or disable the given market.
    ///
    /// This instruction allows a MARKET_KEEPER to toggle whether a market is enabled or disabled.
    /// When disabled, no new positions can be opened in the market, but existing positions can
    /// still be closed.
    ///
    /// # Accounts
    /// [*See the documentation for the accounts.*](ToggleMarket)
    ///
    /// # Arguments
    /// - `enable`: Whether to enable (`true`) or disable (`false`) the market.
    ///
    /// # Errors
    /// - The [`authority`](ToggleMarket::authority) must be a signer and have the MARKET_KEEPER
    ///   role in the store.
    /// - The [`store`](ToggleMarket::store) must be initialized and owned by this program.
    /// - The [`market`](ToggleMarket::market) must be initialized and owned by the store.
    /// - The market's status must not already be in the requested state.
    #[access_control(internal::Authenticate::only_market_keeper(&ctx))]
    pub fn toggle_market(ctx: Context<ToggleMarket>, enable: bool) -> Result<()> {
        instructions::unchecked_toggle_market(ctx, enable)
    }

    /// Transfer tokens into the market and record the amounts in its balance.
    ///
    /// This instruction allows a MARKET_KEEPER to transfer tokens from a source account into one of
    /// the market's vault accounts, updating the market's internal balance tracking.
    ///
    /// # Accounts
    /// [*See the documentation for the accounts.*](MarketTransferIn)
    ///
    /// # Arguments
    /// - `amount`: The amount of tokens to transfer into the market vault.
    ///
    /// # Errors
    /// - The [`authority`](MarketTransferIn::authority) must be a signer and have the MARKET_KEEPER
    ///   role in the store.
    /// - The [`store`](MarketTransferIn::store) must be an initialized store account owned by this program.
    /// - The [`from_authority`](MarketTransferIn::from_authority) must be a signer and own the source
    ///   token account.
    /// - The [`market`](MarketTransferIn::market) must be an initialized market account owned by the store.
    /// - The [`from`](MarketTransferIn::from) must be an initialized token account and cannot be the
    ///   same as the destination vault.
    /// - The [`vault`](MarketTransferIn::vault) must be an initialized and valid market vault token
    ///   account owned by the store.
    /// - The market must be enabled and the token being transferred must be one of the market's
    ///   configured collateral tokens.
    /// - The source token account must have sufficient balance for the transfer amount.
    #[access_control(internal::Authenticate::only_market_keeper(&ctx))]
    pub fn market_transfer_in(ctx: Context<MarketTransferIn>, amount: u64) -> Result<()> {
        instructions::unchecked_market_transfer_in(ctx, amount)
    }

    /// Update an item in the market config.
    ///
    /// This instruction allows a MARKET_KEEPER to update a single configuration value in the market's
    /// configuration. The key must be one of the predefined market config keys.
    ///
    /// # Accounts
    /// [*See the documentation for the accounts.*](UpdateMarketConfig)
    ///
    /// # Arguments
    /// - `key`: The configuration key to update. Must be a valid key defined in
    ///   [`MarketConfigKey`](states::market::config::MarketConfigKey).
    /// - `value`: The new value to set for this configuration key.
    ///
    /// # Errors
    /// - The [`authority`](UpdateMarketConfig::authority) must be a signer and have the MARKET_KEEPER
    ///   role in the store.
    /// - The [`store`](UpdateMarketConfig::store) must be an initialized store account owned by this program.
    /// - The [`market`](UpdateMarketConfig::market) must be an initialized market account owned by the store.
    /// - The provided `key` must be defined in [`MarketConfigKey`](states::market::config::MarketConfigKey).
    #[access_control(internal::Authenticate::only_market_keeper(&ctx))]
    pub fn update_market_config(
        ctx: Context<UpdateMarketConfig>,
        key: String,
        value: u128,
    ) -> Result<()> {
        instructions::unchecked_update_market_config(ctx, &key, value)
    }

    /// Update the market configuration using a pre-populated
    /// [`MarketConfigBuffer`](crate::states::market::config::MarketConfigBuffer) account.
    ///
    /// This instruction allows a MARKET_KEEPER to update multiple market configuration values at once
    /// by applying the changes stored in a buffer account. The buffer must contain valid configuration
    /// keys and values.
    ///
    /// # Accounts
    /// [*See the documentation for the accounts.*](UpdateMarketConfigWithBuffer)
    ///
    /// # Errors
    /// - The [`authority`](UpdateMarketConfigWithBuffer::authority) must be a signer and have the
    ///   MARKET_KEEPER role in the store.
    /// - The [`store`](UpdateMarketConfigWithBuffer::store) must be an initialized store account
    ///   owned by this program.
    /// - The [`market`](UpdateMarketConfigWithBuffer::market) must be an initialized market account
    ///   owned by the store.
    /// - The [`buffer`](UpdateMarketConfigWithBuffer::buffer) must be:
    ///   - An initialized market config buffer account
    ///   - Owned by both the store and the authority
    ///   - Not expired
    /// - All configuration keys in the buffer must be valid keys defined in
    ///   [`MarketConfigKey`](states::market::config::MarketConfigKey).
    #[access_control(internal::Authenticate::only_market_keeper(&ctx))]
    pub fn update_market_config_with_buffer(
        ctx: Context<UpdateMarketConfigWithBuffer>,
    ) -> Result<()> {
        instructions::unchecked_update_market_config_with_buffer(ctx)
    }

    /// Read the current market status.
    ///
    /// This instruction calculates and returns the current status of a market, including metrics like
    /// pool value, PnL, and other key indicators. The calculation can be configured to maximize or
    /// minimize certain values based on the provided flags.
    ///
    /// # Accounts
    /// [*See the documentation for the accounts.*](ReadMarket)
    ///
    /// # Arguments
    /// - `prices`: The current unit prices of tokens in the market, used for calculations.
    /// - `maximize_pnl`: If true, uses the maximum possible PnL values in calculations.
    ///   If false, uses minimum PnL values.
    /// - `maximize_pool_value`: If true, uses the maximum possible pool value in calculations.
    ///   If false, uses minimum pool value.
    ///
    /// # Errors
    /// - The [`market`](ReadMarket::market) account must be properly initialized.
    /// - The market must have valid token configurations.
    /// - The provided prices must match the market's token configurations.
    /// - Numerical overflow/underflow during calculations.
    /// - Invalid pool value or PnL calculations.
    pub fn get_market_status(
        ctx: Context<ReadMarket>,
        prices: Prices<u128>,
        maximize_pnl: bool,
        maximize_pool_value: bool,
    ) -> Result<MarketStatus> {
        instructions::get_market_status(ctx, &prices, maximize_pnl, maximize_pool_value)
    }

    /// Get the current market token price based on the provided token prices and PnL factor.
    ///
    /// This instruction calculates and returns the current price of the market token, taking into
    /// account the provided token prices and PnL factor. The calculation can be configured to
    /// maximize certain values based on the provided flag.
    ///
    /// # Accounts
    /// [*See the documentation for the accounts.*](ReadMarketWithToken)
    ///
    /// # Arguments
    /// - `prices`: The current unit prices of tokens in the market, used for calculations.
    /// - `pnl_factor`: The PnL factor key to use for price calculations, must be a valid
    ///   [`FactorKey`](states::FactorKey).
    /// - `maximize`: If true, uses the maximum possible values in calculations.
    ///   If false, uses minimum values.
    ///
    /// # Errors
    /// - The [`market`](ReadMarketWithToken::market) must be an initialized market account.
    /// - The market must have valid token configurations.
    /// - The provided prices must match the market's token configurations.
    /// - The `pnl_factor` must be a valid factor key.
    /// - Numerical overflow/underflow during calculations.
    /// - Invalid price calculations.
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

    /// Initialize a market config buffer account.
    ///
    /// This instruction creates a new market config buffer account that can be used to stage market
    /// configuration changes before applying them. The buffer has an expiration time after which
    /// it cannot be used.
    ///
    /// # Accounts
    /// [*See the documentation for the accounts.*](InitializeMarketConfigBuffer)
    ///
    /// # Arguments
    /// - `expire_after_secs`: The number of seconds after which this buffer account will expire.
    ///   Once expired, the buffer can no longer be used and must be closed.
    ///
    /// # Errors
    /// - The [`authority`](InitializeMarketConfigBuffer::authority) must be a signer and will be
    ///   set as the owner of the buffer account.
    /// - The [`store`](InitializeMarketConfigBuffer::store) must be an initialized store account
    ///   owned by the program.
    /// - The [`buffer`](InitializeMarketConfigBuffer::buffer) must be an uninitialized account
    ///   that will store the market configuration data.
    /// - The expiration time must be greater than zero.
    pub fn initialize_market_config_buffer(
        ctx: Context<InitializeMarketConfigBuffer>,
        expire_after_secs: u32,
    ) -> Result<()> {
        instructions::initialize_market_config_buffer(ctx, expire_after_secs)
    }

    /// Transfer ownership of a market config buffer account to a new authority.
    ///
    /// This instruction allows the current authority to transfer ownership of the buffer
    /// account to a new authority. After the transfer, only the new authority will be able
    /// to modify or close the buffer.
    ///
    /// # Accounts
    /// [*See the documentation for the accounts.*](SetMarketConfigBufferAuthority)
    ///
    /// # Arguments
    /// - `new_authority`: The public key of the new authority that will own the buffer account.
    ///
    /// # Errors
    /// - The [`authority`](SetMarketConfigBufferAuthority::authority) must be a signer
    ///   and the current owner of the `buffer` account.
    /// - The [`buffer`](SetMarketConfigBufferAuthority::buffer) must be an initialized
    ///   market config buffer account.
    /// - The `new_authority` cannot be the same as the current authority.
    pub fn set_market_config_buffer_authority(
        ctx: Context<SetMarketConfigBufferAuthority>,
        new_authority: Pubkey,
    ) -> Result<()> {
        instructions::set_market_config_buffer_authority(ctx, new_authority)
    }

    /// Close the given market config buffer account and reclaim its rent.
    ///
    /// This instruction allows the authority to close their market config buffer account
    /// and reclaim the rent.
    ///
    /// # Accounts
    /// [*See the documentation for the accounts.*](CloseMarketConfigBuffer)
    ///
    /// # Errors
    /// - The [`authority`](CloseMarketConfigBuffer::authority) must be a signer
    ///   and the owner of the `buffer` account.
    /// - The [`buffer`](CloseMarketConfigBuffer::buffer) must be an initialized
    ///   market config buffer account.
    pub fn close_market_config_buffer(ctx: Context<CloseMarketConfigBuffer>) -> Result<()> {
        instructions::close_market_config_buffer(ctx)
    }

    /// Push config items to the given market config buffer account.
    ///
    /// This instruction allows the authority to add new configuration items to their market
    /// config buffer account. The buffer will be reallocated to accommodate the new items,
    /// with the authority paying for any additional rent.
    ///
    /// # Accounts
    /// [*See the documentation for the accounts.*](PushToMarketConfigBuffer)
    ///
    /// # Arguments
    /// - `new_configs`: The list of new config items to append to the buffer. Each item
    ///   consists of a string key and a factor value.
    ///
    /// # Errors
    /// - The [`authority`](PushToMarketConfigBuffer::authority) must be a signer
    ///   and the owner of the `buffer` account.
    /// - The [`buffer`](PushToMarketConfigBuffer::buffer) must be an initialized
    ///   market config buffer account.
    /// - The buffer must not have expired.
    /// - The authority must have enough SOL to pay for any additional rent needed.
    pub fn push_to_market_config_buffer(
        ctx: Context<PushToMarketConfigBuffer>,
        new_configs: Vec<EntryArgs>,
    ) -> Result<()> {
        instructions::push_to_market_config_buffer(ctx, new_configs)
    }

    /// Enable or disable GT minting for the given market.
    ///
    /// This instruction allows a market keeper to control whether GT (governance token)
    /// minting is enabled for a specific market. When disabled, users cannot mint new GT
    /// tokens through this market.
    ///
    /// # Accounts
    /// [*See the documentation for the accounts.*](ToggleGTMinting)
    ///
    /// # Arguments
    /// - `enable`: Whether to enable (`true`) or disable (`false`) GT minting for the given market.
    ///
    /// # Errors
    /// - The [`authority`](ToggleGTMinting::authority) must be a signer and be a MARKET_KEEPER
    ///   in the store.
    /// - The [`store`](ToggleGTMinting::store) must be an initialized store account.
    /// - The [`market`](ToggleGTMinting::market) must be an initialized market account and owned
    ///   by the store.
    #[access_control(internal::Authenticate::only_market_keeper(&ctx))]
    pub fn toggle_gt_minting(ctx: Context<ToggleGTMinting>, enable: bool) -> Result<()> {
        instructions::unchecked_toggle_gt_minting(ctx, enable)
    }

    /// Claim fees from the given market. The claimed amount remains in the market balance,
    /// and requires a subsequent transfer.
    ///
    /// This instruction allows the fee receiver to claim accumulated fees from a market.
    /// The claimed amount is not immediately transferred - it remains in the market's balance
    /// and must be transferred in a separate instruction.
    ///
    /// # Accounts
    /// [*See the documentation for the accounts.*](ClaimFeesFromMarket)
    ///
    /// # Return
    /// - Returns the claimed amount in base units of the token.
    ///
    /// # Errors
    /// - The [`authority`](ClaimFeesFromMarket::authority) must be a signer and be the designated
    ///   fee receiver in the given store.
    /// - The [`store`](ClaimFeesFromMarket::store) must be an initialized [`Store`](crate::states::Store)
    ///   account owned by this program.
    /// - The [`market`](ClaimFeesFromMarket::market) must be an initialized [`Market`](crate::states::Market)
    ///   account owned by this program and associated with the given store.
    /// - The token being claimed must be one of the market's configured collateral tokens.
    /// - All provided token accounts must match their expected addresses.
    /// - The market must maintain valid balance requirements after the claim.
    pub fn claim_fees_from_market(ctx: Context<ClaimFeesFromMarket>) -> Result<u64> {
        let claimed = instructions::claim_fees_from_market(ctx)?;
        Ok(claimed)
    }

    /// Initialize a new market vault for a specific token.
    ///
    /// This instruction creates a new vault account that will be used to store tokens for a market.
    /// The vault is a PDA (Program Derived Address) account that can only be controlled by this program.
    ///
    /// # Accounts
    /// [*See the documentation for the accounts.*](InitializeMarketVault)
    ///
    /// # Errors
    /// - The [`authority`](InitializeMarketVault::authority) must be a signer and have MARKET_KEEPER
    ///   permissions in the store.
    /// - The [`store`](InitializeMarketVault::store) must be an initialized store account.
    /// - The [`vault`](InitializeMarketVault::vault) must be an uninitialized account and its address
    ///   must match the PDA derived from the expected seeds (store, market, token).
    #[access_control(internal::Authenticate::only_market_keeper(&ctx))]
    pub fn initialize_market_vault(ctx: Context<InitializeMarketVault>) -> Result<()> {
        instructions::unchecked_initialize_market_vault(ctx)
    }

    /// Prepare a claimable account to receive tokens during order execution.
    ///
    /// This instruction serves two purposes:
    /// 1. For uninitialized accounts: Creates and prepares the account to receive tokens
    /// 2. For initialized accounts: Unlocks the funds for the owner to claim
    ///
    /// # Accounts
    /// [*See the documentation for the accounts.*](UseClaimableAccount)
    ///
    /// # Arguments
    /// - `timestamp`: The timestamp when the claimable account was created
    /// - `amount`: The token amount to approve for delegation
    ///
    /// # Errors
    /// - The [`authority`](UseClaimableAccount::authority) must be a signer and have ORDER_KEEPER
    ///   permissions in the store
    /// - The [`store`](UseClaimableAccount::store) must be an initialized store account
    /// - The [`account`](UseClaimableAccount::account) must be a PDA derived from
    ///   the claimable timestamp and other expected seeds
    /// - If initialized, the [`account`](UseClaimableAccount::account) must be owned by the store
    #[access_control(internal::Authenticate::only_order_keeper(&ctx))]
    pub fn use_claimable_account(
        ctx: Context<UseClaimableAccount>,
        timestamp: i64,
        amount: u64,
    ) -> Result<()> {
        instructions::unchecked_use_claimable_account(ctx, timestamp, amount)
    }

    /// Close an empty claimable account.
    ///
    /// # Accounts
    /// [*See the documentation for the accounts.*](CloseEmptyClaimableAccount)
    ///
    /// # Arguments
    /// - `timestamp`: The timestamp when the claimable account was created.
    ///
    /// # Errors
    /// - The [`authority`](CloseEmptyClaimableAccount::authority) must be a signer and have ORDER_KEEPER
    ///   permissions in the store.
    /// - The [`store`](CloseEmptyClaimableAccount::store) must be initialized.
    /// - The [`account`](CloseEmptyClaimableAccount::account) must be a PDA derived from
    ///   the claimable timestamp and other expected seeds.
    /// - The [`account`](CloseEmptyClaimableAccount::account) must be initialized and owned by the store.
    /// - The balance of the [`account`](CloseEmptyClaimableAccount::account) must be zero.
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
    /// *[See the documentation for the accounts.](PrepareAssociatedTokenAccount)*
    ///
    /// # Errors
    /// - The [`payer`](PrepareAssociatedTokenAccount::payer) must be a signer
    /// - The [`mint`](PrepareAssociatedTokenAccount::mint) must be a [`Mint`](anchor_spl::token::Mint) account
    /// - The [`account`](PrepareAssociatedTokenAccount::account) must be an associated token account with:
    ///   - mint = [`mint`](PrepareAssociatedTokenAccount::mint)
    ///   - owner = [`owner`](PrepareAssociatedTokenAccount::owner)
    ///   - It can be uninitialized
    pub fn prepare_associated_token_account(
        ctx: Context<PrepareAssociatedTokenAccount>,
    ) -> Result<()> {
        instructions::prepare_associated_token_account(ctx)
    }

    // ===========================================
    //                  Depsoit
    // ===========================================

    /// Create a deposit by the owner.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](CreateDeposit)*
    ///
    /// # Arguments
    /// - `nonce`: Nonce bytes used to derive the deposit account address.
    /// - `params`: Parameters specifying the deposit details.
    ///
    /// # Errors
    /// This instruction will fail if:
    /// - The [`owner`](CreateDeposit::owner) is not a signer or has insufficient balance
    ///   for the execution fee
    /// - The [`store`](CreateDeposit::store) is not properly initialized
    /// - The [`market`](CreateDeposit::market) is not initialized, not owned by the store,
    ///   or is disabled
    /// - The [`deposit`](CreateDeposit::deposit) account is already initialized or is not
    ///   a valid PDA derived from the provided nonce
    /// - The [`market_token`](CreateDeposit::market_token) does not match the market's
    ///   configured token
    /// - Any required escrow account is not properly initialized or owned by the deposit
    /// - Any source account has insufficient balance or does not match the initial tokens
    /// - The remaining accounts do not form valid swap paths or reference disabled markets
    pub fn create_deposit<'info>(
        mut ctx: Context<'_, '_, 'info, 'info, CreateDeposit<'info>>,
        nonce: [u8; 32],
        params: CreateDepositParams,
    ) -> Result<()> {
        internal::Create::create(&mut ctx, &nonce, &params)
    }

    /// Close a deposit, either by the owner or by keepers.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](CloseDeposit)*
    ///
    /// # Arguments
    /// - `reason`: The reason for closing the deposit.
    ///
    /// # Errors
    /// This instruction will fail if:
    /// - The [`executor`](CloseDeposit::executor) is not a signer or is neither the deposit
    ///   owner nor an ORDER_KEEPER in the store
    /// - The [`store`](CloseDeposit::store) is not properly initialized
    /// - The [`owner`](CloseDeposit::owner) does not match the deposit's owner
    /// - The provided token accounts do not match those recorded in the deposit
    /// - The [`deposit`](CloseDeposit::deposit) is not initialized, not owned by the store,
    ///   or not owned by the specified owner
    /// - Any escrow account is not properly owned or does not match the deposit records
    /// - Any associated token account address is invalid
    /// - The deposit is not in a cancelled or completed state when closed by a non-owner
    pub fn close_deposit<'info>(
        ctx: Context<'_, '_, 'info, 'info, CloseDeposit<'info>>,
        reason: String,
    ) -> Result<()> {
        internal::Close::close(&ctx, &reason)
    }

    /// Execute a deposit by keepers.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](ExecuteDeposit)*
    ///
    /// # Arguments
    /// - `execution_fee`: The execution fee to be paid to the keeper for executing the deposit
    /// - `throw_on_execution_error`: If true, throws an error if execution fails. If false,
    ///   allows execution to continue even if there are errors.
    ///
    /// # Errors
    /// This instruction will fail if:
    /// - The [`authority`](ExecuteDeposit::authority) is not a signer or is not an ORDER_KEEPER
    ///   in the store
    /// - The [`store`](ExecuteDeposit::store) is not properly initialized
    /// - The [`token_map`](ExecuteDeposit::token_map) is not initialized or not authorized by
    ///   the store
    /// - The [`oracle`](ExecuteDeposit::oracle) is not initialized, cleared and owned by the
    ///   store
    /// - The [`market`](ExecuteDeposit::market) is not initialized, is disabled, not owned by
    ///   the store, or does not match the market recorded in the deposit
    /// - The [`deposit`](ExecuteDeposit::deposit) is not initialized, not owned by the store,
    ///   or not in the pending state
    /// - Any token accounts do not match those recorded in the deposit
    /// - Any escrow accounts are not properly owned or not recorded in the deposit
    /// - Any vault accounts are not valid market vaults or do not correspond to the initial tokens
    /// - Any feed accounts in the remaining accounts are invalid or do not match the swap parameters
    /// - Any market accounts in the remaining accounts are disabled, not owned by the store,
    ///   or do not match the swap parameters
    /// - Any oracle prices from the feed accounts are incomplete or invalid
    /// - The execution fails and `throw_on_execution_error` is set to `true`
    #[access_control(internal::Authenticate::only_order_keeper(&ctx))]
    pub fn execute_deposit<'info>(
        ctx: Context<'_, '_, 'info, 'info, ExecuteDeposit<'info>>,
        execution_fee: u64,
        throw_on_execution_error: bool,
    ) -> Result<()> {
        instructions::unchecked_execute_deposit(ctx, execution_fee, throw_on_execution_error)
    }

    // ===========================================
    //                 Withdrawal
    // ===========================================

    /// Create a withdrawal by the owner.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](CreateWithdrawal)*
    ///
    /// # Arguments
    /// - `nonce`: Nonce bytes used to derive the address for the withdrawal.
    /// - `params`: Withdrawal Parameters containing the withdrawal configuration.
    ///
    /// # Errors
    /// This instruction will fail if:
    /// - The [`owner`](CreateWithdrawal::owner) is not a signer or has insufficient balance
    ///   for the withdrawal
    /// - The [`store`](CreateWithdrawal::store) is not properly initialized
    /// - The [`market`](CreateWithdrawal::market) is not initialized, is disabled, or not owned
    ///   by the store
    /// - The [`withdrawal`](CreateWithdrawal::withdrawal) is already initialized or is not a valid
    ///   PDA derived from the provided `nonce` and expected seeds
    /// - The [`market_token`](CreateWithdrawal::market_token) does not match the market token
    ///   of the specified market
    /// - Any required escrow accounts are not properly initialized or not owned by the withdrawal
    /// - The source market token account has insufficient balance
    /// - Any market accounts in the remaining accounts are disabled, not owned by the store,
    ///   or do not form valid swap paths
    pub fn create_withdrawal<'info>(
        mut ctx: Context<'_, '_, 'info, 'info, CreateWithdrawal<'info>>,
        nonce: [u8; 32],
        params: CreateWithdrawalParams,
    ) -> Result<()> {
        internal::Create::create(&mut ctx, &nonce, &params)
    }

    /// Close a withdrawal, either by the owner or by keepers.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](CloseWithdrawal)*
    ///
    /// # Arguments
    /// - `reason`: The reason for closing the withdrawal.
    ///
    /// # Errors
    /// This instruction will fail if:
    /// - The [`executor`](CloseWithdrawal::executor) is not a signer or is neither the withdrawal
    ///   owner nor an ORDER_KEEPER in the store
    /// - The [`store`](CloseWithdrawal::store) is not properly initialized
    /// - The [`owner`](CloseWithdrawal::owner) does not match the withdrawal owner
    /// - The token accounts do not match those recorded in the withdrawal
    /// - The [`withdrawal`](CloseWithdrawal::withdrawal) is not initialized, not owned by the store,
    ///   or not owned by the specified owner
    /// - Any required escrow accounts are not properly initialized or not owned by the withdrawal
    /// - Any associated token accounts have invalid addresses
    /// - The withdrawal is not in a cancelled or completed state when the executor is not the owner
    pub fn close_withdrawal<'info>(
        ctx: Context<'_, '_, 'info, 'info, CloseWithdrawal<'info>>,
        reason: String,
    ) -> Result<()> {
        internal::Close::close(&ctx, &reason)
    }

    /// Execute a withdrawal by keepers.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](ExecuteWithdrawal)*
    ///
    /// # Arguments
    /// - `execution_fee`: The execution fee to be paid to the keeper for executing the withdrawal
    /// - `throw_on_execution_error`: If true, throws an error if execution fails. If false, marks
    ///   the withdrawal as failed but does not throw.
    ///
    /// # Errors
    /// This instruction will fail if:
    /// - The [`authority`](ExecuteWithdrawal::authority) is not a signer or is not an ORDER_KEEPER
    ///   in the store
    /// - The [`store`](ExecuteWithdrawal::store) is not properly initialized
    /// - The [`token_map`](ExecuteWithdrawal::token_map) is not initialized or not authorized by
    ///   the store
    /// - The [`oracle`](ExecuteWithdrawal::oracle) is not initialized, cleared and owned by the
    ///   store
    /// - The [`market`](ExecuteWithdrawal::market) is not initialized, is disabled, not owned by
    ///   the store, or does not match the market recorded in the withdrawal
    /// - The [`withdrawal`](ExecuteWithdrawal::withdrawal) is not initialized, not owned by the
    ///   store, or not in the pending state
    /// - Any token accounts do not match those recorded in the withdrawal
    /// - Any escrow accounts are not properly initialized or not owned by the withdrawal
    /// - Any vault accounts are not valid market vaults or do not correspond to the initial tokens
    /// - Any feed accounts in the remaining accounts are invalid or do not match the swap parameters
    /// - Any market accounts in the remaining accounts are disabled, not owned by the store, or do
    ///   not match the swap parameters
    /// - Any oracle prices from the feed accounts are incomplete or invalid
    /// - The execution fails and `throw_on_execution_error` is set to true
    #[access_control(internal::Authenticate::only_order_keeper(&ctx))]
    pub fn execute_withdrawal<'info>(
        ctx: Context<'_, '_, 'info, 'info, ExecuteWithdrawal<'info>>,
        execution_fee: u64,
        throw_on_execution_error: bool,
    ) -> Result<()> {
        instructions::unchecked_execute_withdrawal(ctx, execution_fee, throw_on_execution_error)
    }

    // ===========================================
    //             Order and Position
    // ===========================================

    /// Prepare the position account for orders.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](PreparePosition)*
    ///
    /// # Arguments
    /// - `params`: Order Parameters specifying the market and position details.
    ///
    /// # Errors
    /// This instruction will fail if:
    /// - The [`owner`](PreparePosition::owner) is not a signer
    /// - The [`store`](PreparePosition::store) is not properly initialized
    /// - The [`market`](PreparePosition::market) is not initialized, is disabled, or not owned by
    ///   the store
    /// - The [`position`](PreparePosition::position) address is not a valid PDA derived from the
    ///   owner and expected seeds
    /// - The position account is neither uninitialized nor validly initialized for the owner
    pub fn prepare_position(
        ctx: Context<PreparePosition>,
        params: CreateOrderParams,
    ) -> Result<()> {
        instructions::prepare_position(ctx, &params)
    }

    /// Create an order by the owner.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](CreateOrder)*
    ///
    /// # Arguments
    /// - `nonce`: Nonce bytes used to derive the address for the order.
    /// - `params`: Order Parameters specifying the market, order kind, and other details.
    ///
    /// # Errors
    /// This instruction will fail if:
    /// - The [`owner`](CreateOrder::owner) is not a signer
    /// - The [`store`](CreateOrder::store) is not properly initialized
    /// - The [`market`](CreateOrder::market) is not initialized, is disabled, or not owned by
    ///   the store
    /// - The [`user`](CreateOrder::user) is not initialized or does not correspond to the owner.
    ///   The address must be a valid PDA derived from the owner and expected seeds.
    /// - The [`order`](CreateOrder::order) is not uninitialized or the address is not a valid
    ///   PDA derived from the owner, nonce and expected seeds
    /// - For increase/decrease orders:
    ///   - The [`position`](CreateOrder::position) is missing, not validly initialized, or not
    ///     owned by both the owner and store
    ///   - The [`long_token`](CreateOrder::long_token) or [`short_token`](CreateOrder::short_token)
    ///     are missing or do not match the market tokens
    ///   - The [`long_token_escrow`](CreateOrder::long_token_escrow) or
    ///     [`short_token_escrow`](CreateOrder::short_token_escrow) are missing or not valid
    ///     escrow accounts owned by the order
    ///   - The [`long_token_ata`](CreateOrder::long_token_ata) or
    ///     [`short_token_ata`](CreateOrder::short_token_ata) are missing or not valid ATAs
    ///     owned by the owner
    /// - For increase/swap orders:
    ///   - The [`initial_collateral_token`](CreateOrder::initial_collateral_token) is missing
    ///     or invalid
    ///   - The [`initial_collateral_token_escrow`](CreateOrder::initial_collateral_token_escrow)
    ///     is missing or not a valid escrow account owned by the order
    ///   - The [`initial_collateral_token_source`](CreateOrder::initial_collateral_token_source)
    ///     is missing or not a valid source account with order authority
    /// - For decrease/swap orders:
    ///   - The [`final_output_token`](CreateOrder::final_output_token) is invalid
    ///   - The [`final_output_token_escrow`](CreateOrder::final_output_token_escrow) is missing
    ///     or not a valid escrow account owned by the order
    ///   - The [`final_output_token_ata`](CreateOrder::final_output_token_ata) is missing or
    ///     not a valid ATA owned by the owner
    /// - The feature for creating this kind of order is not enabled
    /// - The remaining accounts do not match the swap parameters
    pub fn create_order<'info>(
        mut ctx: Context<'_, '_, 'info, 'info, CreateOrder<'info>>,
        nonce: [u8; 32],
        params: CreateOrderParams,
    ) -> Result<()> {
        internal::Create::create(&mut ctx, &nonce, &params)
    }

    /// Close an order, either by the owner or by keepers.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](CloseOrder)*
    ///
    /// # Arguments
    /// - `reason`: The reason for the close.
    ///
    /// # Errors
    /// - The [`executor`](CloseOrder::executor) must be a signer and either the owner
    ///   of the `order` or a ORDER_KEEPER in the store.
    /// - The [`store`](CloseOrder::store) must be initialized.
    /// - The [`owner`](CloseOrder::owner) must be the owner of the `order`.
    /// - The [`user`](CloseOrder::user) must be initialized and correspond to the `owner`.
    /// - The [`referrer_user`](CloseOrder::referrer_user) must be present if the `owner` has a
    ///   referrer, and it must be initialized and correspond to the referrer of the `owner`.
    /// - The [`order`](CloseOrder::order) must be initialized and owned by the store and the
    ///   `owner`.
    /// - The tokens must be those recorded in the `order`.
    /// - The escrow accounts must be owned and recorded in the `order`.
    /// - The addresses of the ATAs must be valid.
    /// - The `order` must be cancelled or completed if the `executor` is not the owner.
    /// - The feature must be enabled for closing the given kind of `order`.
    pub fn close_order<'info>(
        ctx: Context<'_, '_, 'info, 'info, CloseOrder<'info>>,
        reason: String,
    ) -> Result<()> {
        internal::Close::close(&ctx, &reason)
    }

    /// Prepare a trade event buffer.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](PrepareTradeEventBuffer)*
    ///
    /// # Arguments
    /// - `index`: The index of the trade event buffer to prepare. Must be between 0 and 255.
    ///
    /// # Errors
    /// - The [`authority`](PrepareTradeEventBuffer::authority) must be a signer and have
    ///   permission to prepare trade event buffers.
    /// - The [`store`](PrepareTradeEventBuffer::store) must be initialized and active.
    /// - The [`event`](PrepareTradeEventBuffer::event) must be either:
    ///   - Uninitialized, or
    ///   - Already initialized with the `authority` as the authority and the `store` as
    ///     the store
    /// - The `index` must not already be in use by another trade event buffer.
    // FIXME: There is a false positive lint for the doc link of `event`.
    #[allow(rustdoc::broken_intra_doc_links)]
    pub fn prepare_trade_event_buffer(
        ctx: Context<PrepareTradeEventBuffer>,
        index: u8,
    ) -> Result<()> {
        instructions::prepare_trade_event_buffer(ctx, index)
    }

    /// Update an order by the owner.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](UpdateOrder)*
    ///
    /// # Arguments
    /// - `params`: Update Order Parameters. Contains the updated order details like size, price, etc.
    ///
    /// # Errors
    /// - The [`owner`](UpdateOrder::owner) must be a signer and the original creator of the order.
    /// - The [`store`](UpdateOrder::store) must be initialized and active.
    /// - The [`market`](UpdateOrder::market) must be initialized, enabled and owned by the store.
    /// - The [`order`](UpdateOrder::order) must be:
    ///   - Initialized and owned by both the store and the `owner`
    ///   - Associated with the provided `market`
    ///   - In a pending/active state that allows updates
    /// - The feature must be enabled in the store for updating the given kind of `order`.
    /// - The updated parameters must be valid for the order type and market configuration.
    pub fn update_order(ctx: Context<UpdateOrder>, params: UpdateOrderParams) -> Result<()> {
        instructions::update_order(ctx, &params)
    }

    /// Execute an increase/swap order by keepers.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](ExecuteIncreaseOrSwapOrder)*
    ///
    /// # Arguments
    /// - `recent_timestamp`: A recent timestamp used for deriving the claimable accounts.
    /// - `execution_fee`: The execution fee to be paid to the keeper for processing the order.
    /// - `throw_on_execution_error`: If true, throws an error if order execution fails. If false,
    ///   silently cancels the order on execution failure.
    ///
    /// # Errors
    /// - The [`authority`](ExecuteIncreaseOrSwapOrder::authority) must be a signer with ORDER_KEEPER
    ///   permissions in the store.
    /// - The [`store`](ExecuteIncreaseOrSwapOrder::store) must be initialized and active.
    /// - The [`token_map`](ExecuteIncreaseOrSwapOrder::token_map) must be initialized and authorized
    ///   by the store.
    /// - The [`oracle`](ExecuteIncreaseOrSwapOrder::oracle) must be initialized, cleared and owned
    ///   by the store.
    /// - The [`market`](ExecuteIncreaseOrSwapOrder::market) must be initialized, enabled and owned
    ///   by the store.
    /// - The [`owner`](ExecuteIncreaseOrSwapOrder::owner) must be the original creator of the order.
    /// - The [`user`](ExecuteIncreaseOrSwapOrder::user) must be initialized and associated with
    ///   the `owner`.
    /// - The [`order`](ExecuteIncreaseOrSwapOrder::order) must be:
    ///   - Initialized and owned by both the store and owner
    ///   - Associated with the provided market
    ///   - In a pending state
    /// - For increase orders:
    ///   - The [`position`](ExecuteIncreaseOrSwapOrder::position) must exist and be validly owned
    ///   - The [`event`](ExecuteIncreaseOrSwapOrder::event) must be a valid trade event buffer
    ///   - The [`long_token`](ExecuteIncreaseOrSwapOrder::long_token) and [`short_token`](ExecuteIncreaseOrSwapOrder::short_token)
    ///     must match the market tokens
    ///   - The corresponding token escrow and vault accounts must be valid
    /// - For swap orders:
    ///   - The initial and final token accounts must be valid
    ///   - The corresponding escrow and vault accounts must be valid
    /// - All feed accounts must provide valid and complete oracle prices
    /// - The feature for executing this order type must be enabled in the store
    /// - If `throw_on_execution_error` is true, any execution failure will throw an error
    // FIXME: There is a false positive lint for the doc link of `event`.
    #[allow(rustdoc::broken_intra_doc_links)]
    #[access_control(internal::Authenticate::only_order_keeper(&ctx))]
    pub fn execute_increase_or_swap_order<'info>(
        ctx: Context<'_, '_, 'info, 'info, ExecuteIncreaseOrSwapOrder<'info>>,
        recent_timestamp: i64,
        execution_fee: u64,
        throw_on_execution_error: bool,
    ) -> Result<()> {
        instructions::unchecked_execute_increase_or_swap_order(
            ctx,
            recent_timestamp,
            execution_fee,
            throw_on_execution_error,
        )
    }

    /// Execute a decrease order by keepers.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](ExecuteDecreaseOrder)*
    ///
    /// # Arguments
    /// - `recent_timestamp`: A recent timestamp that must be within the valid time window.
    /// - `execution_fee`: The execution fee to be paid to the keeper for processing the order.
    /// - `throw_on_execution_error`: If true, throws an error if order execution fails. If false,
    ///   silently cancels the order on execution failure.
    ///
    /// # Errors
    /// - The [`authority`](ExecuteDecreaseOrder::authority) must be a signer with ORDER_KEEPER
    ///   permissions in the store.
    /// - The [`store`](ExecuteDecreaseOrder::store) must be initialized and active.
    /// - The [`token_map`](ExecuteDecreaseOrder::token_map) must be initialized and authorized
    ///   by the store.
    /// - The [`oracle`](ExecuteDecreaseOrder::oracle) must be initialized, cleared and owned
    ///   by the store.
    /// - The [`market`](ExecuteDecreaseOrder::market) must be initialized, enabled and owned
    ///   by the store.
    /// - The [`owner`](ExecuteDecreaseOrder::owner) must be the original creator of the order.
    /// - The [`user`](ExecuteDecreaseOrder::user) must be initialized and associated with
    ///   the `owner`.
    /// - The [`order`](ExecuteDecreaseOrder::order) must be:
    ///   - Initialized and owned by both the store and owner
    ///   - Associated with the provided market
    ///   - In a pending state
    /// - The [`position`](ExecuteDecreaseOrder::position) must exist and be validly owned
    /// - The [`event`](ExecuteDecreaseOrder::event) must be a valid trade event buffer
    /// - The tokens must match those specified in the order
    /// - All token escrow and vault accounts must be valid and properly owned
    /// - All claimable token accounts must be valid and properly delegated
    /// - All feed accounts must provide valid and complete oracle prices
    /// - The feature for executing decrease orders must be enabled in the store
    /// - If `throw_on_execution_error` is true, any execution failure will throw an error
    // FIXME: There is a false positive lint for the doc link of `event`.
    #[allow(rustdoc::broken_intra_doc_links)]
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

    /// Perform a liquidation by keepers.
    ///
    /// This instruction allows keepers to liquidate positions that have fallen below the required maintenance margin.
    /// When executed, it will close the position and distribute funds according to the liquidation rules.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](PositionCut)*
    ///
    /// # Arguments
    /// - `nonce`: The nonce used to derive the `order` PDA address
    /// - `recent_timestamp`: A recent timestamp that must be within the valid time window
    /// - `execution_fee`: The execution fee to be paid to the keeper for processing the liquidation
    ///
    /// # Errors
    /// - The [`authority`](PositionCut::authority) must be a signer with ORDER_KEEPER permissions in the store
    /// - The [`owner`](PositionCut::owner) must be the owner of the position being liquidated
    /// - The [`user`](PositionCut::user) must be an initialized user account associated with the `owner`
    /// - The [`store`](PositionCut::store) must be initialized and active
    /// - The [`token_map`](PositionCut::token_map) must be initialized and authorized by the store
    /// - The [`oracle`](PositionCut::oracle) must be initialized, cleared and owned by the store
    /// - The [`market`](PositionCut::market) must be:
    ///   - Initialized and enabled
    ///   - Owned by the store
    ///   - The market associated with the position being liquidated
    /// - The [`order`](PositionCut::order) must be:
    ///   - Uninitialized
    ///   - Have an address matching the PDA derived from the store, owner and provided nonce
    /// - The [`position`](PositionCut::position) must be:
    ///   - Validly initialized
    ///   - Owned by both the owner and store
    ///   - In a liquidatable state (below maintenance margin)
    /// - The [`event`](PositionCut::event) must be a valid trade event buffer owned by both the store and authority
    /// - The [`long_token`](PositionCut::long_token) and [`short_token`](PositionCut::short_token) must match the market's configured tokens
    /// - Token escrow accounts must be:
    ///   - Valid for their respective tokens
    ///   - Owned by the order if present
    /// - Token vault accounts must be:
    ///   - Valid for their respective tokens  
    ///   - Owned by the store
    /// - Claimable token accounts must be:
    ///   - Valid for their respective tokens
    ///   - Owned by the store
    ///   - Properly delegated to their respective owners
    /// - Price feed accounts must be:
    ///   - Valid and complete
    ///   - Provided in order matching the market's sorted token list
    /// - The liquidation feature must be enabled in the store
    /// - All oracle prices must be valid and complete
    // FIXME: There is a false positive lint for the doc link of `event`.
    #[allow(rustdoc::broken_intra_doc_links)]
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

    /// Update the ADL (Auto-Deleveraging) state for the market.
    ///
    /// This instruction updates the ADL state for either the long or short side of a market. The ADL
    /// state determines whether positions on that side are eligible for auto-deleveraging based on
    /// current market conditions.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](UpdateAdlState)*
    ///
    /// # Arguments
    /// - `is_long`: Whether to update the ADL state for the long (`true`) or short (`false`) side
    ///   of the market.
    ///
    /// # Errors
    /// - The [`authority`](UpdateAdlState::authority) must be a signer and have the ORDER_KEEPER
    ///   role in the store.
    /// - The [`store`](UpdateAdlState::store) must be an initialized [`Store`](states::Store)
    ///   account owned by the store program.
    /// - The [`oracle`](UpdateAdlState::oracle) must be an initialized [`Oracle`](states::Oracle)
    ///   account that is owned by the store.
    /// - The [`market`](UpdateAdlState::market) must be enabled and owned by the store.
    #[access_control(internal::Authenticate::only_order_keeper(&ctx))]
    pub fn update_adl_state<'info>(
        ctx: Context<'_, '_, 'info, 'info, UpdateAdlState<'info>>,
        is_long: bool,
    ) -> Result<()> {
        instructions::unchecked_update_adl_state(ctx, is_long)
    }

    /// Perform an ADL (Auto-Deleveraging) by keepers.
    ///
    /// This instruction allows keepers to execute auto-deleveraging on positions that meet the ADL criteria.
    /// ADL helps maintain market stability by reducing highly leveraged positions when the market's
    /// overall risk metrics exceed safe thresholds.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](PositionCut)*
    ///
    /// # Arguments
    /// - `nonce`: The nonce used to derive the `order` PDA address.
    /// - `recent_timestamp`: A recent blockchain timestamp for validation.
    /// - `execution_fee`: Fee paid to the keeper for executing the ADL.
    ///
    /// # Errors
    /// - The [`authority`](PositionCut::authority) must be a signer with ORDER_KEEPER role.
    /// - The [`owner`](PositionCut::owner) must be the position owner.
    /// - The [`user`](PositionCut::user) must be initialized and linked to the `owner`.
    /// - The [`store`](PositionCut::store) must be initialized.
    /// - The [`token_map`](PositionCut::token_map) must be initialized and authorized by the store.
    /// - The [`oracle`](PositionCut::oracle) must be initialized, cleared and store-owned.
    /// - The [`market`](PositionCut::market) must be initialized, enabled, store-owned and match
    ///   the position's market.
    /// - The [`order`](PositionCut::order) must be uninitialized with address matching PDA from
    ///   store, owner and nonce.
    /// - The [`position`](PositionCut::position) must be initialized, owned by owner/store and
    ///   eligible for ADL.
    /// - The [`event`](PositionCut::event) must be a valid trade event buffer owned by store/authority.
    /// - The [`long_token`](PositionCut::long_token) and [`short_token`](PositionCut::short_token)
    ///   must match the market's tokens.
    /// - The [`long_token_escrow`](PositionCut::long_token_escrow) and
    ///   [`short_token_escrow`](PositionCut::short_token_escrow) must be valid order-owned escrow
    ///   accounts for their respective tokens.
    /// - The [`long_token_vault`](PositionCut::long_token_vault) and
    ///   [`short_token_vault`](PositionCut::short_token_vault) must be valid store-owned vault
    ///   accounts for their tokens.
    /// - The [`claimable_long_token_account_for_user`](PositionCut::claimable_long_token_account_for_user)
    ///   must be a store-owned, owner-delegated claimable account for long token.
    /// - The [`claimable_short_token_account_for_user`](PositionCut::claimable_short_token_account_for_user)
    ///   must be a store-owned, owner-delegated claimable account for short token.
    /// - The [`claimable_pnl_token_account_for_holding`](PositionCut::claimable_pnl_token_account_for_holding)
    ///   must be a store-owned, holding-delegated claimable account for PnL token.
    /// - Price feed accounts must be valid and provided in market's sorted token order.
    /// - ADL feature must be enabled in store settings.
    /// - Oracle prices must be valid and complete.
    /// - Market must be in ADL state with PnL factor exceeding minimum threshold.
    /// - Execution must complete successfully.
    // FIXME: There is a false positive lint for the doc link of `event`.
    #[allow(rustdoc::broken_intra_doc_links)]
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

    // ===========================================
    //                  Shift
    // ===========================================

    /// Create a shift by the owner.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](CreateShift)*
    ///
    /// # Arguments
    /// - `nonce`: The nonce used to derive the shift's PDA address.
    /// - `params`: The parameters for creating the shift.
    ///
    /// # Errors
    /// - The [`owner`](CreateShift::owner) must be a signer.
    /// - The [`store`](CreateShift::store) must be initialized.
    /// - The [`from_market`](CreateShift::from_market) must be initialized, enabled and store-owned.
    /// - The [`to_market`](CreateShift::to_market) must be initialized, enabled and store-owned.
    /// - The [`from_market`](CreateShift::from_market) must be configured as shiftable to the [`to_market`](CreateShift::to_market).
    /// - The [`shift`](CreateShift::shift) must be uninitialized.
    /// - The [`from_market_token`](CreateShift::from_market_token) must be the market token of the [`from_market`](CreateShift::from_market).
    /// - The [`to_market_token`](CreateShift::to_market_token) must be the market token of the [`to_market`](CreateShift::to_market).
    /// - The [`from_market_token_escrow`](CreateShift::from_market_token_escrow) must be a valid shift-owned escrow account for the [`from_market_token`](CreateShift::from_market_token).
    /// - The [`to_market_token_escrow`](CreateShift::to_market_token_escrow) must be a valid shift-owned escrow account for the [`to_market_token`](CreateShift::to_market_token).
    /// - The [`from_market_token_source`](CreateShift::from_market_token_source) must be a token account for [`from_market_token`](CreateShift::from_market_token) with `owner` as authority.
    /// - The [`to_market_token_ata`](CreateShift::to_market_token_ata) must be a valid associated token account for [`to_market_token`](CreateShift::to_market_token) owned by `owner`.
    pub fn create_shift<'info>(
        mut ctx: Context<'_, '_, 'info, 'info, CreateShift<'info>>,
        nonce: [u8; 32],
        params: CreateShiftParams,
    ) -> Result<()> {
        internal::Create::create(&mut ctx, &nonce, &params)
    }

    /// Execute a shift by keepers.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](ExecuteShift)*
    ///
    /// # Arguments
    /// - `execution_lamports`: The execution fee in lamports claimed by the keeper.
    /// - `throw_on_execution_error`: Whether to throw an error if the execution fails.
    ///
    /// # Errors
    /// - The [`authority`](ExecuteShift::authority) must be a signer and have ORDER_KEEPER role in the store.
    /// - The [`store`](ExecuteShift::store) must be initialized.
    /// - The [`token_map`](ExecuteShift::token_map) must be initialized and authorized by the store.
    /// - The [`oracle`](ExecuteShift::oracle) must be initialized, cleared and store-owned.
    /// - The [`from_market`](ExecuteShift::from_market) must be initialized, enabled and store-owned.
    /// - The [`to_market`](ExecuteShift::to_market) must be initialized, enabled and store-owned.
    /// - The [`from_market`](ExecuteShift::from_market) must be configured as shiftable to the [`to_market`](ExecuteShift::to_market).
    /// - The [`shift`](ExecuteShift::shift) must be initialized and store-owned.
    /// - The [`from_market_token`](ExecuteShift::from_market_token) must be the market token of the [`from_market`](ExecuteShift::from_market).
    /// - The [`to_market_token`](ExecuteShift::to_market_token) must be the market token of the [`to_market`](ExecuteShift::to_market).
    /// - The [`from_market_token_escrow`](ExecuteShift::from_market_token_escrow) must be a valid shift-owned escrow account for the [`from_market_token`](ExecuteShift::from_market_token).
    /// - The [`to_market_token_escrow`](ExecuteShift::to_market_token_escrow) must be a valid shift-owned escrow account for the [`to_market_token`](ExecuteShift::to_market_token).
    /// - The [`from_market_token_vault`](ExecuteShift::from_market_token_vault) must be the token vault for the [`from_market_token`](ExecuteShift::from_market_token) and store-owned.
    /// - The feed accounts must be valid and provided in the same order as the unique sorted list of tokens in the `from_market`.
    /// - The oracle prices from the feed accounts must be complete and valid.
    /// - If `throw_on_execution_error` is `true`, returns an error if execution fails.
    #[access_control(internal::Authenticate::only_order_keeper(&ctx))]
    pub fn execute_shift<'info>(
        ctx: Context<'_, '_, 'info, 'info, ExecuteShift<'info>>,
        execution_lamports: u64,
        throw_on_execution_error: bool,
    ) -> Result<()> {
        instructions::unchecked_execute_shift(ctx, execution_lamports, throw_on_execution_error)
    }

    /// Close a shift, either by the owner or by keepers.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](CloseShift)*
    ///
    /// # Arguments
    /// - `reason`: The reason for closing the shift.
    ///
    /// # Errors
    /// - The [`executor`](CloseShift::executor) must be a signer.
    /// - The [`store`](CloseShift::store) must be initialized.
    /// - The [`owner`](CloseShift::owner) must be the owner of the shift.
    /// - The [`shift`](CloseShift::shift) must be initialized and owned by both the `store` and `owner`.
    /// - The [`from_market_token`](CloseShift::from_market_token) and [`to_market_token`](CloseShift::to_market_token) must be valid and match those recorded in the [`shift`](CloseShift::shift).
    /// - The [`from_market_token_escrow`](CloseShift::from_market_token_escrow) and [`to_market_token_escrow`](CloseShift::to_market_token_escrow) must be valid escrow accounts owned by the `shift` and match those recorded in the [`shift`](CloseShift::shift).
    /// - The [`from_market_token_ata`](CloseShift::from_market_token_ata) must be a valid associated token account for [`from_market_token`](CloseShift::from_market_token) owned by `owner`.
    /// - The [`to_market_token_ata`](CloseShift::to_market_token_ata) must be a valid associated token account for [`to_market_token`](CloseShift::to_market_token) owned by `owner`.
    /// - If the `executor` is not the `owner`, the `shift` must be in either cancelled or completed state.
    pub fn close_shift<'info>(
        ctx: Context<'_, '_, 'info, 'info, CloseShift<'info>>,
        reason: String,
    ) -> Result<()> {
        internal::Close::close(&ctx, &reason)
    }

    // ===========================================
    //                The GT Model
    // ===========================================

    /// Initialize GT Mint.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](InitializeGt)*
    ///
    /// # Arguments
    /// - `decimals`: The number of decimal places for the GT token.
    /// - `initial_minting_cost`: The base cost to mint the GT token.
    /// - `grow_factor`: The multiplier that increases minting cost for each step.
    /// - `grow_step`: The minted amount between each cost increase.
    /// - `ranks`: Array of GT token thresholds that define user rank boundaries.
    ///
    /// # Errors
    /// - The [`authority`](InitializeGt::authority) must be a signer and have MARKET_KEEPER permissions in the store
    /// - The [`store`](InitializeGt::store) must be properly initialized
    /// - The GT state account must not already be initialized
    /// - The arguments must be valid. See `init` method of [`GtState`](states::gt::GtState) for detailed validation logic.
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
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](ConfigurateGt)*
    ///
    /// # Arguments
    /// - `factors`: The order fee discount factors for each rank.
    ///
    /// # Errors
    /// - The [`authority`](ConfigurateGt::authority) must be a signer and a
    ///   MARKET_KEEPER in the store.
    /// - The [`store`](ConfigurateGt::store) must be initialized.
    /// - The GT state must be initialized.
    /// - The number of `factors` must match the number of ranks defined in GT state.
    /// - Each factor must be less than or equal to [`MARKET_USD_UNIT`](crate::constants::MARKET_USD_UNIT).
    #[access_control(internal::Authenticate::only_market_keeper(&ctx))]
    pub fn gt_set_order_fee_discount_factors(
        ctx: Context<ConfigurateGt>,
        factors: Vec<u128>,
    ) -> Result<()> {
        instructions::unchecked_gt_set_order_fee_discount_factors(ctx, &factors)
    }

    /// Set referral reward factors.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](ConfigurateGt)*
    ///
    /// # Arguments
    /// - `factors`: The referral reward factors for each rank.
    ///
    /// # Errors
    /// - The [`authority`](ConfigurateGt::authority) must be a signer and a
    ///   GT_CONTROLLER in the store.
    /// - The [`store`](ConfigurateGt::store) must be initialized.
    /// - The GT state must be initialized.
    /// - The number of `factors` must match the number of ranks defined in GT state.
    /// - Each factor must be less than or equal to [`MARKET_USD_UNIT`](crate::constants::MARKET_USD_UNIT).
    #[access_control(internal::Authenticate::only_gt_controller(&ctx))]
    pub fn gt_set_referral_reward_factors(
        ctx: Context<ConfigurateGt>,
        factors: Vec<u128>,
    ) -> Result<()> {
        instructions::unchecked_gt_set_referral_reward_factors(ctx, &factors)
    }

    /// Set esGT receiver factor.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](ConfigurateGt)*
    ///
    /// # Arguments
    /// - `factor`: The factor determining what ratio of esGT rewards are minted to the receiver.
    ///
    /// # Errors
    /// - The [`authority`](ConfigurateGt::authority) must be a signer and a
    ///   GT_CONTROLLER in the store.
    /// - The [`store`](ConfigurateGt::store) must be initialized.
    /// - The GT state must be initialized.
    /// - The `factor` must be less than or equal to [`MARKET_USD_UNIT`](crate::constants::MARKET_USD_UNIT).
    #[access_control(internal::Authenticate::only_gt_controller(&ctx))]
    pub fn gt_set_es_receiver_factor(ctx: Context<ConfigurateGt>, factor: u128) -> Result<()> {
        instructions::unchecked_gt_set_es_receiver_factor(ctx, factor)
    }

    /// Set GT exchange time window (in seconds).
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](ConfigurateGt)*
    ///
    /// # Arguments
    /// - `window`: The time window in seconds for one GT exchange period.
    ///
    /// # Errors
    /// - The [`authority`](ConfigurateGt::authority) must be a signer and have
    ///   GT_CONTROLLER privileges in the store.
    /// - The [`store`](ConfigurateGt::store) must be properly initialized.
    /// - The GT state must be properly initialized.
    /// - The `window` must be greater than 0 seconds to ensure a valid exchange period.
    #[access_control(internal::Authenticate::only_gt_controller(&ctx))]
    pub fn gt_set_exchange_time_window(ctx: Context<ConfigurateGt>, window: u32) -> Result<()> {
        instructions::unchecked_gt_set_exchange_time_window(ctx, window)
    }

    /// Set esGT vault receiver.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](ConfigurateGt)*
    ///
    /// # Arguments
    /// - `receiver`: The public key of the account that can claim esGT rewards from the esGT vault.
    ///
    /// # Errors
    /// - The [`authority`](ConfigurateGt::authority) must be a signer and have
    ///   GT_CONTROLLER privileges in the store.
    /// - The [`store`](ConfigurateGt::store) must be properly initialized.
    /// - The GT state must be properly initialized.
    /// - The `receiver` must be a valid account that can receive esGT tokens.
    #[access_control(internal::Authenticate::only_gt_controller(&ctx))]
    pub fn gt_set_receiver(ctx: Context<ConfigurateGt>, receiver: Pubkey) -> Result<()> {
        instructions::unchecked_gt_set_receiver(ctx, &receiver)
    }

    /// Prepare GT Exchange Vault.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](PrepareGtExchangeVault)*
    ///
    /// # Arguments
    /// - `time_window_index`: The index of the current time window.
    /// - `time_window`: The current GT exchange time window in seconds.
    ///
    /// # Errors
    /// - The [`payer`](PrepareGtExchangeVault::payer) must be a signer.
    /// - The [`store`](PrepareGtExchangeVault::store) must be properly initialized.
    /// - The GT state must be properly initialized.
    /// - The [`vault`](PrepareGtExchangeVault::vault) must be either:
    ///   - Uninitialized, or
    ///   - Properly initialized, owned by the `store`, and have matching `time_window_index`
    ///     and `time_window` values
    /// - The provided `time_window_index` must match the current time window index
    /// - The provided `time_window` must match the current GT exchange time window
    pub fn prepare_gt_exchange_vault(
        ctx: Context<PrepareGtExchangeVault>,
        time_window_index: i64,
        time_window: u32,
    ) -> Result<()> {
        instructions::prepare_gt_exchange_vault(ctx, time_window_index, time_window)
    }

    /// Confirm GT exchange vault.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](ConfirmGtExchangeVault)*
    ///
    /// # Errors
    /// - The [`authority`](ConfirmGtExchangeVault::authority) must be a signer and have
    ///   GT_CONTROLLER privileges in the store.
    /// - The [`store`](ConfirmGtExchangeVault::store) must be properly initialized.
    /// - The GT state must be properly initialized.
    /// - The [`vault`](ConfirmGtExchangeVault::vault) must be validly initialized and owned by
    ///   the `store`.
    /// - The `vault` must be in a confirmable state (deposit window has passed but not yet confirmed).
    #[access_control(internal::Authenticate::only_gt_controller(&ctx))]
    pub fn confirm_gt_exchange_vault(ctx: Context<ConfirmGtExchangeVault>) -> Result<()> {
        instructions::unchecked_confirm_gt_exchange_vault(ctx)
    }

    /// Request a GT exchange.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](RequestGtExchange)*
    ///
    /// # Arguments
    /// - `amount`: The amount of GT to exchange for rewards.
    ///
    /// # Errors
    /// - The [`owner`](RequestGtExchange::owner) must be a signer.
    /// - The [`store`](RequestGtExchange::store) must be properly initialized with an initialized GT state.
    /// - The [`user`](RequestGtExchange::user) must be properly initialized and owned by the `owner`.
    /// - The [`vault`](RequestGtExchange::vault) must be properly initialized, owned by the `store`,
    ///   and currently accepting deposits.
    /// - The [`exchange`](RequestGtExchange::exchange) must be either:
    ///   - Uninitialized, or
    ///   - Properly initialized and owned by both the `owner` and `vault`
    /// - The `amount` must be:
    ///   - Greater than 0
    ///   - Not exceed the owner's available (non-reserved) GT balance in their user account
    pub fn request_gt_exchange(ctx: Context<RequestGtExchange>, amount: u64) -> Result<()> {
        instructions::request_gt_exchange(ctx, amount)
    }

    /// Close a confirmed GT exchange.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](CloseGtExchange)*
    ///
    /// # Errors
    /// - The [`authority`](CloseGtExchange::authority) must be a signer and have
    ///   the GT_CONTROLLER role in the store.
    /// - The [`store`](CloseGtExchange::store) must be properly initialized with an initialized GT state.
    /// - The [`vault`](CloseGtExchange::vault) must be properly initialized, owned by the `store`,
    ///   and in a confirmed state.
    /// - The [`exchange`](CloseGtExchange::exchange) must be properly initialized and owned by both
    ///   the `owner` and `vault`.
    #[access_control(internal::Authenticate::only_gt_controller(&ctx))]
    pub fn close_gt_exchange(ctx: Context<CloseGtExchange>) -> Result<()> {
        instructions::unchecked_close_gt_exchange(ctx)
    }

    /// Claim pending esGT of the owner.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](ClaimEsGt)*
    ///
    /// # Errors
    /// - The [`owner`](ClaimEsGt::owner) must be a signer.
    /// - The [`store`](ClaimEsGt::store) must be properly initialized with an initialized GT state.
    /// - The [`user`](ClaimEsGt::user) must be properly initialized and owned by the `owner`.
    /// - The `user` must have pending esGT to claim.
    pub fn claim_es_gt(ctx: Context<ClaimEsGt>) -> Result<()> {
        instructions::claim_es_gt(ctx)
    }

    /// Request GT vesting.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](RequestGtVesting)*
    ///
    /// # Arguments
    /// - `amount`: The amount of esGT to vest into GT.
    ///
    /// # Errors
    /// - The [`owner`](RequestGtVesting::owner) must be a signer.
    /// - The [`store`](RequestGtVesting::store) must be properly initialized with an initialized GT state.
    /// - The [`user`](RequestGtVesting::user) must be properly initialized and owned by the `owner`.
    /// - The [`vesting`](RequestGtVesting::vesting) must be either:
    ///   - Uninitialized, or
    ///   - Properly initialized and owned by both the `owner` and `store`
    /// - The `amount` must not exceed the owner's esGT balance in their user account.
    /// - The owner must have sufficient GT reserved in their user account to cover the total vesting amount
    ///   after this request is processed (i.e., `reserve_factor * total_vesting_esgt <= gt_balance`).
    pub fn request_gt_vesting(ctx: Context<RequestGtVesting>, amount: u64) -> Result<()> {
        instructions::request_gt_vesting(ctx, amount)
    }

    /// Update GT vesting state for the owner. This can be used to claim the vested GT.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](UpdateGtVesting)*
    ///
    /// # Errors
    /// - The [`owner`](UpdateGtVesting::owner) must be a signer.
    /// - The [`store`](UpdateGtVesting::store) must be properly initialized with an initialized GT state.
    /// - The [`user`](UpdateGtVesting::user) must be properly initialized and owned by the `owner`.
    /// - The [`vesting`](UpdateGtVesting::vesting) must be properly initialized and owned by both
    ///   the `owner` and `store`.
    pub fn update_gt_vesting(ctx: Context<UpdateGtVesting>) -> Result<()> {
        instructions::update_gt_vesting(ctx)
    }

    /// Close GT vesting account.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](CloseGtVesting)*
    ///
    /// # Errors
    /// - The [`owner`](CloseGtVesting::owner) must be a signer.
    /// - The [`store`](CloseGtVesting::store) must be properly initialized with an initialized GT state.
    /// - The [`user`](CloseGtVesting::user) must be properly initialized and owned by the `owner`.
    /// - The [`vesting`](CloseGtVesting::vesting) must be properly initialized and owned by both
    ///   the `owner` and `store`. The vesting account must have no remaining unvested esGT.
    pub fn close_gt_vesting(ctx: Context<CloseGtVesting>) -> Result<()> {
        instructions::close_gt_vesting(ctx)
    }

    /// Claim esGT from a vault through a vesting account.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](ClaimEsGtVaultViaVesting)*
    ///
    /// # Arguments
    /// - `amount`: The amount of esGT to claim from the vault.
    ///
    /// # Errors
    /// - The [`owner`](ClaimEsGtVaultViaVesting::owner) must be a signer and the designated recipient of the esGT vault.
    /// - The [`store`](ClaimEsGtVaultViaVesting::store) must be properly initialized with an initialized GT state.
    /// - The [`user`](ClaimEsGtVaultViaVesting::user) must be properly initialized and owned by the `owner`.
    /// - The [`vesting`](ClaimEsGtVaultViaVesting::vesting) must be properly initialized and owned by both
    ///   the `owner` and `store`.
    /// - The requested `amount` must not exceed the available esGT balance in the vault.
    pub fn claim_es_gt_vault_via_vesting(
        ctx: Context<ClaimEsGtVaultViaVesting>,
        amount: u64,
    ) -> Result<()> {
        instructions::claim_es_gt_vault_via_vesting(ctx, amount)
    }

    // ===========================================
    //              User & Referral
    // ===========================================

    /// Initialize or validate a User Account.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](PrepareUser)*
    ///
    /// # Errors
    /// - The [`owner`](PrepareUser::owner) must be a signer.
    /// - The [`store`](PrepareUser::store) must be properly initialized.
    /// - The [`user`](PrepareUser::user) must be either:
    ///   - Uninitialized (for new account creation)
    ///   - Or validly initialized and owned by the `store`
    pub fn prepare_user(ctx: Context<PrepareUser>) -> Result<()> {
        instructions::prepare_user(ctx)
    }

    /// Initialize referral code.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](InitializeReferralCode)*
    ///
    /// # Arguments
    /// - `code`: The referral code to initialize and associate with the user.
    ///
    /// # Errors
    /// - The [`owner`](InitializeReferralCode::owner) must be a signer.
    /// - The [`store`](InitializeReferralCode::store) must be properly initialized.
    /// - The [`referral_code`](InitializeReferralCode::referral_code) account must be uninitialized.
    /// - The [`user`](InitializeReferralCode::user) account must be:
    ///   - Properly initialized
    ///   - Owned by the `owner`
    ///   - Not already have an associated referral code
    /// - The provided `code` must not already be in use by another user.
    pub fn initialize_referral_code(
        ctx: Context<InitializeReferralCode>,
        code: [u8; 8],
    ) -> Result<()> {
        instructions::initialize_referral_code(ctx, code)
    }

    /// Set referrer.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](SetReferrer)*
    ///
    /// # Arguments
    /// - `code`: The referral code of the referrer.
    ///
    /// # Errors
    /// - The [`owner`](SetReferrer::owner) must be a signer.
    /// - The [`store`](SetReferrer::store) must be properly initialized.
    /// - The [`user`](SetReferrer::user) must be:
    ///   - Properly initialized
    ///   - Owned by the `owner`
    ///   - Must not already have a referrer set
    /// - The [`referral_code`](SetReferrer::referral_code) must be:
    ///   - Properly initialized
    ///   - Owned by the `store`
    ///   - Match the provided `code`
    ///   - Be owned by the `referrer_user`
    /// - The [`referrer_user`](SetReferrer::referrer_user) must be:
    ///   - Properly initialized
    ///   - Different from the `user`
    ///   - Not have the `user` as their referrer (no circular references)
    pub fn set_referrer(ctx: Context<SetReferrer>, code: [u8; 8]) -> Result<()> {
        instructions::set_referrer(ctx, code)
    }

    /// Transfer referral code.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](TransferReferralCode)*
    ///
    /// # Errors
    /// - The [`owner`](TransferReferralCode::owner) must be a signer.
    /// - The [`store`](TransferReferralCode::store) must be properly initialized.
    /// - The [`user`](TransferReferralCode::user) account must be:
    ///   - Properly initialized
    ///   - Owned by the `owner`
    ///   - Different from the [`receiver_user`](TransferReferralCode::receiver_user)
    /// - The [`referral_code`](TransferReferralCode::referral_code) account must be:
    ///   - Properly initialized
    ///   - Owned by the `store`
    ///   - Owned by the `user`
    /// - The [`receiver_user`](TransferReferralCode::receiver_user) account must be:
    ///   - Properly initialized
    ///   - Not have an associated referral code
    pub fn transfer_referral_code(ctx: Context<TransferReferralCode>) -> Result<()> {
        instructions::transfer_referral_code(ctx)
    }

    // ===========================================
    //                GLV Operations
    // ===========================================

    /// Initialize GLV.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](InitializeGlv)*
    ///
    /// # Arguments
    /// - `index`: The index of the GLV. Used to derive the GLV token address.
    /// - `length`: The number of markets to include in the GLV.
    ///
    /// # Errors
    /// - The [`authority`](InitializeGlv::authority) must be a signer and have
    ///   MARKET_KEEPER role in the store
    /// - The [`store`](InitializeGlv::store) must be properly initialized
    /// - The [`glv_token`](InitializeGlv::glv_token) must be:
    ///   - Uninitialized
    ///   - Address must be PDA derived from [`GLV_TOKEN_SEED`](crate::states::Glv::GLV_TOKEN_SEED),
    ///     [`store`] and `index`
    /// - The [`glv`](InitializeGlv::glv) must be:
    ///   - Uninitialized  
    ///   - Address must be PDA derived from the SEED of [`Glv`](states::Glv) and [`glv_token`](InitializeGlv::glv_token)
    /// - The remaining required accounts are documented in [`InitializeGlv`]
    /// - The `length` must be:
    ///   - Greater than 0
    ///   - Less than or equal to [`Glv::MAX_ALLOWED_NUMBER_OF_MARKETS`](crate::states::Glv::MAX_ALLOWED_NUMBER_OF_MARKETS)
    #[access_control(internal::Authenticate::only_market_keeper(&ctx))]
    pub fn initialize_glv<'info>(
        ctx: Context<'_, '_, 'info, 'info, InitializeGlv<'info>>,
        index: u8,
        length: u16,
    ) -> Result<()> {
        instructions::unchecked_initialize_glv(ctx, index, length as usize)
    }

    /// Update a market config of a GLV.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](UpdateGlvMarketConfig)*
    ///
    /// # Arguments
    /// - `max_amount`: The maximum amount of the market token that can be stored in the GLV.
    /// - `max_value`: The maximum value of the market token that can be stored in the GLV.
    ///
    /// # Errors
    /// - The [`authority`](UpdateGlvMarketConfig::authority) must be:
    ///   - A signer
    ///   - Have MARKET_KEEPER role in the store
    /// - The [`store`](UpdateGlvMarketConfig::store) must be properly initialized
    /// - The [`glv`](UpdateGlvMarketConfig::glv) must be:
    ///   - Properly initialized
    ///   - Owned by the `store`
    ///   - Have the market token in its list of markets
    /// - At least one of `max_amount` or `max_value` must be provided
    #[access_control(internal::Authenticate::only_market_keeper(&ctx))]
    pub fn update_glv_market_config(
        ctx: Context<UpdateGlvMarketConfig>,
        max_amount: Option<u64>,
        max_value: Option<u128>,
    ) -> Result<()> {
        instructions::unchecked_update_glv_market_config(ctx, max_amount, max_value)
    }

    /// Create GLV deposit.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](CreateGlvDeposit)*
    ///
    /// # Arguments
    /// - `nonce`: A 32-byte unique identifier for the GLV deposit.
    /// - `params`: The parameters for creating the GLV deposit, including initial token amounts and configuration.
    ///
    /// # Errors
    /// - The [`owner`](CreateGlvDeposit::owner) must be a signer
    /// - The [`store`](CreateGlvDeposit::store) must be properly initialized
    /// - The [`market`](CreateGlvDeposit::market) must be:
    ///   - Properly initialized
    ///   - Owned by the `store`
    ///   - Listed as a valid market in the [`glv`](CreateGlvDeposit::glv)
    /// - The [`glv`](CreateGlvDeposit::glv) must be:
    ///   - Properly initialized
    ///   - Owned by the `store`
    /// - The [`glv_deposit`](CreateGlvDeposit::glv_deposit) must be:
    ///   - Uninitialized
    ///   - Address must be PDA derived from the SEED of [`GlvDeposit`](states::GlvDeposit),
    ///     [`store`](CreateGlvDeposit::store), [`owner`](CreateGlvDeposit::owner) and `nonce`
    /// - The [`glv_token`](CreateGlvDeposit::glv_token) must be:
    ///   - Properly initialized
    ///   - Associated with the provided [`glv`](CreateGlvDeposit::glv)
    /// - The [`market_token`](CreateGlvDeposit::market_token) must be:
    ///   - Properly initialized
    ///   - Associated with the provided [`market`](CreateGlvDeposit::market)
    /// - Token account requirements:
    ///   - [`initial_long_token`](CreateGlvDeposit::initial_long_token) must be provided if initial long amount > 0
    ///   - [`initial_short_token`](CreateGlvDeposit::initial_short_token) must be provided if initial short amount > 0
    ///   - Source token accounts must be provided for any non-zero initial token amounts
    /// - Escrow account requirements:
    ///   - [`glv_token_escrow`](CreateGlvDeposit::glv_token_escrow) must be:
    ///     - Provided
    ///     - Owned by the [`glv_deposit`](CreateGlvDeposit::glv_deposit)
    ///   - Other escrow accounts must be:
    ///     - Provided for any non-zero initial token amounts
    ///     - Have sufficient balance
    ///     - Owned by the [`glv_deposit`](CreateGlvDeposit::glv_deposit)
    /// - All token programs must match their corresponding token accounts
    pub fn create_glv_deposit<'info>(
        mut ctx: Context<'_, '_, 'info, 'info, CreateGlvDeposit<'info>>,
        nonce: [u8; 32],
        params: CreateGlvDepositParams,
    ) -> Result<()> {
        internal::Create::create(&mut ctx, &nonce, &params)
    }

    /// Close GLV deposit.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](CloseGlvDeposit)*
    ///
    /// # Arguments
    /// - `reason`: The reason for closing the GLV deposit.
    ///
    /// # Errors
    /// - The [`executor`](CloseGlvDeposit::executor) must be a signer, and must be
    ///   either the owner of the GLV deposit or a `ORDER_KEEPER` in the store
    /// - The [`store`](CloseGlvDeposit::store) must be properly initialized
    /// - The [`owner`](CloseGlvDeposit::owner) must be the owner of the GLV deposit
    /// - The [`glv_deposit`](CloseGlvDeposit::glv_deposit) must be:
    ///   - Properly initialized
    ///   - Owned by the `owner` and `store`
    ///   - In cancelled or executed state if the `executor` is not the `owner`
    /// - Token account requirements:
    ///   - All tokens must be valid and recorded in the [`glv_deposit`](CloseGlvDeposit::glv_deposit)
    ///   - [`initial_long_token`](CloseGlvDeposit::initial_long_token) must be provided if initial long amount > 0
    ///   - [`initial_short_token`](CloseGlvDeposit::initial_short_token) must be provided if initial short amount > 0
    /// - Escrow account requirements:
    ///   - Must correspond to their respective tokens
    ///   - Must be owned by the [`glv_deposit`](CloseGlvDeposit::glv_deposit)
    ///   - Must be recorded in the [`glv_deposit`](CloseGlvDeposit::glv_deposit)
    /// - Associated Token Account requirements:
    ///   - Must correspond to their respective tokens
    ///   - Must be owned by the `owner`
    /// - All token programs must match their corresponding token accounts
    pub fn close_glv_deposit<'info>(
        ctx: Context<'_, '_, 'info, 'info, CloseGlvDeposit<'info>>,
        reason: String,
    ) -> Result<()> {
        internal::Close::close(&ctx, &reason)
    }

    /// Execute GLV deposit.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](ExecuteGlvDeposit)*
    ///
    /// # Arguments
    /// - `execution_lamports`: The execution fee in lamports to be claimed by the keeper.
    /// - `throw_on_execution_error`: Whether to throw an error if the execution fails.
    ///
    /// # Errors
    /// - The [`authority`](ExecuteGlvDeposit::authority) must be a signer and have `ORDER_KEEPER` role in the store
    /// - The [`store`](ExecuteGlvDeposit::store) must be properly initialized
    /// - The [`token_map`](ExecuteGlvDeposit::token_map) must be:
    ///   - Properly initialized
    ///   - Authorized by the store
    /// - The [`oracle`](ExecuteGlvDeposit::oracle) must be:
    ///   - Cleared
    ///   - Owned by the store
    /// - The [`glv`](ExecuteGlvDeposit::glv) must be:
    ///   - Properly initialized
    ///   - Owned by the store
    ///   - Match the expected GLV of the deposit
    /// - The [`market`](ExecuteGlvDeposit::market) must be:
    ///   - Properly initialized
    ///   - Owned by the store
    ///   - Match the expected market of the deposit
    /// - The [`glv_deposit`](ExecuteGlvDeposit::glv_deposit) must be:
    ///   - Properly initialized
    ///   - Owned by the store
    ///   - In pending state
    /// - Token requirements:
    ///   - All tokens must be valid and recorded in the [`glv_deposit`](ExecuteGlvDeposit::glv_deposit)
    ///   - [`glv_token`](ExecuteGlvDeposit::glv_token) must be the GLV token of the [`glv`](ExecuteGlvDeposit::glv)
    ///   - [`market_token`](ExecuteGlvDeposit::market_token) must be the market token of the [`market`](ExecuteGlvDeposit::market)
    /// - Vault requirements:
    ///   - [`initial_long_token_vault`](ExecuteGlvDeposit::initial_long_token_vault) must be:
    ///     - The market vault for the initial long token
    ///     - Owned by the store
    ///   - [`initial_short_token_vault`](ExecuteGlvDeposit::initial_short_token_vault) must be:
    ///     - The market vault for the initial short token
    ///     - Owned by the store
    ///   - [`market_token_vault`](ExecuteGlvDeposit::market_token_vault) must be:
    ///     - The market token vault in the GLV
    ///     - Owned by the GLV
    /// - Escrow requirements:
    ///   - Must correspond to their respective tokens
    ///   - Must be owned by the GLV deposit
    ///   - Must be recorded in the GLV deposit
    /// - All token programs must match their corresponding token accounts
    /// - All remaining accounts must be valid per [`ExecuteGlvDeposit`] documentation
    /// - Returns error if execution fails and `throw_on_execution_error` is `true`
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
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](CreateGlvWithdrawal)*
    ///
    /// # Arguments
    /// - `nonce`: The nonce for the GLV withdrawal.
    /// - `params`: The parameters for the GLV withdrawal.
    ///
    /// # Errors
    /// - The [`owner`](CreateGlvWithdrawal::owner) must be:
    ///   - A signer
    /// - The [`store`](CreateGlvWithdrawal::store) must be:
    ///   - Properly initialized
    /// - The [`market`](CreateGlvWithdrawal::market) must be:
    ///   - Properly initialized
    ///   - Enabled
    ///   - Owned by the store
    ///   - One of the markets in the GLV
    /// - The [`glv`](CreateGlvWithdrawal::glv) must be:
    ///   - Properly initialized
    ///   - Owned by the store
    /// - The [`glv_withdrawal`](CreateGlvWithdrawal::glv_withdrawal) must be:
    ///   - Uninitialized
    ///   - A PDA derived from:
    ///     - the SEED of [`GlvWithdrawal`](states::GlvWithdrawal)
    ///     - [`store`](CreateGlvWithdrawal::store)
    ///     - [`owner`](CreateGlvWithdrawal::owner)
    ///     - `nonce`
    /// - Token requirements:
    ///   - [`glv_token`](CreateGlvWithdrawal::glv_token) must be:
    ///     - Properly initialized
    ///     - The GLV token of the [`glv`](CreateGlvWithdrawal::glv)
    ///   - [`market_token`](CreateGlvWithdrawal::market_token) must be:
    ///     - Properly initialized  
    ///     - The market token of the [`market`](CreateGlvWithdrawal::market)
    ///   - All other tokens must be properly initialized
    /// - Source requirements:
    ///   - [`glv_token_source`](CreateGlvWithdrawal::glv_token_source) must be:
    ///     - Properly initialized
    ///     - A GLV token account
    ///     - Have sufficient balance
    ///     - Have the `owner` as its authority
    /// - Escrow requirements:
    ///   - Must correspond to their respective tokens
    ///   - Must be owned by the [`glv_withdrawal`](CreateGlvWithdrawal::glv_withdrawal)
    /// - All token programs must match their corresponding token accounts
    pub fn create_glv_withdrawal<'info>(
        mut ctx: Context<'_, '_, 'info, 'info, CreateGlvWithdrawal<'info>>,
        nonce: [u8; 32],
        params: CreateGlvWithdrawalParams,
    ) -> Result<()> {
        internal::Create::create(&mut ctx, &nonce, &params)
    }

    /// Close GLV withdrawal.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](CloseGlvWithdrawal)*
    ///
    /// # Arguments
    /// - `reason`: The reason for closing the GLV withdrawal.
    ///
    /// # Errors
    /// - The [`executor`](CloseGlvWithdrawal::executor) must be:
    ///   - A signer
    ///   - Either:
    ///     - The owner of the GLV withdrawal
    ///     - A `ORDER_KEEPER` in the store
    /// - The [`store`](CloseGlvWithdrawal::store) must be:
    ///   - Properly initialized
    /// - The [`owner`](CloseGlvWithdrawal::owner) must be:
    ///   - The owner of the GLV withdrawal
    /// - The [`glv_withdrawal`](CloseGlvWithdrawal::glv_withdrawal) must be:
    ///   - Properly initialized
    ///   - Owned by the `owner`
    ///   - Owned by the `store`
    /// - Token requirements:
    ///   - All tokens must be valid and recorded in the [`glv_withdrawal`](CloseGlvWithdrawal::glv_withdrawal)
    /// - Escrow requirements:
    ///   - Must correspond to their respective tokens
    ///   - Must be owned by the [`glv_withdrawal`](CloseGlvWithdrawal::glv_withdrawal)
    ///   - Must be recorded in the [`glv_withdrawal`](CloseGlvWithdrawal::glv_withdrawal)
    /// - ATA requirements:
    ///   - Must correspond to their respective tokens
    ///   - Must be owned by the `owner`
    /// - All token programs must match their corresponding token accounts
    /// - State requirements:
    ///   - If the `executor` is not the `owner`, the `glv_withdrawal` must be:
    ///     - Either cancelled or executed
    pub fn close_glv_withdrawal<'info>(
        ctx: Context<'_, '_, 'info, 'info, CloseGlvWithdrawal<'info>>,
        reason: String,
    ) -> Result<()> {
        internal::Close::close(&ctx, &reason)
    }

    /// Execute GLV withdrawal.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](ExecuteGlvWithdrawal)*
    ///
    /// # Arguments
    /// - `execution_lamports`: The amount of lamports to be paid as execution fee to the keeper.
    /// - `throw_on_execution_error`: If true, throws an error when execution fails. If false, returns success even if execution fails.
    ///
    /// # Errors
    /// - The [`authority`](ExecuteGlvWithdrawal::authority) must be:
    ///   - A signer
    ///   - A `ORDER_KEEPER` in the store
    /// - The [`store`](ExecuteGlvWithdrawal::store) must be:
    ///   - Properly initialized
    /// - The [`token_map`](ExecuteGlvWithdrawal::token_map) must be:
    ///   - Properly initialized
    ///   - Authorized by the store
    /// - The [`oracle`](ExecuteGlvWithdrawal::oracle) must be:
    ///   - Cleared
    ///   - Owned by the store
    /// - The [`glv`](ExecuteGlvWithdrawal::glv) must be:
    ///   - Properly initialized
    ///   - Owned by the store
    ///   - The expected GLV of the withdrawal
    /// - The [`market`](ExecuteGlvWithdrawal::market) must be:
    ///   - Properly initialized
    ///   - Owned by the store
    ///   - The expected market of the withdrawal
    /// - The [`glv_withdrawal`](ExecuteGlvWithdrawal::glv_withdrawal) must be:
    ///   - Properly initialized
    ///   - Owned by the store
    ///   - In pending state
    /// - Token requirements:
    ///   - All tokens must be valid and recorded in the withdrawal
    ///   - [`glv_token`](ExecuteGlvWithdrawal::glv_token) must be the GLV token of the GLV
    ///   - [`market_token`](ExecuteGlvWithdrawal::market_token) must be the market token of the market
    /// - Vault requirements:
    ///   - Escrow accounts must correspond to their tokens
    ///   - Escrow accounts must be owned by the withdrawal
    ///   - Escrow accounts must be recorded in the withdrawal
    ///   - [`market_token_withdrawal_vault`](ExecuteGlvWithdrawal::market_token_withdrawal_vault) must be the market vault for market token, owned by store
    ///   - [`final_long_token_vault`](ExecuteGlvWithdrawal::final_long_token_vault) must be the market vault for final long token, owned by store
    ///   - [`final_short_token_vault`](ExecuteGlvWithdrawal::final_short_token_vault) must be the market vault for final short token, owned by store
    ///   - [`market_token_vault`](ExecuteGlvWithdrawal::market_token_vault) must be the GLV's market token vault, owned by GLV
    /// - All token programs must match their corresponding token accounts
    /// - All remaining accounts must be valid per [`ExecuteGlvWithdrawal`] documentation
    /// - If `throw_on_execution_error` is true, execution failures will throw an error
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
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](CreateGlvShift)*
    ///
    /// # Arguments
    /// - `nonce`: The nonce for the GLV shift.
    /// - `params`: The parameters for the GLV shift.
    ///
    /// # Requirements
    /// - The [`authority`](CreateGlvShift::authority) must be:
    ///   - A signer
    ///   - A `ORDER_KEEPER` in the store
    /// - The [`store`](CreateGlvShift::store) must be:
    ///   - Properly initialized
    /// - The [`glv`](CreateGlvShift::glv) must be:
    ///   - Properly initialized
    ///   - Owned by the store
    /// - Market requirements:
    ///   - [`from_market`](CreateGlvShift::from_market) must be:
    ///     - Enabled
    ///     - Owned by the store
    ///     - One of the markets in the GLV
    ///   - [`to_market`](CreateGlvShift::to_market) must be:
    ///     - Enabled
    ///     - Owned by the store
    ///     - One of the markets in the GLV
    ///     - Different from `from_market`
    /// - The [`glv_shift`](CreateGlvShift::glv_shift) must be:
    ///   - Uninitialized
    ///   - PDA derived from the SEED of [`GlvShift`](states::GlvShift), `store`, `glv`, and `nonce`
    /// - Token requirements:
    ///   - [`from_market_token`](CreateGlvShift::from_market_token) must be:
    ///     - Properly initialized
    ///     - The market token of `from_market`
    ///   - [`to_market_token`](CreateGlvShift::to_market_token) must be:
    ///     - Properly initialized
    ///     - The market token of `to_market`
    /// - Vault requirements:
    ///   - [`from_market_token_vault`](CreateGlvShift::from_market_token_vault) must be:
    ///     - The market token vault for `from_market_token` in the GLV
    ///     - Owned by the GLV
    ///   - [`to_market_token_vault`](CreateGlvShift::to_market_token_vault) must be:
    ///     - The market token vault for `to_market_token` in the GLV
    ///     - Owned by the GLV
    #[access_control(internal::Authenticate::only_order_keeper(&ctx))]
    pub fn create_glv_shift<'info>(
        mut ctx: Context<'_, '_, 'info, 'info, CreateGlvShift<'info>>,
        nonce: [u8; 32],
        params: CreateShiftParams,
    ) -> Result<()> {
        internal::Create::create(&mut ctx, &nonce, &params)
    }

    /// Close a GLV shift.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](CloseGlvShift)*
    ///
    /// # Arguments
    /// - `reason`: The reason for closing the GLV shift.
    ///
    /// # Requirements
    /// - The [`authority`](CloseGlvShift::authority) must be:
    ///   - A signer
    ///   - A `ORDER_KEEPER` in the store
    /// - The [`funder`](CloseGlvShift::funder) must be:
    ///   - The funder of the GLV shift
    /// - The [`store`](CloseGlvShift::store) must be:
    ///   - Properly initialized
    /// - The [`glv`](CloseGlvShift::glv) must be:
    ///   - Properly initialized
    ///   - Owned by the store
    ///   - The expected GLV of the GLV shift
    /// - The [`glv_shift`](CloseGlvShift::glv_shift) must be:
    ///   - Properly initialized
    ///   - Owned by the store
    /// - Token requirements:
    ///   - [`from_market_token`](CloseGlvShift::from_market_token) must be:
    ///     - Recorded in the GLV shift
    ///   - [`to_market_token`](CloseGlvShift::to_market_token) must be:
    ///     - Recorded in the GLV shift
    #[access_control(internal::Authenticate::only_order_keeper(&ctx))]
    pub fn close_glv_shift<'info>(
        ctx: Context<'_, '_, 'info, 'info, CloseGlvShift<'info>>,
        reason: String,
    ) -> Result<()> {
        internal::Close::close(&ctx, &reason)
    }

    /// Execute GLV shift.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](ExecuteGlvShift)*
    ///
    /// # Arguments
    /// - `execution_lamports`: The execution fee claimed by the keeper.
    /// - `throw_on_execution_error`: Whether to throw an error if execution fails.
    ///
    /// # Requirements
    /// - The [`authority`](ExecuteGlvShift::authority) must be:
    ///   - A signer
    ///   - A `ORDER_KEEPER` in the store
    /// - The [`store`](ExecuteGlvShift::store) must be:
    ///   - Properly initialized
    /// - The [`token_map`](ExecuteGlvShift::token_map) must be:
    ///   - Properly initialized
    ///   - Authorized by the store
    /// - The [`oracle`](ExecuteGlvShift::oracle) must be:
    ///   - Cleared
    ///   - Owned by the store
    /// - The [`glv`](ExecuteGlvShift::glv) must be:
    ///   - Properly initialized
    ///   - Owned by the store
    ///   - The expected GLV of the GLV shift
    /// - The [`from_market`](ExecuteGlvShift::from_market) must be:
    ///   - Enabled
    ///   - Owned by the store
    ///   - One of the markets in the GLV
    /// - The [`to_market`](ExecuteGlvShift::to_market) must be:
    ///   - Enabled
    ///   - Owned by the store
    ///   - One of the markets in the GLV
    /// - The [`glv_shift`](ExecuteGlvShift::glv_shift) must be:
    ///   - Properly initialized
    ///   - Owned by the store
    /// - Token requirements:
    ///   - [`from_market_token`](ExecuteGlvShift::from_market_token) must be:
    ///     - The market token of `from_market`
    ///     - Recorded in the GLV shift
    ///   - [`to_market_token`](ExecuteGlvShift::to_market_token) must be:
    ///     - The market token of `to_market`
    ///     - Recorded in the GLV shift
    ///   - [`from_market_token_glv_vault`](ExecuteGlvShift::from_market_token_glv_vault) must be:
    ///     - The escrow account for `from_market_token` in the GLV
    ///     - Owned by the GLV
    ///   - [`to_market_token_glv_vault`](ExecuteGlvShift::to_market_token_glv_vault) must be:
    ///     - The escrow account for `to_market_token` in the GLV
    ///     - Owned by the GLV
    ///   - [`from_market_token_vault`](ExecuteGlvShift::from_market_token_vault) must be:
    ///     - The market vault for `from_market_token`
    ///     - Owned by the store
    ///   - Token programs must match the tokens and token accounts
    /// - The remaining accounts must be valid (see [`ExecuteGlvShift`] docs)
    /// - Returns error if execution fails and `throw_on_execution_error` is `true`
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
    // ===========================================
    //                Common Errors
    // ===========================================
    /// Non-defualt store is not allowed.
    #[msg("non-default store is not allowed")]
    NonDefaultStore,
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
    // ===========================================
    //                 Store Errors
    // ===========================================
    /// Invalid Store Config Key.
    #[msg("invalid store config key")]
    InvalidStoreConfigKey,
    // ===========================================
    //                Oracle Errors
    // ===========================================
    /// Invalid Provider Kind Index.
    #[msg("invalid provider kind index")]
    InvalidProviderKindIndex,
    /// Chainlink Program is required.
    #[msg("chainlink program is required")]
    ChainlinkProgramIsRequired,
    /// Not supported price provider for custom price feed.
    #[msg("this price provider is not supported to be used with custom price feed")]
    NotSupportedCustomPriceProvider,
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
    /// Price Overflow.
    #[msg("price overflow")]
    PriceOverflow,
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
    /// Invalid Price Report.
    #[msg("invalid price report")]
    InvalidPriceReport,
    /// Market not opened.
    #[msg("market is not open")]
    MarketNotOpen,
    // ===========================================
    //                Deposit Errors
    // ===========================================
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
    // ===========================================
    //               Withdrawal Errors
    // ===========================================
    /// Empty Withdrawal.
    #[msg("emtpy withdrawal")]
    EmptyWithdrawal,
    // ===========================================
    //                 Order Errors
    // ===========================================
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
    // ===========================================
    //                 Shift Errors
    // ===========================================
    /// Empty Shift.
    #[msg("emtpy shift")]
    EmptyShift,
    /// Invalid Shift Markets
    #[msg("invalid shift markets")]
    InvalidShiftMarkets,
    // ===========================================
    //        GT and User Accounts Errors
    // ===========================================
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
    // ===========================================
    //               Referral Errors
    // ===========================================
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
    // ===========================================
    //                Market Errors
    // ===========================================
    /// Invalid market config key.
    #[msg("invalid market config key")]
    InvalidMarketConfigKey,
    /// Invalid collateral token.
    #[msg("invalid collateral token")]
    InvalidCollateralToken,
    /// Disabled market.
    #[msg("disabled market")]
    DisabledMarket,
    // ===========================================
    //                  GLV Errors
    // ===========================================
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
    // ===========================================
    //                Other Errors
    // ===========================================
    /// The decimals of token is immutable.
    #[msg("The decimals of token is immutable")]
    TokenDecimalsChanged,
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
