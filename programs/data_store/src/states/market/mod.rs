use std::str::FromStr;

use anchor_lang::{prelude::*, Bump};
use anchor_spl::token::Mint;
use bitmaps::Bitmap;
use borsh::{BorshDeserialize, BorshSerialize};
use gmx_core::{ClockKind, PoolKind};

use crate::DataStoreError;

use super::{Factor, InitSpace, Seed};

pub use self::{
    config::{MarketConfig, MarketConfigKey},
    ops::AsMarket,
};

/// Market Operations.
pub mod ops;

/// Market Config.
pub mod config;

/// Max number of flags.
pub const MAX_FLAGS: usize = 8;

/// Market Flag Value.
pub type MarketFlagValue = u8;

/// Market Flag Bitmap.
pub type MarketFlagBitmap = Bitmap<MAX_FLAGS>;

/// Market.
#[account(zero_copy)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Market {
    /// Bump Seed.
    pub(crate) bump: u8,
    flag: MarketFlagValue,
    padding: [u8; 14],
    pub(crate) meta: MarketMeta,
    pub(crate) store: Pubkey,
    pools: Pools,
    clocks: Clocks,
    state: MarketState,
    config: MarketConfig,
}

impl Bump for Market {
    fn seed(&self) -> u8 {
        self.bump
    }
}

impl Seed for Market {
    const SEED: &'static [u8] = b"market";
}

impl InitSpace for Market {
    const INIT_SPACE: usize = std::mem::size_of::<Self>();
}

impl Market {
    /// Initialize the market.
    #[allow(clippy::too_many_arguments)]
    pub fn init(
        &mut self,
        bump: u8,
        store: Pubkey,
        market_token_mint: Pubkey,
        index_token_mint: Pubkey,
        long_token_mint: Pubkey,
        short_token_mint: Pubkey,
        is_enabled: bool,
    ) -> Result<()> {
        self.bump = bump;
        self.store = store;
        self.set_enabled(is_enabled);
        self.meta.market_token_mint = market_token_mint;
        self.meta.index_token_mint = index_token_mint;
        self.meta.long_token_mint = long_token_mint;
        self.meta.short_token_mint = short_token_mint;
        let is_pure = self.meta.long_token_mint == self.meta.short_token_mint;
        self.set_flag(MarketFlag::Pure, is_pure);
        self.pools.init(is_pure);
        self.clocks.init_to_current()?;
        self.config.init();
        Ok(())
    }

    /// Get meta.
    pub fn meta(&self) -> &MarketMeta {
        &self.meta
    }

    /// Record transferred in by the given token.
    pub fn record_transferred_in_by_token(&mut self, token: &Pubkey, amount: u64) -> Result<()> {
        if self.meta.long_token_mint == *token {
            self.record_transferred_in(true, amount)
        } else if self.meta.short_token_mint == *token {
            self.record_transferred_in(false, amount)
        } else {
            Err(error!(DataStoreError::InvalidCollateralToken))
        }
    }

    /// Record transferred out by the given token.
    pub fn record_transferred_out_by_token(&mut self, token: &Pubkey, amount: u64) -> Result<()> {
        if self.meta.long_token_mint == *token {
            self.record_transferred_out(true, amount)
        } else if self.meta.short_token_mint == *token {
            self.record_transferred_out(false, amount)
        } else {
            Err(error!(DataStoreError::InvalidCollateralToken))
        }
    }

    /// Get flag.
    pub fn flag(&self, flag: MarketFlag) -> bool {
        let bitmap = MarketFlagBitmap::from_value(self.flag);
        bitmap.get(usize::from(flag as u8))
    }

    /// Set flag.
    pub fn set_flag(&mut self, flag: MarketFlag, value: bool) {
        let mut bitmap = MarketFlagBitmap::from_value(self.flag);
        bitmap.set(usize::from(flag as u8), value);
        self.flag = bitmap.into_value();
    }

    /// Is this market a pure market, i.e., a single token market.
    pub fn is_pure(&self) -> bool {
        self.flag(MarketFlag::Pure)
    }

    /// Is this market enabled.
    pub fn is_enabled(&self) -> bool {
        self.flag(MarketFlag::Enabled)
    }

    /// Set enabled.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.set_flag(MarketFlag::Enabled, enabled);
    }

    /// Record transferred in.
    fn record_transferred_in(&mut self, is_long_token: bool, amount: u64) -> Result<()> {
        // TODO: use event
        msg!(
            "{}: {},{}(+{},{})",
            self.meta.market_token_mint,
            self.state.long_token_balance,
            self.state.short_token_balance,
            amount,
            is_long_token
        );
        if self.is_pure() || is_long_token {
            self.state.long_token_balance = self
                .state
                .long_token_balance
                .checked_add(amount)
                .ok_or(error!(DataStoreError::AmountOverflow))?;
        } else {
            self.state.short_token_balance = self
                .state
                .short_token_balance
                .checked_add(amount)
                .ok_or(error!(DataStoreError::AmountOverflow))?;
        }
        msg!(
            "{}: {},{}",
            self.meta.market_token_mint,
            self.state.long_token_balance,
            self.state.short_token_balance
        );
        Ok(())
    }

    /// Record transferred out.
    fn record_transferred_out(&mut self, is_long_token: bool, amount: u64) -> Result<()> {
        // TODO: use event
        msg!(
            "{}: {},{}(-{},{})",
            self.meta.market_token_mint,
            self.state.long_token_balance,
            self.state.short_token_balance,
            amount,
            is_long_token
        );
        if self.is_pure() || is_long_token {
            self.state.long_token_balance = self
                .state
                .long_token_balance
                .checked_sub(amount)
                .ok_or(error!(DataStoreError::AmountOverflow))?;
        } else {
            self.state.short_token_balance = self
                .state
                .short_token_balance
                .checked_sub(amount)
                .ok_or(error!(DataStoreError::AmountOverflow))?;
        }
        msg!(
            "{}: {},{}",
            self.meta.market_token_mint,
            self.state.long_token_balance,
            self.state.short_token_balance
        );
        Ok(())
    }

    /// Get pool of the given kind.
    #[inline]
    pub fn pool(&self, kind: PoolKind) -> Option<Pool> {
        self.pools.get(kind).copied()
    }

    pub(crate) fn as_market<'a, 'info>(
        &'a mut self,
        mint: &'a mut Account<'info, Mint>,
    ) -> AsMarket<'a, 'info> {
        AsMarket::new(self, mint)
    }

    /// Validate the market.
    pub fn validate(&self, store: &Pubkey) -> Result<()> {
        require_eq!(*store, self.store, DataStoreError::InvalidMarket);
        require!(self.is_enabled(), DataStoreError::DisabledMarket);
        Ok(())
    }

    /// Get config.
    pub fn get_config(&self, key: &str) -> Result<&Factor> {
        let key = MarketConfigKey::from_str(key).map_err(|_| error!(DataStoreError::InvalidKey))?;
        Ok(self.get_config_by_key(key))
    }

    /// Get config by key.
    #[inline]
    pub fn get_config_by_key(&self, key: MarketConfigKey) -> &Factor {
        self.config.get(key)
    }

    /// Get config mutably by key
    pub fn get_config_mut(&mut self, key: &str) -> Result<&mut Factor> {
        let key = MarketConfigKey::from_str(key).map_err(|_| error!(DataStoreError::InvalidKey))?;
        Ok(self.config.get_mut(key))
    }
}

/// Market Flags.
#[repr(u8)]
pub enum MarketFlag {
    /// Is enabled.
    Enabled,
    /// Is Pure.
    Pure,
    // CHECK: cannot have more than `MAX_FLAGS` flags.
}

/// Market State.
#[zero_copy]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct MarketState {
    long_token_balance: u64,
    short_token_balance: u64,
    funding_factor_per_second: i128,
    reserved: [u8; 32],
}

/// Market Metadata.
#[zero_copy]
#[derive(BorshSerialize, BorshDeserialize)]
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

impl MarketMeta {
    /// Check if the given token is a valid collateral token.
    #[inline]
    pub fn is_collateral_token(&self, token: &Pubkey) -> bool {
        *token == self.long_token_mint || *token == self.short_token_mint
    }

    /// Get pnl token.
    pub fn pnl_token(&self, is_long: bool) -> Pubkey {
        if is_long {
            self.long_token_mint
        } else {
            self.short_token_mint
        }
    }

    /// Check if the given token is a valid collateral token,
    /// return error if it is not.
    pub fn validate_collateral_token(&self, token: &Pubkey) -> Result<()> {
        if self.is_collateral_token(token) {
            Ok(())
        } else {
            Err(DataStoreError::InvalidCollateralToken.into())
        }
    }
}

/// Market Pools.
#[zero_copy]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Pools {
    /// Primary Pool.
    primary: Pool,
    /// Swap Impact Pool.
    swap_impact: Pool,
    /// Claimable Fee Pool.
    claimable_fee: Pool,
    /// Long open interest.
    open_interest_for_long: Pool,
    /// Short open interest.
    open_interest_for_short: Pool,
    /// Long open interest in tokens.
    open_interest_in_tokens_for_long: Pool,
    /// Short open interest in tokens.
    open_interest_in_tokens_for_short: Pool,
    /// Position Impact.
    position_impact: Pool,
    /// Borrowing Factor.
    borrowing_factor: Pool,
    /// Funding Amount Per Size for long.
    funding_amount_per_size_for_long: Pool,
    /// Funding Amount Per Size for short.
    funding_amount_per_size_for_short: Pool,
    /// Claimable Funding Amount Per Size for long.
    claimable_funding_amount_per_size_for_long: Pool,
    /// Claimable Funding Amount Per Size for short.
    claimable_funding_amount_per_size_for_short: Pool,
    reserved: [Pool; 8],
}

impl Pools {
    fn init(&mut self, is_pure: bool) {
        self.primary.set_is_pure(is_pure);
        self.swap_impact.set_is_pure(is_pure);
        self.claimable_fee.set_is_pure(is_pure);
        self.open_interest_for_long.set_is_pure(is_pure);
        self.open_interest_for_short.set_is_pure(is_pure);
        self.open_interest_in_tokens_for_long.set_is_pure(is_pure);
        self.claimable_funding_amount_per_size_for_short
            .set_is_pure(is_pure);
        self.position_impact.set_is_pure(is_pure);
        // Borrowing factor must be not pure.
        self.borrowing_factor.set_is_pure(false);
        self.funding_amount_per_size_for_long.set_is_pure(is_pure);
        self.funding_amount_per_size_for_short.set_is_pure(is_pure);
        self.claimable_funding_amount_per_size_for_long
            .set_is_pure(is_pure);
        self.claimable_funding_amount_per_size_for_short
            .set_is_pure(is_pure);
    }

    fn get(&self, kind: PoolKind) -> Option<&Pool> {
        let pool = match kind {
            PoolKind::Primary => &self.primary,
            PoolKind::SwapImpact => &self.swap_impact,
            PoolKind::ClaimableFee => &self.claimable_fee,
            PoolKind::OpenInterestForLong => &self.open_interest_for_long,
            PoolKind::OpenInterestForShort => &self.open_interest_for_short,
            PoolKind::OpenInterestInTokensForLong => &self.open_interest_in_tokens_for_long,
            PoolKind::OpenInterestInTokensForShort => &self.open_interest_in_tokens_for_short,
            PoolKind::PositionImpact => &self.position_impact,
            PoolKind::BorrowingFactor => &self.borrowing_factor,
            PoolKind::FundingAmountPerSizeForLong => &self.funding_amount_per_size_for_long,
            PoolKind::FundingAmountPerSizeForShort => &self.funding_amount_per_size_for_short,
            PoolKind::ClaimableFundingAmountPerSizeForLong => {
                &self.claimable_funding_amount_per_size_for_long
            }
            PoolKind::ClaimableFundingAmountPerSizeForShort => {
                &self.claimable_funding_amount_per_size_for_short
            }
            _ => return None,
        };
        Some(pool)
    }

    fn get_mut(&mut self, kind: PoolKind) -> Option<&mut Pool> {
        let pool = match kind {
            PoolKind::Primary => &mut self.primary,
            PoolKind::SwapImpact => &mut self.swap_impact,
            PoolKind::ClaimableFee => &mut self.claimable_fee,
            PoolKind::OpenInterestForLong => &mut self.open_interest_for_long,
            PoolKind::OpenInterestForShort => &mut self.open_interest_for_short,
            PoolKind::OpenInterestInTokensForLong => &mut self.open_interest_in_tokens_for_long,
            PoolKind::OpenInterestInTokensForShort => &mut self.open_interest_in_tokens_for_short,
            PoolKind::PositionImpact => &mut self.position_impact,
            PoolKind::BorrowingFactor => &mut self.borrowing_factor,
            PoolKind::FundingAmountPerSizeForLong => &mut self.funding_amount_per_size_for_long,
            PoolKind::FundingAmountPerSizeForShort => &mut self.funding_amount_per_size_for_short,
            PoolKind::ClaimableFundingAmountPerSizeForLong => {
                &mut self.claimable_funding_amount_per_size_for_long
            }
            PoolKind::ClaimableFundingAmountPerSizeForShort => {
                &mut self.claimable_funding_amount_per_size_for_short
            }
            _ => return None,
        };
        Some(pool)
    }
}

/// Market clocks.
#[zero_copy]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Clocks {
    /// Price impact distribution clock.
    price_impact_distribution: i64,
    /// Borrowing clock.
    borrowing: i64,
    /// Funding clock.
    funding: i64,
    reserved: [i64; 5],
}

impl Clocks {
    fn init_to_current(&mut self) -> Result<()> {
        let current = Clock::get()?.unix_timestamp;
        self.price_impact_distribution = current;
        self.borrowing = current;
        self.funding = current;
        Ok(())
    }

    fn get_mut(&mut self, kind: ClockKind) -> Option<&mut i64> {
        let clock = match kind {
            ClockKind::PriceImpactDistribution => &mut self.price_impact_distribution,
            ClockKind::Borrowing => &mut self.borrowing,
            ClockKind::Funding => &mut self.funding,
            _ => return None,
        };
        Some(clock)
    }
}

/// A pool for market.
#[zero_copy]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Pool {
    /// Whether the pool only contains one kind of token,
    /// i.e. a pure pool.
    /// For a pure pool, only the `long_token_amount` field is used.
    is_pure: u8,
    padding: [u8; 15],
    /// Long token amount.
    long_token_amount: u128,
    /// Short token amount.
    short_token_amount: u128,
}

const PURE_VALUE: u8 = 1;

impl Pool {
    /// Set the pure flag.
    fn set_is_pure(&mut self, is_pure: bool) {
        self.is_pure = if is_pure { PURE_VALUE } else { 0 };
    }

    /// Is this a pure pool.
    fn is_pure(&self) -> bool {
        !matches!(self.is_pure, 0)
    }
}

impl gmx_core::Balance for Pool {
    type Num = u128;

    type Signed = i128;

    /// Get the long token amount.
    fn long_amount(&self) -> gmx_core::Result<Self::Num> {
        if self.is_pure() {
            debug_assert_eq!(
                self.short_token_amount, 0,
                "short token amount must be zero"
            );
            Ok(self.long_token_amount / 2)
        } else {
            Ok(self.long_token_amount)
        }
    }

    /// Get the short token amount.
    fn short_amount(&self) -> gmx_core::Result<Self::Num> {
        if self.is_pure() {
            debug_assert_eq!(
                self.short_token_amount, 0,
                "short token amount must be zero"
            );
            Ok(self.long_token_amount / 2)
        } else {
            Ok(self.short_token_amount)
        }
    }
}

impl gmx_core::Pool for Pool {
    fn apply_delta_to_long_amount(&mut self, delta: &Self::Signed) -> gmx_core::Result<()> {
        self.long_token_amount = self
            .long_token_amount
            .checked_add_signed(*delta)
            .ok_or(gmx_core::Error::Computation("apply delta to long amount"))?;
        Ok(())
    }

    fn apply_delta_to_short_amount(&mut self, delta: &Self::Signed) -> gmx_core::Result<()> {
        let amount = if self.is_pure() {
            &mut self.long_token_amount
        } else {
            &mut self.short_token_amount
        };
        *amount = amount
            .checked_add_signed(*delta)
            .ok_or(gmx_core::Error::Computation("apply delta to short amount"))?;
        Ok(())
    }
}

#[event]
pub struct MarketChangeEvent {
    pub address: Pubkey,
    pub action: super::Action,
}
