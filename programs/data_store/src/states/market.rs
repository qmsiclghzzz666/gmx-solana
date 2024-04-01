use anchor_lang::{prelude::*, Bump};
use dual_vec_map::DualVecMap;
use gmx_core::PoolKind;
use gmx_solana_utils::to_seed;

use crate::DataStoreError;

use super::{Data, Seed};

/// Market.
#[account]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Market {
    /// Bump Seed.
    pub(crate) bump: u8,
    pub(crate) meta: MarketMeta,
    pools: Pools,
}

impl Market {
    pub(crate) fn init_space(num_pools: u8) -> usize {
        1 + MarketMeta::INIT_SPACE + Pools::init_space(num_pools)
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct MarketMeta {
    /// Market token.
    pub market_token_mint: Pubkey,
    /// Index token.
    pub index_token_mint: Pubkey,
    /// Long token.
    pub long_token_mint: Pubkey,
    /// Short token.
    pub short_token_mint: Pubkey,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Pools {
    pools: Vec<Pool>,
    keys: Vec<u8>,
}

impl Pools {
    pub(crate) fn init_space(num_pools: u8) -> usize {
        let len = num_pools as usize;
        (4 + Pool::INIT_SPACE * len) + (4 + len)
    }

    fn init(&mut self, is_pure: bool, num_pools: u8) {
        let mut map: DualVecMap<Vec<u8>, Vec<Pool>> = DualVecMap::new_vecs();
        for kind in 0..num_pools {
            map.insert(kind, Pool::default().with_is_pure(is_pure));
        }
        let (keys, pools) = map.into_inner();
        self.keys = keys;
        self.pools = pools;
    }

    fn as_map(&self) -> DualVecMap<&Vec<u8>, &Vec<Pool>> {
        DualVecMap::from_sorted_stores_unchecked(&self.keys, &self.pools)
    }

    fn as_map_mut(&mut self) -> DualVecMap<&mut Vec<u8>, &mut Vec<Pool>> {
        DualVecMap::from_sorted_stores_unchecked(&mut self.keys, &mut self.pools)
    }

    fn pool(&self, kind: PoolKind) -> Option<Pool> {
        self.as_map().get(&(kind as u8)).cloned()
    }

    fn with_pool_mut<T>(&mut self, kind: PoolKind, f: impl FnOnce(&mut Pool) -> T) -> Option<T> {
        let mut map = self.as_map_mut();
        let Some(pool) = map.get_mut(&(kind as u8)) else {
            return None;
        };
        Some(f(pool))
    }
}

impl Market {
    /// Initialize the market.
    pub fn init(
        &mut self,
        bump: u8,
        market_token_mint: Pubkey,
        index_token_mint: Pubkey,
        long_token_mint: Pubkey,
        short_token_mint: Pubkey,
        num_pools: u8,
    ) {
        self.bump = bump;
        self.meta.market_token_mint = market_token_mint;
        self.meta.index_token_mint = index_token_mint;
        self.meta.long_token_mint = long_token_mint;
        self.meta.short_token_mint = short_token_mint;
        let is_pure = self.meta.long_token_mint == self.meta.short_token_mint;
        self.pools.init(is_pure, num_pools);
    }

    /// Get pool of the given kind.
    #[inline]
    pub fn pool(&self, kind: PoolKind) -> Option<Pool> {
        self.pools.pool(kind)
    }

    /// Get mutable reference to the pool of the given kind.
    #[inline]
    pub(crate) fn with_pool_mut<T>(
        &mut self,
        kind: PoolKind,
        f: impl FnOnce(&mut Pool) -> T,
    ) -> Option<T> {
        self.pools.with_pool_mut(kind, f)
    }

    /// Get the expected key.
    pub fn expected_key(&self) -> String {
        Self::create_key(&self.meta.market_token_mint)
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

/// A pool for market.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace, Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Pool {
    /// Whether the pool only contains one kind of token,
    /// i.e. a pure pool.
    /// For a pure pool, only the `long_token_amount` field is used.
    pub is_pure: bool,
    /// Long token amount.
    long_token_amount: u128,
    /// Short token amount.
    short_token_amount: u128,
}

impl Pool {
    /// Set the pure flag.
    fn with_is_pure(mut self, is_pure: bool) -> Self {
        self.is_pure = is_pure;
        self
    }

    /// Get the long token amount.
    pub fn long_token_amount(&self) -> u128 {
        if self.is_pure {
            debug_assert_eq!(
                self.short_token_amount, 0,
                "short token amount must be zero"
            );
            self.long_token_amount / 2
        } else {
            self.long_token_amount
        }
    }

    /// Get the short token amount.
    pub fn short_token_amount(&self) -> u128 {
        if self.is_pure {
            debug_assert_eq!(
                self.short_token_amount, 0,
                "short token amount must be zero"
            );
            self.long_token_amount / 2
        } else {
            self.short_token_amount
        }
    }

    pub(crate) fn apply_delta_to_long_token_amount(&mut self, delta: i128) -> Result<()> {
        self.long_token_amount = self
            .long_token_amount
            .checked_add_signed(delta)
            .ok_or(DataStoreError::Computation)?;
        Ok(())
    }

    pub(crate) fn apply_delta_to_short_token_amount(&mut self, delta: i128) -> Result<()> {
        let amount = if self.is_pure {
            &mut self.long_token_amount
        } else {
            &mut self.short_token_amount
        };
        *amount = amount
            .checked_add_signed(delta)
            .ok_or(DataStoreError::Computation)?;
        Ok(())
    }
}

#[event]
pub struct MarketChangeEvent {
    pub address: Pubkey,
    pub action: super::Action,
    pub(crate) market: Market,
}
