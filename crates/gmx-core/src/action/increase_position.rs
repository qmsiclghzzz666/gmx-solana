use num_traits::{CheckedAdd, CheckedNeg, Signed, Zero};
use std::fmt;

use crate::{
    num::Unsigned,
    params::fee::PositionFees,
    position::{CollateralDelta, Position, PositionExt},
    MarketExt,
};

use super::Prices;

/// Increase the position.
#[must_use]
pub struct IncreasePosition<P: Position<DECIMALS>, const DECIMALS: u8> {
    position: P,
    params: IncreasePositionParams<P::Num>,
}

/// Increase Position Params.
#[derive(Debug, Clone, Copy)]
pub struct IncreasePositionParams<T> {
    collateral_increment_amount: T,
    size_delta_usd: T,
    acceptable_price: Option<T>,
    prices: Prices<T>,
}

/// Report of the execution of position increasing.
pub struct IncreasePositionReport<T: Unsigned> {
    params: IncreasePositionParams<T>,
    execution: ExecutionParams<T>,
    collateral_delta_amount: T,
    fees: PositionFees<T>,
}

impl<T: Unsigned + fmt::Debug> fmt::Debug for IncreasePositionReport<T>
where
    T::Signed: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IncreasePositionReport")
            .field("params", &self.params)
            .field("execution", &self.execution)
            .field("collateral_delta_amount", &self.collateral_delta_amount)
            .field("fees", &self.fees)
            .finish()
    }
}

impl<T: Unsigned> IncreasePositionReport<T> {
    fn new(
        params: IncreasePositionParams<T>,
        execution: ExecutionParams<T>,
        collateral_delta_amount: T,
        fees: PositionFees<T>,
    ) -> Self {
        Self {
            params,
            execution,
            collateral_delta_amount,
            fees,
        }
    }

    /// Get params.
    pub fn params(&self) -> &IncreasePositionParams<T> {
        &self.params
    }

    /// Get execution params.
    pub fn execution(&self) -> &ExecutionParams<T> {
        &self.execution
    }

    /// Get collateral delta amount.
    pub fn collateral_delta_amount(&self) -> &T {
        &self.collateral_delta_amount
    }

    /// Get position fees.
    pub fn fees(&self) -> &PositionFees<T> {
        &self.fees
    }
}

/// Exeuction Params for increasing position.
#[derive(Debug, Clone, Copy)]
pub struct ExecutionParams<T: Unsigned> {
    price_impact_value: T::Signed,
    price_impact_amount: T::Signed,
    size_delta_in_tokens: T,
    execution_price: T,
}

impl<T: Unsigned> ExecutionParams<T> {
    /// Get execution price.
    pub fn execution_price(&self) -> &T {
        &self.execution_price
    }

    /// Get price impact amount.
    pub fn price_impact_amount(&self) -> &T::Signed {
        &self.price_impact_amount
    }
}

impl<const DECIMALS: u8, P: Position<DECIMALS>> IncreasePosition<P, DECIMALS> {
    /// Create a new action to increase the given position.
    pub fn try_new(
        position: P,
        prices: Prices<P::Num>,
        collateral_increment_amount: P::Num,
        size_delta_usd: P::Num,
        acceptable_price: Option<P::Num>,
    ) -> crate::Result<Self> {
        if !prices.is_valid() {
            return Err(crate::Error::invalid_argument("invalid prices"));
        }
        Ok(Self {
            position,
            params: IncreasePositionParams {
                collateral_increment_amount,
                size_delta_usd,
                acceptable_price,
                prices,
            },
        })
    }

    /// Execute.
    pub fn execute(mut self) -> crate::Result<IncreasePositionReport<P::Num>> {
        // TODO: update funding and borrowing state

        let execution = self.get_execution_params()?;

        let (collateral_delta_amount, fees) =
            self.process_collateral(&execution.price_impact_value)?;
        *self.position.collateral_amount_mut() = self
            .position
            .collateral_amount_mut()
            .checked_add(&collateral_delta_amount)
            .ok_or(crate::Error::Computation("collateral amount overflow"))?;

        self.position
            .market_mut()
            .apply_delta_to_position_impact_pool(
                &execution
                    .price_impact_amount()
                    .checked_neg()
                    .ok_or(crate::Error::Computation(
                        "calculating position impact pool delta amount",
                    ))?,
            )?;

        let next_position_size_in_usd = self
            .position
            .size_in_usd_mut()
            .checked_add(&self.params.size_delta_usd)
            .ok_or(crate::Error::Computation("size in usd overflow"))?;

        // TODO: update total borrowing

        // TODO: update claimable funding amount

        *self.position.size_in_usd_mut() = next_position_size_in_usd;
        *self.position.size_in_tokens_mut() = self
            .position
            .size_in_tokens_mut()
            .checked_add(&execution.size_delta_in_tokens)
            .ok_or(crate::Error::Computation("size in tokens overflow"))?;
        // TODO: update other position state

        self.position.apply_delta_to_open_interest(
            &self.params.size_delta_usd.to_signed()?,
            &execution.size_delta_in_tokens.to_signed()?,
        )?;

        if !self.params.size_delta_usd.is_zero() {
            // TODO: validate reserve.
            // TODO: validate open interset reserve.

            let delta = CollateralDelta::new(
                self.position.size_in_usd().clone(),
                self.position.collateral_amount().clone(),
                Zero::zero(),
                Zero::zero(),
            );
            let will_collateral_be_sufficient = self
                .position
                .will_collateral_be_sufficient(&self.params.prices, &delta)?;

            if !will_collateral_be_sufficient.is_sufficient() {
                return Err(crate::Error::invalid_argument(
                    "insufficient collateral usd",
                ));
            }
        }

        // TODO: handle referral

        self.position
            .validate_position(&self.params.prices, true, true)?;

        self.position.increased()?;

        Ok(IncreasePositionReport::new(
            self.params,
            execution,
            collateral_delta_amount,
            fees,
        ))
    }

    fn get_execution_params(&self) -> crate::Result<ExecutionParams<P::Num>> {
        let index_token_price = &self.params.prices.index_token_price;
        if self.params.size_delta_usd.is_zero() {
            return Ok(ExecutionParams {
                price_impact_value: Zero::zero(),
                price_impact_amount: Zero::zero(),
                size_delta_in_tokens: Zero::zero(),
                // TODO: pick price by position side
                execution_price: index_token_price.clone(),
            });
        }

        let price_impact_value = self.position.capped_positive_position_price_impact(
            index_token_price,
            &self.params.size_delta_usd.to_signed()?,
        )?;

        let price_impact_amount = if price_impact_value.is_positive() {
            let price: P::Signed = self
                .params
                .prices
                .index_token_price
                .clone()
                .try_into()
                .map_err(|_| crate::Error::Convert)?;
            debug_assert!(
                !price.is_zero(),
                "price must have been checked to be non-zero"
            );
            price_impact_value.clone() / price
        } else {
            self.params
                .prices
                .index_token_price
                .as_divisor_to_round_up_magnitude_div(&price_impact_value)
                .ok_or(crate::Error::Computation("calculating price impact amount"))?
        };

        // Base size delta in tokens.
        let mut size_delta_in_tokens = if self.position.is_long() {
            // TODO: select max price.
            let price = &self.params.prices.index_token_price;
            debug_assert!(
                !price.is_zero(),
                "price must have been checked to be non-zero"
            );
            self.params.size_delta_usd.clone() / price.clone()
        } else {
            // TODO: select min price.
            let price = &self.params.prices.index_token_price;
            self.params
                .size_delta_usd
                .checked_round_up_div(price)
                .ok_or(crate::Error::Computation(
                    "calculating size delta in tokens",
                ))?
        };

        // Apply price impact.
        size_delta_in_tokens = if self.position.is_long() {
            size_delta_in_tokens.checked_add_with_signed(&price_impact_amount)
        } else {
            size_delta_in_tokens.checked_sub_with_signed(&price_impact_amount)
        }
        .ok_or(crate::Error::Computation(
            "price impact larger than order size",
        ))?;

        let execution_price = get_execution_price_for_increase(
            &self.params.size_delta_usd,
            &size_delta_in_tokens,
            self.params.acceptable_price.as_ref(),
            self.position.is_long(),
        )?;

        Ok(ExecutionParams {
            price_impact_value,
            price_impact_amount,
            size_delta_in_tokens,
            execution_price,
        })
    }

    fn process_collateral(
        &self,
        _price_impact_value: &P::Signed,
    ) -> crate::Result<(P::Num, PositionFees<P::Num>)> {
        let collateral_delta_amount = self.params.collateral_increment_amount.clone();
        // TODO: get position fees params and use to get position fees.
        let fees = PositionFees::default();
        // TODO: apply the fees.
        // TODO: apply delta to collateral pool.
        Ok((collateral_delta_amount, fees))
    }
}

fn get_execution_price_for_increase<T>(
    size_delta_usd: &T,
    size_delta_in_tokens: &T,
    acceptable_price: Option<&T>,
    is_long: bool,
) -> crate::Result<T>
where
    T: num_traits::Num + Ord + Clone,
{
    if size_delta_usd.is_zero() {
        return Err(crate::Error::Computation("empty size delta in tokens"));
    }

    let execution_price = size_delta_usd.clone() / size_delta_in_tokens.clone();

    let Some(acceptable_price) = acceptable_price else {
        return Ok(execution_price);
    };

    if (is_long && execution_price <= *acceptable_price)
        || (!is_long && execution_price >= *acceptable_price)
    {
        Ok(execution_price)
    } else {
        Err(crate::Error::invalid_argument(
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
                100_000_000,
                8_000_000_000,
                None,
            )?
            .execute()?;
        println!("{report:#?}");
        println!("{position:#?}");
        Ok(())
    }
}
