use anchor_lang::prelude::*;

/// Instructions.
pub mod instructions;

use data_store::utils::Authenticate;
use instructions::*;

declare_id!("AFxZM92h6tryw4hZx2puJRWjA4CQSkxmVkzJrDWJgJAL");

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
}

/// Errors of market program.
#[error_code]
pub enum ExchangeError {
    #[msg("Permission denied")]
    PermissionDenied,
    // Deposit.
    #[msg("Empty deposit amounts")]
    EmptyDepositAmounts,
}
