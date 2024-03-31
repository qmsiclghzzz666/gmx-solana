use anchor_lang::prelude::*;

/// Instructions.
pub mod instructions;

/// Utils.
pub mod utils;

use data_store::utils::Authenticate;
use instructions::*;

declare_id!("HY9NoGiu68nqu3H44UySTX3rZ1db8Mx3b2CFcDNAmSQJ");

#[program]
pub mod exchange {
    use super::*;

    // Market.
    #[access_control(Authenticate::only_market_keeper(&ctx))]
    pub fn create_market(ctx: Context<CreateMarket>, index_token_mint: Pubkey) -> Result<()> {
        instructions::create_market(ctx, index_token_mint)
    }

    // Deposit.
    #[access_control(Authenticate::only_controller(&ctx))]
    pub fn create_deposit(
        ctx: Context<CreateDeposit>,
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

    #[access_control(Authenticate::only_controller(&ctx))]
    pub fn cancel_deposit(ctx: Context<CancelDeposit>, execution_fee: u64) -> Result<()> {
        instructions::cancel_deposit(ctx, execution_fee)
    }

    // Withdrawal.
    #[access_control(Authenticate::only_controller(&ctx))]
    pub fn create_withdrawal(
        ctx: Context<CreateWithdrawal>,
        nonce: [u8; 32],
        params: CreateWithdrawalParams,
    ) -> Result<()> {
        instructions::create_withdrawal(ctx, nonce, params)
    }

    #[access_control(Authenticate::only_controller(&ctx))]
    pub fn cancel_withdrawal(ctx: Context<CancelWithdrawal>, execution_fee: u64) -> Result<()> {
        instructions::cancel_withdrawal(ctx, execution_fee)
    }

    #[access_control(Authenticate::only_controller(&ctx))]
    pub fn execute_withdrawal<'info>(
        ctx: Context<'_, '_, 'info, 'info, ExecuteWithdrawal<'info>>,
        execution_fee: u64,
    ) -> Result<()> {
        instructions::execute_withdrawal(ctx, execution_fee)
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
    // Deposit.
    #[msg("Empty deposit amounts")]
    EmptyDepositAmounts,
    #[msg("Failed to execute deposit")]
    FailedToExecuteDeposit,
    #[msg("Invalid deposit to cancel")]
    InvalidDepositToCancel,
    // Withdrawal.
    #[msg("Market token mint mismached")]
    MismatchedMarketTokenMint,
    #[msg("Empty withdrawal amount")]
    EmptyWithdrawalAmount,
    #[msg("Invalid withdrawal to cancel")]
    InvalidWithdrawalToCancel,
    #[msg("Invalid output amount")]
    InvalidOutputAmount,
    #[msg("Output amount too small")]
    OutputAmountTooSmall,
    #[msg("Invalid withdrawal to execute")]
    InvalidWIthdrawalToExecute,
}
