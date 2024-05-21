use num_traits::{CheckedAdd, CheckedMul, CheckedSub};

use crate::{
    fixed::FixedPointOps, num::Unsigned, params::PriceImpactParams, utils, Balance, BalanceExt,
};

/// Usd values of pool.
pub struct PoolValue<T> {
    long_token_usd_value: T,
    short_token_usd_value: T,
}

impl<T> PoolValue<T> {
    /// Get long token usd value.
    pub fn long_value(&self) -> &T {
        &self.long_token_usd_value
    }

    /// Get short token usd value.
    pub fn short_value(&self) -> &T {
        &self.short_token_usd_value
    }
}

impl<T: Unsigned + Clone> PoolValue<T> {
    /// Get usd value (abs) difference.
    #[inline]
    pub fn diff_value(&self) -> T {
        self.long_token_usd_value
            .clone()
            .diff(self.short_token_usd_value.clone())
    }
}

impl<T: Unsigned> PoolValue<T> {
    /// Create a new [`PoolValue`] from the given pool and prices.
    pub fn try_new<P>(pool: &P, long_token_price: &T, short_token_price: &T) -> crate::Result<Self>
    where
        P: Balance<Num = T, Signed = T::Signed> + ?Sized,
    {
        let long_token_usd_value = pool.long_usd_value(long_token_price)?;
        let short_token_usd_value = pool.short_usd_value(short_token_price)?;
        Ok(Self {
            long_token_usd_value,
            short_token_usd_value,
        })
    }
}

/// Delta of pool usd values.
pub struct PoolDelta<T: Unsigned> {
    current: PoolValue<T>,
    next: PoolValue<T>,
    delta: PoolValue<T::Signed>,
}

impl<T: Unsigned> PoolDelta<T> {
    /// Create a new [`PoolDelta`].
    pub fn try_new<P>(
        pool: &P,
        delta_long_token_usd_value: T::Signed,
        delta_short_token_usd_value: T::Signed,
        long_token_price: &T,
        short_token_price: &T,
    ) -> crate::Result<Self>
    where
        T: CheckedAdd + CheckedSub + CheckedMul,
        P: Balance<Num = T, Signed = T::Signed> + ?Sized,
    {
        let current = PoolValue::try_new(pool, long_token_price, short_token_price)?;

        let next = PoolValue {
            long_token_usd_value: current
                .long_token_usd_value
                .checked_add_with_signed(&delta_long_token_usd_value)
                .ok_or(crate::Error::Computation("next delta long usd value"))?,
            short_token_usd_value: current
                .short_token_usd_value
                .checked_add_with_signed(&delta_short_token_usd_value)
                .ok_or(crate::Error::Computation("next delta short usd value"))?,
        };

        let delta = PoolValue {
            long_token_usd_value: delta_long_token_usd_value,
            short_token_usd_value: delta_short_token_usd_value,
        };

        Ok(Self {
            current,
            next,
            delta,
        })
    }

    /// Create a new [`PoolDelta`].
    pub fn try_from_delta_amounts<P>(
        pool: &P,
        long_delta_amount: &T::Signed,
        short_delta_amount: &T::Signed,
        long_token_price: &T,
        short_token_price: &T,
    ) -> crate::Result<Self>
    where
        T: CheckedAdd + CheckedSub + CheckedMul,
        P: Balance<Num = T, Signed = T::Signed> + ?Sized,
    {
        let delta_long_token_usd_value = long_token_price
            .checked_mul_with_signed(long_delta_amount)
            .ok_or(crate::Error::Computation("delta long token usd value"))?;
        let delta_short_token_usd_value = short_token_price
            .checked_mul_with_signed(short_delta_amount)
            .ok_or(crate::Error::Computation("delta short token usd value"))?;
        Self::try_new(
            pool,
            delta_long_token_usd_value,
            delta_short_token_usd_value,
            long_token_price,
            short_token_price,
        )
    }

    /// Get delta values.
    pub fn delta(&self) -> &PoolValue<T::Signed> {
        &self.delta
    }
}

impl<T: Unsigned + Clone + Ord> PoolDelta<T> {
    /// Initial diff usd value.
    #[inline]
    pub fn initial_diff_value(&self) -> T {
        self.current.diff_value()
    }

    /// Next diff usd value.
    #[inline]
    pub fn next_diff_value(&self) -> T {
        self.next.diff_value()
    }

    /// Returns whether it is a same side rebalance.
    #[inline]
    pub fn is_same_side_rebalance(&self) -> bool {
        (self.current.long_token_usd_value <= self.current.short_token_usd_value)
            == (self.next.long_token_usd_value <= self.next.short_token_usd_value)
    }

    /// Calculate price impact.
    pub fn price_impact<const DECIMALS: u8>(
        &self,
        params: &PriceImpactParams<T>,
    ) -> crate::Result<T::Signed>
    where
        T: FixedPointOps<DECIMALS>,
    {
        if self.is_same_side_rebalance() {
            self.price_impact_for_same_side_rebalance(params)
        } else {
            self.price_impact_for_cross_over_rebalance(params)
        }
    }

    #[inline]
    fn price_impact_for_same_side_rebalance<const DECIMALS: u8>(
        &self,
        params: &PriceImpactParams<T>,
    ) -> crate::Result<T::Signed>
    where
        T: FixedPointOps<DECIMALS>,
    {
        let initial = self.initial_diff_value();
        let next = self.next_diff_value();
        let has_positive_impact = next < initial;
        let (positive_factor, negative_factor) = params.adjusted_factors();

        let factor = if has_positive_impact {
            positive_factor
        } else {
            negative_factor
        };
        let exponent_factor = params.exponent();

        let initial = utils::apply_factors(initial, factor.clone(), exponent_factor.clone())?;
        let next = utils::apply_factors(next, factor.clone(), exponent_factor.clone())?;
        let delta: T::Signed = initial
            .diff(next)
            .try_into()
            .map_err(|_| crate::Error::Convert)?;
        Ok(if has_positive_impact { delta } else { -delta })
    }

    #[inline]
    fn price_impact_for_cross_over_rebalance<const DECIMALS: u8>(
        &self,
        params: &PriceImpactParams<T>,
    ) -> crate::Result<T::Signed>
    where
        T: FixedPointOps<DECIMALS>,
    {
        let initial = self.initial_diff_value();
        let next = self.next_diff_value();
        let (positive_factor, negative_factor) = params.adjusted_factors();
        let exponent_factor = params.exponent();
        let positive_impact =
            utils::apply_factors(initial, positive_factor.clone(), exponent_factor.clone())?;
        let negative_impact =
            utils::apply_factors(next, negative_factor.clone(), exponent_factor.clone())?;
        let has_positive_impact = positive_impact > negative_impact;
        let delta: T::Signed = positive_impact
            .diff(negative_impact)
            .try_into()
            .map_err(|_| crate::Error::Convert)?;
        Ok(if has_positive_impact { delta } else { -delta })
    }
}
