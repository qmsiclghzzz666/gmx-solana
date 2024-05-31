use std::{collections::HashMap, fmt, time::Instant};

use crate::{
    clock::ClockKind,
    fixed::FixedPointOps,
    market::{Market, PnlFactorKind},
    num::{MulDiv, Num, Unsigned, UnsignedAbs},
    params::{
        fee::{BorrowingFeeParams, FundingFeeParams},
        position::PositionImpactDistributionParams,
        FeeParams, PositionParams, PriceImpactParams,
    },
    pool::{Balance, Pool, PoolKind},
    position::Position,
};
use num_traits::{CheckedSub, Signed};

/// Test Pool.
#[derive(Debug, Default, Clone, Copy)]
pub struct TestPool<T> {
    long_amount: T,
    short_amount: T,
}

impl<T> Balance for TestPool<T>
where
    T: MulDiv + Num + CheckedSub,
{
    type Num = T;

    type Signed = T::Signed;

    fn long_amount(&self) -> crate::Result<Self::Num> {
        Ok(self.long_amount.clone())
    }

    fn short_amount(&self) -> crate::Result<Self::Num> {
        Ok(self.short_amount.clone())
    }
}

impl<T> Pool for TestPool<T>
where
    T: MulDiv + Num + CheckedSub,
{
    fn apply_delta_to_long_amount(&mut self, delta: &Self::Signed) -> Result<(), crate::Error> {
        if delta.is_positive() {
            self.long_amount = self
                .long_amount
                .checked_add(&delta.unsigned_abs())
                .ok_or(crate::Error::Overflow)?;
        } else {
            self.long_amount = self
                .long_amount
                .checked_sub(&delta.unsigned_abs())
                .ok_or(crate::Error::Underflow)?;
        }
        Ok(())
    }

    fn apply_delta_to_short_amount(&mut self, delta: &Self::Signed) -> Result<(), crate::Error> {
        if delta.is_positive() {
            self.short_amount = self
                .short_amount
                .checked_add(&delta.unsigned_abs())
                .ok_or(crate::Error::Overflow)?;
        } else {
            self.short_amount = self
                .short_amount
                .checked_sub(&delta.unsigned_abs())
                .ok_or(crate::Error::Underflow)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct MaxPnlFactors<T> {
    deposit: T,
    withdrawal: T,
}

/// Test Market.
#[derive(Debug, Clone)]
pub struct TestMarket<T: Unsigned, const DECIMALS: u8> {
    total_supply: T,
    value_to_amount_divisor: T,
    funding_amount_per_size_adjustment: T,
    swap_impact_params: PriceImpactParams<T>,
    swap_fee_params: FeeParams<T>,
    position_params: PositionParams<T>,
    position_impact_params: PriceImpactParams<T>,
    order_fee_params: FeeParams<T>,
    position_impact_distribution_params: PositionImpactDistributionParams<T>,
    borrowing_fee_params: BorrowingFeeParams<T>,
    funding_fee_params: FundingFeeParams<T>,
    reserve_factor: T,
    open_interest_reserve_factor: T,
    max_pnl_factors: MaxPnlFactors<T>,
    max_pool_amount: T,
    max_pool_value_for_deposit: T,
    max_open_interest: T,
    primary: TestPool<T>,
    swap_impact: TestPool<T>,
    fee: TestPool<T>,
    open_interest: (TestPool<T>, TestPool<T>),
    open_interest_in_tokens: (TestPool<T>, TestPool<T>),
    position_impact: TestPool<T>,
    borrowing_factor: TestPool<T>,
    funding_factor_per_second: T::Signed,
    funding_amount_per_size: (TestPool<T>, TestPool<T>),
    claimable_funding_amount_per_size: (TestPool<T>, TestPool<T>),
    clocks: HashMap<ClockKind, Instant>,
}

impl Default for TestMarket<u64, 9> {
    fn default() -> Self {
        Self {
            total_supply: Default::default(),
            value_to_amount_divisor: 1,
            funding_amount_per_size_adjustment: 10_000,
            swap_impact_params: PriceImpactParams::builder()
                .with_exponent(2_000_000_000)
                .with_positive_factor(4)
                .with_negative_factor(8)
                .build()
                .unwrap(),
            swap_fee_params: FeeParams::builder()
                .with_fee_receiver_factor(370_000_000)
                .with_positive_impact_fee_factor(500_000)
                .with_negative_impact_fee_factor(700_000)
                .build(),
            position_params: PositionParams::new(
                1_000_000_000,
                1_000_000_000,
                10_000_000,
                5_000_000,
                5_000_000,
                2_500_000,
            ),
            position_impact_params: PriceImpactParams::builder()
                .with_exponent(2_000_000_000)
                .with_positive_factor(1)
                .with_negative_factor(2)
                .build()
                .unwrap(),
            order_fee_params: FeeParams::builder()
                .with_fee_receiver_factor(370_000_000)
                .with_positive_impact_fee_factor(500_000)
                .with_negative_impact_fee_factor(700_000)
                .build(),
            position_impact_distribution_params: PositionImpactDistributionParams::builder()
                .distribute_factor(1_000_000_000)
                .min_position_impact_pool_amount(1_000_000_000)
                .build(),
            borrowing_fee_params: BorrowingFeeParams::builder()
                .factor_for_long(28)
                .factor_for_short(28)
                .exponent_for_long(1_000_000_000)
                .exponent_for_short(1_000_000_000)
                .build(),
            funding_fee_params: FundingFeeParams::builder()
                .exponent(1_000_000_000)
                .funding_factor(20)
                .max_factor_per_second(10)
                .min_factor_per_second(1)
                .increase_factor_per_second(10)
                .decrease_factor_per_second(0)
                .threshold_for_stable_funding(50_000_000)
                .threshold_for_decrease_funding(0)
                .build(),
            reserve_factor: 1_000_000_000,
            max_pnl_factors: MaxPnlFactors {
                deposit: 600_000_000,
                withdrawal: 300_000_000,
            },
            open_interest_reserve_factor: 1_000_000_000,
            max_pool_amount: 1_000_000_000 * 1_000_000_000,
            max_pool_value_for_deposit: u64::MAX,
            max_open_interest: u64::MAX,
            primary: Default::default(),
            swap_impact: Default::default(),
            fee: Default::default(),
            open_interest: Default::default(),
            open_interest_in_tokens: Default::default(),
            position_impact: Default::default(),
            borrowing_factor: Default::default(),
            funding_factor_per_second: Default::default(),
            funding_amount_per_size: Default::default(),
            claimable_funding_amount_per_size: Default::default(),
            clocks: Default::default(),
        }
    }
}

#[cfg(feature = "u128")]
impl Default for TestMarket<u128, 20> {
    fn default() -> Self {
        Self {
            total_supply: Default::default(),
            value_to_amount_divisor: 10u128.pow(20 - 9),
            funding_amount_per_size_adjustment: 10u128.pow(10),
            swap_impact_params: PriceImpactParams::builder()
                .with_exponent(200_000_000_000_000_000_000)
                .with_positive_factor(400_000_000_000)
                .with_negative_factor(800_000_000_000)
                .build()
                .unwrap(),
            swap_fee_params: FeeParams::builder()
                .with_fee_receiver_factor(37_000_000_000_000_000_000)
                .with_positive_impact_fee_factor(50_000_000_000_000_000)
                .with_negative_impact_fee_factor(70_000_000_000_000_000)
                .build(),
            position_params: PositionParams::new(
                100_000_000_000_000_000_000,
                100_000_000_000_000_000_000,
                1_000_000_000_000_000_000,
                500_000_000_000_000_000,
                500_000_000_000_000_000,
                250_000_000_000_000_000,
            ),
            position_impact_params: PriceImpactParams::builder()
                .with_exponent(200_000_000_000_000_000_000)
                .with_positive_factor(100_000_000_000)
                .with_negative_factor(200_000_000_000)
                .build()
                .unwrap(),
            order_fee_params: FeeParams::builder()
                .with_fee_receiver_factor(37_000_000_000_000_000_000)
                .with_positive_impact_fee_factor(50_000_000_000_000_000)
                .with_negative_impact_fee_factor(70_000_000_000_000_000)
                .build(),
            position_impact_distribution_params: PositionImpactDistributionParams::builder()
                .distribute_factor(100_000_000_000_000_000_000)
                .min_position_impact_pool_amount(1_000_000_000)
                .build(),
            borrowing_fee_params: BorrowingFeeParams::builder()
                .factor_for_long(2_820_000_000_000)
                .factor_for_short(2_820_000_000_000)
                .exponent_for_long(100_000_000_000_000_000_000)
                .exponent_for_short(100_000_000_000_000_000_000)
                .build(),
            funding_fee_params: FundingFeeParams::builder()
                .exponent(100_000_000_000_000_000_000)
                .funding_factor(2_000_000_000_000)
                .max_factor_per_second(1_000_000_000_000)
                .min_factor_per_second(30_000_000_000)
                .increase_factor_per_second(790_000_000)
                .decrease_factor_per_second(0)
                .threshold_for_stable_funding(5_000_000_000_000_000_000)
                .threshold_for_decrease_funding(0)
                .build(),
            reserve_factor: 10u128.pow(20),
            open_interest_reserve_factor: 10u128.pow(20),
            max_pnl_factors: MaxPnlFactors {
                deposit: 60_000_000_000_000_000_000,
                withdrawal: 30_000_000_000_000_000_000,
            },
            max_pool_amount: 1_000_000_000 * 10u128.pow(20),
            max_pool_value_for_deposit: 1_000_000_000_000_000 * 10u128.pow(20),
            max_open_interest: 1_000_000_000 * 10u128.pow(20),
            primary: Default::default(),
            swap_impact: Default::default(),
            fee: Default::default(),
            open_interest: Default::default(),
            open_interest_in_tokens: Default::default(),
            position_impact: Default::default(),
            borrowing_factor: Default::default(),
            funding_factor_per_second: Default::default(),
            funding_amount_per_size: Default::default(),
            claimable_funding_amount_per_size: Default::default(),
            clocks: Default::default(),
        }
    }
}

impl<T, const DECIMALS: u8> Market<DECIMALS> for TestMarket<T, DECIMALS>
where
    T: CheckedSub + fmt::Display + FixedPointOps<DECIMALS>,
    T::Signed: Num + std::fmt::Debug,
{
    type Num = T;

    type Signed = T::Signed;

    type Pool = TestPool<T>;

    fn pool(&self, kind: PoolKind) -> crate::Result<Option<&Self::Pool>> {
        let pool = match kind {
            PoolKind::Primary => &self.primary,
            PoolKind::SwapImpact => &self.swap_impact,
            PoolKind::ClaimableFee => &self.fee,
            PoolKind::OpenInterestForLong => &self.open_interest.0,
            PoolKind::OpenInterestForShort => &self.open_interest.1,
            PoolKind::OpenInterestInTokensForLong => &self.open_interest_in_tokens.0,
            PoolKind::OpenInterestInTokensForShort => &self.open_interest_in_tokens.1,
            PoolKind::PositionImpact => &self.position_impact,
            PoolKind::BorrowingFactor => &self.borrowing_factor,
            PoolKind::FundingAmountPerSizeForLong => &self.funding_amount_per_size.0,
            PoolKind::FundingAmountPerSizeForShort => &self.funding_amount_per_size.1,
            PoolKind::ClaimableFundingAmountPerSizeForLong => {
                &self.claimable_funding_amount_per_size.0
            }
            PoolKind::ClaimableFundingAmountPerSizeForShort => {
                &self.claimable_funding_amount_per_size.1
            }
        };
        Ok(Some(pool))
    }

    fn pool_mut(&mut self, kind: PoolKind) -> crate::Result<Option<&mut Self::Pool>> {
        let pool = match kind {
            PoolKind::Primary => &mut self.primary,
            PoolKind::SwapImpact => &mut self.swap_impact,
            PoolKind::ClaimableFee => &mut self.fee,
            PoolKind::OpenInterestForLong => &mut self.open_interest.0,
            PoolKind::OpenInterestForShort => &mut self.open_interest.1,
            PoolKind::OpenInterestInTokensForLong => &mut self.open_interest_in_tokens.0,
            PoolKind::OpenInterestInTokensForShort => &mut self.open_interest_in_tokens.1,
            PoolKind::PositionImpact => &mut self.position_impact,
            PoolKind::BorrowingFactor => &mut self.borrowing_factor,
            PoolKind::FundingAmountPerSizeForLong => &mut self.funding_amount_per_size.0,
            PoolKind::FundingAmountPerSizeForShort => &mut self.funding_amount_per_size.1,
            PoolKind::ClaimableFundingAmountPerSizeForLong => {
                &mut self.claimable_funding_amount_per_size.0
            }
            PoolKind::ClaimableFundingAmountPerSizeForShort => {
                &mut self.claimable_funding_amount_per_size.1
            }
        };
        Ok(Some(pool))
    }

    fn total_supply(&self) -> Self::Num {
        self.total_supply.clone()
    }

    fn mint(&mut self, amount: &Self::Num) -> Result<(), crate::Error> {
        self.total_supply = self
            .total_supply
            .checked_add(amount)
            .ok_or(crate::Error::Overflow)?;
        Ok(())
    }

    fn burn(&mut self, amount: &Self::Num) -> crate::Result<()> {
        self.total_supply = self
            .total_supply
            .checked_sub(amount)
            .ok_or(crate::Error::Underflow)?;
        Ok(())
    }

    fn just_passed_in_seconds(&mut self, clock: ClockKind) -> crate::Result<u64> {
        let now = Instant::now();
        let clock = self.clocks.entry(clock).or_insert(now);
        let duration = now.saturating_duration_since(*clock);
        *clock = now;
        Ok(duration.as_secs())
    }

    fn usd_to_amount_divisor(&self) -> Self::Num {
        self.value_to_amount_divisor.clone()
    }

    fn funding_amount_per_size_adjustment(&self) -> Self::Num {
        self.funding_amount_per_size_adjustment.clone()
    }

    fn swap_impact_params(&self) -> PriceImpactParams<Self::Num> {
        self.swap_impact_params.clone()
    }

    fn swap_fee_params(&self) -> crate::params::FeeParams<Self::Num> {
        self.swap_fee_params.clone()
    }

    fn position_params(&self) -> crate::params::PositionParams<Self::Num> {
        self.position_params.clone()
    }

    fn position_impact_params(&self) -> PriceImpactParams<Self::Num> {
        self.position_impact_params.clone()
    }

    fn order_fee_params(&self) -> FeeParams<Self::Num> {
        self.order_fee_params.clone()
    }

    fn position_impact_distribution_params(&self) -> PositionImpactDistributionParams<Self::Num> {
        self.position_impact_distribution_params.clone()
    }

    fn borrowing_fee_params(&self) -> BorrowingFeeParams<Self::Num> {
        self.borrowing_fee_params.clone()
    }

    fn funding_fee_params(&self) -> FundingFeeParams<Self::Num> {
        self.funding_fee_params.clone()
    }

    fn funding_factor_per_second(&self) -> &Self::Signed {
        &self.funding_factor_per_second
    }

    fn funding_factor_per_second_mut(&mut self) -> &mut Self::Signed {
        &mut self.funding_factor_per_second
    }

    fn reserve_factor(&self) -> &Self::Num {
        &self.reserve_factor
    }

    fn open_interest_reserve_factor(&self) -> &Self::Num {
        &self.open_interest_reserve_factor
    }

    fn max_pnl_factor(&self, kind: PnlFactorKind, _is_long: bool) -> crate::Result<Self::Num> {
        let factor = match kind {
            PnlFactorKind::Deposit => self.max_pnl_factors.deposit.clone(),
            PnlFactorKind::Withdrawal => self.max_pnl_factors.withdrawal.clone(),
        };
        Ok(factor)
    }

    fn max_pool_amount(&self, _is_long_token: bool) -> crate::Result<Self::Num> {
        Ok(self.max_pool_amount.clone())
    }

    fn max_pool_value_for_deposit(&self, _is_long_token: bool) -> crate::Result<Self::Num> {
        Ok(self.max_pool_value_for_deposit.clone())
    }

    fn max_open_interest(&self, _is_long: bool) -> crate::Result<Self::Num> {
        Ok(self.max_open_interest.clone())
    }
}

/// Test Position
#[derive(Debug, Clone, Copy, Default)]
pub struct TestPosition<T, const DECIMALS: u8> {
    is_long: bool,
    is_collateral_token_long: bool,
    collateral_token_amount: T,
    size_in_usd: T,
    size_in_tokens: T,
    borrowing_factor: T,
    funding_fee_amount_per_size: T,
    claimable_funding_fee_amount_per_size: (T, T),
}

impl<T: Unsigned, const DECIMALS: u8> TestPosition<T, DECIMALS>
where
    T::Signed: fmt::Debug,
{
    /// Create a [`TestPositionOps`] for ops.
    pub fn ops<'a>(
        &'a mut self,
        market: &'a mut TestMarket<T, DECIMALS>,
    ) -> TestPositionOps<T, DECIMALS> {
        TestPositionOps {
            market,
            position: self,
        }
    }

    /// Create an empty long position.
    pub fn long(long_token_as_collateral: bool) -> Self
    where
        T: Default,
    {
        Self {
            is_long: true,
            is_collateral_token_long: long_token_as_collateral,
            ..Default::default()
        }
    }

    /// Create an empty short position.
    pub fn short(long_token_as_collateral: bool) -> Self
    where
        T: Default,
    {
        Self {
            is_long: false,
            is_collateral_token_long: long_token_as_collateral,
            ..Default::default()
        }
    }
}

/// Test Position.
#[derive(Debug)]
pub struct TestPositionOps<'a, T: Unsigned, const DECIMALS: u8>
where
    T::Signed: fmt::Debug,
{
    market: &'a mut TestMarket<T, DECIMALS>,
    position: &'a mut TestPosition<T, DECIMALS>,
}

impl<'a, T, const DECIMALS: u8> Position<DECIMALS> for TestPositionOps<'a, T, DECIMALS>
where
    T: CheckedSub + fmt::Display + FixedPointOps<DECIMALS>,
    T::Signed: Num + std::fmt::Debug,
{
    type Num = T;

    type Signed = T::Signed;

    type Market = TestMarket<T, DECIMALS>;

    fn market(&self) -> &Self::Market {
        self.market
    }

    fn market_mut(&mut self) -> &mut Self::Market {
        self.market
    }

    fn is_collateral_token_long(&self) -> bool {
        self.position.is_collateral_token_long
    }

    fn collateral_amount(&self) -> &Self::Num {
        &self.position.collateral_token_amount
    }

    fn collateral_amount_mut(&mut self) -> &mut Self::Num {
        &mut self.position.collateral_token_amount
    }

    fn size_in_usd(&self) -> &Self::Num {
        &self.position.size_in_usd
    }

    fn size_in_tokens(&self) -> &Self::Num {
        &self.position.size_in_tokens
    }

    fn size_in_usd_mut(&mut self) -> &mut Self::Num {
        &mut self.position.size_in_usd
    }

    fn size_in_tokens_mut(&mut self) -> &mut Self::Num {
        &mut self.position.size_in_tokens
    }

    fn is_long(&self) -> bool {
        self.position.is_long
    }

    fn increased(&mut self) -> crate::Result<()> {
        Ok(())
    }

    fn decreased(&mut self) -> crate::Result<()> {
        Ok(())
    }

    fn borrowing_factor(&self) -> &Self::Num {
        &self.position.borrowing_factor
    }

    fn borrowing_factor_mut(&mut self) -> &mut Self::Num {
        &mut self.position.borrowing_factor
    }

    fn funding_fee_amount_per_size(&self) -> &Self::Num {
        &self.position.funding_fee_amount_per_size
    }

    fn funding_fee_amount_per_size_mut(&mut self) -> &mut Self::Num {
        &mut self.position.funding_fee_amount_per_size
    }

    fn claimable_funding_fee_amount_per_size(&self, is_long_collateral: bool) -> &Self::Num {
        if is_long_collateral {
            &self.position.claimable_funding_fee_amount_per_size.0
        } else {
            &self.position.claimable_funding_fee_amount_per_size.1
        }
    }

    fn claimable_funding_fee_amount_per_size_mut(
        &mut self,
        is_long_collateral: bool,
    ) -> &mut Self::Num {
        if is_long_collateral {
            &mut self.position.claimable_funding_fee_amount_per_size.0
        } else {
            &mut self.position.claimable_funding_fee_amount_per_size.1
        }
    }
}
