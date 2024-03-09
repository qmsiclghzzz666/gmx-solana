use anchor_lang::prelude::*;

use crate::PriceMap;

/// Oracle Account.
#[account]
#[derive(InitSpace)]
pub struct Oracle {
    pub bump: u8,
    pub role_store: Pubkey,
    pub data_store: Pubkey,
    pub primary: PriceMap,
}

impl Oracle {
    /// Seed for PDA.
    pub const SEED: &'static [u8] = b"oracle";
}
