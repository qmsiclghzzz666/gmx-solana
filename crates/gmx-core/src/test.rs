use std::fmt;

use crate::{
    fixed::FixedPointOps,
    market::Market,
    num::{MulDiv, Num, UnsignedAbs},
    params::{FeeParams, SwapImpactParams},
    pool::{Pool, PoolKind},
    position::Position,
};
use num_traits::{CheckedSub, Signed};

/// Test Pool.
#[derive(Debug, Default, Clone, Copy)]
pub struct TestPool<T> {
    long_token_amount: T,
    short_token_amount: T,
}

impl<T> Pool for TestPool<T>
where
    T: MulDiv + Num + CheckedSub,
{
    type Num = T;

    type Signed = T::Signed;

    fn long_token_amount(&self) -> crate::Result<Self::Num> {
        Ok(self.long_token_amount.clone())
    }

    fn short_token_amount(&self) -> crate::Result<Self::Num> {
        Ok(self.short_token_amount.clone())
    }

    fn apply_delta_to_long_token_amount(
        &mut self,
        delta: &Self::Signed,
    ) -> Result<(), crate::Error> {
        if delta.is_positive() {
            self.long_token_amount = self
                .long_token_amount
                .checked_add(&delta.unsigned_abs())
                .ok_or(crate::Error::Overflow)?;
        } else {
            self.long_token_amount = self
                .long_token_amount
                .checked_sub(&delta.unsigned_abs())
                .ok_or(crate::Error::Underflow)?;
        }
        Ok(())
    }

    fn apply_delta_to_short_token_amount(
        &mut self,
        delta: &Self::Signed,
    ) -> Result<(), crate::Error> {
        if delta.is_positive() {
            self.short_token_amount = self
                .short_token_amount
                .checked_add(&delta.unsigned_abs())
                .ok_or(crate::Error::Overflow)?;
        } else {
            self.short_token_amount = self
                .short_token_amount
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
    swap_impact_params: SwapImpactParams<T>,
    swap_fee_params: FeeParams<T>,
    primary: TestPool<T>,
    price_impact: TestPool<T>,
    fee: TestPool<T>,
}

impl Default for TestMarket<u64, 9> {
    fn default() -> Self {
        Self {
            total_supply: Default::default(),
            value_to_amount_divisor: 1,
            swap_impact_params: SwapImpactParams::builder()
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
            primary: Default::default(),
            price_impact: Default::default(),
            fee: Default::default(),
        }
    }
}

#[cfg(feature = "u128")]
impl Default for TestMarket<u128, 20> {
    fn default() -> Self {
        Self {
            total_supply: Default::default(),
            value_to_amount_divisor: 10u128.pow(20 - 9),
            swap_impact_params: SwapImpactParams::builder()
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
            primary: Default::default(),
            price_impact: Default::default(),
            fee: Default::default(),
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
            PoolKind::SwapImpact => &self.price_impact,
            PoolKind::ClaimableFee => &self.fee,
        };
        Ok(Some(pool))
    }

    fn pool_mut(&mut self, kind: PoolKind) -> crate::Result<Option<&mut Self::Pool>> {
        let pool = match kind {
            PoolKind::Primary => &mut self.primary,
            PoolKind::SwapImpact => &mut self.price_impact,
            PoolKind::ClaimableFee => &mut self.fee,
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

    fn swap_impact_params(&self) -> SwapImpactParams<Self::Num> {
        self.swap_impact_params.clone()
    }

    fn swap_fee_params(&self) -> crate::params::FeeParams<Self::Num> {
        self.swap_fee_params.clone()
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
    pub fn long() -> Self
    where
        T: Default,
    {
        Self {
            is_long: true,
            ..Default::default()
        }
    }

    /// Create an empty short position.
    pub fn short() -> Self
    where
        T: Default,
    {
        Self {
            is_long: false,
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

    fn market_mut(&mut self) -> &mut Self::Market {
        self.market
    }

    fn is_collateral_token_long(&self) -> bool {
        self.position.is_collateral_token_long
    }

    fn collateral_amount_mut(&mut self) -> &mut Self::Num {
        &mut self.position.collateral_token_amount
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
}
