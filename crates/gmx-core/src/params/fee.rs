use num_traits::Zero;

use crate::{fixed::FixedPointOps, utils};

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
        let factor = self.factor(is_positive_impact);
        let fee_amount = utils::apply_factor(amount, factor)?;
        let fee_receiver_amount = utils::apply_factor(&fee_amount, &self.fee_receiver_factor)?;
        let fees = Fees {
            fee_amount_for_pool: fee_amount.checked_sub(&fee_receiver_amount)?,
            fee_receiver_amount,
        };
        Some((amount.checked_sub(&fee_amount)?, fees))
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
