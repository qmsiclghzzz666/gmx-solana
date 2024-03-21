#[cfg(feature = "cpi")]
mod cpi;

#[cfg(feature = "cpi")]
pub use cpi::*;

use anchor_lang::{solana_program::pubkey::Pubkey, Id};

/// The Chainlink Program.
pub struct Chainlink;

impl Id for Chainlink {
    fn id() -> Pubkey {
        chainlink_solana::ID
    }
}
