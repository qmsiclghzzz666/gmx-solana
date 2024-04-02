use crate::{
    action::{deposit::Deposit, swap::Swap, withdraw::Withdrawal},
    fixed::FixedPointOps,
    num::{MulDiv, Num, UnsignedAbs},
    params::{FeeParams, SwapImpactParams},
    pool::{Pool, PoolExt, PoolKind},
};
use num_traits::{CheckedAdd, CheckedSub, One, Signed, Zero};

/// A GMX Market.
///
/// - The constant generic `DECIMALS` is the number of decimals of USD values.
pub trait Market<const DECIMALS: u8> {
    /// Unsigned number type used in the market.
    type Num: MulDiv<Signed = Self::Signed> + FixedPointOps<DECIMALS>;

    /// Signed number type used in the market.
    type Signed: UnsignedAbs<Unsigned = Self::Num> + TryFrom<Self::Num> + Num;

    /// Pool type.
    type Pool: Pool<Num = Self::Num, Signed = Self::Signed>;

    /// Get the reference to the pool of the given kind.
    fn pool(&self, kind: PoolKind) -> crate::Result<Option<&Self::Pool>>;

    /// Get the mutable reference to the pool of the given kind.
    fn pool_mut(&mut self, kind: PoolKind) -> crate::Result<Option<&mut Self::Pool>>;

    /// Get total supply of the market token.
    fn total_supply(&self) -> Self::Num;

    /// Perform mint.
    fn mint(&mut self, amount: &Self::Num) -> Result<(), crate::Error>;

    /// Perform burn.
    fn burn(&mut self, amount: &Self::Num) -> crate::Result<()>;

    /// USD value to market token amount divisor.
    ///
    /// One should make sure it is non-zero.
    fn usd_to_amount_divisor(&self) -> Self::Num;

    /// Get the swap impact params.
    fn swap_impact_params(&self) -> SwapImpactParams<Self::Num>;

    /// Get the swap fee params.
    fn swap_fee_params(&self) -> FeeParams<Self::Num>;
}

impl<'a, const DECIMALS: u8, M: Market<DECIMALS>> Market<DECIMALS> for &'a mut M {
    type Num = M::Num;

    type Signed = M::Signed;

    type Pool = M::Pool;

    fn pool(&self, kind: PoolKind) -> crate::Result<Option<&Self::Pool>> {
        (**self).pool(kind)
    }

    fn pool_mut(&mut self, kind: PoolKind) -> crate::Result<Option<&mut Self::Pool>> {
        (**self).pool_mut(kind)
    }

    fn total_supply(&self) -> Self::Num {
        (**self).total_supply()
    }

    fn mint(&mut self, amount: &Self::Num) -> Result<(), crate::Error> {
        (**self).mint(amount)
    }

    fn burn(&mut self, amount: &Self::Num) -> crate::Result<()> {
        (**self).burn(amount)
    }

    fn usd_to_amount_divisor(&self) -> Self::Num {
        (**self).usd_to_amount_divisor()
    }

    fn swap_impact_params(&self) -> SwapImpactParams<Self::Num> {
        (**self).swap_impact_params()
    }

    fn swap_fee_params(&self) -> FeeParams<Self::Num> {
        (**self).swap_fee_params()
    }
}

/// Extension trait for [`Market`] with utils.
pub trait MarketExt<const DECIMALS: u8>: Market<DECIMALS> {
    /// Unit USD value used in the market, i.e. the fixed-point deciamls amount of `one` USD,
    /// not the amount unit of market token.
    fn unit(&self) -> Self::Num {
        Self::Num::UNIT
    }

    /// Get the primary pool.
    #[inline]
    fn primary_pool(&self) -> crate::Result<&Self::Pool> {
        self.pool(PoolKind::Primary)?
            .ok_or(crate::Error::MissingPoolKind(PoolKind::Primary))
    }

    /// Get the swap impact pool.
    #[inline]
    fn swap_impact_pool(&self) -> crate::Result<&Self::Pool> {
        self.pool(PoolKind::PriceImpact)?
            .ok_or(crate::Error::MissingPoolKind(PoolKind::PriceImpact))
    }

    /// Get the claimable fee pool.
    #[inline]
    fn claimable_fee_pool(&self) -> crate::Result<&Self::Pool> {
        self.pool(PoolKind::ClaimableFee)?
            .ok_or(crate::Error::MissingPoolKind(PoolKind::ClaimableFee))
    }

    /// Get the mutable reference of the claimable fee pool.
    #[inline]
    fn claimable_fee_pool_mut(&mut self) -> crate::Result<&mut Self::Pool> {
        self.pool_mut(PoolKind::ClaimableFee)?
            .ok_or(crate::Error::MissingPoolKind(PoolKind::ClaimableFee))
    }

    /// Get the usd value of primary pool.
    fn pool_value(
        &self,
        long_token_price: &Self::Num,
        short_token_price: &Self::Num,
    ) -> crate::Result<Self::Num> {
        let long_value = self
            .primary_pool()?
            .long_token_usd_value(long_token_price)?;
        let short_value = self
            .primary_pool()?
            .short_token_usd_value(short_token_price)?;
        long_value
            .checked_add(&short_value)
            .ok_or(crate::Error::Computation)
    }

    /// Create a [`Deposit`] action.
    fn deposit(
        &mut self,
        long_token_amount: Self::Num,
        short_token_amount: Self::Num,
        long_token_price: Self::Num,
        short_token_price: Self::Num,
    ) -> Result<Deposit<&mut Self, DECIMALS>, crate::Error>
    where
        Self: Sized,
    {
        Deposit::try_new(
            self,
            long_token_amount,
            short_token_amount,
            long_token_price,
            short_token_price,
        )
    }

    /// Create a [`Withdrawal`].
    fn withdraw(
        &mut self,
        market_token_amount: Self::Num,
        long_token_price: Self::Num,
        short_token_price: Self::Num,
    ) -> crate::Result<Withdrawal<&mut Self, DECIMALS>>
    where
        Self: Sized,
    {
        Withdrawal::try_new(
            self,
            market_token_amount,
            long_token_price,
            short_token_price,
        )
    }

    /// Create a [`Swap`].
    fn swap(
        &mut self,
        is_token_in_long: bool,
        token_in_amount: Self::Num,
        long_token_price: Self::Num,
        short_token_price: Self::Num,
    ) -> crate::Result<Swap<&mut Self, DECIMALS>>
    where
        Self: Sized,
    {
        Swap::try_new(
            self,
            is_token_in_long,
            token_in_amount,
            long_token_price,
            short_token_price,
        )
    }

    /// Get the swap impact amount with cap.
    fn swap_impact_amount_with_cap(
        &self,
        is_long_token: bool,
        price: &Self::Num,
        usd_impact: &Self::Signed,
    ) -> crate::Result<Self::Signed> {
        if price.is_zero() {
            return Err(crate::Error::DividedByZero);
        }
        if usd_impact.is_positive() {
            let mut amount = usd_impact.clone()
                / price
                    .clone()
                    .try_into()
                    .map_err(|_| crate::Error::Convert)?;
            let max_amount = if is_long_token {
                self.pool(PoolKind::PriceImpact)?
                    .ok_or(crate::Error::MissingPoolKind(PoolKind::PriceImpact))?
                    .long_token_amount()?
            } else {
                self.pool(PoolKind::PriceImpact)?
                    .ok_or(crate::Error::MissingPoolKind(PoolKind::PriceImpact))?
                    .short_token_amount()?
            };
            if amount.unsigned_abs() > max_amount {
                amount = max_amount.try_into().map_err(|_| crate::Error::Convert)?;
            }
            Ok(amount)
        } else if usd_impact.is_negative() {
            let price: Self::Signed = price
                .clone()
                .try_into()
                .map_err(|_| crate::Error::Convert)?;
            // Round up div.
            let amount = (usd_impact
                .checked_sub(&price)
                .ok_or(crate::Error::Computation)?
                + One::one())
                / price;
            Ok(amount)
        } else {
            Ok(Zero::zero())
        }
    }

    /// Apply a swap impact value to the price impact pool.
    ///
    /// - If it is a positive impact amount, cap the impact amount to the amount available in the price impact pool,
    /// and the price impact pool will be decreased by this amount and return.
    /// - If it is a negative impact amount, the price impact pool will be increased by this amount and return.
    fn apply_swap_impact_value_with_cap(
        &mut self,
        is_long_token: bool,
        price: &Self::Num,
        usd_impact: &Self::Signed,
    ) -> crate::Result<Self::Num> {
        let delta = self.swap_impact_amount_with_cap(is_long_token, price, usd_impact)?;
        if is_long_token {
            self.pool_mut(PoolKind::PriceImpact)?
                .ok_or(crate::Error::MissingPoolKind(PoolKind::PriceImpact))?
                .apply_delta_to_long_token_amount(&-delta.clone())?;
        } else {
            self.pool_mut(PoolKind::PriceImpact)?
                .ok_or(crate::Error::MissingPoolKind(PoolKind::PriceImpact))?
                .apply_delta_to_short_token_amount(&-delta.clone())?;
        }
        Ok(delta.unsigned_abs())
    }

    /// Apply delta to the pool.
    fn apply_delta(&mut self, is_long_token: bool, delta: &Self::Signed) -> crate::Result<()> {
        if is_long_token {
            self.pool_mut(PoolKind::Primary)?
                .ok_or(crate::Error::MissingPoolKind(PoolKind::Primary))?
                .apply_delta_to_long_token_amount(delta)?;
        } else {
            self.pool_mut(PoolKind::Primary)?
                .ok_or(crate::Error::MissingPoolKind(PoolKind::Primary))?
                .apply_delta_to_short_token_amount(delta)?;
        }
        Ok(())
    }
}

impl<const DECIMALS: u8, M: Market<DECIMALS>> MarketExt<DECIMALS> for M {}
