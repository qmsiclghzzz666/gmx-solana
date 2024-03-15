use anchor_lang::{prelude::*, Bump};
use gmx_solana_utils::to_seed;

use super::{Data, Seed};

#[account]
#[derive(InitSpace)]
pub struct Market {
    /// Bump Seed.
    pub bump: u8,
    /// Market token.
    pub market_token_mint: Pubkey,
    /// Index token.
    pub index_token_mint: Pubkey,
    /// Long token.
    pub long_token_mint: Pubkey,
    /// Short token.
    pub short_token_mint: Pubkey,
}

impl Market {
    /// Get the expected key.
    pub fn expected_key(&self) -> String {
        Self::create_key(&self.market_token_mint)
    }

    /// Get the expected key seed.
    pub fn expected_key_seed(&self) -> [u8; 32] {
        to_seed(&self.expected_key())
    }

    /// Create key from tokens.
    pub fn create_key(market_token: &Pubkey) -> String {
        market_token.to_string()
    }

    /// Create key seed from tokens.
    pub fn create_key_seed(market_token: &Pubkey) -> [u8; 32] {
        let key = Self::create_key(market_token);
        to_seed(&key)
    }
}

impl Bump for Market {
    fn seed(&self) -> u8 {
        self.bump
    }
}

impl Seed for Market {
    const SEED: &'static [u8] = b"market";
}

impl Data for Market {
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
    pub action: super::Action,
    pub market: Market,
}
