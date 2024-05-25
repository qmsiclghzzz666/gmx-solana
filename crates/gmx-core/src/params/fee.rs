use num_traits::{CheckedAdd, Zero};
use typed_builder::TypedBuilder;

use crate::{fixed::FixedPointOps, num::Unsigned, utils};

/// Fee Parameters.
#[derive(Debug, Clone, Copy)]
pub struct FeeParams<T> {
    positive_impact_fee_factor: T,
    negative_impact_fee_factor: T,
    fee_receiver_factor: T,
}

impl<T> FeeParams<T> {
    /// Builder for [`FeeParams`].
    pub fn builder() -> Builder<T>
    where
        T: Zero,
    {
        Builder {
            positive_impact_factor: Zero::zero(),
            negative_impact_factor: Zero::zero(),
            fee_receiver_factor: Zero::zero(),
        }
    }
}

/// Borrowing Fee Parameters.
#[derive(Debug, Clone, Copy, TypedBuilder)]
pub struct BorrowingFeeParams<T> {
    exponent_for_long: T,
    exponent_for_short: T,
    factor_for_long: T,
    factor_for_short: T,
    #[builder(default = true)]
    skip_borrowing_fee_for_smaller_side: bool,
}

impl<T> BorrowingFeeParams<T> {
    /// Get borrowing exponent factor.
    pub fn exponent(&self, is_long: bool) -> &T {
        if is_long {
            &self.exponent_for_long
        } else {
            &self.exponent_for_short
        }
    }

    /// Get borrowing factor.
    pub fn factor(&self, is_long: bool) -> &T {
        if is_long {
            &self.factor_for_long
        } else {
            &self.factor_for_short
        }
    }

    /// Get whether to skip borrowing fee for smaller side.
    pub fn skip_borrowing_fee_for_smaller_side(&self) -> bool {
        self.skip_borrowing_fee_for_smaller_side
    }
}

/// Funding Fee Parameters.
#[derive(Debug, Clone, Copy, TypedBuilder)]
pub struct FundingFeeParams<T> {
    exponent: T,
    funding_factor: T,
    increase_factor_per_second: T,
    decrease_factor_per_second: T,
    max_factor_per_second: T,
    min_factor_per_second: T,
    threshold_for_stable_funding: T,
    threshold_for_decrease_funding: T,
}

impl<T> FundingFeeParams<T> {
    /// Get funding exponent factor.
    pub fn exponent(&self) -> &T {
        &self.exponent
    }

    /// Get funding increase factor per second.
    pub fn increase_factor_per_second(&self) -> &T {
        &self.increase_factor_per_second
    }

    /// Get funding decrease factor per second.
    pub fn decrease_factor_per_second(&self) -> &T {
        &self.decrease_factor_per_second
    }

    /// Get max funding factor per second.
    pub fn max_factor_per_second(&self) -> &T {
        &self.max_factor_per_second
    }

    /// Get min funding factor per second.
    pub fn min_factor_per_second(&self) -> &T {
        &self.min_factor_per_second
    }

    /// Fallback funding factor.
    pub fn factor(&self) -> &T {
        &self.funding_factor
    }

    /// Threshold for stable funding.
    pub fn threshold_for_stable_funding(&self) -> &T {
        &self.threshold_for_stable_funding
    }

    /// Threshold for decrease funding.
    pub fn threshold_for_decrease_funding(&self) -> &T {
        &self.threshold_for_decrease_funding
    }

    /// Get change type for next funding rate.
    pub fn change(
        &self,
        funding_factor_per_second: &T::Signed,
        long_open_interest: &T,
        short_open_interest: &T,
        diff_factor: &T,
    ) -> FundingRateChangeType
    where
        T: Ord + Unsigned,
    {
        use num_traits::Signed;

        let is_skew_the_same_direction_as_funding = (funding_factor_per_second.is_positive()
            && *long_open_interest > *short_open_interest)
            || (funding_factor_per_second.is_negative()
                && *long_open_interest < *short_open_interest);

        if is_skew_the_same_direction_as_funding {
            if *diff_factor > self.threshold_for_stable_funding {
                FundingRateChangeType::Increase
            } else if *diff_factor < self.threshold_for_decrease_funding {
                FundingRateChangeType::Decrease
            } else {
                FundingRateChangeType::NoChange
            }
        } else {
            FundingRateChangeType::Increase
        }
    }
}

/// Funding Rate Change Type.
#[derive(Default, Debug)]
pub enum FundingRateChangeType {
    /// No Change.
    #[default]
    NoChange,
    /// Increase.
    Increase,
    /// Decrease.
    Decrease,
}

/// Fees.
#[derive(Debug, Clone, Copy)]
pub struct Fees<T> {
    fee_receiver_amount: T,
    fee_amount_for_pool: T,
}

impl<T: Zero> Default for Fees<T> {
    fn default() -> Self {
        Self {
            fee_receiver_amount: Zero::zero(),
            fee_amount_for_pool: Zero::zero(),
        }
    }
}

impl<T> Fees<T> {
    /// Create a new [`Fees`].
    pub fn new(pool: T, receiver: T) -> Self {
        Self {
            fee_amount_for_pool: pool,
            fee_receiver_amount: receiver,
        }
    }

    /// Get fee receiver amount.
    pub fn fee_receiver_amount(&self) -> &T {
        &self.fee_receiver_amount
    }

    /// Get fee amount for pool.
    pub fn fee_amount_for_pool(&self) -> &T {
        &self.fee_amount_for_pool
    }
}

impl<T> FeeParams<T> {
    #[inline]
    fn factor(&self, is_positive_impact: bool) -> &T {
        if is_positive_impact {
            &self.positive_impact_fee_factor
        } else {
            &self.negative_impact_fee_factor
        }
    }

    /// Get basic fee.
    #[inline]
    pub fn fee<const DECIMALS: u8>(&self, is_positive_impact: bool, amount: &T) -> Option<T>
    where
        T: FixedPointOps<DECIMALS>,
    {
        let factor = self.factor(is_positive_impact);
        utils::apply_factor(amount, factor)
    }

    /// Get receiver fee.
    #[inline]
    pub fn receiver_fee<const DECIMALS: u8>(&self, fee_amount: &T) -> Option<T>
    where
        T: FixedPointOps<DECIMALS>,
    {
        utils::apply_factor(fee_amount, &self.fee_receiver_factor)
    }

    /// Apply fees to `amount`.
    /// - `DECIMALS` is the decimals of the parameters.
    ///
    /// Returns `None` if the computation fails, otherwise `amount` after fees and the fees are returned.
    pub fn apply_fees<const DECIMALS: u8>(
        &self,
        is_positive_impact: bool,
        amount: &T,
    ) -> Option<(T, Fees<T>)>
    where
        T: FixedPointOps<DECIMALS>,
    {
        let fee_amount = self.fee(is_positive_impact, amount)?;
        let fee_receiver_amount = self.receiver_fee(&fee_amount)?;
        let fees = Fees {
            fee_amount_for_pool: fee_amount.checked_sub(&fee_receiver_amount)?,
            fee_receiver_amount,
        };
        Some((amount.checked_sub(&fee_amount)?, fees))
    }

    /// Get order fees.
    fn order_fees<const DECIMALS: u8>(
        &self,
        collateral_token_price: &T,
        size_delta_usd: &T,
        is_positive_impact: bool,
    ) -> crate::Result<OrderFees<T>>
    where
        T: FixedPointOps<DECIMALS>,
    {
        if collateral_token_price.is_zero() {
            return Err(crate::Error::InvalidPrices);
        }

        // TODO: use min price.
        let fee_amount = self
            .fee(is_positive_impact, size_delta_usd)
            .ok_or(crate::Error::Computation("calculating order fee usd"))?
            / collateral_token_price.clone();

        // TODO: apply rebase.

        let receiver_fee_amount = self
            .receiver_fee(&fee_amount)
            .ok_or(crate::Error::Computation("calculating order receiver fee"))?;
        Ok(OrderFees {
            base: Fees::new(
                fee_amount
                    .checked_sub(&receiver_fee_amount)
                    .ok_or(crate::Error::Computation("calculating order fee for pool"))?,
                receiver_fee_amount,
            ),
        })
    }

    /// Get base position fees.
    pub fn base_position_fees<const DECIMALS: u8>(
        &self,
        collateral_token_price: &T,
        size_delta_usd: &T,
        is_positive_impact: bool,
    ) -> crate::Result<PositionFees<T>>
    where
        T: FixedPointOps<DECIMALS>,
    {
        let OrderFees { base } =
            self.order_fees(collateral_token_price, size_delta_usd, is_positive_impact)?;
        Ok(PositionFees {
            base,
            borrowing: Default::default(),
        })
    }
}

/// Order Fees.
pub struct OrderFees<T> {
    base: Fees<T>,
}

/// Borrowing Fee.
#[derive(Debug, Clone, Copy)]
pub struct BorrowingFee<T> {
    amount: T,
}

impl<T> BorrowingFee<T> {
    /// Get borrowing fee amount.
    pub fn amount(&self) -> &T {
        &self.amount
    }
}

impl<T: Zero> Default for BorrowingFee<T> {
    fn default() -> Self {
        Self {
            amount: Zero::zero(),
        }
    }
}

/// Position Fees.
#[derive(Debug, Clone, Copy)]
pub struct PositionFees<T> {
    base: Fees<T>,
    borrowing: BorrowingFee<T>,
}

impl<T> PositionFees<T> {
    /// Get fee for receiver.
    pub fn for_receiver(&self) -> &T {
        &self.base.fee_receiver_amount
    }

    /// Get fee for pool.
    pub fn for_pool<const DECIMALS: u8>(&self) -> crate::Result<T>
    where
        T: FixedPointOps<DECIMALS>,
    {
        let amount = self
            .base
            .fee_amount_for_pool
            .checked_add(&self.borrowing.amount)
            .ok_or(crate::Error::Computation("calculating fee for pool"))?;
        Ok(amount)
    }

    /// Get borrowing fee.
    pub fn borrowing(&self) -> &BorrowingFee<T> {
        &self.borrowing
    }

    /// Get total cost amount in collateral tokens.
    pub fn total_cost_amount(&self) -> crate::Result<T>
    where
        T: CheckedAdd,
    {
        self.total_cost_excluding_funding()
    }

    /// Get total cost excluding funding fee.
    pub fn total_cost_excluding_funding(&self) -> crate::Result<T>
    where
        T: CheckedAdd,
    {
        self.base
            .fee_amount_for_pool
            .checked_add(&self.base.fee_receiver_amount)
            .and_then(|acc| acc.checked_add(&self.borrowing.amount))
            .ok_or(crate::Error::Overflow)
    }

    /// Clear fees excluding funding fee.
    pub fn clear_fees_excluding_funding(&mut self)
    where
        T: Zero,
    {
        self.base = Fees::default();
        self.borrowing = BorrowingFee::default();
    }

    /// Apply borrowing fee.
    pub fn apply_borrowing_fee<const DECIMALS: u8>(
        mut self,
        price: &T,
        value: T,
    ) -> crate::Result<Self>
    where
        T: FixedPointOps<DECIMALS>,
    {
        debug_assert!(!price.is_zero(), "must be non-zero");
        let amount = value / price.clone();
        self.borrowing.amount = amount;
        Ok(self)
    }
}

impl<T: Zero> Default for PositionFees<T> {
    fn default() -> Self {
        Self {
            base: Default::default(),
            borrowing: Default::default(),
        }
    }
}

/// Builder for [`FeeParams`].
pub struct Builder<T> {
    positive_impact_factor: T,
    negative_impact_factor: T,
    fee_receiver_factor: T,
}

impl<T> Builder<T> {
    /// Set the fee factor for positive impact.
    pub fn with_positive_impact_fee_factor(mut self, factor: T) -> Self {
        self.positive_impact_factor = factor;
        self
    }

    /// Set the fee factor for negative impact.
    pub fn with_negative_impact_fee_factor(mut self, factor: T) -> Self {
        self.negative_impact_factor = factor;
        self
    }

    /// Set the fee receiver factor.
    pub fn with_fee_receiver_factor(mut self, factor: T) -> Self {
        self.fee_receiver_factor = factor;
        self
    }

    /// Build [`FeeParams`].
    pub fn build(self) -> FeeParams<T> {
        FeeParams {
            positive_impact_fee_factor: self.positive_impact_factor,
            negative_impact_fee_factor: self.negative_impact_factor,
            fee_receiver_factor: self.fee_receiver_factor,
        }
    }
}
