use anchor_lang::prelude::*;

use crate::StoreError;

use super::{
    common::{
        action::{Action, ActionHeader},
        swap::SwapParamsV2,
        token::TokenAndAccount,
    },
    Seed,
};

/// Withdrawal.
#[account(zero_copy)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct WithdrawalV2 {
    /// Action header.
    pub(crate) header: ActionHeader,
    /// Token accounts.
    pub(crate) tokens: TokenAccounts,
    /// Withdrawal params.
    pub(crate) params: WithdrawalParams,
    /// Swap params.
    pub(crate) swap: SwapParamsV2,
    padding_1: [u8; 4],
    reserve: [u8; 128],
}

impl WithdrawalV2 {
    /// Init space.
    pub const INIT_SPACE: usize = core::mem::size_of::<Self>();

    /// Min execution lamports.
    pub const MIN_EXECUTION_LAMPORTS: u64 = 200_000;

    /// Get tokens and accounts.
    pub fn tokens(&self) -> &TokenAccounts {
        &self.tokens
    }

    /// Get the swap params.
    pub fn swap(&self) -> &SwapParamsV2 {
        &self.swap
    }

    pub(crate) fn validate_output_amounts(
        &self,
        long_amount: u64,
        short_amount: u64,
    ) -> Result<()> {
        let params = &self.params;
        require_gte!(
            long_amount,
            params.min_long_token_amount,
            StoreError::InsufficientOutputAmount
        );
        require_gte!(
            short_amount,
            params.min_short_token_amount,
            StoreError::InsufficientOutputAmount
        );
        Ok(())
    }
}

impl Seed for WithdrawalV2 {
    /// Seed.
    const SEED: &'static [u8] = b"withdrawal";
}

impl Action for WithdrawalV2 {
    fn header(&self) -> &ActionHeader {
        &self.header
    }
}

/// Token Accounts.
#[cfg_attr(feature = "debug", derive(Debug))]
#[account(zero_copy)]
pub struct TokenAccounts {
    /// Final long token accounts.
    pub(crate) final_long_token: TokenAndAccount,
    /// Final short token accounts.
    pub(crate) final_short_token: TokenAndAccount,
    /// Market token account.
    pub(crate) market_token: TokenAndAccount,
}

impl TokenAccounts {
    /// Get market token.
    pub fn market_token(&self) -> Pubkey {
        self.market_token.token().expect("must exist")
    }

    /// Get market token account.
    pub fn market_token_account(&self) -> Pubkey {
        self.market_token.account().expect("must exist")
    }

    /// Get final_long token.
    pub fn final_long_token(&self) -> Pubkey {
        self.final_long_token.token().expect("must exist")
    }

    /// Get final_long token account.
    pub fn final_long_token_account(&self) -> Pubkey {
        self.final_long_token.account().expect("must exist")
    }

    /// Get final_short token.
    pub fn final_short_token(&self) -> Pubkey {
        self.final_short_token.token().expect("must exist")
    }

    /// Get final_short token account.
    pub fn final_short_token_account(&self) -> Pubkey {
        self.final_short_token.account().expect("must exist")
    }
}

/// Withdrawal params.
#[cfg_attr(feature = "debug", derive(Debug))]
#[account(zero_copy)]
pub struct WithdrawalParams {
    /// Market token amount to burn.
    pub market_token_amount: u64,
    /// The minimum acceptable amount of final long tokens to receive.
    pub min_long_token_amount: u64,
    /// The minimum acceptable amount of final short tokens to receive.
    pub min_short_token_amount: u64,
    reserved: [u8; 64],
}

impl Default for WithdrawalParams {
    fn default() -> Self {
        Self {
            reserved: [0; 64],
            market_token_amount: 0,
            min_long_token_amount: 0,
            min_short_token_amount: 0,
        }
    }
}
