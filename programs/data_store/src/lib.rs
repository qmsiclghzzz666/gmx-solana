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
        market::MarketMeta,
        order::{OrderParams, TransferOut},
        token_config::TokenConfigBuilder,
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

    #[access_control(internal::Authenticate::only_market_keeper(&ctx))]
    pub fn set_token_map(ctx: Context<SetTokenMap>) -> Result<()> {
        instructions::unchecked_set_token_map(ctx)
    }

    pub fn get_token_map(ctx: Context<ReadStore>) -> Result<Option<Pubkey>> {
        instructions::get_token_map(ctx)
    }

    // Roles.
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
    pub fn initialize_token_map(ctx: Context<InitializeTokenMap>) -> Result<()> {
        instructions::initialize_token_map(ctx)
    }

    #[access_control(internal::Authenticate::only_market_keeper(&ctx))]
    pub fn push_to_token_map(
        ctx: Context<PushToTokenMap>,
        builder: TokenConfigBuilder,
        enable: bool,
        new: bool,
    ) -> Result<()> {
        instructions::unchecked_push_to_token_map(ctx, builder, enable, new)
    }

    #[access_control(internal::Authenticate::only_market_keeper(&ctx))]
    pub fn push_to_token_map_synthetic(
        ctx: Context<PushToTokenMapSynthetic>,
        token: Pubkey,
        token_decimals: u8,
        builder: TokenConfigBuilder,
        enable: bool,
        new: bool,
    ) -> Result<()> {
        instructions::unchecked_push_to_token_map_synthetic(
            ctx,
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
                .map_err(|_| DataStoreError::InvalidProviderKindIndex)?,
        )
    }

    pub fn token_timestamp_adjustment(ctx: Context<ReadTokenMap>, token: Pubkey) -> Result<u32> {
        instructions::token_timestamp_adjustment(ctx, &token)
    }

    #[access_control(internal::Authenticate::only_market_keeper(&ctx))]
    pub fn toggle_token_config(
        ctx: Context<ToggleTokenConfig>,
        token: Pubkey,
        enable: bool,
    ) -> Result<()> {
        instructions::unchecked_toggle_token_config(ctx, token, enable)
    }

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
                .map_err(|_| DataStoreError::InvalidProviderKindIndex)?,
        )
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

    pub fn get_validated_market_meta(ctx: Context<GetValidatedMarketMeta>) -> Result<MarketMeta> {
        instructions::get_validated_market_meta(ctx)
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
    #[access_control(internal::Authenticate::only_controller(&ctx))]
    pub fn execute_deposit<'info>(
        ctx: Context<'_, '_, 'info, 'info, ExecuteDeposit<'info>>,
    ) -> Result<()> {
        instructions::execute_deposit(ctx)
    }

    #[access_control(internal::Authenticate::only_controller(&ctx))]
    pub fn execute_withdrawal<'info>(
        ctx: Context<'_, '_, 'info, 'info, ExecuteWithdrawal<'info>>,
    ) -> Result<(u64, u64)> {
        instructions::execute_withdrawal(ctx)
    }

    #[access_control(internal::Authenticate::only_controller(&ctx))]
    pub fn execute_order<'info>(
        ctx: Context<'_, '_, 'info, 'info, ExecuteOrder<'info>>,
        recent_timestamp: i64,
    ) -> Result<(bool, Box<TransferOut>)> {
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
    #[msg("Aready exist")]
    AlreadyExist,
    #[msg("Exceed max length limit")]
    ExceedMaxLengthLimit,
    #[msg("Exceed max string length limit")]
    ExceedMaxStringLengthLimit,
    #[msg("No space for new data")]
    NoSpaceForNewData,
    #[msg("Invalid argument")]
    InvalidArgument,
    #[msg("Lamports not enough")]
    LamportsNotEnough,
    #[msg("Required resource not found")]
    RequiredResourceNotFound,
    #[msg("Amount overflow")]
    AmountOverflow,
    #[msg("Unknown error")]
    Unknown,
    #[msg("Gmx Core Error")]
    Core,
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
    #[msg("No such role")]
    NoSuchRole,
    #[msg("The role is disabled")]
    DisabledRole,
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
    #[msg("Invalid market")]
    InvalidMarket,
    #[msg("Disabled market")]
    DisabledMarket,
    #[msg("Unknown swap out market")]
    UnknownSwapOutMarket,
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
    #[msg("invalid token map")]
    InvalidTokenMap,
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
