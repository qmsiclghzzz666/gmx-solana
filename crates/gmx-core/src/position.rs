use std::ops::Deref;

use crate::{
    action::{decrease_position::DecreasePosition, increase_position::IncreasePosition, Prices},
    fixed::FixedPointOps,
    num::{MulDiv, Num, Unsigned, UnsignedAbs},
    Market,
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

        Ok(WillCollateralBeSufficient::Sufficient(
            remaining_collateral_value,
        ))
    }
}

impl<const DECIMALS: u8, P: Position<DECIMALS>> PositionExt<DECIMALS> for P {}

/// Collateral Delta Values.
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

impl<T> Deref for WillCollateralBeSufficient<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Sufficient(v) => v,
            Self::Insufficient(v) => v,
        }
    }
}
