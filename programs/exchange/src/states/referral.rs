use anchor_lang::prelude::*;

/// The Root of Referral Tree.
#[zero_copy]
pub struct ReferralRoot {
    /// Total Number of nodes.
    pub size: u128,
    /// Height of the tree (maximum depth from the root to any leaf node).
    pub height: u128,
    /// Reserved.
    reserved: [u8; 128],
}
