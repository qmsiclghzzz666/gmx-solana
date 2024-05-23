use std::fmt;

use crate::{
    fixed::FixedPointOps,
    market::Market,
    num::{MulDiv, Num, UnsignedAbs},
    params::{FeeParams, PositionParams, PriceImpactParams},
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

/// Test Market.
#[derive(Debug, Clone)]
pub struct TestMarket<T, const DECIMALS: u8> {
    total_supply: T,
    value_to_amount_divisor: T,
    swap_impact_params: PriceImpactParams<T>,
    swap_fee_params: FeeParams<T>,
    position_params: PositionParams<T>,
    position_impact_params: PriceImpactParams<T>,
    order_fee_params: FeeParams<T>,
    primary: TestPool<T>,
    swap_impact: TestPool<T>,
    fee: TestPool<T>,
    open_interest: (TestPool<T>, TestPool<T>),
    open_interest_in_tokens: (TestPool<T>, TestPool<T>),
    position_impact: TestPool<T>,
}

impl Default for TestMarket<u64, 9> {
    fn default() -> Self {
        Self {
            total_supply: Default::default(),
            value_to_amount_divisor: 1,
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
            primary: Default::default(),
            swap_impact: Default::default(),
            fee: Default::default(),
            open_interest: Default::default(),
            open_interest_in_tokens: Default::default(),
            position_impact: Default::default(),
        }
    }
}

#[cfg(feature = "u128")]
impl Default for TestMarket<u128, 20> {
    fn default() -> Self {
        Self {
            total_supply: Default::default(),
            value_to_amount_divisor: 10u128.pow(20 - 9),
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
            primary: Default::default(),
            swap_impact: Default::default(),
            fee: Default::default(),
            open_interest: Default::default(),
            open_interest_in_tokens: Default::default(),
            position_impact: Default::default(),
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
            PoolKind::OpenInterestForLongCollateral => &self.open_interest.0,
            PoolKind::OpenInterestForShortCollateral => &self.open_interest.1,
            PoolKind::OpenInterestInTokensForLongCollateral => &self.open_interest_in_tokens.0,
            PoolKind::OpenInterestInTokensForShortCollateral => &self.open_interest_in_tokens.1,
            PoolKind::PositionImpact => &self.position_impact,
        };
        Ok(Some(pool))
    }

    fn pool_mut(&mut self, kind: PoolKind) -> crate::Result<Option<&mut Self::Pool>> {
        let pool = match kind {
            PoolKind::Primary => &mut self.primary,
            PoolKind::SwapImpact => &mut self.swap_impact,
            PoolKind::ClaimableFee => &mut self.fee,
            PoolKind::OpenInterestForLongCollateral => &mut self.open_interest.0,
            PoolKind::OpenInterestForShortCollateral => &mut self.open_interest.1,
            PoolKind::OpenInterestInTokensForLongCollateral => &mut self.open_interest_in_tokens.0,
            PoolKind::OpenInterestInTokensForShortCollateral => &mut self.open_interest_in_tokens.1,
            PoolKind::PositionImpact => &mut self.position_impact,
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

    fn usd_to_amount_divisor(&self) -> Self::Num {
        self.value_to_amount_divisor.clone()
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
}

/// Test Position
#[derive(Debug, Clone, Copy, Default)]
pub struct TestPosition<T, const DECIMALS: u8> {
    is_long: bool,
    is_collateral_token_long: bool,
    collateral_token_amount: T,
    size_in_usd: T,
    size_in_tokens: T,
}

impl<T, const DECIMALS: u8> TestPosition<T, DECIMALS> {
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
pub struct TestPositionOps<'a, T, const DECIMALS: u8> {
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
}
