use anchor_lang::prelude::*;
use dual_vec_map::DualVecMap;
use gmx_solana_utils::price::Price;

use crate::DataStoreError;

use super::Seed;

/// Maximum number of tokens for a single `Price Map` to store.
const MAX_TOKENS: usize = 32;

/// Price Map.
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

    fn as_map(&self) -> DualVecMap<&Vec<Pubkey>, &Vec<Price>> {
        // CHECK: All the insert operations is done by `FlatMap`.
        DualVecMap::from_sorted_stores_unchecked(&self.tokens, &self.prices)
    }

    fn as_map_mut(&mut self) -> DualVecMap<&mut Vec<Pubkey>, &mut Vec<Price>> {
        // CHECK: All the insert operations is done by `FlatMap`.
        DualVecMap::from_sorted_stores_unchecked(&mut self.tokens, &mut self.prices)
    }

    /// Get price of the given token key.
    pub fn get(&self, token: &Pubkey) -> Option<Price> {
        self.as_map().get(token).copied()
    }

    /// Set the price of the given token.
    /// # Error
    /// Return error if it already set.
    pub fn set(&mut self, token: &Pubkey, price: Price) -> Result<()> {
        self.as_map_mut()
            .try_insert(*token, price)
            .map_err(|_| DataStoreError::PriceAlreadySet)?;
        Ok(())
    }

    /// Clear all prices.
    pub fn clear(&mut self) {
        self.tokens.clear();
        self.prices.clear();
    }

    /// Is empty.
    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }
}

/// Oracle Account.
#[account]
#[derive(InitSpace)]
pub struct Oracle {
    pub bump: u8,
    pub index: u8,
    pub primary: PriceMap,
}

impl Seed for Oracle {
    const SEED: &'static [u8] = b"oracle";
}

impl Oracle {
    /// Initialize the [`Oracle`].
    pub fn init(&mut self, bump: u8, index: u8) {
        self.primary.clear();
        self.bump = bump;
        self.index = index;
    }
}
