use anchor_lang::prelude::*;

use super::{
    common::action::{Action, ActionHeader},
    Seed,
};

/// Shift.
#[account(zero_copy)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Shift {
    /// Action header.
    pub(crate) header: ActionHeader,
    /// From market token.
    pub(crate) from_market_token: Pubkey,
    /// To market token.
    pub(crate) to_market_token: Pubkey,
    /// Token accounts.
    pub(crate) tokens: TokenAccounts,
    /// Shift params.
    pub(crate) params: ShiftParams,
}

impl Seed for Shift {
    const SEED: &'static [u8] = b"shift";
}

impl Action for Shift {
    fn header(&self) -> &ActionHeader {
        &self.header
    }
}

impl Shift {
    /// Init Space.
    pub const INIT_SPACE: usize = core::mem::size_of::<Self>();

    /// Min execution lamports.
    pub const MIN_EXECUTION_LAMPORTS: u64 = 200_000;
}

#[account(zero_copy)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct TokenAccounts {}

#[account(zero_copy)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct ShiftParams {
    from_market_token_amount: u64,
    min_to_market_token_amount: u64,
}
