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

declare_id!("exYLDKzzpXkp8FBghLxJkM4xvuGViAvGUTkQ7UTzFt1");

#[program]
pub mod gmsol_exchange {
    use super::*;

    // Treasury.
    /// Claim fees from the given market.
    ///
    /// # Accounts
    /// *[See the documentation for the accounts.](ClaimFees)*
    ///
    /// # Errors
    /// - *TODO*
    pub fn claim_fees(ctx: Context<ClaimFees>) -> Result<()> {
        instructions::claim_fees(ctx)
    }

    // Market.
    #[access_control(Authenticate::only_market_keeper(&ctx))]
    pub fn create_market(
        ctx: Context<CreateMarket>,
        name: String,
        index_token_mint: Pubkey,
        enable: bool,
    ) -> Result<()> {
        instructions::unchecked_create_market(ctx, name, index_token_mint, enable)
    }

    /// Fund the given market.
    pub fn fund_market(ctx: Context<FundMarket>, amount: u64) -> Result<()> {
        instructions::fund_market(ctx, amount)
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
    #[msg("Feature disabled")]
    FeatureDisabled,
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
