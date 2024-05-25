use std::{fmt, ops::Deref};

use num_traits::{One, Zero};

use crate::{
    action::{
        decrease_position::DecreasePosition, increase_position::IncreasePosition,
        update_funding_state::unpack_to_funding_amount, Prices,
    },
    fixed::FixedPointOps,
    num::{MulDiv, Num, Unsigned, UnsignedAbs},
    params::fee::{FundingFees, PositionFees},
    BalanceExt, Market, MarketExt, Pool,
};

/// A position.
pub trait Position<const DECIMALS: u8> {
    /// Unsigned number type.
    type Num: MulDiv<Signed = Self::Signed> + FixedPointOps<DECIMALS>;

    /// Signed number type.
    type Signed: UnsignedAbs<Unsigned = Self::Num> + TryFrom<Self::Num> + Num;

    /// Market type.
    type Market: Market<DECIMALS, Num = Self::Num, Signed = Self::Signed>;

    /// Get a reference to the market.
    fn market(&self) -> &Self::Market;

    /// Get a mutable reference to the market.
    fn market_mut(&mut self) -> &mut Self::Market;

    /// Returns whether the collateral token is the long token of the market.
    fn is_collateral_token_long(&self) -> bool;

    /// Get the collateral amount.
    fn collateral_amount(&self) -> &Self::Num;

    /// Get a mutable reference to the collateral amount.
    fn collateral_amount_mut(&mut self) -> &mut Self::Num;

    /// Get a reference to the size (in USD) of the position.
    fn size_in_usd(&self) -> &Self::Num;

    /// Get a reference to the size (in tokens) of the position.
    fn size_in_tokens(&self) -> &Self::Num;

    /// Get a mutable reference to the size (in USD) of the position.
    fn size_in_usd_mut(&mut self) -> &mut Self::Num;

    /// Get a mutable reference to the size (in tokens) of the position.
    fn size_in_tokens_mut(&mut self) -> &mut Self::Num;

    /// Returns whether the position is a long position.
    fn is_long(&self) -> bool;

    /// Get a reference to last borrowing factor applied by the position.
    fn borrowing_factor(&self) -> &Self::Num;

    /// Get a mutable reference to last borrowing factor applied by the position.
    fn borrowing_factor_mut(&mut self) -> &mut Self::Num;

    /// Get a reference to the funding fee amount per size.
    fn funding_fee_amount_per_size(&self) -> &Self::Num;

    /// Get a mutable reference to the funding fee amount per size.
    fn funding_fee_amount_per_size_mut(&mut self) -> &mut Self::Num;

    /// Get a reference to claimable funding fee amount per size of the given collateral.
    fn claimable_funding_fee_amount_per_size(&self, is_long_collateral: bool) -> &Self::Num;

    /// Get a mutable reference to claimable funding fee amount per size of the given collateral.
    fn claimable_funding_fee_amount_per_size_mut(
        &mut self,
        is_long_collateral: bool,
    ) -> &mut Self::Num;

    /// Increased callback.
    fn increased(&mut self) -> crate::Result<()>;

    /// Decreased callback.
    fn decreased(&mut self) -> crate::Result<()>;
}

impl<'a, const DECIMALS: u8, P: Position<DECIMALS>> Position<DECIMALS> for &'a mut P {
    type Num = P::Num;

    type Signed = P::Signed;

    type Market = P::Market;

    fn market(&self) -> &Self::Market {
        (**self).market()
    }

    fn market_mut(&mut self) -> &mut Self::Market {
        (**self).market_mut()
    }

    fn is_collateral_token_long(&self) -> bool {
        (**self).is_collateral_token_long()
    }

    fn collateral_amount(&self) -> &Self::Num {
        (**self).collateral_amount()
    }

    fn collateral_amount_mut(&mut self) -> &mut Self::Num {
        (**self).collateral_amount_mut()
    }

    fn size_in_usd(&self) -> &Self::Num {
        (**self).size_in_usd()
    }

    fn size_in_tokens(&self) -> &Self::Num {
        (**self).size_in_tokens()
    }

    fn size_in_usd_mut(&mut self) -> &mut Self::Num {
        (**self).size_in_usd_mut()
    }

    fn size_in_tokens_mut(&mut self) -> &mut Self::Num {
        (**self).size_in_tokens_mut()
    }

    fn is_long(&self) -> bool {
        (**self).is_long()
    }

    fn borrowing_factor(&self) -> &Self::Num {
        (**self).borrowing_factor()
    }

    fn borrowing_factor_mut(&mut self) -> &mut Self::Num {
        (**self).borrowing_factor_mut()
    }

    fn funding_fee_amount_per_size(&self) -> &Self::Num {
        (**self).funding_fee_amount_per_size()
    }

    fn funding_fee_amount_per_size_mut(&mut self) -> &mut Self::Num {
        (**self).funding_fee_amount_per_size_mut()
    }

    fn claimable_funding_fee_amount_per_size(&self, is_long_collateral: bool) -> &Self::Num {
        (**self).claimable_funding_fee_amount_per_size(is_long_collateral)
    }

    fn claimable_funding_fee_amount_per_size_mut(
        &mut self,
        is_long_collateral: bool,
    ) -> &mut Self::Num {
        (**self).claimable_funding_fee_amount_per_size_mut(is_long_collateral)
    }

    fn increased(&mut self) -> crate::Result<()> {
        (**self).increased()
    }

    fn decreased(&mut self) -> crate::Result<()> {
        (**self).decreased()
    }
}

/// Extension trait for [`Position`] with utils.
pub trait PositionExt<const DECIMALS: u8>: Position<DECIMALS> {
    /// Create an action to increase the position.
    fn increase(
        &mut self,
        prices: Prices<Self::Num>,
        collateral_increment_amount: Self::Num,
        size_delta_usd: Self::Num,
        acceptable_price: Option<Self::Num>,
    ) -> crate::Result<IncreasePosition<&mut Self, DECIMALS>>
    where
        Self: Sized,
    {
        IncreasePosition::try_new(
            self,
            prices,
            collateral_increment_amount,
            size_delta_usd,
            acceptable_price,
        )
    }

    /// Create an action to decrease the position.
    fn decrease(
        &mut self,
        prices: Prices<Self::Num>,
        size_delta_usd: Self::Num,
        acceptable_price: Option<Self::Num>,
        collateral_withdrawal_amount: Self::Num,
        is_insolvent_close_allowed: bool,
        is_liquidation_order: bool,
    ) -> crate::Result<DecreasePosition<&mut Self, DECIMALS>>
    where
        Self: Sized,
    {
        DecreasePosition::try_new(
            self,
            prices,
            size_delta_usd,
            acceptable_price,
            collateral_withdrawal_amount,
            is_insolvent_close_allowed,
            is_liquidation_order,
        )
    }

    /// Calculate the pnl value when decreased by the given delta size.
    ///
    /// Returns `(pnl_value, uncapped_pnl_value, size_delta_in_tokens)`
    fn pnl_value(
        &self,
        prices: &Prices<Self::Num>,
        size_delta_usd: &Self::Num,
    ) -> crate::Result<(Self::Signed, Self::Signed, Self::Num)> {
        use num_traits::{CheckedMul, CheckedSub, Signed};

        // TODO: pick by position side.
        let execution_price = &prices.index_token_price;

        let position_value: Self::Signed = self
            .size_in_tokens()
            .checked_mul(execution_price)
            .ok_or(crate::Error::Computation(
                "overflow calculating position value",
            ))?
            .try_into()
            .map_err(|_| crate::Error::Convert)?;
        let size_in_usd = self
            .size_in_usd()
            .clone()
            .try_into()
            .map_err(|_| crate::Error::Convert)?;
        let total_pnl = if self.is_long() {
            position_value.checked_sub(&size_in_usd)
        } else {
            size_in_usd.checked_sub(&position_value)
        }
        .ok_or(crate::Error::Computation("calculating total pnl"))?;
        let uncapped_total_pnl = total_pnl.clone();

        if total_pnl.is_positive() {
            // TODO: cap `total_pnl`.
        }

        let size_delta_in_tokens = if *self.size_in_usd() == *size_delta_usd {
            self.size_in_tokens().clone()
        } else if self.is_long() {
            self.size_in_tokens()
                .checked_mul_div_ceil(size_delta_usd, self.size_in_usd())
                .ok_or(crate::Error::Computation(
                    "calculating size delta in tokens for long",
                ))?
        } else {
            self.size_in_tokens()
                .checked_mul_div(size_delta_usd, self.size_in_usd())
                .ok_or(crate::Error::Computation(
                    "calculating size delta in tokens for short",
                ))?
        };

        let pnl_usd = size_delta_in_tokens
            .checked_mul_div_with_signed_numberator(&total_pnl, self.size_in_tokens())
            .ok_or(crate::Error::Computation("calculating pnl_usd"))?;

        let uncapped_pnl_usd = size_delta_in_tokens
            .checked_mul_div_with_signed_numberator(&uncapped_total_pnl, self.size_in_tokens())
            .ok_or(crate::Error::Computation("calculating uncapped_pnl_usd"))?;

        Ok((pnl_usd, uncapped_pnl_usd, size_delta_in_tokens))
    }

    /// Check that whether the collateral will be sufficient after paying the given `realized_pnl` and applying `delta_size`.
    ///
    /// - Returns the remaining collateral value if sufficient, `None` otherwise.
    /// - Returns `Err` if failed to finish the calculation.
    fn will_collateral_be_sufficient(
        &self,
        prices: &Prices<Self::Num>,
        delta: &CollateralDelta<Self::Num>,
    ) -> crate::Result<WillCollateralBeSufficient<Self::Signed>> {
        use num_traits::{CheckedAdd, CheckedMul, Signed};

        let collateral_token_price = if self.is_collateral_token_long() {
            &prices.long_token_price
        } else {
            &prices.short_token_price
        };

        // TODO: pick min value of the price.
        let mut remaining_collateral_value = delta
            .next_collateral_amount
            .checked_mul(collateral_token_price)
            .ok_or(crate::Error::Computation(
                "overflow calculating collateral value",
            ))?
            .to_signed()?;

        if delta.realized_pnl_value.is_negative() {
            remaining_collateral_value = remaining_collateral_value
                .checked_add(&delta.realized_pnl_value)
                .ok_or(crate::Error::Computation("adding realized pnl"))?;
        }

        if remaining_collateral_value.is_negative() {
            return Ok(WillCollateralBeSufficient::Insufficient(
                remaining_collateral_value,
            ));
        }

        // TODO: check leverage.
        if remaining_collateral_value.is_positive() {
            Ok(WillCollateralBeSufficient::Sufficient(
                remaining_collateral_value,
            ))
        } else {
            Ok(WillCollateralBeSufficient::Insufficient(
                remaining_collateral_value,
            ))
        }
    }

    /// Validate the position state.
    fn validate_position(
        &self,
        prices: &Prices<Self::Num>,
        should_validate_min_position_size: bool,
        should_validate_min_collateral_usd: bool,
    ) -> crate::Result<()> {
        if self.size_in_usd().is_zero() || self.size_in_tokens().is_zero() {
            return Err(crate::Error::InvalidPosition(
                "size_in_usd or size_in_tokens is zero",
            ));
        }

        // TODO: validate market is enabled.

        if should_validate_min_position_size
            && self.size_in_usd() < self.market().position_params().min_position_size_usd()
        {
            return Err(crate::Error::InvalidPosition("size in usd too small"));
        }

        if let Some(reason) = self.check_liquidatable(prices, should_validate_min_collateral_usd)? {
            return Err(crate::Error::Liquidatable(reason));
        }

        Ok(())
    }

    /// Get collateral price.
    fn collateral_price<'a>(&self, prices: &'a Prices<Self::Num>) -> &'a Self::Num {
        if self.is_collateral_token_long() {
            &prices.long_token_price
        } else {
            &prices.short_token_price
        }
    }

    /// Get collateral value.
    fn collateral_value(&self, prices: &Prices<Self::Num>) -> crate::Result<Self::Num> {
        use num_traits::CheckedMul;

        let collateral_token_price = self.collateral_price(prices);

        let collateral_value = self
            .collateral_amount()
            .checked_mul(collateral_token_price)
            .ok_or(crate::Error::Computation(
                "overflow calculating collateral value",
            ))?;

        Ok(collateral_value)
    }

    /// Check if the position is liquidatable.
    ///
    /// Return [`LiquidatableReason`] if it is liquidatable, `None` otherwise.
    fn check_liquidatable(
        &self,
        prices: &Prices<Self::Num>,
        should_validate_min_collateral_usd: bool,
    ) -> crate::Result<Option<LiquidatableReason>> {
        use num_traits::{CheckedAdd, CheckedMul, CheckedSub, Signed};

        let (pnl, _, _) = self.pnl_value(prices, self.size_in_usd())?;

        let collateral_value = self.collateral_value(prices)?;

        let size_delta_usd = self.size_in_usd().to_opposite_signed()?;
        let mut price_impact_value = self.position_price_impact(&size_delta_usd)?;

        let _has_positive_impact = price_impact_value.is_positive();

        if price_impact_value.is_negative() {
            self.cap_negative_position_price_impact(
                &size_delta_usd,
                &mut price_impact_value,
                true,
            )?;
        } else {
            price_impact_value = Zero::zero();
        }

        // TODO: get position fees.
        let fees = PositionFees::<Self::Num>::default();

        let collateral_cost_value = fees
            .total_cost_amount()?
            .checked_mul(self.collateral_price(prices))
            .ok_or(crate::Error::Computation(
                "overflow calculating collateral cost value",
            ))?;

        let remaining_collateral_value = collateral_value
            .to_signed()?
            .checked_add(&pnl)
            .and_then(|v| {
                v.checked_add(&price_impact_value)?
                    .checked_sub(&collateral_cost_value.to_signed().ok()?)
            })
            .ok_or(crate::Error::Computation(
                "calculating remaining collateral value",
            ))?;

        let params = self.market().position_params();

        let min_collateral_usd_for_leverage =
            crate::utils::apply_factor(self.size_in_usd(), params.min_collateral_factor()).ok_or(
                crate::Error::Computation("calculating min collateral usd for leverage"),
            )?;

        if !remaining_collateral_value.is_positive() {
            return Ok(Some(LiquidatableReason::NotPositive));
        }

        let remaining_collateral_value = remaining_collateral_value.unsigned_abs();

        if should_validate_min_collateral_usd
            && remaining_collateral_value < *params.min_collateral_value()
        {
            return Ok(Some(LiquidatableReason::MinCollateral));
        }

        if remaining_collateral_value < min_collateral_usd_for_leverage {
            return Ok(Some(LiquidatableReason::MinCollateralForLeverage));
        }

        Ok(None)
    }

    /// Apply delta to open interest.
    fn apply_delta_to_open_interest(
        &mut self,
        size_delta_usd: &Self::Signed,
        size_delta_in_tokens: &Self::Signed,
    ) -> crate::Result<()> {
        if size_delta_usd.is_zero() {
            return Ok(());
        }
        let is_long_collateral = self.is_collateral_token_long();
        let is_long = self.is_long();
        let open_interest = self.market_mut().open_interest_pool_mut(is_long)?;
        if is_long_collateral {
            open_interest.apply_delta_to_long_amount(size_delta_usd)?;
        } else {
            open_interest.apply_delta_to_short_amount(size_delta_usd)?;
        }

        let open_interest_in_tokens = self
            .market_mut()
            .open_interest_in_tokens_pool_mut(is_long)?;
        if is_long_collateral {
            open_interest_in_tokens.apply_delta_to_long_amount(size_delta_in_tokens)?;
        } else {
            open_interest_in_tokens.apply_delta_to_short_amount(size_delta_in_tokens)?;
        }

        Ok(())
    }

    /// Get position price impact.
    fn position_price_impact(&self, size_delta_usd: &Self::Signed) -> crate::Result<Self::Signed> {
        // Since the amounts of open interest are already usd amounts,
        // the price should be `one`.
        let usd_price = One::one();
        let (delta_long_usd_value, delta_short_usd_value) = if self.is_long() {
            (size_delta_usd.clone(), Zero::zero())
        } else {
            (Zero::zero(), size_delta_usd.clone())
        };
        let price_impact_value = self
            .market()
            .open_interest()?
            .pool_delta_with_values(
                delta_long_usd_value,
                delta_short_usd_value,
                &usd_price,
                &usd_price,
            )?
            .price_impact(&self.market().position_impact_params())?;
        Ok(price_impact_value)
    }

    /// Cap positive position price impact.
    fn cap_positive_position_price_impact(
        &self,
        index_token_price: &Self::Num,
        size_delta_usd: &Self::Signed,
        impact: &mut Self::Signed,
    ) -> crate::Result<()> {
        use crate::utils;
        use num_traits::{CheckedMul, Signed};
        if impact.is_positive() {
            let impact_pool_amount = self.market().position_impact_pool_amount()?;
            // Cap price impact based on pool amount.
            // TODO: use min price.
            let max_impact = impact_pool_amount
                .checked_mul(index_token_price)
                .ok_or(crate::Error::Computation(
                    "overflow calculating max positive position impact based on pool amount",
                ))?
                .to_signed()?;
            if *impact > max_impact {
                *impact = max_impact;
            }

            // Cap price impact based on max factor.
            let params = self.market().position_params();
            let max_impact_factor = params.max_positive_position_impact_factor();
            let max_impact = utils::apply_factor(&size_delta_usd.unsigned_abs(), max_impact_factor)
                .ok_or(crate::Error::Computation(
                    "calculating max positive position impact based on max factor",
                ))?
                .to_signed()?;
            if *impact > max_impact {
                *impact = max_impact;
            }
        }
        Ok(())
    }

    /// Cap negative position price impact.
    fn cap_negative_position_price_impact(
        &self,
        size_delta_usd: &Self::Signed,
        impact: &mut Self::Signed,
        for_liquidations: bool,
    ) -> crate::Result<Self::Num> {
        use crate::utils;
        use num_traits::{CheckedSub, Signed};

        let mut impact_diff = Zero::zero();
        if impact.is_negative() {
            let params = self.market().position_params();
            let max_impact_factor = if for_liquidations {
                params.max_position_impact_factor_for_liquidations()
            } else {
                params.max_negative_position_impact_factor()
            };
            let min_impact = utils::apply_factor(&size_delta_usd.unsigned_abs(), max_impact_factor)
                .ok_or(crate::Error::Computation(
                    "calculating max negative position impact based on max factor",
                ))?
                .to_opposite_signed()?;
            if *impact < min_impact {
                impact_diff = min_impact
                    .checked_sub(impact)
                    .ok_or(crate::Error::Computation(
                        "overflow calculating impact diff",
                    ))?
                    .unsigned_abs();
                *impact = min_impact;
            }
        }
        Ok(impact_diff)
    }

    /// Get position price impact usd and cap the value if it is positive.
    #[inline]
    fn capped_positive_position_price_impact(
        &self,
        index_token_price: &Self::Num,
        size_delta_usd: &Self::Signed,
    ) -> crate::Result<Self::Signed> {
        let mut impact = self.position_price_impact(size_delta_usd)?;
        self.cap_positive_position_price_impact(index_token_price, size_delta_usd, &mut impact)?;
        Ok(impact)
    }

    /// Get capped position price impact usd.
    ///
    /// Compare to [`PositionExt::capped_positive_position_price_impact`],
    /// this method will also cap the negative impact and return the difference before capping.
    #[inline]
    fn capped_position_price_impact(
        &self,
        index_token_price: &Self::Num,
        size_delta_usd: &Self::Signed,
    ) -> crate::Result<(Self::Signed, Self::Num)> {
        let mut impact =
            self.capped_positive_position_price_impact(index_token_price, size_delta_usd)?;
        let impact_diff =
            self.cap_negative_position_price_impact(size_delta_usd, &mut impact, false)?;
        Ok((impact, impact_diff))
    }

    /// Get borrowing fee value.
    fn borrowing_fee_value(&self) -> crate::Result<Self::Num> {
        use crate::utils;
        use num_traits::CheckedSub;

        let latest_factor = self.market().borrowing_factor(self.is_long())?;
        let diff_factor = latest_factor
            .checked_sub(self.borrowing_factor())
            .ok_or(crate::Error::Computation("invalid latest borrowing factor"))?;
        utils::apply_factor(self.size_in_usd(), &diff_factor)
            .ok_or(crate::Error::Computation("calculating borrowing fee value"))
    }

    /// Get funding fees.
    fn funding_fees(&self) -> crate::Result<FundingFees<Self::Num>> {
        let adjustment = self.market().funding_amount_per_size_adjustment();
        let fees = FundingFees::builder()
            .amount(
                unpack_to_funding_amount(
                    &adjustment,
                    &self.market().funding_fee_amount_per_size(
                        self.is_long(),
                        self.is_collateral_token_long(),
                    )?,
                    self.funding_fee_amount_per_size(),
                    self.size_in_usd(),
                    true,
                )
                .ok_or(crate::Error::Computation("calculating funding fee amount"))?,
            )
            .claimable_long_token_amount(
                unpack_to_funding_amount(
                    &adjustment,
                    &self
                        .market()
                        .claimable_funding_fee_amount_per_size(self.is_long(), true)?,
                    self.claimable_funding_fee_amount_per_size(true),
                    self.size_in_usd(),
                    false,
                )
                .ok_or(crate::Error::Computation(
                    "calculating claimable long token funding fee amount",
                ))?,
            )
            .claimable_short_token_amount(
                unpack_to_funding_amount(
                    &adjustment,
                    &self
                        .market()
                        .claimable_funding_fee_amount_per_size(self.is_long(), false)?,
                    self.claimable_funding_fee_amount_per_size(false),
                    self.size_in_usd(),
                    false,
                )
                .ok_or(crate::Error::Computation(
                    "calculating claimable short token funding fee amount",
                ))?,
            )
            .build();
        Ok(fees)
    }

    /// Get position fees.
    fn position_fees(
        &self,
        collateral_token_price: &Self::Num,
        size_delta_usd: &Self::Num,
        is_positive_impact: bool,
    ) -> crate::Result<PositionFees<Self::Num>> {
        debug_assert!(!collateral_token_price.is_zero(), "must be non-zero");
        let fees = self
            .market()
            .order_fee_params()
            .base_position_fees(collateral_token_price, size_delta_usd, is_positive_impact)?
            .apply_borrowing_fee(collateral_token_price, self.borrowing_fee_value()?)?
            .apply_funding_fees(self.funding_fees()?);
        Ok(fees)
    }
}

impl<const DECIMALS: u8, P: Position<DECIMALS>> PositionExt<DECIMALS> for P {}

/// Collateral Delta Values.
#[allow(unused)]
pub struct CollateralDelta<T: Unsigned> {
    next_size_in_usd: T,
    next_collateral_amount: T,
    realized_pnl_value: T::Signed,
    open_interest_delta: T::Signed,
}

impl<T: Unsigned> CollateralDelta<T> {
    /// Create a new collateral delta.
    pub fn new(
        next_size_in_usd: T,
        next_collateral_amount: T,
        realized_pnl_value: T::Signed,
        open_interest_delta: T::Signed,
    ) -> Self {
        Self {
            next_size_in_usd,
            next_collateral_amount,
            realized_pnl_value,
            open_interest_delta,
        }
    }
}

/// Will collateral be sufficient.
#[derive(Clone, Copy)]
pub enum WillCollateralBeSufficient<T> {
    /// Will be sufficient.
    Sufficient(T),
    /// Won't be sufficient.
    Insufficient(T),
}

impl<T> WillCollateralBeSufficient<T> {
    /// Returns whether it is sufficient.
    pub fn is_sufficient(&self) -> bool {
        matches!(self, Self::Sufficient(_))
    }
}

impl<T> Deref for WillCollateralBeSufficient<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Sufficient(v) => v,
            Self::Insufficient(v) => v,
        }
    }
}

/// Liquidatable reason.
#[derive(Debug, Clone, Copy)]
pub enum LiquidatableReason {
    /// Min collateral.
    MinCollateral,
    /// Remaining collateral not positive.
    NotPositive,
    /// Min collateral for leverage.
    MinCollateralForLeverage,
}

impl fmt::Display for LiquidatableReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MinCollateral => write!(f, "min colltateral"),
            Self::NotPositive => write!(f, "<= 0"),
            Self::MinCollateralForLeverage => write!(f, "min collateral for leverage"),
        }
    }
}
