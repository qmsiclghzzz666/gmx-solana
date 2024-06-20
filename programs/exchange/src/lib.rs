use anchor_lang::prelude::*;

/// Instructions.
pub mod instructions;

/// Utils.
pub mod utils;

/// Constants.
pub mod constants;

/// Events.
pub mod events;

use data_store::utils::Authenticate;
use instructions::*;

declare_id!("hnxiNKTc515NHvuq5fEUAc62dWkEu3m623FbwemWNJd");

#[program]
pub mod exchange {
    use super::*;

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
    ) -> Result<()> {
        instructions::execute_deposit(ctx, execution_fee)
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
    ) -> Result<()> {
        instructions::execute_withdrawal(ctx, execution_fee)
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
    ) -> Result<()> {
        instructions::execute_order(ctx, recent_timestamp, execution_fee)
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
