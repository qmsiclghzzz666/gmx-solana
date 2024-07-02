use anchor_lang::prelude::*;

use super::Referral;

/// Header of `User` Account.
#[account(zero_copy)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct UserHeader {
    /// Version of the account.
    pub version: u8,
    /// The bump seed.
    pub bump: u8,
    padding_0: [u8; 14],
    /// The authority of the user account.
    pub authority: Pubkey,
    /// The authorized store.
    pub store: Pubkey,
    /// Referral.
    pub referral: Referral,
    /// Deposit count.
    pub deposit_count: u32,
    /// Withdrawal count.
    pub withdrawal_count: u32,
    /// Order count.
    pub order_count: u32,
    padding_1: [u8; 4],
    reserved: [u8; 128],
}
