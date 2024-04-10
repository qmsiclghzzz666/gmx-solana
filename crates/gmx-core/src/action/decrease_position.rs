use num_traits::{CheckedAdd, CheckedMul, CheckedSub, Signed, Zero};
use std::fmt;

use crate::{
    num::{MulDiv, Unsigned, UnsignedAbs},
    params::fee::PositionFees,
    pool::PoolExt,
    position::Position,
    MarketExt,
};

use super::Prices;

/// Decrease the position.
#[must_use]
pub struct DecreasePosition<P: Position<DECIMALS>, const DECIMALS: u8> {
    position: P,
    params: DecreasePositionParams<P::Num>,
}

/// Decrease Position Params.
#[derive(Debug, Clone, Copy)]
pub struct DecreasePositionParams<T> {
    size_delta_usd: T,
    acceptable_price: Option<T>,
    prices: Prices<T>,
}

/// Report of the execution of posiiton decreasing.
#[must_use = "`should_remove` and `output_amount` must use"]
pub struct DecreasePositionReport<T: Unsigned> {
    should_remove: bool,
    params: DecreasePositionParams<T>,
    execution: ExecutionResult<T>,
}

impl<T: Unsigned + fmt::Debug> fmt::Debug for DecreasePositionReport<T>
where
    T::Signed: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DecreasePositionReport")
            .field("should_remove", &self.should_remove)
            .field("params", &self.params)
            .field("execution", &self.execution)
            .finish()
    }
}

impl<T: Unsigned> DecreasePositionReport<T> {
    fn new(
        should_remove: bool,
        params: DecreasePositionParams<T>,
        execution: ExecutionResult<T>,
    ) -> Self {
        Self {
            should_remove,
            params,
            execution,
        }
    }

    /// Get params.
    pub fn params(&self) -> &DecreasePositionParams<T> {
        &self.params
    }

    /// Get execution result.
    pub fn execution(&self) -> &ExecutionResult<T> {
        &self.execution
    }

    /// Get output amount.
    pub fn output_amount(&self) -> &T {
        &self.execution.output_amount
    }

    /// Get should remove.
    pub fn should_remove(&self) -> bool {
        self.should_remove
    }
}

/// Exeuction Result for decreasing position.
#[derive(Debug, Clone, Copy)]
pub struct ExecutionResult<T: Unsigned> {
    price_impact_value: T::Signed,
    execution_price: T,
    size_delta_in_tokens: T,
    remaining_collateral_amount: T,
    is_output_token_long: bool,
    output_amount: T,
    is_secondary_output_token_long: bool,
    secondary_output_amount: T,
}

impl<T: Unsigned> ExecutionResult<T> {
    /// Get execution price.
    pub fn execution_price(&self) -> &T {
        &self.execution_price
    }
}

impl<const DECIMALS: u8, P: Position<DECIMALS>> DecreasePosition<P, DECIMALS> {
    /// Create a new action to decrease the given position.
    pub fn try_new(
        mut position: P,
        prices: Prices<P::Num>,
        size_delta_usd: P::Num,
        acceptable_price: Option<P::Num>,
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
            position,
            params: DecreasePositionParams {
                size_delta_usd,
                acceptable_price,
                prices,
            },
        })
    }

    /// Execute.
    pub fn execute(mut self) -> crate::Result<DecreasePositionReport<P::Num>> {
        debug_assert!(
            self.params.size_delta_usd <= *self.position.size_in_usd_mut(),
            "must have been checked or capped by the position size"
        );
        self.check_partial()?;
        self.prepare_close()?;

        let is_pnl_token_long = self.position.is_long();
        let pnl_token_price = if is_pnl_token_long {
            &self.params.prices.long_token_price
        } else {
            &self.params.prices.short_token_price
        };

        // TODO: handle NoSwap.

        // TODO: distribute position impact pool.

        // TODO: update funding and borrowing state.

        // TODO: handle liquidation order.

        let initial_collateral_amount = self.position.collateral_amount_mut().clone();

        let (mut execution, fees) = self.process_collateral(is_pnl_token_long)?;

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
        let next_position_collateral_amount = execution.remaining_collateral_amount.clone();

        // TODO: update claimable funding amount.

        let should_remove =
            if next_position_size_in_usd.is_zero() || next_position_size_in_tokens.is_zero() {
                *self.position.size_in_usd_mut() = Zero::zero();
                *self.position.size_in_tokens_mut() = Zero::zero();
                *self.position.collateral_amount_mut() = Zero::zero();
                execution.output_amount = execution
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
        ))
    }

    /// Do a check when the position will be partially decreased.
    fn check_partial(&mut self) -> crate::Result<()> {
        // TODO: do the check.
        Ok(())
    }

    /// FIXME: not sure what to do here.
    fn prepare_close(&mut self) -> crate::Result<()> {
        // TODO: set initial collateral delta amount to zero?
        Ok(())
    }

    #[allow(clippy::type_complexity)]
    fn process_collateral(
        &mut self,
        is_pnl_token_long: bool,
    ) -> crate::Result<(ExecutionResult<P::Num>, PositionFees<P::Num>)> {
        let pnl_token_price = if is_pnl_token_long {
            &self.params.prices.long_token_price
        } else {
            &self.params.prices.short_token_price
        };
        // TODO: handle insolvent close.

        let (price_impact_value, price_impact_diff_usd, execution_price) =
            self.get_execution_params()?;

        // TODO: calculate position pnl usd.
        let (base_pnl_usd, unpacced_base_pnl_usd, size_delta_in_tokens) = self.get_pnl_usd()?;

        // TODO: calcualte fees.
        let fees = PositionFees::default();

        let is_output_token_long = self.position.is_collateral_token_long();
        let mut output_amount: P::Num = Zero::zero();
        let mut secondary_output_amount: P::Num = Zero::zero();

        // Pay positive pnl.
        if base_pnl_usd.is_positive() {
            // TODO: pick max pnl token price.
            let deduction_amount_for_pool = base_pnl_usd.unsigned_abs() / pnl_token_price.clone();

            self.position.market_mut().apply_delta(
                is_pnl_token_long,
                &deduction_amount_for_pool.to_opposite_signed()?,
            )?;

            if is_output_token_long == is_pnl_token_long {
                output_amount = output_amount
                    .checked_add(&deduction_amount_for_pool)
                    .ok_or(crate::Error::Computation(
                        "overflow adding deduction amount to output_amount",
                    ))?;
            } else {
                secondary_output_amount = secondary_output_amount
                    .checked_add(&deduction_amount_for_pool)
                    .ok_or(crate::Error::Computation(
                        "overflow adding deduction amount to secondary_output_amount",
                    ))?;
            }
        }

        // TODO: pay positive price impact.

        // FIXME: We might not have to do the swapping of profit to collateral token here.

        let mut remaining_collateral_amount = self.position.collateral_amount_mut().clone();

        // TODO: pay for funding fees.

        // Pay negative pnl.
        if base_pnl_usd.is_negative() {
            self.pay_for_cost(&base_pnl_usd.unsigned_abs())?;
        }

        // TODO: pay for other fees.
        // TODO: pay ofr negative price impact.
        // TODO: pay for price impact diff.
        // TODO: handle initial collateral delta amount.
        Ok((
            ExecutionResult {
                price_impact_value,
                execution_price,
                size_delta_in_tokens,
                remaining_collateral_amount,
                is_output_token_long,
                output_amount,
                is_secondary_output_token_long: is_pnl_token_long,
                secondary_output_amount,
            },
            fees,
        ))
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

    fn pay_for_cost(&self, cost: &P::Num) -> crate::Result<()> {
        todo!()
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
            )?
            .execute()?;
        println!("{report:#?}");
        println!("{position:#?}");
        println!("{market:#?}");
        Ok(())
    }
}
