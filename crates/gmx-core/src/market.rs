use crate::{
    action::{
        deposit::Deposit, distribute_position_impact::DistributePositionImpact, swap::Swap,
        update_borrowing_state::UpdateBorrowingState, update_funding_state::UpdateFundingState,
        withdraw::Withdrawal, Prices,
    },
    clock::ClockKind,
    fixed::FixedPointOps,
    num::{MulDiv, Num, UnsignedAbs},
    params::{
        fee::{BorrowingFeeParams, FundingFeeParams},
        position::{PositionImpactDistributionParams, PositionParams},
        FeeParams, PriceImpactParams,
    },
    pool::{balance::Merged, Balance, BalanceExt, Pool, PoolKind},
    PoolExt,
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

    /// Get the just passed time in seconds for the given kind of clock.
    fn just_passed_in_seconds(&mut self, clock: ClockKind) -> crate::Result<u64>;

    /// USD value to market token amount divisor.
    ///
    /// One should make sure it is non-zero.
    fn usd_to_amount_divisor(&self) -> Self::Num;

    /// Adjustment factor for packing funding amount per size.
    fn funding_amount_per_size_adjustment(&self) -> Self::Num;

    /// Get the swap impact params.
    fn swap_impact_params(&self) -> PriceImpactParams<Self::Num>;

    /// Get the swap fee params.
    fn swap_fee_params(&self) -> FeeParams<Self::Num>;

    /// Get basic market params.
    fn position_params(&self) -> PositionParams<Self::Num>;

    /// Get the position impact params.
    fn position_impact_params(&self) -> PriceImpactParams<Self::Num>;

    /// Get the order fee params.
    fn order_fee_params(&self) -> FeeParams<Self::Num>;

    /// Get position impact distribution params.
    fn position_impact_distribution_params(&self) -> PositionImpactDistributionParams<Self::Num>;

    /// Get borrowing fee params.
    fn borrowing_fee_params(&self) -> BorrowingFeeParams<Self::Num>;

    /// Get funding fee params.
    fn funding_fee_params(&self) -> FundingFeeParams<Self::Num>;

    /// Get funding factor per second.
    fn funding_factor_per_second(&self) -> &Self::Signed;

    /// Get the mutable reference to funding factor per second.
    fn funding_factor_per_second_mut(&mut self) -> &mut Self::Signed;

    /// Get reserve factor.
    fn reserve_factor(&self) -> &Self::Num;

    /// Get open interest reserve factor.
    fn open_interest_reserve_factor(&self) -> &Self::Num;

    /// Get max pnl factor.
    fn max_pnl_factor(&self, kind: PnlFactorKind, is_long: bool) -> crate::Result<Self::Num>;

    /// Get max pool amount.
    fn max_pool_amount(&self, is_long_token: bool) -> crate::Result<Self::Num>;

    /// Get max pool value for deposit.
    fn max_pool_value_for_deposit(&self, is_long_token: bool) -> crate::Result<Self::Num>;

    /// Get max open interest.
    fn max_open_interest(&self, is_long: bool) -> crate::Result<Self::Num>;
}

/// Pnl Factor Kind.
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum PnlFactorKind {
    /// For deposit.
    Deposit,
    /// For withdrawal.
    Withdrawal,
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

    fn just_passed_in_seconds(&mut self, clock: ClockKind) -> crate::Result<u64> {
        (**self).just_passed_in_seconds(clock)
    }

    fn usd_to_amount_divisor(&self) -> Self::Num {
        (**self).usd_to_amount_divisor()
    }

    fn funding_amount_per_size_adjustment(&self) -> Self::Num {
        (**self).funding_amount_per_size_adjustment()
    }

    fn swap_impact_params(&self) -> PriceImpactParams<Self::Num> {
        (**self).swap_impact_params()
    }

    fn swap_fee_params(&self) -> FeeParams<Self::Num> {
        (**self).swap_fee_params()
    }

    fn position_params(&self) -> PositionParams<Self::Num> {
        (**self).position_params()
    }

    fn position_impact_params(&self) -> PriceImpactParams<Self::Num> {
        (**self).position_impact_params()
    }

    fn order_fee_params(&self) -> FeeParams<Self::Num> {
        (**self).order_fee_params()
    }

    fn position_impact_distribution_params(&self) -> PositionImpactDistributionParams<Self::Num> {
        (**self).position_impact_distribution_params()
    }

    fn borrowing_fee_params(&self) -> BorrowingFeeParams<Self::Num> {
        (**self).borrowing_fee_params()
    }

    fn funding_fee_params(&self) -> FundingFeeParams<Self::Num> {
        (**self).funding_fee_params()
    }

    fn funding_factor_per_second(&self) -> &Self::Signed {
        (**self).funding_factor_per_second()
    }

    fn funding_factor_per_second_mut(&mut self) -> &mut Self::Signed {
        (**self).funding_factor_per_second_mut()
    }

    fn reserve_factor(&self) -> &Self::Num {
        (**self).reserve_factor()
    }

    fn open_interest_reserve_factor(&self) -> &Self::Num {
        (**self).open_interest_reserve_factor()
    }

    fn max_pnl_factor(&self, kind: PnlFactorKind, is_long: bool) -> crate::Result<Self::Num> {
        (**self).max_pnl_factor(kind, is_long)
    }

    fn max_pool_amount(&self, is_long_token: bool) -> crate::Result<Self::Num> {
        (**self).max_pool_amount(is_long_token)
    }

    fn max_pool_value_for_deposit(&self, is_long_token: bool) -> crate::Result<Self::Num> {
        (**self).max_pool_value_for_deposit(is_long_token)
    }

    fn max_open_interest(&self, is_long: bool) -> crate::Result<Self::Num> {
        (**self).max_open_interest(is_long)
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
        self.pool(PoolKind::SwapImpact)?
            .ok_or(crate::Error::MissingPoolKind(PoolKind::SwapImpact))
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

    /// Get the reference of open interest pool.
    #[inline]
    fn open_interest_pool(&self, is_long: bool) -> crate::Result<&Self::Pool> {
        let kind = if is_long {
            PoolKind::OpenInterestForLong
        } else {
            PoolKind::OpenInterestForShort
        };
        self.pool(kind)?.ok_or(crate::Error::MissingPoolKind(kind))
    }

    /// Get the reference of open interest pool.
    #[inline]
    fn open_interest_in_tokens_pool(&self, is_long: bool) -> crate::Result<&Self::Pool> {
        let kind = if is_long {
            PoolKind::OpenInterestInTokensForLong
        } else {
            PoolKind::OpenInterestInTokensForShort
        };
        self.pool(kind)?.ok_or(crate::Error::MissingPoolKind(kind))
    }

    /// Get mutable reference of open interest pool.
    #[inline]
    fn open_interest_pool_mut(&mut self, is_long: bool) -> crate::Result<&mut Self::Pool> {
        let kind = if is_long {
            PoolKind::OpenInterestForLong
        } else {
            PoolKind::OpenInterestForShort
        };
        self.pool_mut(kind)?
            .ok_or(crate::Error::MissingPoolKind(kind))
    }

    /// Get mutable reference of open interest pool.
    #[inline]
    fn open_interest_in_tokens_pool_mut(
        &mut self,
        is_long: bool,
    ) -> crate::Result<&mut Self::Pool> {
        let kind = if is_long {
            PoolKind::OpenInterestInTokensForLong
        } else {
            PoolKind::OpenInterestInTokensForShort
        };
        self.pool_mut(kind)?
            .ok_or(crate::Error::MissingPoolKind(kind))
    }

    /// Get position impact pool.
    #[inline]
    fn position_impact_pool(&self) -> crate::Result<&Self::Pool> {
        self.pool(PoolKind::PositionImpact)?
            .ok_or(crate::Error::MissingPoolKind(PoolKind::PositionImpact))
    }

    /// Get the mutable reference to position impact pool.
    #[inline]
    fn position_impact_pool_mut(&mut self) -> crate::Result<&mut Self::Pool> {
        self.pool_mut(PoolKind::PositionImpact)?
            .ok_or(crate::Error::MissingPoolKind(PoolKind::PositionImpact))
    }

    /// Get position impact pool amount.
    #[inline]
    fn position_impact_pool_amount(&self) -> crate::Result<Self::Num> {
        self.position_impact_pool()?.long_amount()
    }

    /// Get borrowing factor pool.
    #[inline]
    fn borrowing_factor_pool(&self) -> crate::Result<&Self::Pool> {
        self.pool(PoolKind::BorrowingFactor)?
            .ok_or(crate::Error::MissingPoolKind(PoolKind::BorrowingFactor))
    }

    /// Get the mutable reference to borrowing factor pool.
    #[inline]
    fn borrowing_factor_pool_mut(&mut self) -> crate::Result<&mut Self::Pool> {
        self.pool_mut(PoolKind::BorrowingFactor)?
            .ok_or(crate::Error::MissingPoolKind(PoolKind::BorrowingFactor))
    }

    /// Get a reference to funding amount per size pool.
    #[inline]
    fn funding_amount_per_size_pool(&self, is_long: bool) -> crate::Result<&Self::Pool> {
        let kind = if is_long {
            PoolKind::FundingAmountPerSizeForLong
        } else {
            PoolKind::FundingAmountPerSizeForShort
        };
        self.pool(kind)?.ok_or(crate::Error::MissingPoolKind(kind))
    }

    /// Get a reference to claimable funding amount per size pool.
    #[inline]
    fn claimable_funding_amount_per_size_pool(&self, is_long: bool) -> crate::Result<&Self::Pool> {
        let kind = if is_long {
            PoolKind::ClaimableFundingAmountPerSizeForLong
        } else {
            PoolKind::ClaimableFundingAmountPerSizeForShort
        };
        self.pool(kind)?.ok_or(crate::Error::MissingPoolKind(kind))
    }

    /// Get the mutable reference to funding amount per size pool.
    #[inline]
    fn funding_amount_per_size_pool_mut(
        &mut self,
        is_long: bool,
    ) -> crate::Result<&mut Self::Pool> {
        let kind = if is_long {
            PoolKind::FundingAmountPerSizeForLong
        } else {
            PoolKind::FundingAmountPerSizeForShort
        };
        self.pool_mut(kind)?
            .ok_or(crate::Error::MissingPoolKind(kind))
    }

    /// Get the mutable reference to claimable funding amount per size pool.
    #[inline]
    fn claimable_funding_amount_per_size_pool_mut(
        &mut self,
        is_long: bool,
    ) -> crate::Result<&mut Self::Pool> {
        let kind = if is_long {
            PoolKind::ClaimableFundingAmountPerSizeForLong
        } else {
            PoolKind::ClaimableFundingAmountPerSizeForShort
        };
        self.pool_mut(kind)?
            .ok_or(crate::Error::MissingPoolKind(kind))
    }

    /// Get the usd value of primary pool for one side.
    #[inline]
    fn pool_value_for_one_side(
        &self,
        prices: &Prices<Self::Num>,
        is_long: bool,
        _maximize: bool,
    ) -> crate::Result<Self::Num> {
        // TODO: apply maximize by choosing price.
        if is_long {
            self.primary_pool()?
                .long_usd_value(&prices.long_token_price)
        } else {
            self.primary_pool()?
                .short_usd_value(&prices.short_token_price)
        }
    }

    /// Get the usd value of primary pool.
    fn pool_value(
        &self,
        long_token_price: &Self::Num,
        short_token_price: &Self::Num,
    ) -> crate::Result<Self::Num> {
        let long_value = self.primary_pool()?.long_usd_value(long_token_price)?;
        let short_value = self.primary_pool()?.short_usd_value(short_token_price)?;
        long_value
            .checked_add(&short_value)
            .ok_or(crate::Error::Overflow)
    }

    /// Get total open interest as a [`Balance`].
    fn open_interest(&self) -> crate::Result<Merged<&Self::Pool, &Self::Pool>> {
        Ok(self
            .open_interest_pool(true)?
            .merge(self.open_interest_pool(false)?))
    }

    /// Get total open interest in tokens as a merged [`Balance`].
    ///
    /// The long amount is the total long open interest in tokens,
    /// while the short amount is the total short open interest in tokens.
    fn open_interest_in_tokens(&self) -> crate::Result<Merged<&Self::Pool, &Self::Pool>> {
        Ok(self
            .open_interest_in_tokens_pool(true)?
            .merge(self.open_interest_in_tokens_pool(false)?))
    }

    /// Create a [`Deposit`] action.
    fn deposit(
        &mut self,
        long_token_amount: Self::Num,
        short_token_amount: Self::Num,
        prices: Prices<Self::Num>,
    ) -> Result<Deposit<&mut Self, DECIMALS>, crate::Error>
    where
        Self: Sized,
    {
        Deposit::try_new(self, long_token_amount, short_token_amount, prices)
    }

    /// Create a [`Withdrawal`].
    fn withdraw(
        &mut self,
        market_token_amount: Self::Num,
        prices: Prices<Self::Num>,
    ) -> crate::Result<Withdrawal<&mut Self, DECIMALS>>
    where
        Self: Sized,
    {
        Withdrawal::try_new(self, market_token_amount, prices)
    }

    /// Create a [`Swap`].
    fn swap(
        &mut self,
        is_token_in_long: bool,
        token_in_amount: Self::Num,
        prices: Prices<Self::Num>,
    ) -> crate::Result<Swap<&mut Self, DECIMALS>>
    where
        Self: Sized,
    {
        Swap::try_new(self, is_token_in_long, token_in_amount, prices)
    }

    /// Create a [`DistributePositionImpact`] action.
    fn distribute_position_impact(
        &mut self,
    ) -> crate::Result<DistributePositionImpact<&mut Self, DECIMALS>>
    where
        Self: Sized,
    {
        Ok(DistributePositionImpact::from(self))
    }

    /// Create a [`UpdateBorrowingState`] action.
    fn update_borrowing(
        &mut self,
        prices: &Prices<Self::Num>,
    ) -> crate::Result<UpdateBorrowingState<&mut Self, DECIMALS>>
    where
        Self: Sized,
    {
        UpdateBorrowingState::try_new(self, prices)
    }

    /// Create a [`UpdateFundingState`] action.
    fn update_funding(
        &mut self,
        prices: &Prices<Self::Num>,
    ) -> crate::Result<UpdateFundingState<&mut Self, DECIMALS>>
    where
        Self: Sized,
    {
        UpdateFundingState::try_new(self, prices)
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
                self.pool(PoolKind::SwapImpact)?
                    .ok_or(crate::Error::MissingPoolKind(PoolKind::SwapImpact))?
                    .long_amount()?
            } else {
                self.pool(PoolKind::SwapImpact)?
                    .ok_or(crate::Error::MissingPoolKind(PoolKind::SwapImpact))?
                    .short_amount()?
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
                .ok_or(crate::Error::Underflow)?
                + One::one())
                / price;
            Ok(amount)
        } else {
            Ok(Zero::zero())
        }
    }

    /// Get pending position impact pool distribution amount.
    fn pending_position_impact_pool_distribution_amount(
        &self,
        duration_in_secs: u64,
    ) -> crate::Result<(Self::Num, Self::Num)> {
        use crate::utils;
        use num_traits::FromPrimitive;

        let current_amount = self.position_impact_pool_amount()?;
        let params = self.position_impact_distribution_params();
        if params.distribute_factor().is_zero()
            || current_amount <= *params.min_position_impact_pool_amount()
        {
            return Ok((Zero::zero(), current_amount));
        }
        let max_distribution_amount = current_amount
            .checked_sub(params.min_position_impact_pool_amount())
            .ok_or(crate::Error::Computation(
                "calculating max distribution amount",
            ))?;

        let duration_value = Self::Num::from_u64(duration_in_secs).ok_or(crate::Error::Convert)?;
        let mut distribution_amount =
            utils::apply_factor(&duration_value, params.distribute_factor())
                .ok_or(crate::Error::Computation("calculating distribution amount"))?;
        if distribution_amount > max_distribution_amount {
            distribution_amount = max_distribution_amount;
        }
        let next_amount =
            current_amount
                .checked_sub(&distribution_amount)
                .ok_or(crate::Error::Computation(
                    "calculating next position impact amount",
                ))?;
        Ok((distribution_amount, next_amount))
    }

    /// Get reseved value.
    fn reserved(&self, is_long: bool, index_token_price: &Self::Num) -> crate::Result<Self::Num> {
        use num_traits::CheckedMul;

        if is_long {
            let amount = self.open_interest_in_tokens()?.amount(is_long)?;
            // TODO: use max price.
            amount
                .checked_mul(index_token_price)
                .ok_or(crate::Error::Computation("calculating reserved value"))
        } else {
            self.open_interest()?.amount(is_long)
        }
    }

    /// Get borrowing factor per second.
    fn calc_borrowing_factor_per_second(
        &self,
        is_long: bool,
        prices: &Prices<Self::Num>,
    ) -> crate::Result<Self::Num> {
        use crate::utils;

        let reserved_value = self.reserved(is_long, &prices.index_token_price)?;

        if reserved_value.is_zero() {
            return Ok(Zero::zero());
        }

        let params = self.borrowing_fee_params();

        if params.skip_borrowing_fee_for_smaller_side() {
            let open_interest = self.open_interest()?;
            let long_interest = open_interest.long_amount()?;
            let short_interest = open_interest.short_amount()?;
            if (is_long && long_interest < short_interest)
                || (!is_long && short_interest < long_interest)
            {
                return Ok(Zero::zero());
            }
        }

        let pool_value = self.pool_value(&prices.long_token_price, &prices.short_token_price)?;

        if pool_value.is_zero() {
            return Err(crate::Error::UnableToGetBorrowingFactorEmptyPoolValue);
        }

        // TODO: apply optimal usage factor.

        let reserved_value_after_exponent =
            utils::apply_exponent_factor(reserved_value, params.exponent(is_long).clone()).ok_or(
                crate::Error::Computation("calculating reserved value after exponent"),
            )?;
        let reversed_value_to_pool_factor =
            utils::div_to_factor(&reserved_value_after_exponent, &pool_value, false).ok_or(
                crate::Error::Computation("calculating reserved value to pool factor"),
            )?;
        utils::apply_factor(&reversed_value_to_pool_factor, params.factor(is_long)).ok_or(
            crate::Error::Computation("calculating borrowing factort per second"),
        )
    }

    /// Get next cumulative borrowing factor of the given side.
    fn calc_next_cumulative_borrowing_factor(
        &self,
        is_long: bool,
        prices: &Prices<Self::Num>,
        duration_in_second: u64,
    ) -> crate::Result<(Self::Num, Self::Num)> {
        use num_traits::{CheckedMul, FromPrimitive};

        let borrowing_factor_per_second = self.calc_borrowing_factor_per_second(is_long, prices)?;
        let current_factor = self.borrowing_factor(is_long)?;

        let duration_value =
            Self::Num::from_u64(duration_in_second).ok_or(crate::Error::Convert)?;
        let delta = borrowing_factor_per_second
            .checked_mul(&duration_value)
            .ok_or(crate::Error::Computation(
                "calculating borrowing factor delta",
            ))?;
        let next_cumulative_borrowing_factor =
            current_factor
                .checked_add(&delta)
                .ok_or(crate::Error::Computation(
                    "calculating next borrowing factor",
                ))?;
        Ok((next_cumulative_borrowing_factor, delta))
    }

    /// Get current borrowing factor.
    #[inline]
    fn borrowing_factor(&self, is_long: bool) -> crate::Result<Self::Num> {
        self.borrowing_factor_pool()?.amount(is_long)
    }

    /// Get current funding fee amount per size.
    #[inline]
    fn funding_fee_amount_per_size(
        &self,
        is_long: bool,
        is_long_collateral: bool,
    ) -> crate::Result<Self::Num> {
        self.funding_amount_per_size_pool(is_long)?
            .amount(is_long_collateral)
    }

    /// Get current claimable funding fee amount per size.
    #[inline]
    fn claimable_funding_fee_amount_per_size(
        &self,
        is_long: bool,
        is_long_collateral: bool,
    ) -> crate::Result<Self::Num> {
        self.claimable_funding_amount_per_size_pool(is_long)?
            .amount(is_long_collateral)
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
            self.pool_mut(PoolKind::SwapImpact)?
                .ok_or(crate::Error::MissingPoolKind(PoolKind::SwapImpact))?
                .apply_delta_to_long_amount(&-delta.clone())?;
        } else {
            self.pool_mut(PoolKind::SwapImpact)?
                .ok_or(crate::Error::MissingPoolKind(PoolKind::SwapImpact))?
                .apply_delta_to_short_amount(&-delta.clone())?;
        }
        Ok(delta.unsigned_abs())
    }

    /// Apply delta to the primary pool.
    fn apply_delta(&mut self, is_long_token: bool, delta: &Self::Signed) -> crate::Result<()> {
        if is_long_token {
            self.pool_mut(PoolKind::Primary)?
                .ok_or(crate::Error::MissingPoolKind(PoolKind::Primary))?
                .apply_delta_to_long_amount(delta)?;
        } else {
            self.pool_mut(PoolKind::Primary)?
                .ok_or(crate::Error::MissingPoolKind(PoolKind::Primary))?
                .apply_delta_to_short_amount(delta)?;
        }
        Ok(())
    }

    /// Apply delta to claimable fee pool.
    fn apply_delta_to_claimable_fee_pool(
        &mut self,
        is_long_token: bool,
        delta: &Self::Signed,
    ) -> crate::Result<()> {
        self.claimable_fee_pool_mut()?
            .apply_delta_amount(is_long_token, delta)?;
        Ok(())
    }

    /// Apply delta to the position impact pool.
    fn apply_delta_to_position_impact_pool(&mut self, delta: &Self::Signed) -> crate::Result<()> {
        self.position_impact_pool_mut()?
            .apply_delta_to_long_amount(delta)
    }

    /// Apply delta to borrowing factor.
    fn apply_delta_to_borrowing_factor(
        &mut self,
        is_long: bool,
        delta: &Self::Signed,
    ) -> crate::Result<()> {
        self.borrowing_factor_pool_mut()?
            .apply_delta_amount(is_long, delta)
    }

    /// Apply delta to funding amount per size.
    fn apply_delta_to_funding_amount_per_size(
        &mut self,
        is_long: bool,
        is_long_collateral: bool,
        delta: &Self::Signed,
    ) -> crate::Result<()> {
        self.funding_amount_per_size_pool_mut(is_long)?
            .apply_delta_amount(is_long_collateral, delta)
    }

    /// Apply delta to claimable funding amount per size.
    fn apply_delta_to_claimable_funding_amount_per_size(
        &mut self,
        is_long: bool,
        is_long_collateral: bool,
        delta: &Self::Signed,
    ) -> crate::Result<()> {
        self.claimable_funding_amount_per_size_pool_mut(is_long)?
            .apply_delta_amount(is_long_collateral, delta)
    }

    /// Get reserved value.
    fn reserved_value(
        &self,
        index_token_price: &Self::Num,
        is_long: bool,
    ) -> crate::Result<Self::Num> {
        // TODO: add comment to explain the difference.
        if is_long {
            // TODO: use max price.
            self.open_interest_in_tokens()?
                .long_usd_value(index_token_price)
        } else {
            self.open_interest()?.short_amount()
        }
    }

    /// Validate reserve.
    fn validate_reserve(&self, prices: &Prices<Self::Num>, is_long: bool) -> crate::Result<()> {
        let pool_value = self.pool_value_for_one_side(prices, is_long, false)?;

        let max_reserved_value = crate::utils::apply_factor(&pool_value, self.reserve_factor())
            .ok_or(crate::Error::Computation("calculating max reserved value"))?;

        let reserved_value = self.reserved_value(&prices.index_token_price, is_long)?;

        if reserved_value > max_reserved_value {
            Err(crate::Error::InsufficientReserve)
        } else {
            Ok(())
        }
    }

    /// Validate open interest reserve.
    fn validate_open_interest_reserve(
        &self,
        prices: &Prices<Self::Num>,
        is_long: bool,
    ) -> crate::Result<()> {
        let pool_value = self.pool_value_for_one_side(prices, is_long, false)?;

        let max_reserved_value =
            crate::utils::apply_factor(&pool_value, self.open_interest_reserve_factor())
                .ok_or(crate::Error::Computation("calculating max reserved value"))?;

        let reserved_value = self.reserved_value(&prices.index_token_price, is_long)?;

        if reserved_value > max_reserved_value {
            Err(crate::Error::InsufficientReserveForOpenInterest)
        } else {
            Ok(())
        }
    }

    /// Get total pnl of the market for one side.
    fn pnl(
        &self,
        index_token_price: &Self::Num,
        is_long: bool,
        _maximize: bool,
    ) -> crate::Result<Self::Signed> {
        use crate::num::Unsigned;
        use num_traits::CheckedMul;

        let open_interest = self.open_interest()?.amount(is_long)?;
        let open_interest_in_tokens = self.open_interest_in_tokens()?.amount(is_long)?;
        if open_interest.is_zero() && open_interest_in_tokens.is_zero() {
            return Ok(Zero::zero());
        }

        // TODO: pick price according to the `maximize` flag.
        let price = index_token_price;

        let open_interest_value = open_interest_in_tokens
            .checked_mul(price)
            .ok_or(crate::Error::Computation("calculating open interest value"))?;

        if is_long {
            open_interest_value
                .to_signed()?
                .checked_sub(&open_interest.to_signed()?)
                .ok_or(crate::Error::Computation("calculating pnl for long"))
        } else {
            open_interest
                .to_signed()?
                .checked_sub(&open_interest_value.to_signed()?)
                .ok_or(crate::Error::Computation("calculating pnl for short"))
        }
    }

    /// Get pnl factor.
    fn pnl_factor(
        &self,
        prices: &Prices<Self::Num>,
        is_long: bool,
        maximize: bool,
    ) -> crate::Result<Self::Signed> {
        let pool_value = self.pool_value_for_one_side(prices, is_long, !maximize)?;
        let pnl = self.pnl(&prices.index_token_price, is_long, maximize)?;
        crate::utils::div_to_factor_signed(&pnl, &pool_value)
            .ok_or(crate::Error::Computation("calculating pnl factor"))
    }

    /// Validate pnl factor.
    fn validate_pnl_factor(
        &self,
        prices: &Prices<Self::Num>,
        kind: PnlFactorKind,
        is_long: bool,
    ) -> crate::Result<()> {
        let pnl_factor = self.pnl_factor(prices, is_long, true)?;
        let max_pnl_factor = self.max_pnl_factor(kind, is_long)?;

        if pnl_factor.is_positive() && pnl_factor.unsigned_abs() > max_pnl_factor {
            Err(crate::Error::PnlFactorExceeded(
                kind,
                get_msg_by_side(is_long),
            ))
        } else {
            Ok(())
        }
    }

    /// Validate max pnl.
    fn validate_max_pnl(
        &self,
        prices: &Prices<Self::Num>,
        long_kind: PnlFactorKind,
        short_kind: PnlFactorKind,
    ) -> crate::Result<()> {
        self.validate_pnl_factor(prices, long_kind, true)?;
        self.validate_pnl_factor(prices, short_kind, false)?;
        Ok(())
    }

    /// Validate (primary) pool value for deposit.
    fn validate_pool_value_for_deposit(
        &self,
        prices: &Prices<Self::Num>,
        is_long_token: bool,
    ) -> crate::Result<()> {
        let pool_value = self.pool_value_for_one_side(prices, is_long_token, true)?;
        let max_pool_value = self.max_pool_value_for_deposit(is_long_token)?;
        if pool_value > max_pool_value {
            Err(crate::Error::MaxPoolAmountExceeded(get_msg_by_side(
                is_long_token,
            )))
        } else {
            Ok(())
        }
    }

    /// Validate (primary) pool amount.
    fn validate_pool_amount(&self, is_long_token: bool) -> crate::Result<()> {
        let amount = self.primary_pool()?.amount(is_long_token)?;
        let max_pool_amount = self.max_pool_amount(is_long_token)?;
        if amount > max_pool_amount {
            Err(crate::Error::MaxPoolAmountExceeded(get_msg_by_side(
                is_long_token,
            )))
        } else {
            Ok(())
        }
    }
}

impl<const DECIMALS: u8, M: Market<DECIMALS>> MarketExt<DECIMALS> for M {}

#[inline]
fn get_msg_by_side(is_long: bool) -> &'static str {
    if is_long {
        "for long"
    } else {
        "for short"
    }
}
