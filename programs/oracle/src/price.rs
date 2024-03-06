use anchor_lang::prelude::*;

use crate::{decimal::Decimal, OracleError};

/// Price type.
#[derive(Debug, AnchorSerialize, AnchorDeserialize, Clone, Copy, InitSpace)]
pub struct Price {
    /// Min Price.
    pub min: Decimal,
    /// Max Price.
    pub max: Decimal,
}

/// Maximum number of tokens for a single `Price Map` to store.
const MAX_TOKENS: usize = 32;

/// Price Map.
// If the `PriceMap` is initialized with empty vecs or is cleared,
// then the following invariants are preserved after all public operations.
// - Invariant 1: tokens are sorted
// - Invariant 2: the orders of `prices` and `tokens` matched
#[derive(Debug, AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct PriceMap {
    #[max_len(MAX_TOKENS)]
    prices: Vec<Price>,
    #[max_len(MAX_TOKENS)]
    tokens: Vec<Pubkey>,
}

impl PriceMap {
    /// Maximum number of tokens for a single `Price Map` to store.
    pub const MAX_TOKENS: usize = MAX_TOKENS;

    /// Get price of the given token key.
    pub fn get(&self, token: &Pubkey) -> Option<Price> {
        // Apply the invariant 1: tokens are sorted.
        let idx = self.tokens.binary_search(token).ok()?;
        // Apply the invariant 2: the orders of `prices` and `tokens` matched.
        Some(self.prices[idx])
    }

    /// Set the price of the given token.
    /// # Error
    /// Return error if it already set.
    pub fn set(&mut self, token: &Pubkey, price: Price) -> Result<()> {
        match self.tokens.binary_search(token) {
            Ok(_) => Err(OracleError::PriceAlreadySet.into()),
            Err(idx) => {
                // The returned `idx` is where the token/price can be
                // inserted while keeps the order sorted, which
                // preserves the invariant 1. And since the operations
                // of `prices` and `tokens` are the same, the invariant 2
                // also preserves.
                self.tokens.insert(idx, *token);
                self.prices.insert(idx, price);
                Ok(())
            }
        }
    }

    /// Clear all prices.
    pub fn clear(&mut self) {
        // Clearly, the invariant 1 and 2 are both preserved trivially.
        self.tokens.clear();
        self.prices.clear();
    }
}
