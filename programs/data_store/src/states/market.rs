use anchor_lang::{prelude::*, Bump};
use gmx_solana_utils::{price::Decimal, to_seed};

use crate::{constants, DataStoreError};

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
    /// Primary Pool.
    pub primary: Pool,
    /// Price impact pool.
    pub price_impact: Pool,
}

impl Market {
    /// USD value to amount divisor.
    pub const USD_TO_AMOUNT_DIVISOR: u128 =
        10u128.pow((Decimal::MAX_DECIMALS - constants::MARKET_TOKEN_DECIMALS) as u32);

    /// Initialize the market.
    pub fn init(
        &mut self,
        bump: u8,
        market_token_mint: Pubkey,
        index_token_mint: Pubkey,
        long_token_mint: Pubkey,
        short_token_mint: Pubkey,
    ) {
        self.bump = bump;
        self.market_token_mint = market_token_mint;
        self.index_token_mint = index_token_mint;
        self.long_token_mint = long_token_mint;
        self.short_token_mint = short_token_mint;
        let is_pure = self.long_token_mint == self.short_token_mint;
        self.primary = Pool::default().with_is_pure(is_pure);
        self.price_impact = Pool::default().with_is_pure(is_pure);
    }

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

/// A pool for market.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace, Default)]
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

/// Pool kind.
#[derive(Debug, Clone, Copy, Default, num_enum::TryFromPrimitive, PartialEq, Eq)]
#[repr(u8)]
pub enum PoolKind {
    /// Primary.
    #[default]
    Primary,
    /// Price impact.
    PriceImpact,
}

#[event]
pub struct MarketChangeEvent {
    pub address: Pubkey,
    pub action: super::Action,
    pub market: Market,
}
