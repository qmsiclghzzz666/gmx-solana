use anchor_lang::prelude::*;

/// The Root of Referral Tree.
#[zero_copy]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct ReferralRoot {
    /// Total Number of nodes.
    pub size: u128,
    /// Height of the tree (maximum depth from the root to any leaf node).
    pub height: u128,
    /// Reserved.
    reserved: [u8; 128],
}

/// Referral.
#[zero_copy]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Referral {
    /// Referrer.
    ///
    /// `Pubkey::default()` means no referrer.
    pub referrer: Pubkey,
    /// Referee count.
    pub referee_count: u128,
    /// Depth from the root.
    pub depth: u128,
    reserved: [u8; 64],
}
