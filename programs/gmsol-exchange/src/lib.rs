use anchor_lang::prelude::*;

/// Instructions.
pub mod instructions;

/// Utils.
pub mod utils;

/// Constants.
pub mod constants;

/// Events.
pub mod events;

/// States.
pub mod states;

use gmsol_store::utils::Authenticate;
use instructions::*;

declare_id!("hnxiNKTc515NHvuq5fEUAc62dWkEu3m623FbwemWNJd");

#[program]
pub mod gmsol_exchange {
    use super::*;

    // Controller.
    pub fn initialize_controller(ctx: Context<InitializeController>) -> Result<()> {
        instructions::initialize_controller(ctx)
    }

    // Market.
    #[access_control(Authenticate::only_market_keeper(&ctx))]
    pub fn create_market(
        ctx: Context<CreateMarket>,
        name: String,
        index_token_mint: Pubkey,
        enable: bool,
    ) -> Result<()> {
        instructions::create_market(ctx, name, index_token_mint, enable)
    }

    // Deposit.
    // #[access_control(Authenticate::only_controller(&ctx))]
    pub fn create_deposit<'info>(
        ctx: Context<'_, '_, 'info, 'info, CreateDeposit<'info>>,
        nonce: [u8; 32],
        params: CreateDepositParams,
    ) -> Result<()> {
        instructions::create_deposit(ctx, nonce, params)
    }

    #[access_control(Authenticate::only_order_keeper(&ctx))]
    pub fn execute_deposit<'info>(
        ctx: Context<'_, '_, 'info, 'info, ExecuteDeposit<'info>>,
        execution_fee: u64,
        cancel_on_execution_error: bool,
    ) -> Result<()> {
        instructions::execute_deposit(ctx, execution_fee, cancel_on_execution_error)
    }

    pub fn cancel_deposit(ctx: Context<CancelDeposit>) -> Result<()> {
        instructions::cancel_deposit(ctx)
    }

    // Withdrawal.
    pub fn create_withdrawal<'info>(
        ctx: Context<'_, '_, 'info, 'info, CreateWithdrawal<'info>>,
        nonce: [u8; 32],
        params: CreateWithdrawalParams,
    ) -> Result<()> {
        instructions::create_withdrawal(ctx, nonce, params)
    }

    pub fn cancel_withdrawal(ctx: Context<CancelWithdrawal>) -> Result<()> {
        instructions::cancel_withdrawal(ctx)
    }

    #[access_control(Authenticate::only_order_keeper(&ctx))]
    pub fn execute_withdrawal<'info>(
        ctx: Context<'_, '_, 'info, 'info, ExecuteWithdrawal<'info>>,
        execution_fee: u64,
        cancel_on_execution_error: bool,
    ) -> Result<()> {
        instructions::execute_withdrawal(ctx, execution_fee, cancel_on_execution_error)
    }

    // Order.
    pub fn create_order<'info>(
        ctx: Context<'_, '_, 'info, 'info, CreateOrder<'info>>,
        nonce: [u8; 32],
        params: CreateOrderParams,
    ) -> Result<()> {
        instructions::create_order(ctx, nonce, params)
    }

    #[access_control(Authenticate::only_order_keeper(&ctx))]
    pub fn execute_order<'info>(
        ctx: Context<'_, '_, 'info, 'info, ExecuteOrder<'info>>,
        recent_timestamp: i64,
        execution_fee: u64,
        cancel_on_execution_error: bool,
    ) -> Result<()> {
        instructions::execute_order(
            ctx,
            recent_timestamp,
            execution_fee,
            cancel_on_execution_error,
        )
    }

    /// Cancel an order.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](CancelOrder)*
    ///
    /// # Checks
    /// - The [`user`](CancelOrder::user) must be a signer and the owner of the
    /// order.
    /// - The [`controller`](CancelOrder::controller) must be derived for the
    /// `store`.
    /// - *[See also the checks done by the `remove_order` instruction.](gmsol_store::gmsol_store::remove_order)*
    pub fn cancel_order(ctx: Context<CancelOrder>) -> Result<()> {
        instructions::cancel_order(ctx)
    }

    /// Liquidate a position.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](Liquidate)*
    ///
    /// # Arguments
    /// - `recent_timestmap`: The timestamp used to derive the claimable collateral accounts.
    /// - `nonce`: Nonce bytes used to derive the order account.
    /// - `execution_fee`: Execution fee claimed by Keeper for its usage.
    ///
    /// # Checks
    /// - The [`authority`](Liquidate::authority) must be a signer and has the `ORDER_KEEPER` role.
    /// - *TODO*
    #[access_control(Authenticate::only_order_keeper(&ctx))]
    pub fn liquidate<'info>(
        ctx: Context<'_, '_, 'info, 'info, Liquidate<'info>>,
        recent_timestamp: i64,
        nonce: [u8; 32],
        execution_fee: u64,
    ) -> Result<()> {
        instructions::unchecked_liquidate(ctx, recent_timestamp, nonce, execution_fee)
    }

    /// Auto-deleverage a position.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](AutoDeleverage)*
    ///
    /// # Arguments
    /// - `size_delta_usd`: The amount by which the size will be decreased.
    /// - `recent_timestmap`: The timestamp used to derive the claimable collateral accounts.
    /// - `nonce`: Nonce bytes used to derive the order account.
    /// - `execution_fee`: Execution fee claimed by Keeper for its usage.
    ///
    /// # Checks
    /// - The [`authority`](AutoDeleverage::authority) must be a signer and has the `ORDER_KEEPER` role.
    /// - *TODO*
    #[access_control(Authenticate::only_order_keeper(&ctx))]
    pub fn auto_deleverage<'info>(
        ctx: Context<'_, '_, 'info, 'info, AutoDeleverage<'info>>,
        size_delta_usd: u128,
        recent_timestamp: i64,
        nonce: [u8; 32],
        execution_fee: u64,
    ) -> Result<()> {
        instructions::unchecked_auto_deleverage(
            ctx,
            size_delta_usd,
            recent_timestamp,
            nonce,
            execution_fee,
        )
    }

    /// Update the ADL state for the given market.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](UpdateAdlState)*
    ///
    /// # Arguments
    /// - `is_long`: The market side to update for.
    ///
    /// # Checks
    /// *TODO*
    #[access_control(Authenticate::only_order_keeper(&ctx))]
    pub fn update_adl_state<'info>(
        ctx: Context<'_, '_, 'info, 'info, UpdateAdlState<'info>>,
        is_long: bool,
    ) -> Result<()> {
        instructions::unchecked_update_adl_state(ctx, is_long)
    }
}

/// Errors of market program.
#[error_code]
pub enum ExchangeError {
    #[msg("Permission denied")]
    PermissionDenied,
    #[msg("Not enough execution fee")]
    NotEnoughExecutionFee,
    #[msg("Resource not found")]
    ResourceNotFound,
    #[msg("Not enough remaining accounts")]
    NotEnoughRemainingAccounts,
    #[msg("Invalid swap path")]
    InvalidSwapPath,
    #[msg("Missing oracle price")]
    MissingOraclePrice,
    #[msg("Amount overflow")]
    AmountOverflow,
    #[msg("Invalid Argument")]
    InvalidArgument,
    // Deposit.
    #[msg("Empty deposit amounts")]
    EmptyDepositAmounts,
    #[msg("Failed to execute deposit")]
    FailedToExecuteDeposit,
    // Withdrawal.
    #[msg("Market token mint mismached")]
    MismatchedMarketTokenMint,
    #[msg("Empty withdrawal amount")]
    EmptyWithdrawalAmount,
    #[msg("Invalid output amount")]
    InvalidOutputAmount,
    #[msg("Output amount too small")]
    OutputAmountTooSmall,
    #[msg("Invalid withdrawal to execute")]
    InvalidWithdrawalToExecute,
    // Order.
    #[msg("Unsupported order kind")]
    UnsupportedOrderKind,
    #[msg("Invalid secondary output token")]
    InvalidSecondaryOutputToken,
    #[msg("Invalid output token")]
    InvalidOutputToken,
    #[msg("Position is not provided")]
    PositionNotProvided,
    #[msg("Missing token account for order")]
    MissingTokenAccountForOrder,
}
