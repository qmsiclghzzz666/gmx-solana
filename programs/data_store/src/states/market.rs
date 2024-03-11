use anchor_lang::{prelude::*, Bump};
use gmx_solana_utils::to_seed;

use super::Data;

#[account]
#[derive(InitSpace)]
pub struct Market {
    /// Bump Seed.
    pub bump: u8,
    /// Market token.
    pub market_token: Pubkey,
    /// Index token.
    pub index_token: Pubkey,
    /// Long token.
    pub long_token: Pubkey,
    /// Short token.
    pub short_token: Pubkey,
}

impl Market {
    /// Get the expected key.
    pub fn expected_key(&self) -> String {
        Self::create_key(&self.index_token, &self.long_token, &self.short_token)
    }

    /// Get the expected key seed.
    pub fn expected_key_seed(&self) -> [u8; 32] {
        to_seed(&self.expected_key())
    }

    /// Create key from tokens.
    pub fn create_key(index_token: &Pubkey, long_token: &Pubkey, short_token: &Pubkey) -> String {
        let mut key = index_token.to_string();
        key.push_str(&long_token.to_string());
        key.push_str(&short_token.to_string());
        key
    }

    /// Create key seed from tokens.
    pub fn create_key_seed(
        index_token: &Pubkey,
        long_token: &Pubkey,
        short_token: &Pubkey,
    ) -> [u8; 32] {
        to_seed(&Self::create_key(index_token, long_token, short_token))
    }
}

impl Bump for Market {
    fn seed(&self) -> u8 {
        self.bump
    }
}

impl Data for Market {
    const SEED: &'static [u8] = b"market";

    fn verify(&self, key: &str) -> Result<()> {
        // FIXME: is there a better way to verify the key?
        let expected = self.expected_key();
        require_eq!(key, &expected, crate::DataStoreError::InvalidKey);
        Ok(())
    }
}

#[event]
pub struct MarketChangeEvent {
    pub address: Pubkey,
    pub init: bool,
    pub market: Market,
}
