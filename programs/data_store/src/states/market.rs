use anchor_lang::{prelude::*, Bump};

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

impl Bump for Market {
    fn seed(&self) -> u8 {
        self.bump
    }
}

impl Data for Market {
    const SEED: &'static [u8] = b"market";

    fn verify(&self, key: &str) -> Result<()> {
        // FIXME: is there a better way to verify the key?
        let mut expected = self.index_token.to_string();
        expected.push_str(&self.long_token.to_string());
        expected.push_str(&self.short_token.to_string());
        require_eq!(key, &expected, crate::DataStoreError::InvalidKey);
        Ok(())
    }
}
