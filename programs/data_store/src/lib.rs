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
        common::{SwapParams, TokenRecord},
        deposit::TokenParams as DepositTokenParams,
        market::{MarketMeta, Pool},
        order::OrderParams,
        token_config::{TokenConfig, TokenConfigBuilder},
        withdrawal::TokenParams as WithdrawalTokenParams,
        PriceProviderKind,
    },
    utils::internal,
};
use gmx_solana_utils::price::Price;

#[cfg_attr(test, macro_use)]
extern crate static_assertions;

declare_id!("hndKzPMrB9Xzs3mwarnPdkSWpZPZN3gLeeNzHDcHotT");

#[program]
pub mod data_store {
    use super::*;

    // Data Store.
    pub fn initialize(ctx: Context<Initialize>, key: String) -> Result<()> {
        instructions::initialize(ctx, key)
    }

    // Roles.
    pub fn initialize_roles(ctx: Context<InitializeRoles>, authority: Pubkey) -> Result<()> {
        instructions::initialize_roles(ctx, authority)
    }

    pub fn check_admin(ctx: Context<CheckRole>) -> Result<bool> {
        instructions::check_admin(ctx)
    }

    pub fn check_role(ctx: Context<CheckRole>, role: String) -> Result<bool> {
        instructions::check_role(ctx, role)
    }

    pub fn has_admin(ctx: Context<HasRole>, authority: Pubkey) -> Result<bool> {
        instructions::has_admin(ctx, authority)
    }

    pub fn has_role(ctx: Context<HasRole>, authority: Pubkey, role: String) -> Result<bool> {
        instructions::has_role(ctx, authority, role)
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

    // Config.
    #[access_control(internal::Authenticate::only_controller(&ctx))]
    pub fn initialize_config(ctx: Context<InitializeConfig>) -> Result<()> {
        instructions::initialize_config(ctx)
    }

    #[access_control(internal::Authenticate::only_controller(&ctx))]
    pub fn insert_amount(
        ctx: Context<InsertAmount>,
        key: String,
        amount: u64,
        new: bool,
    ) -> Result<()> {
        instructions::insert_amount(ctx, &key, amount, new)
    }

    #[access_control(internal::Authenticate::only_controller(&ctx))]
    pub fn insert_factor(
        ctx: Context<InsertFactor>,
        key: String,
        amount: u128,
        new: bool,
    ) -> Result<()> {
        instructions::insert_factor(ctx, &key, amount, new)
    }

    #[access_control(internal::Authenticate::only_controller(&ctx))]
    pub fn insert_address(
        ctx: Context<InsertAddress>,
        key: String,
        address: Pubkey,
        new: bool,
    ) -> Result<()> {
        instructions::insert_address(ctx, &key, address, new)
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
        builder: TokenConfigBuilder,
        enable: bool,
    ) -> Result<()> {
        instructions::insert_token_config(ctx, builder, enable)
    }

    #[access_control(internal::Authenticate::only_controller(&ctx))]
    pub fn insert_synthetic_token_config(
        ctx: Context<InsertSyntheticTokenConfig>,
        token: Pubkey,
        decimals: u8,
        builder: TokenConfigBuilder,
        enable: bool,
    ) -> Result<()> {
        instructions::insert_synthetic_token_config(ctx, token, decimals, builder, enable)
    }

    #[access_control(internal::Authenticate::only_controller(&ctx))]
    pub fn toggle_token_config(
        ctx: Context<ToggleTokenConfig>,
        token: Pubkey,
        enable: bool,
    ) -> Result<()> {
        instructions::toggle_token_config(ctx, token, enable)
    }

    #[access_control(internal::Authenticate::only_controller(&ctx))]
    pub fn set_expected_provider(
        ctx: Context<SetExpectedProvider>,
        token: Pubkey,
        provider: u8,
    ) -> Result<()> {
        instructions::set_expected_provider(
            ctx,
            token,
            PriceProviderKind::try_from(provider)
                .map_err(|_| DataStoreError::InvalidProviderKindIndex)?,
        )
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

    #[access_control(internal::Authenticate::only_controller(&ctx))]
    pub fn insert_token_config_amount(
        ctx: Context<InsertTokenConfigAmount>,
        token: Pubkey,
        key: String,
        amount: u64,
    ) -> Result<()> {
        instructions::insert_token_config_amount(ctx, &token, &key, amount)
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

    #[access_control(internal::Authenticate::only_controller(&ctx))]
    pub fn market_transfer_in(ctx: Context<MarketTransferIn>, amount: u64) -> Result<()> {
        instructions::market_transfer_in(ctx, amount)
    }

    #[access_control(internal::Authenticate::only_controller(&ctx))]
    pub fn market_transfer_out(ctx: Context<MarketTransferOut>, amount: u64) -> Result<()> {
        instructions::market_transfer_out(ctx, amount)
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

    #[access_control(internal::Authenticate::only_controller(&ctx))]
    pub fn use_claimable_account(
        ctx: Context<UseClaimableAccount>,
        timestamp: i64,
        amount: u64,
    ) -> Result<()> {
        instructions::use_claimable_account(ctx, timestamp, amount)
    }

    #[access_control(internal::Authenticate::only_controller(&ctx))]
    pub fn close_empty_claimable_account(
        ctx: Context<CloseEmptyClaimableAccount>,
        user: Pubkey,
        timestamp: i64,
    ) -> Result<()> {
        instructions::close_empty_claimable_account(ctx, user, timestamp)
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

    #[access_control(internal::Authenticate::only_controller(&ctx))]
    pub fn set_prices_from_price_feed<'info>(
        ctx: Context<'_, '_, 'info, 'info, SetPricesFromPriceFeed<'info>>,
        tokens: Vec<Pubkey>,
    ) -> Result<()> {
        instructions::set_prices_from_price_feed(ctx, tokens)
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
        tokens_with_feed: Vec<TokenRecord>,
        swap_params: SwapParams,
        token_params: DepositTokenParams,
        ui_fee_receiver: Pubkey,
    ) -> Result<()> {
        instructions::initialize_deposit(
            ctx,
            nonce,
            tokens_with_feed,
            swap_params,
            token_params,
            ui_fee_receiver,
        )
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
        swap_params: SwapParams,
        tokens_with_feed: Vec<TokenRecord>,
        token_params: WithdrawalTokenParams,
        market_token_amount: u64,
        ui_fee_receiver: Pubkey,
    ) -> Result<()> {
        instructions::initialize_withdrawal(
            ctx,
            nonce,
            swap_params,
            tokens_with_feed,
            token_params,
            market_token_amount,
            ui_fee_receiver,
        )
    }

    #[access_control(internal::Authenticate::only_controller(&ctx))]
    pub fn remove_withdrawal(ctx: Context<RemoveWithdrawal>, refund: u64) -> Result<()> {
        instructions::remove_withdrawal(ctx, refund)
    }

    // Exchange.
    #[access_control(internal::Authenticate::only_order_keeper(&ctx))]
    pub fn execute_deposit<'info>(
        ctx: Context<'_, '_, 'info, 'info, ExecuteDeposit<'info>>,
    ) -> Result<()> {
        instructions::execute_deposit(ctx)
    }

    #[access_control(internal::Authenticate::only_order_keeper(&ctx))]
    pub fn execute_withdrawal<'info>(
        ctx: Context<'_, '_, 'info, 'info, ExecuteWithdrawal<'info>>,
    ) -> Result<()> {
        instructions::execute_withdrawal(ctx)
    }

    #[access_control(internal::Authenticate::only_order_keeper(&ctx))]
    pub fn execute_order<'info>(
        ctx: Context<'_, '_, 'info, 'info, ExecuteOrder<'info>>,
        recent_timestamp: i64,
    ) -> Result<bool> {
        instructions::execute_order(ctx, recent_timestamp)
    }

    #[access_control(internal::Authenticate::only_controller(&ctx))]
    pub fn initialize_order(
        ctx: Context<InitializeOrder>,
        nonce: [u8; 32],
        tokens_with_feed: Vec<TokenRecord>,
        swap: SwapParams,
        params: OrderParams,
        output_token: Pubkey,
        ui_fee_receiver: Pubkey,
    ) -> Result<()> {
        instructions::initialize_order(
            ctx,
            nonce,
            tokens_with_feed,
            swap,
            params,
            output_token,
            ui_fee_receiver,
        )
    }

    #[access_control(internal::Authenticate::only_controller(&ctx))]
    pub fn remove_order(ctx: Context<RemoveOrder>, refund: u64) -> Result<()> {
        instructions::remove_order(ctx, refund)
    }

    // Position.
    #[access_control(internal::Authenticate::only_controller(&ctx))]
    pub fn remove_position(ctx: Context<RemovePosition>, refund: u64) -> Result<()> {
        instructions::remove_position(ctx, refund)
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
    #[msg("amount overflow")]
    AmountOverflow,
    #[msg("Unknown error")]
    Unknown,
    #[msg("Missing amount")]
    MissingAmount,
    #[msg("Missing factor")]
    MissingFactor,
    #[msg("Cannot be zero")]
    CannotBeZero,
    #[msg("Missing Market Account")]
    MissingMarketAccount,
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
    #[msg("Oracle is not empty")]
    PricesAlreadySet,
    #[msg("Price of the given token already set")]
    PriceAlreadySet,
    #[msg("Invalid price feed account")]
    InvalidPriceFeedAccount,
    #[msg("Invalid price feed price")]
    InvalidPriceFeedPrice,
    #[msg("Price feed not updated")]
    PriceFeedNotUpdated,
    #[msg("Token config disabled")]
    TokenConfigDisabled,
    #[msg("Negative price is not allowed")]
    NegativePrice,
    #[msg("Price overflow")]
    PriceOverflow,
    #[msg("Price feed is not set for the given provider")]
    PriceFeedNotSet,
    #[msg("Not enough feeds")]
    NotEnoughFeeds,
    #[msg("Max price age exceeded")]
    MaxPriceAgeExceeded,
    #[msg("Invalid oracle timestamp range")]
    InvalidOracleTsTrange,
    #[msg("Max oracle timestamp range exceeded")]
    MaxOracleTimeStampRangeExceeded,
    #[msg("Oracle timestamps are smaller than required")]
    OracleTimestampsAreSmallerThanRequired,
    #[msg("Oracle timestamps are larger than requried")]
    OracleTimestampsAreLargerThanRequired,
    #[msg("Oracle not updated")]
    OracleNotUpdated,
    #[msg("Invalid oracle slot")]
    InvalidOracleSlot,
    // Market.
    #[msg("Computation error")]
    Computation,
    #[msg("Unsupported pool kind")]
    UnsupportedPoolKind,
    #[msg("Invalid collateral token")]
    InvalidCollateralToken,
    // Exchange Common.
    #[msg("Invalid swap path")]
    InvalidSwapPath,
    #[msg("Output amount too small")]
    OutputAmountTooSmall,
    #[msg("Amount is not zero but swap in token not provided")]
    AmountNonZeroMissingToken,
    #[msg("Missing token mint")]
    MissingTokenMint,
    #[msg("Missing oracle price")]
    MissingOracelPrice,
    // Withdrawal.
    #[msg("User mismach")]
    UserMismatch,
    #[msg("Empty withdrawal")]
    EmptyWithdrawal,
    #[msg("Invalid withdrawal to remove")]
    InvalidWithdrawalToRemove,
    #[msg("Unable to transfer out remaining withdrawal amount")]
    UnableToTransferOutRemainingWithdrawalAmount,
    // Deposit.
    #[msg("Empty deposit")]
    EmptyDeposit,
    #[msg("Missing deposit token account")]
    MissingDepositTokenAccount,
    #[msg("Invalid deposit to remove")]
    InvalidDepositToRemove,
    // Exchange.
    #[msg("Invalid position kind")]
    InvalidPositionKind,
    #[msg("Invalid position collateral token")]
    InvalidPositionCollateralToken,
    #[msg("Invalid position market")]
    InvalidPositionMarket,
    #[msg("Position account not provided")]
    PositionNotProvided,
    #[msg("Same secondary tokens not merged")]
    SameSecondaryTokensNotMerged,
    #[msg("Missing receivers")]
    MissingReceivers,
    // Position.
    #[msg("position is not initialized")]
    PositionNotInitalized,
    #[msg("position has been initialized")]
    PositionHasBeenInitialized,
    #[msg("position is not required")]
    PositionIsNotRequried,
    #[msg("position is not provided")]
    PositionIsNotProvided,
    #[msg("invalid position initialization params")]
    InvalidPositionInitailziationParams,
    #[msg("invalid position")]
    InvalidPosition,
    // Order.
    #[msg("missing initialial token account for order")]
    MissingInitializeTokenAccountForOrder,
    #[msg("missing claimable time window")]
    MissingClaimableTimeWindow,
    #[msg("missing recent time window")]
    MissingRecentTimeWindow,
    #[msg("missing holding address")]
    MissingHoldingAddress,
    #[msg("missing sender")]
    MissingSender,
    #[msg("missing position")]
    MissingPosition,
    #[msg("missing claimable long collateral account for user")]
    MissingClaimableLongCollateralAccountForUser,
    #[msg("missing claimable short collateral account for user")]
    MissingClaimableShortCollateralAccountForUser,
    #[msg("missing claimable pnl token account for holding")]
    MissingClaimablePnlTokenAccountForHolding,
    #[msg("claimable collateral in output token for holding is not supported")]
    ClaimbleCollateralInOutputTokenForHolding,
    #[msg("no delegated authority is set")]
    NoDelegatedAuthorityIsSet,
    #[msg("invalid order to remove")]
    InvalidOrderToRemove,
    // Token Config.
    #[msg("synthetic flag does not match")]
    InvalidSynthetic,
    // Invalid Provider Kind.
    #[msg("invalid provider kind index")]
    InvalidProviderKindIndex,
}

impl DataStoreError {
    #[inline]
    pub(crate) const fn invalid_position_kind(_kind: u8) -> Self {
        Self::InvalidPositionKind
    }
}
