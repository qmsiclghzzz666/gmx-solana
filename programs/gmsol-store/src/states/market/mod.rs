use std::str::FromStr;

use anchor_lang::{prelude::*, Bump};
use bitmaps::Bitmap;
use borsh::{BorshDeserialize, BorshSerialize};
use config::MarketConfigBuffer;
use gmsol_model::{ClockKind, PoolKind};

use crate::{
    utils::fixed_str::{bytes_to_fixed_str, fixed_str_to_bytes},
    StoreError,
};

use super::{Factor, InitSpace, Seed};

pub use self::{
    config::{MarketConfig, MarketConfigKey},
    ops::ValidateMarketBalances,
};

/// Market Operations.
pub mod ops;

/// Clock ops.
pub mod clock;

/// Market Config.
pub mod config;

/// Revertible Market Operations.
pub mod revertible;

/// Max number of flags.
pub const MAX_FLAGS: usize = 8;

/// Market Flag Value.
pub type MarketFlagValue = u8;

/// Market Flag Bitmap.
pub type MarketFlagBitmap = Bitmap<MAX_FLAGS>;

const MAX_NAME_LEN: usize = 64;

/// Find PDA for [`Market`] account.
pub fn find_market_address(
    store: &Pubkey,
    token: &Pubkey,
    store_program_id: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[Market::SEED, store.as_ref(), token.as_ref()],
        store_program_id,
    )
}

/// Market.
#[account(zero_copy)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Market {
    /// Bump Seed.
    pub(crate) bump: u8,
    flag: MarketFlagValue,
    padding: [u8; 14],
    name: [u8; MAX_NAME_LEN],
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
        name: &str,
        market_token_mint: Pubkey,
        index_token_mint: Pubkey,
        long_token_mint: Pubkey,
        short_token_mint: Pubkey,
        is_enabled: bool,
    ) -> Result<()> {
        self.bump = bump;
        self.store = store;
        self.name = fixed_str_to_bytes(name)?;
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

    /// Get name.
    pub fn name(&self) -> Result<&str> {
        bytes_to_fixed_str(&self.name)
    }

    /// Record transferred in by the given token.
    pub fn record_transferred_in_by_token(&mut self, token: &Pubkey, amount: u64) -> Result<()> {
        if self.meta.long_token_mint == *token {
            self.record_transferred_in(true, amount)
        } else if self.meta.short_token_mint == *token {
            self.record_transferred_in(false, amount)
        } else {
            Err(error!(StoreError::InvalidCollateralToken))
        }
    }

    /// Record transferred out by the given token.
    pub fn record_transferred_out_by_token(&mut self, token: &Pubkey, amount: u64) -> Result<()> {
        if self.meta.long_token_mint == *token {
            self.record_transferred_out(true, amount)
        } else if self.meta.short_token_mint == *token {
            self.record_transferred_out(false, amount)
        } else {
            Err(error!(StoreError::InvalidCollateralToken))
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
                .ok_or(error!(StoreError::AmountOverflow))?;
        } else {
            self.state.short_token_balance = self
                .state
                .short_token_balance
                .checked_add(amount)
                .ok_or(error!(StoreError::AmountOverflow))?;
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
                .ok_or(error!(StoreError::AmountOverflow))?;
        } else {
            self.state.short_token_balance = self
                .state
                .short_token_balance
                .checked_sub(amount)
                .ok_or(error!(StoreError::AmountOverflow))?;
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

    /// Validate the market.
    pub fn validate(&self, store: &Pubkey) -> Result<()> {
        require_eq!(*store, self.store, StoreError::InvalidMarket);
        require!(self.is_enabled(), StoreError::DisabledMarket);
        Ok(())
    }

    /// Get config.
    pub fn get_config(&self, key: &str) -> Result<&Factor> {
        let key = MarketConfigKey::from_str(key).map_err(|_| error!(StoreError::InvalidKey))?;
        Ok(self.get_config_by_key(key))
    }

    /// Get config by key.
    #[inline]
    pub fn get_config_by_key(&self, key: MarketConfigKey) -> &Factor {
        self.config.get(key)
    }

    /// Get config mutably by key
    pub fn get_config_mut(&mut self, key: &str) -> Result<&mut Factor> {
        let key = MarketConfigKey::from_str(key).map_err(|_| error!(StoreError::InvalidKey))?;
        Ok(self.config.get_mut(key))
    }

    /// Get market state.
    pub fn state(&self) -> &MarketState {
        &self.state
    }

    /// Get market state mutably.
    pub fn state_mut(&mut self) -> &mut MarketState {
        &mut self.state
    }

    /// Update config with buffer.
    pub fn update_config_with_buffer(&mut self, buffer: &MarketConfigBuffer) -> Result<()> {
        for entry in buffer.iter() {
            let key = entry.key()?;
            let current_value = self.config.get_mut(key);
            let new_value = entry.value();
            msg!(
                "{}: update config `{}` from {} to {}",
                self.meta.market_token_mint,
                key,
                current_value,
                new_value
            );
            *current_value = new_value;
        }
        Ok(())
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
    counters_updated_at_slot: u64,
    deposit_count: u64,
    withdrawal_count: u64,
    order_count: u64,
}

impl MarketState {
    /// Get long token balance.
    pub fn long_token_balance_raw(&self) -> u64 {
        self.long_token_balance
    }

    /// Get short token balance.
    pub fn short_token_balance_raw(&self) -> u64 {
        self.short_token_balance
    }

    /// Get funding factor per second.
    pub fn funding_factor_per_second(&self) -> i128 {
        self.funding_factor_per_second
    }

    /// Get updated slot for counters.
    pub fn counters_updated_at_slot(&self) -> u64 {
        self.counters_updated_at_slot
    }

    /// Get current deposit count.
    pub fn deposit_count(&self) -> u64 {
        self.deposit_count
    }

    /// Get current withdrawal count.
    pub fn withdrawal_count(&self) -> u64 {
        self.withdrawal_count
    }

    /// Get current order count.
    pub fn order_count(&self) -> u64 {
        self.order_count
    }

    fn post_update(&mut self) -> Result<()> {
        self.counters_updated_at_slot = Clock::get()?.slot;
        Ok(())
    }

    /// Next deposit id.
    pub fn next_deposit_id(&mut self) -> Result<u64> {
        let next_id = self
            .deposit_count
            .checked_add(1)
            .ok_or(error!(StoreError::AmountOverflow))?;
        self.deposit_count = next_id;
        self.post_update()?;
        Ok(next_id)
    }

    /// Next withdrawal id.
    pub fn next_withdrawal_id(&mut self) -> Result<u64> {
        let next_id = self
            .withdrawal_count
            .checked_add(1)
            .ok_or(error!(StoreError::AmountOverflow))?;
        self.withdrawal_count = next_id;
        self.post_update()?;
        Ok(next_id)
    }

    /// Next order id.
    pub fn next_order_id(&mut self) -> Result<u64> {
        let next_id = self
            .order_count
            .checked_add(1)
            .ok_or(error!(StoreError::AmountOverflow))?;
        self.order_count = next_id;
        self.post_update()?;
        Ok(next_id)
    }
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

    /// Check if the given token is long token or short token, and return it's side.
    pub fn to_token_side(&self, token: &Pubkey) -> Result<bool> {
        if *token == self.long_token_mint {
            Ok(true)
        } else if *token == self.short_token_mint {
            Ok(false)
        } else {
            err!(StoreError::InvalidArgument)
        }
    }

    /// Get opposite token.
    pub fn opposite_token(&self, token: &Pubkey) -> Result<&Pubkey> {
        if *token == self.long_token_mint {
            Ok(&self.short_token_mint)
        } else if *token == self.short_token_mint {
            Ok(&self.long_token_mint)
        } else {
            err!(StoreError::InvalidArgument)
        }
    }

    /// Check if the given token is a valid collateral token,
    /// return error if it is not.
    pub fn validate_collateral_token(&self, token: &Pubkey) -> Result<()> {
        if self.is_collateral_token(token) {
            Ok(())
        } else {
            Err(StoreError::InvalidCollateralToken.into())
        }
    }
}

/// Type that has market meta.
pub trait HasMarketMeta {
    fn is_pure(&self) -> bool;

    fn market_meta(&self) -> &MarketMeta;
}

impl HasMarketMeta for Market {
    fn is_pure(&self) -> bool {
        self.is_pure()
    }

    fn market_meta(&self) -> &MarketMeta {
        &self.meta
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

    fn get(&self, kind: ClockKind) -> Option<&i64> {
        let clock = match kind {
            ClockKind::PriceImpactDistribution => &self.price_impact_distribution,
            ClockKind::Borrowing => &self.borrowing,
            ClockKind::Funding => &self.funding,
            _ => return None,
        };
        Some(clock)
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

impl gmsol_model::Balance for Pool {
    type Num = u128;

    type Signed = i128;

    /// Get the long token amount.
    fn long_amount(&self) -> gmsol_model::Result<Self::Num> {
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
    fn short_amount(&self) -> gmsol_model::Result<Self::Num> {
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

impl gmsol_model::Pool for Pool {
    fn apply_delta_to_long_amount(&mut self, delta: &Self::Signed) -> gmsol_model::Result<()> {
        self.long_token_amount = self.long_token_amount.checked_add_signed(*delta).ok_or(
            gmsol_model::Error::Computation("apply delta to long amount"),
        )?;
        Ok(())
    }

    fn apply_delta_to_short_amount(&mut self, delta: &Self::Signed) -> gmsol_model::Result<()> {
        let amount = if self.is_pure() {
            &mut self.long_token_amount
        } else {
            &mut self.short_token_amount
        };
        *amount = amount
            .checked_add_signed(*delta)
            .ok_or(gmsol_model::Error::Computation(
                "apply delta to short amount",
            ))?;
        Ok(())
    }
}

#[event]
pub struct MarketChangeEvent {
    pub address: Pubkey,
    pub action: super::Action,
}
