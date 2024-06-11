use anchor_lang::{prelude::*, Bump};
use anchor_spl::token::Mint;
use bitmaps::Bitmap;
use borsh::{BorshDeserialize, BorshSerialize};
use gmx_core::{
    params::{
        fee::{BorrowingFeeParams, FundingFeeParams},
        position::PositionImpactDistributionParams,
        FeeParams, PositionParams, PriceImpactParams,
    },
    ClockKind, MarketExt, PoolKind,
};

use crate::{constants, utils::internal::TransferUtils, DataStoreError, GmxCoreError};

use super::{
    position::{Position, PositionOps},
    InitSpace, Seed, Store,
};

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
        AsMarket {
            meta: &self.meta,
            long_token_balance: &self.state.long_token_balance,
            short_token_balance: &self.state.short_token_balance,
            pools: &mut self.pools,
            clocks: &mut self.clocks,
            mint,
            transfer: None,
            receiver: None,
            vault: None,
            funding_factor_per_second: &mut self.state.funding_factor_per_second,
        }
    }

    /// Validate the market.
    pub fn validate(&self, store: &Pubkey) -> Result<()> {
        require_eq!(*store, self.store, DataStoreError::InvalidMarket);
        require!(self.is_enabled(), DataStoreError::DisabledMarket);
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

/// Convert to a [`Market`](gmx_core::Market).
pub struct AsMarket<'a, 'info> {
    meta: &'a MarketMeta,
    long_token_balance: &'a u64,
    short_token_balance: &'a u64,
    funding_factor_per_second: &'a mut i128,
    pools: &'a mut Pools,
    clocks: &'a mut Clocks,
    mint: &'a mut Account<'info, Mint>,
    transfer: Option<TransferUtils<'a, 'info>>,
    receiver: Option<AccountInfo<'info>>,
    vault: Option<AccountInfo<'info>>,
}

impl<'a, 'info> AsMarket<'a, 'info> {
    pub(crate) fn enable_transfer(
        mut self,
        token_program: AccountInfo<'info>,
        store: &'a AccountLoader<'info, Store>,
    ) -> Self {
        self.transfer = Some(TransferUtils::new(
            token_program,
            store,
            Some(self.mint.to_account_info()),
        ));
        self
    }

    pub(crate) fn with_receiver(mut self, receiver: AccountInfo<'info>) -> Self {
        self.receiver = Some(receiver);
        self
    }

    pub(crate) fn with_vault(mut self, vault: AccountInfo<'info>) -> Self {
        self.vault = Some(vault);
        self
    }

    pub(crate) fn meta(&self) -> &MarketMeta {
        self.meta
    }

    pub(crate) fn into_position_ops(
        self,
        position: &'a mut AccountLoader<'info, Position>,
    ) -> Result<PositionOps<'a, 'info>> {
        PositionOps::try_new(self, position)
    }

    fn balance_after_excluding(&self, is_long_token: bool, excluding_amount: u64) -> Result<u64> {
        let balance = if is_long_token {
            self.long_token_balance
        } else {
            self.short_token_balance
        };
        if excluding_amount != 0 {
            balance
                .checked_sub(excluding_amount)
                .ok_or(error!(DataStoreError::AmountOverflow))
        } else {
            Ok(*balance)
        }
    }

    pub(crate) fn validate_market_balances(
        &self,
        long_excluding_amount: u64,
        short_excluding_amount: u64,
    ) -> Result<()> {
        let long_token_balance = self.balance_after_excluding(true, long_excluding_amount)? as u128;
        self.validate_token_balance_for_one_side(&long_token_balance, true)
            .map_err(GmxCoreError::from)?;
        let short_token_balance =
            self.balance_after_excluding(false, short_excluding_amount)? as u128;
        self.validate_token_balance_for_one_side(&short_token_balance, false)
            .map_err(GmxCoreError::from)?;
        Ok(())
    }

    pub(crate) fn validate_market_balances_excluding_token_amount(
        &self,
        token: &Pubkey,
        amount: u64,
    ) -> Result<()> {
        if *token == self.meta.long_token_mint {
            self.validate_market_balances(amount, 0)
        } else if *token == self.meta.short_token_mint {
            self.validate_market_balances(0, amount)
        } else {
            Err(error!(DataStoreError::InvalidCollateralToken))
        }
    }
}

impl<'a, 'info> gmx_core::Market<{ constants::MARKET_DECIMALS }> for AsMarket<'a, 'info> {
    type Num = u128;

    type Signed = i128;

    type Pool = Pool;

    fn pool(&self, kind: PoolKind) -> gmx_core::Result<Option<&Self::Pool>> {
        Ok(self.pools.get(kind))
    }

    fn pool_mut(&mut self, kind: PoolKind) -> gmx_core::Result<Option<&mut Self::Pool>> {
        Ok(self.pools.get_mut(kind))
    }

    fn total_supply(&self) -> Self::Num {
        self.mint.supply.into()
    }

    fn mint(&mut self, amount: &Self::Num) -> gmx_core::Result<()> {
        let Some(transfer) = self.transfer.as_ref() else {
            return Err(gmx_core::Error::invalid_argument("transfer not enabled"));
        };
        let Some(receiver) = self.receiver.as_ref() else {
            return Err(gmx_core::Error::MintReceiverNotSet);
        };
        transfer.mint_to(
            receiver,
            (*amount)
                .try_into()
                .map_err(|_| gmx_core::Error::Overflow)?,
        )?;
        self.mint.reload()?;
        Ok(())
    }

    fn burn(&mut self, amount: &Self::Num) -> gmx_core::Result<()> {
        let Some(transfer) = self.transfer.as_ref() else {
            return Err(gmx_core::Error::invalid_argument("transfer not enabled"));
        };
        let Some(vault) = self.vault.as_ref() else {
            return Err(gmx_core::Error::WithdrawalVaultNotSet);
        };
        transfer.burn_from(
            vault,
            (*amount)
                .try_into()
                .map_err(|_| gmx_core::Error::Overflow)?,
        )?;
        self.mint.reload()?;
        Ok(())
    }

    fn just_passed_in_seconds(&mut self, clock: ClockKind) -> gmx_core::Result<u64> {
        let current = Clock::get().map_err(Error::from)?.unix_timestamp;
        let last = self
            .clocks
            .get_mut(clock)
            .ok_or(gmx_core::Error::MissingClockKind(clock))?;
        let duration = current.saturating_sub(*last);
        if duration > 0 {
            *last = current;
        }
        Ok(duration as u64)
    }

    fn usd_to_amount_divisor(&self) -> Self::Num {
        constants::MARKET_USD_TO_AMOUNT_DIVISOR
    }

    fn funding_amount_per_size_adjustment(&self) -> Self::Num {
        constants::FUNDING_AMOUNT_PER_SIZE_ADJUSTMENT
    }

    fn swap_impact_params(&self) -> gmx_core::Result<PriceImpactParams<Self::Num>> {
        PriceImpactParams::builder()
            .with_exponent(2 * constants::MARKET_USD_UNIT)
            .with_positive_factor(400_000_000_000)
            .with_negative_factor(800_000_000_000)
            .build()
    }

    fn swap_fee_params(&self) -> gmx_core::Result<FeeParams<Self::Num>> {
        Ok(FeeParams::builder()
            .with_fee_receiver_factor(37_000_000_000_000_000_000)
            .with_positive_impact_fee_factor(50_000_000_000_000_000)
            .with_negative_impact_fee_factor(70_000_000_000_000_000)
            .build())
    }

    fn position_params(&self) -> gmx_core::Result<PositionParams<Self::Num>> {
        Ok(PositionParams::new(
            constants::MARKET_USD_UNIT,
            constants::MARKET_USD_UNIT,
            constants::MARKET_USD_UNIT / 100,
            500_000_000_000_000_000,
            500_000_000_000_000_000,
            250_000_000_000_000_000,
        ))
    }

    fn position_impact_params(&self) -> gmx_core::Result<PriceImpactParams<Self::Num>> {
        PriceImpactParams::builder()
            .with_exponent(2 * constants::MARKET_USD_UNIT)
            .with_positive_factor(9_000_000_000)
            .with_negative_factor(15_000_000_000)
            .build()
    }

    fn order_fee_params(&self) -> gmx_core::Result<FeeParams<Self::Num>> {
        Ok(FeeParams::builder()
            .with_fee_receiver_factor(37_000_000_000_000_000_000)
            .with_positive_impact_fee_factor(50_000_000_000_000_000)
            .with_negative_impact_fee_factor(70_000_000_000_000_000)
            .build())
    }

    fn position_impact_distribution_params(
        &self,
    ) -> gmx_core::Result<PositionImpactDistributionParams<Self::Num>> {
        Ok(PositionImpactDistributionParams::builder()
            .distribute_factor(constants::MARKET_USD_UNIT)
            .min_position_impact_pool_amount(1_000_000_000)
            .build())
    }

    fn borrowing_fee_params(&self) -> gmx_core::Result<BorrowingFeeParams<Self::Num>> {
        Ok(BorrowingFeeParams::builder()
            .receiver_factor(37_000_000_000_000_000_000)
            .factor_for_long(2_820_000_000_000)
            .factor_for_short(2_820_000_000_000)
            .exponent_for_long(100_000_000_000_000_000_000)
            .exponent_for_short(100_000_000_000_000_000_000)
            .build())
    }

    fn funding_fee_params(&self) -> gmx_core::Result<FundingFeeParams<Self::Num>> {
        Ok(FundingFeeParams::builder()
            .exponent(100_000_000_000_000_000_000)
            .funding_factor(2_000_000_000_000)
            .max_factor_per_second(1_000_000_000_000)
            .min_factor_per_second(30_000_000_000)
            .increase_factor_per_second(790_000_000)
            .decrease_factor_per_second(0)
            .threshold_for_stable_funding(5_000_000_000_000_000_000)
            .threshold_for_decrease_funding(0)
            .build())
    }

    fn funding_factor_per_second(&self) -> &Self::Signed {
        self.funding_factor_per_second
    }

    fn funding_factor_per_second_mut(&mut self) -> &mut Self::Signed {
        self.funding_factor_per_second
    }

    fn reserve_factor(&self) -> gmx_core::Result<Self::Num> {
        Ok(constants::MARKET_USD_UNIT)
    }

    fn open_interest_reserve_factor(&self) -> gmx_core::Result<Self::Num> {
        Ok(constants::MARKET_USD_UNIT)
    }

    fn max_pnl_factor(
        &self,
        kind: gmx_core::PnlFactorKind,
        _is_long: bool,
    ) -> gmx_core::Result<Self::Num> {
        use gmx_core::PnlFactorKind;

        match kind {
            PnlFactorKind::Deposit => Ok(60_000_000_000_000_000_000),
            PnlFactorKind::Withdrawal => Ok(30_000_000_000_000_000_000),
            _ => Err(error!(DataStoreError::RequiredResourceNotFound).into()),
        }
    }

    fn max_pool_amount(&self, _is_long_token: bool) -> gmx_core::Result<Self::Num> {
        Ok(1_000_000_000 * constants::MARKET_USD_UNIT)
    }

    fn max_pool_value_for_deposit(&self, _is_long_token: bool) -> gmx_core::Result<Self::Num> {
        Ok(1_000_000_000_000_000 * constants::MARKET_USD_UNIT)
    }

    fn max_open_interest(&self, _is_long: bool) -> gmx_core::Result<Self::Num> {
        Ok(1_000_000_000 * constants::MARKET_USD_UNIT)
    }
}

#[event]
pub struct MarketChangeEvent {
    pub address: Pubkey,
    pub action: super::Action,
}
