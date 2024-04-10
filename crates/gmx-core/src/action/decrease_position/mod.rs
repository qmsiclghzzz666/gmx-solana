use num_traits::{CheckedAdd, CheckedMul, CheckedSub, Signed, Zero};
use std::fmt;

use crate::{
    num::{MulDiv, Unsigned, UnsignedAbs},
    params::fee::PositionFees,
    position::Position,
};

use self::collateral_processor::{CollateralProcessor, ProcessReport};

use super::Prices;

mod collateral_processor;
mod debt;

/// Decrease the position.
#[must_use]
pub struct DecreasePosition<P: Position<DECIMALS>, const DECIMALS: u8> {
    position: P,
    params: DecreasePositionParams<P::Num>,
    withdrawable_collateral_amount: P::Num,
}

/// Decrease Position Params.
#[derive(Debug, Clone, Copy)]
pub struct DecreasePositionParams<T> {
    collateral_withdrawal_amount: T,
    size_delta_usd: T,
    acceptable_price: Option<T>,
    prices: Prices<T>,
}

impl<T> DecreasePositionParams<T> {
    /// Get collateral withdrawal amount.
    pub fn collateral_withdrawal_amount(&self) -> &T {
        &self.collateral_withdrawal_amount
    }
}

/// Report of the execution of posiiton decreasing.
#[must_use = "`should_remove`, `output_amount`, `secondary_output_amount` must use"]
pub struct DecreasePositionReport<T: Unsigned> {
    should_remove: bool,
    params: DecreasePositionParams<T>,
    price_impact_value: T::Signed,
    execution_price: T,
    size_delta_in_tokens: T,
    fees: PositionFees<T>,
    withdrawable_collateral_amount: T,

    // Output.
    is_output_token_long: bool,
    output_amount: T,
    secondary_output_amount: T,
}

impl<T: Unsigned + fmt::Debug> fmt::Debug for DecreasePositionReport<T>
where
    T::Signed: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DecreasePositionReport")
            .field("should_remove", &self.should_remove)
            .field("params", &self.params)
            .field("price_impact_value", &self.price_impact_value)
            .field("execution_price", &self.execution_price)
            .field("size_delta_in_tokens", &self.size_delta_in_tokens)
            .field("fees", &self.fees)
            .field(
                "withdrawable_collateral_amount",
                &self.withdrawable_collateral_amount,
            )
            .field("is_output_token_long", &self.is_output_token_long)
            .field("output_amount", &self.output_amount)
            .field("secondary_output_amount", &self.secondary_output_amount)
            .finish()
    }
}

impl<T: Unsigned> DecreasePositionReport<T> {
    fn new(
        should_remove: bool,
        params: DecreasePositionParams<T>,
        execution: ProcessCollateralResult<T>,
        withdrawable_collateral_amount: T,
    ) -> Self {
        Self {
            should_remove,
            params,
            price_impact_value: execution.price_impact_value,
            execution_price: execution.execution_price,
            size_delta_in_tokens: execution.size_delta_in_tokens,
            fees: execution.fees,
            is_output_token_long: execution.is_output_token_long,
            output_amount: execution.collateral.output_amount,
            secondary_output_amount: execution.collateral.secondary_output_amount,
            withdrawable_collateral_amount,
        }
    }

    /// Get params.
    pub fn params(&self) -> &DecreasePositionParams<T> {
        &self.params
    }

    /// Get size delta in tokens.
    pub fn size_delta_in_tokens(&self) -> &T {
        &self.size_delta_in_tokens
    }

    /// Get execution price.
    pub fn execution_price(&self) -> &T {
        &self.execution_price
    }

    /// Get price impact value.
    pub fn price_impact_value(&self) -> &T::Signed {
        &self.price_impact_value
    }

    /// Get execution fees.
    pub fn fees(&self) -> &PositionFees<T> {
        &self.fees
    }

    /// Returns whether the output token is long token.
    pub fn is_output_token_long(&self) -> bool {
        self.is_output_token_long
    }

    /// Get output amount.
    pub fn output_amount(&self) -> &T {
        &self.output_amount
    }

    /// Get secondary output amount.
    pub fn secondary_output_amount(&self) -> &T {
        &self.secondary_output_amount
    }

    /// Get should remove.
    pub fn should_remove(&self) -> bool {
        self.should_remove
    }

    /// Get withdrawable collateral amount.
    pub fn withdrawable_collateral_amount(&self) -> &T {
        &self.withdrawable_collateral_amount
    }
}

struct ProcessCollateralResult<T: Unsigned> {
    price_impact_value: T::Signed,
    execution_price: T,
    size_delta_in_tokens: T,
    is_output_token_long: bool,
    collateral: ProcessReport<T>,
    fees: PositionFees<T>,
}

impl<const DECIMALS: u8, P: Position<DECIMALS>> DecreasePosition<P, DECIMALS> {
    /// Create a new action to decrease the given position.
    pub fn try_new(
        mut position: P,
        prices: Prices<P::Num>,
        size_delta_usd: P::Num,
        acceptable_price: Option<P::Num>,
        collateral_withdrawal_amount: P::Num,
    ) -> crate::Result<Self> {
        if !prices.is_valid() {
            return Err(crate::Error::invalid_argument("invalid prices"));
        }
        if size_delta_usd > *position.size_in_usd_mut() {
            return Err(crate::Error::invalid_argument(
                "invalid decrease position size",
            ));
        }
        Ok(Self {
            params: DecreasePositionParams {
                size_delta_usd,
                acceptable_price,
                prices,
                collateral_withdrawal_amount: collateral_withdrawal_amount.clone(),
            },
            withdrawable_collateral_amount: collateral_withdrawal_amount
                .min(position.collateral_amount_mut().clone()),
            position,
        })
    }

    /// Execute.
    pub fn execute(mut self) -> crate::Result<DecreasePositionReport<P::Num>> {
        debug_assert!(
            self.params.size_delta_usd <= *self.position.size_in_usd_mut(),
            "must have been checked or capped by the position size"
        );
        debug_assert!(
            self.withdrawable_collateral_amount <= *self.position.collateral_amount_mut(),
            "must have been capped by the position collateral amount"
        );

        self.check_partial_close()?;
        self.check_close()?;

        let is_pnl_token_long = self.position.is_long();

        // TODO: handle NoSwap.

        // TODO: distribute position impact pool.

        // TODO: update funding and borrowing state.

        // TODO: handle liquidation order.

        // let initial_collateral_amount = self.position.collateral_amount_mut().clone();

        let mut execution = self.process_collateral(is_pnl_token_long)?;

        let next_position_size_in_usd = self
            .position
            .size_in_usd_mut()
            .checked_sub(&self.params.size_delta_usd)
            .ok_or(crate::Error::Computation(
                "calculating next position size in usd",
            ))?;

        // TODO: update borrowing state.

        // *self.position.size_in_usd_mut() = next_position_size_in_usd;
        let next_position_size_in_tokens = self
            .position
            .size_in_tokens_mut()
            .checked_sub(&execution.size_delta_in_tokens)
            .ok_or(crate::Error::Computation("calculating next size in tokens"))?;
        let next_position_collateral_amount =
            execution.collateral.remaining_collateral_amount.clone();

        // TODO: update claimable funding amount.

        let should_remove =
            if next_position_size_in_usd.is_zero() || next_position_size_in_tokens.is_zero() {
                *self.position.size_in_usd_mut() = Zero::zero();
                *self.position.size_in_tokens_mut() = Zero::zero();
                *self.position.collateral_amount_mut() = Zero::zero();
                execution.collateral.output_amount = execution
                    .collateral
                    .output_amount
                    .checked_add(&next_position_collateral_amount)
                    .ok_or(crate::Error::Computation("calculating output amount"))?;
                true
            } else {
                // TODO: update funding and borrowing states of position.
                *self.position.size_in_usd_mut() = next_position_size_in_usd;
                *self.position.size_in_tokens_mut() = next_position_size_in_tokens;
                *self.position.collateral_amount_mut() = next_position_collateral_amount;
                false
            };

        // TODO: update global states.
        // TODO: handle referral.

        // TODO: validate position if not `should_remove`.

        Ok(DecreasePositionReport::new(
            should_remove,
            self.params,
            execution,
            self.withdrawable_collateral_amount,
        ))
    }

    /// Do a check when the position will be partially decreased.
    fn check_partial_close(&mut self) -> crate::Result<()> {
        // TODO: make sure the collateral amount after withdraw will be sufficient.
        Ok(())
    }

    fn check_close(&mut self) -> crate::Result<()> {
        if self.params.size_delta_usd == *self.position.size_in_usd()
            && !self.withdrawable_collateral_amount.is_zero()
        {
            // Help ensure that the order can be executed.
            self.withdrawable_collateral_amount = Zero::zero();
        }
        Ok(())
    }

    #[allow(clippy::type_complexity)]
    fn process_collateral(
        &mut self,
        is_pnl_token_long: bool,
    ) -> crate::Result<ProcessCollateralResult<P::Num>> {
        // TODO: handle insolvent close.

        let (price_impact_value, _price_impact_diff_usd, execution_price) =
            self.get_execution_params()?;

        // TODO: calculate position pnl usd.
        let (base_pnl_usd, _uncapped_base_pnl_usd, size_delta_in_tokens) = self.get_pnl_usd()?;

        // TODO: calcualte fees.
        let fees = PositionFees::default();

        let is_output_token_long = self.position.is_collateral_token_long();

        let remaining_collateral_amount = self.position.collateral_amount_mut().clone();
        let mut processor = CollateralProcessor::new(
            self.position.market_mut(),
            remaining_collateral_amount,
            is_output_token_long,
            is_pnl_token_long,
            &self.params.prices.long_token_price,
            &self.params.prices.short_token_price,
        );

        processor.apply_pnl(&base_pnl_usd)?;
        // TODO: pay positive price impact.
        // TODO: pay for funding fees.
        // TODO: pay for other fees.
        // TODO: pay ofr negative price impact.
        // TODO: pay for price impact diff.
        let mut report = processor.process()?;

        // TODO: handle initial collateral delta amount with price impact diff.

        // Cap the withdrawal amount to the remaining collateral amount.
        if self.withdrawable_collateral_amount > report.remaining_collateral_amount {
            self.withdrawable_collateral_amount = report.remaining_collateral_amount.clone();
        }

        if !self.withdrawable_collateral_amount.is_zero() {
            report.remaining_collateral_amount = report
                .remaining_collateral_amount
                .checked_sub(&self.withdrawable_collateral_amount)
                .expect("must be success");
            report.output_amount = report
                .output_amount
                .checked_add(&self.withdrawable_collateral_amount)
                .ok_or(crate::Error::Computation(
                    "overflow adding withdrawable amount",
                ))?;
        }

        Ok(ProcessCollateralResult {
            price_impact_value,
            execution_price,
            size_delta_in_tokens,
            is_output_token_long,
            collateral: report,
            fees,
        })
    }

    fn get_execution_params(&self) -> crate::Result<(P::Signed, P::Signed, P::Num)> {
        let index_token_price = &self.params.prices.index_token_price;
        let size_delta_usd = &self.params.size_delta_usd;

        if size_delta_usd.is_zero() {
            // TODO: pick price by position side.
            return Ok((Zero::zero(), Zero::zero(), index_token_price.clone()));
        }

        // TODO: calculate capped price impact value.
        let price_impact_value = Zero::zero();

        // TODO: bound negative price impact value.

        let execution_price = get_execution_price_for_decrease(
            index_token_price,
            self.position.size_in_usd(),
            self.position.size_in_tokens(),
            size_delta_usd,
            &price_impact_value,
            self.params.acceptable_price.as_ref(),
            self.position.is_long(),
        )?;

        Ok((price_impact_value, Zero::zero(), execution_price))
    }

    fn get_pnl_usd(&self) -> crate::Result<(P::Signed, P::Signed, P::Num)> {
        // TODO: pick by position side.
        let execution_price = &self.params.prices.index_token_price;

        let position_value: P::Signed = self
            .position
            .size_in_tokens()
            .checked_mul(execution_price)
            .ok_or(crate::Error::Computation(
                "overflow calculating position value",
            ))?
            .try_into()
            .map_err(|_| crate::Error::Convert)?;
        let size_in_usd: P::Signed = self
            .position
            .size_in_usd()
            .clone()
            .try_into()
            .map_err(|_| crate::Error::Convert)?;
        let total_pnl = if self.position.is_long() {
            position_value.checked_sub(&size_in_usd)
        } else {
            size_in_usd.checked_sub(&position_value)
        }
        .ok_or(crate::Error::Computation("calculating total pnl"))?;
        let uncapped_total_pnl = total_pnl.clone();

        if total_pnl.is_positive() {
            // TODO: cap `total_pnl`.
        }

        let size_delta_in_tokens = if *self.position.size_in_usd() == self.params.size_delta_usd {
            self.position.size_in_tokens().clone()
        } else if self.position.is_long() {
            self.position
                .size_in_tokens()
                .checked_mul(&self.params.size_delta_usd)
                .and_then(|v| v.checked_round_up_div(self.position.size_in_usd()))
                .ok_or(crate::Error::Computation(
                    "calculating size delta in tokens for long",
                ))?
        } else {
            self.position
                .size_in_tokens()
                .checked_mul_div(&self.params.size_delta_usd, self.position.size_in_usd())
                .ok_or(crate::Error::Computation(
                    "calculating size delta in tokens for short",
                ))?
        };

        let pnl_usd = size_delta_in_tokens
            .checked_mul_div_with_signed_numberator(&total_pnl, self.position.size_in_tokens())
            .ok_or(crate::Error::Computation("calculating pnl_usd"))?;

        let uncapped_pnl_usd = size_delta_in_tokens
            .checked_mul_div_with_signed_numberator(
                &uncapped_total_pnl,
                self.position.size_in_tokens(),
            )
            .ok_or(crate::Error::Computation("calculating uncapped_pnl_usd"))?;

        Ok((pnl_usd, uncapped_pnl_usd, size_delta_in_tokens))
    }
}

fn get_execution_price_for_decrease<T: Unsigned>(
    index_price: &T,
    size_in_usd: &T,
    size_in_tokens: &T,
    size_delta_usd: &T,
    price_impact_value: &T::Signed,
    acceptable_price: Option<&T>,
    is_long: bool,
) -> crate::Result<T>
where
    T: Clone + MulDiv + Ord + CheckedAdd + CheckedSub,
    T::Signed: CheckedSub + Clone + Ord + UnsignedAbs,
{
    // TODO: pick index price by position side.
    let mut execution_price = index_price.clone();
    if !size_delta_usd.is_zero() && !size_in_tokens.is_zero() {
        let adjusted_price_impact_value = if is_long {
            price_impact_value.clone()
        } else {
            T::Signed::zero()
                .checked_sub(price_impact_value)
                .ok_or(crate::Error::Computation("price impact too large"))?
        };
        if adjusted_price_impact_value.is_negative()
            && adjusted_price_impact_value.unsigned_abs() > *size_delta_usd
        {
            return Err(crate::Error::Computation(
                "price impact larger than order size",
            ));
        }

        let adjustment = size_in_usd
            .checked_mul_div_with_signed_numberator(&adjusted_price_impact_value, size_in_tokens)
            .ok_or(crate::Error::Computation(
                "calculating execution price adjustment",
            ))?
            / (size_delta_usd.clone())
                .try_into()
                .map_err(|_| crate::Error::Convert)?;
        execution_price = execution_price
            .checked_add_with_signed(&adjustment)
            .ok_or(crate::Error::Computation("adjusting execution price"))?;
    }
    let Some(acceptable_prcie) = acceptable_price else {
        return Ok(execution_price);
    };
    if (is_long && execution_price >= *acceptable_prcie)
        || (!is_long && execution_price <= *acceptable_prcie)
    {
        Ok(execution_price)
    } else {
        Err(crate::Error::InvalidArgument(
            "order not fulfillable at acceptable price",
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        position::PositionExt,
        test::{TestMarket, TestPosition},
        MarketExt,
    };

    use super::*;

    #[test]
    fn basic() -> crate::Result<()> {
        let mut market = TestMarket::<u64, 9>::default();
        market.deposit(1_000_000_000, 0, 120, 1)?.execute()?;
        market.deposit(0, 1_000_000_000, 120, 1)?.execute()?;
        println!("{market:#?}");
        let mut position = TestPosition::long(true);
        let report = position
            .ops(&mut market)
            .increase(
                Prices {
                    index_token_price: 123,
                    long_token_price: 123,
                    short_token_price: 1,
                },
                1_000_000,
                80_000_000,
                None,
            )?
            .execute()?;
        println!("{report:#?}");
        println!("{position:#?}");

        let report = position
            .ops(&mut market)
            .decrease(
                Prices {
                    index_token_price: 125,
                    long_token_price: 125,
                    short_token_price: 1,
                },
                40_000_000,
                None,
                100,
            )?
            .execute()?;
        println!("{report:#?}");
        println!("{position:#?}");
        println!("{market:#?}");

        let report = position
            .ops(&mut market)
            .decrease(
                Prices {
                    index_token_price: 118,
                    long_token_price: 118,
                    short_token_price: 1,
                },
                40_000_000,
                None,
                0,
            )?
            .execute()?;
        println!("{report:#?}");
        println!("{position:#?}");
        println!("{market:#?}");
        Ok(())
    }
}
