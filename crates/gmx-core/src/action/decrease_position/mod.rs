use num_traits::{CheckedAdd, CheckedSub, Zero};

use crate::{
    num::{MulDiv, Unsigned},
    params::fee::PositionFees,
    position::{CollateralDelta, Position, PositionExt, WillCollateralBeSufficient},
    Market,
};

use self::{
    collateral_processor::{CollateralProcessor, ProcessReport},
    report::DecreasePositionReport,
};

use super::Prices;

mod collateral_processor;
mod debt;
mod report;
mod utils;

/// Decrease the position.
#[must_use]
pub struct DecreasePosition<P: Position<DECIMALS>, const DECIMALS: u8> {
    position: P,
    params: DecreasePositionParams<P::Num>,
    withdrawable_collateral_amount: P::Num,
    size_delta_usd: P::Num,
    is_insolvent_close_allowed: bool,
    is_liquidation_order: bool,
}

/// Decrease Position Params.
#[derive(Debug, Clone, Copy)]
pub struct DecreasePositionParams<T> {
    initial_collateral_withdrawal_amount: T,
    initial_size_delta_usd: T,
    acceptable_price: Option<T>,
    prices: Prices<T>,
}

impl<T> DecreasePositionParams<T> {
    /// Get initial collateral withdrawal amount.
    pub fn initial_collateral_withdrawal_amount(&self) -> &T {
        &self.initial_collateral_withdrawal_amount
    }

    /// Get inital size delta usd.
    pub fn initial_size_delta_usd(&self) -> &T {
        &self.initial_size_delta_usd
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
        position: P,
        prices: Prices<P::Num>,
        size_delta_usd: P::Num,
        acceptable_price: Option<P::Num>,
        collateral_withdrawal_amount: P::Num,
        is_insolvent_close_allowed: bool,
        is_liquidation_order: bool,
    ) -> crate::Result<Self> {
        if !prices.is_valid() {
            return Err(crate::Error::invalid_argument("invalid prices"));
        }
        let size_delta_usd = size_delta_usd.min(position.size_in_usd().clone());
        Ok(Self {
            params: DecreasePositionParams {
                initial_size_delta_usd: size_delta_usd.clone(),
                acceptable_price,
                prices,
                initial_collateral_withdrawal_amount: collateral_withdrawal_amount.clone(),
            },
            withdrawable_collateral_amount: collateral_withdrawal_amount
                .min(position.collateral_amount().clone()),
            is_insolvent_close_allowed: is_insolvent_close_allowed
                && (size_delta_usd == *position.size_in_usd())
                && is_liquidation_order,
            size_delta_usd,
            position,
            is_liquidation_order,
        })
    }

    /// Execute.
    pub fn execute(mut self) -> crate::Result<DecreasePositionReport<P::Num>> {
        debug_assert!(
            self.size_delta_usd <= *self.position.size_in_usd_mut(),
            "must have been checked or capped by the position size"
        );
        debug_assert!(
            self.withdrawable_collateral_amount <= *self.position.collateral_amount_mut(),
            "must have been capped by the position collateral amount"
        );

        self.check_partial_close()?;
        self.check_close()?;

        // TODO: handle NoSwap.

        // TODO: distribute position impact pool.

        // TODO: update funding and borrowing state.

        self.check_liquiation()?;

        // let initial_collateral_amount = self.position.collateral_amount_mut().clone();

        let mut execution = self.process_collateral()?;

        let next_position_size_in_usd = self
            .position
            .size_in_usd_mut()
            .checked_sub(&self.size_delta_usd)
            .ok_or(crate::Error::Computation(
                "calculating next position size in usd",
            ))?;

        // TODO: update borrowing state.

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

        if !should_remove {
            self.position
                .validate_position(&self.params.prices, false, false)?;
        }

        self.position.decreased()?;

        Ok(DecreasePositionReport::new(
            should_remove,
            self.params,
            execution,
            self.withdrawable_collateral_amount,
            self.size_delta_usd,
        ))
    }

    /// Do a check when the position will be partially decreased.
    fn check_partial_close(&mut self) -> crate::Result<()> {
        use num_traits::CheckedMul;

        if self.size_delta_usd < *self.position.size_in_usd() {
            let (estimated_pnl, _, _) = self
                .position
                .pnl_value(&self.params.prices, self.position.size_in_usd())?;
            let estimated_realized_pnl = self
                .size_delta_usd
                .checked_mul_div_with_signed_numberator(&estimated_pnl, self.position.size_in_usd())
                .ok_or(crate::Error::Computation("estiamting realized pnl"))?;
            let estimated_remaining_pnl = estimated_pnl
                .checked_sub(&estimated_realized_pnl)
                .ok_or(crate::Error::Underflow)?;

            let delta = CollateralDelta::new(
                self.position
                    .size_in_usd()
                    .checked_sub(&self.size_delta_usd)
                    .expect("should have been capped"),
                self.position
                    .collateral_amount()
                    .checked_sub(&self.withdrawable_collateral_amount)
                    .expect("should have been capped"),
                estimated_realized_pnl,
                self.size_delta_usd.to_opposite_signed()?,
            );

            let mut will_be_sufficient = self
                .position
                .will_collateral_be_sufficient(&self.params.prices, &delta)?;

            if let WillCollateralBeSufficient::Insufficient(remaining_collateral_value) =
                &mut will_be_sufficient
            {
                if self.size_delta_usd.is_zero() {
                    return Err(crate::Error::invalid_argument(
                        "unable to withdraw collateral: insufficient collateral",
                    ));
                }

                let collateral_token_price = if self.position.is_collateral_token_long() {
                    &self.params.prices.long_token_price
                } else {
                    &self.params.prices.short_token_price
                };
                // Add back to the estimated remaining collateral value && set withdrawable collateral amount to zero.
                let add_back = self
                    .withdrawable_collateral_amount
                    .checked_mul(collateral_token_price)
                    .ok_or(crate::Error::Computation("overflow calculating add back"))?
                    .to_signed()?;
                *remaining_collateral_value = remaining_collateral_value
                    .checked_add(&add_back)
                    .ok_or(crate::Error::Computation("adding back"))?;
                self.withdrawable_collateral_amount = Zero::zero();
            }

            // Close all if collateral or position size too small.

            let params = self.position.market().position_params();

            let remaining_value = will_be_sufficient
                .checked_add(&estimated_remaining_pnl)
                .ok_or(crate::Error::Computation("calculating remaining value"))?;
            if remaining_value < params.min_collateral_value().to_signed()? {
                self.size_delta_usd = self.position.size_in_usd().clone();
            }

            if *self.position.size_in_usd() > self.size_delta_usd
                && self
                    .position
                    .size_in_usd()
                    .checked_sub(&self.size_delta_usd)
                    .expect("must success")
                    < *params.min_position_size_usd()
            {
                self.size_delta_usd = self.position.size_in_usd().clone();
            }
        }
        Ok(())
    }

    fn check_close(&mut self) -> crate::Result<()> {
        if self.size_delta_usd == *self.position.size_in_usd()
            && !self.withdrawable_collateral_amount.is_zero()
        {
            // Help ensure that the order can be executed.
            self.withdrawable_collateral_amount = Zero::zero();
        }
        Ok(())
    }

    fn check_liquiation(&self) -> crate::Result<()> {
        if self.is_liquidation_order {
            // FIXME: should we check that whether this is a close all order?
            let Some(_reason) = self
                .position
                .check_liquidatable(&self.params.prices, true)?
            else {
                return Err(crate::Error::NotLiquidatable);
            };
            Ok(())
        } else {
            Ok(())
        }
    }

    #[allow(clippy::type_complexity)]
    fn process_collateral(&mut self) -> crate::Result<ProcessCollateralResult<P::Num>> {
        // TODO: handle insolvent close.

        let (price_impact_value, _price_impact_diff_usd, execution_price) =
            self.get_execution_params()?;

        // TODO: calculate position pnl usd.
        let (base_pnl_usd, _uncapped_base_pnl_usd, size_delta_in_tokens) = self
            .position
            .pnl_value(&self.params.prices, &self.size_delta_usd)?;

        // TODO: calcualte fees.
        let fees = PositionFees::default();

        let is_output_token_long = self.position.is_collateral_token_long();
        let is_pnl_token_long = self.position.is_long();

        let remaining_collateral_amount = self.position.collateral_amount_mut().clone();
        let mut processor = CollateralProcessor::new(
            self.position.market_mut(),
            remaining_collateral_amount,
            is_output_token_long,
            is_pnl_token_long,
            &self.params.prices.long_token_price,
            &self.params.prices.short_token_price,
            self.is_insolvent_close_allowed,
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
        let size_delta_usd = &self.size_delta_usd;

        if size_delta_usd.is_zero() {
            // TODO: pick price by position side.
            return Ok((Zero::zero(), Zero::zero(), index_token_price.clone()));
        }

        // TODO: calculate capped price impact value.
        let price_impact_value = Zero::zero();

        // TODO: bound negative price impact value.

        let execution_price = utils::get_execution_price_for_decrease(
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
                80_000_000_000,
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
                40_000_000_000,
                None,
                100_000_000,
                false,
                false,
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
                40_000_000_000,
                None,
                0,
                false,
                false,
            )?
            .execute()?;
        println!("{report:#?}");
        println!("{position:#?}");
        println!("{market:#?}");
        Ok(())
    }
}
