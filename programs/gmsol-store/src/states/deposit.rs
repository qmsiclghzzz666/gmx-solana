use anchor_lang::prelude::*;

use super::{
    common::{
        action::{Action, ActionHeader},
        swap::SwapParamsV2,
        token::TokenAndAccount,
    },
    Seed,
};

/// Deposit V2.
#[cfg_attr(feature = "debug", derive(Debug))]
#[account(zero_copy)]
pub struct DepositV2 {
    /// Header.
    pub(crate) header: ActionHeader,
    /// Token accounts.
    pub(crate) tokens: TokenAccounts,
    /// Deposit params.
    pub(crate) params: DepositParams,
    /// Swap params.
    pub(crate) swap: SwapParamsV2,
    padding_1: [u8; 4],
    reserve: [u8; 128],
}

impl DepositV2 {
    /// Max execution lamports.
    pub const MIN_EXECUTION_LAMPORTS: u64 = 200_000;

    /// Init Space.
    pub const INIT_SPACE: usize = core::mem::size_of::<Self>();

    /// Get tokens.
    pub fn tokens(&self) -> &TokenAccounts {
        &self.tokens
    }

    /// Get swap params.
    pub fn swap(&self) -> &SwapParamsV2 {
        &self.swap
    }
}

impl Seed for DepositV2 {
    /// Seed.
    const SEED: &'static [u8] = b"deposit";
}

impl Action for DepositV2 {
    fn header(&self) -> &ActionHeader {
        &self.header
    }
}

/// Token Accounts.
#[cfg_attr(feature = "debug", derive(Debug))]
#[account(zero_copy)]
pub struct TokenAccounts {
    /// Initial long token accounts.
    pub initial_long_token: TokenAndAccount,
    /// Initial short token accounts.
    pub initial_short_token: TokenAndAccount,
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
}

/// Deposit Params.
#[cfg_attr(feature = "debug", derive(Debug))]
#[account(zero_copy)]
pub struct DepositParams {
    /// The amount of initial long tokens to deposit.
    pub(crate) initial_long_token_amount: u64,
    /// The amount of initial short tokens to deposit.
    pub(crate) initial_short_token_amount: u64,
    /// The minimum acceptable amount of market tokens to receive.
    pub(crate) min_market_token_amount: u64,
    reserved: [u8; 64],
}

impl Default for DepositParams {
    fn default() -> Self {
        Self {
            initial_long_token_amount: 0,
            initial_short_token_amount: 0,
            min_market_token_amount: 0,
            reserved: [0; 64],
        }
    }
}
