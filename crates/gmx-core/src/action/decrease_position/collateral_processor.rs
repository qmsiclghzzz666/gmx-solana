use std::ops::{Deref, DerefMut};

use crate::{
    action::Prices,
    num::{MulDiv, Num, Unsigned, UnsignedAbs},
    params::fee::PositionFees,
    Market, MarketExt,
};

use num_traits::{CheckedAdd, Signed, Zero};

/// Collateral Processor.
#[must_use]
pub(super) struct CollateralProcessor<'a, M: Market<DECIMALS>, const DECIMALS: u8> {
    market: &'a mut M,
    state: State<M::Num>,
    is_insolvent_close_allowed: bool,
}

/// Collateral Process Report.
#[derive(Debug, Clone, Copy)]
pub(super) struct ProcessReport<T> {
    pub(super) output_amount: T,
    pub(super) secondary_output_amount: T,
    pub(super) remaining_collateral_amount: T,
}

struct State<T> {
    long_token_price: T,
    short_token_price: T,
    index_token_price: T,
    is_pnl_token_long: bool,
    is_output_token_long: bool,
    output_amount: T,
    secondary_output_amount: T,
    remaining_collateral_amount: T,
}

impl<T> State<T> {
    #[inline]
    fn output_token_is_pnl_token(&self) -> bool {
        self.is_output_token_long == self.is_pnl_token_long
    }

    #[inline]
    fn pnl_token_price(&self) -> &T {
        if self.is_pnl_token_long {
            &self.long_token_price
        } else {
            &self.short_token_price
        }
    }

    #[inline]
    fn output_token_price(&self) -> &T {
        if self.is_output_token_long {
            &self.long_token_price
        } else {
            &self.short_token_price
        }
    }

    #[inline]
    fn secondary_output_token_price(&self) -> &T {
        if self.is_pnl_token_long {
            &self.long_token_price
        } else {
            &self.short_token_price
        }
    }
}

impl<T> State<T>
where
    T: MulDiv + Num,
{
    /// Pay for cost with `output_amount`, `collateral_amount` and `pnl_amount` in order.
    fn do_pay_for_cost(&mut self, cost: &mut T) -> crate::Result<(T, T, T)> {
        let mut paid_in_output_amount = Zero::zero();
        let mut paid_in_collateral_amount = Zero::zero();
        let mut paid_in_secondary_output_amount = Zero::zero();

        if cost.is_zero() {
            return Ok((
                paid_in_output_amount,
                paid_in_collateral_amount,
                paid_in_secondary_output_amount,
            ));
        }

        let mut remaining_cost_in_output_token =
            cost.checked_round_up_div(self.output_token_price()).ok_or(
                crate::Error::Computation("initializing cost in output tokens"),
            )?;

        if !self.output_amount.is_zero() {
            if self.output_amount > remaining_cost_in_output_token {
                paid_in_output_amount = paid_in_output_amount
                    .checked_add(&remaining_cost_in_output_token)
                    .ok_or(crate::Error::Computation(
                        "overflow increasing paid amount for output token [1]",
                    ))?;
                self.output_amount = self
                    .output_amount
                    .checked_sub(&remaining_cost_in_output_token)
                    .ok_or(crate::Error::Computation(
                        "underflow deducting output amount",
                    ))?;
                remaining_cost_in_output_token = Zero::zero();
            } else {
                paid_in_output_amount = paid_in_output_amount
                    .checked_add(&self.output_amount)
                    .ok_or(crate::Error::Computation(
                        "overflow increasing paid amount for output token [2]",
                    ))?;
                remaining_cost_in_output_token = remaining_cost_in_output_token
                    .checked_sub(&self.output_amount)
                    .ok_or(crate::Error::Computation(
                        "underflow deducting remaining cost in output token [1]",
                    ))?;
                self.output_amount = Zero::zero();
            }
        }

        if remaining_cost_in_output_token.is_zero() {
            *cost = Zero::zero();
            return Ok((
                paid_in_output_amount,
                paid_in_collateral_amount,
                paid_in_secondary_output_amount,
            ));
        }

        if !self.remaining_collateral_amount.is_zero() {
            if self.remaining_collateral_amount > remaining_cost_in_output_token {
                paid_in_collateral_amount = paid_in_collateral_amount
                    .checked_add(&remaining_cost_in_output_token)
                    .ok_or(crate::Error::Computation(
                        "overflow increasing paid amount in collateral token [1]",
                    ))?;
                self.remaining_collateral_amount = self
                    .remaining_collateral_amount
                    .checked_sub(&remaining_cost_in_output_token)
                    .ok_or(crate::Error::Computation(
                        "underflow deducting collateral amount",
                    ))?;
                remaining_cost_in_output_token = Zero::zero();
            } else {
                paid_in_collateral_amount = paid_in_collateral_amount
                    .checked_add(&self.remaining_collateral_amount)
                    .ok_or(crate::Error::Computation(
                        "overflow increasing paid amount for collateral token [2]",
                    ))?;
                remaining_cost_in_output_token = remaining_cost_in_output_token
                    .checked_sub(&self.remaining_collateral_amount)
                    .ok_or(crate::Error::Computation(
                        "underflow deducting remaining cost in output token [2]",
                    ))?;
                self.remaining_collateral_amount = Zero::zero();
            }
        }

        if remaining_cost_in_output_token.is_zero() {
            *cost = Zero::zero();
            return Ok((
                paid_in_output_amount,
                paid_in_collateral_amount,
                paid_in_secondary_output_amount,
            ));
        }

        let mut remaining_cost_in_secondary_output_token = remaining_cost_in_output_token
            .checked_mul_div(
                self.output_token_price(),
                self.secondary_output_token_price(),
            )
            .ok_or(crate::Error::Computation(
                "initalizing remaing cost in secondary output token",
            ))?;

        if !self.secondary_output_amount.is_zero() {
            if self.secondary_output_amount > remaining_cost_in_secondary_output_token {
                paid_in_secondary_output_amount = paid_in_secondary_output_amount
                    .checked_add(&remaining_cost_in_secondary_output_token)
                    .ok_or(crate::Error::Computation(
                        "overflow increasing paid amount in secondary token [1]",
                    ))?;
                self.secondary_output_amount = self
                    .secondary_output_amount
                    .checked_sub(&remaining_cost_in_secondary_output_token)
                    .ok_or(crate::Error::Computation(
                        "underflow deducting secondary amount",
                    ))?;
                remaining_cost_in_secondary_output_token = Zero::zero();
            } else {
                paid_in_secondary_output_amount = paid_in_secondary_output_amount
                    .checked_add(&self.secondary_output_amount)
                    .ok_or(crate::Error::Computation(
                        "overflow increasing paid amount for secondary token [2]",
                    ))?;
                remaining_cost_in_secondary_output_token = remaining_cost_in_secondary_output_token
                    .checked_sub(&self.secondary_output_amount)
                    .ok_or(crate::Error::Computation(
                        "underflow deducting remaining cost in secondary token [2]",
                    ))?;
                self.secondary_output_amount = Zero::zero();
            }
        }

        *cost = remaining_cost_in_secondary_output_token
            .checked_mul(self.secondary_output_token_price())
            .ok_or(crate::Error::Computation("calculating remaing cost"))?;

        Ok((
            paid_in_output_amount,
            paid_in_collateral_amount,
            paid_in_secondary_output_amount,
        ))
    }
}

impl<'a, M, const DECIMALS: u8> CollateralProcessor<'a, M, DECIMALS>
where
    M: Market<DECIMALS>,
{
    pub(super) fn new(
        market: &'a mut M,
        remaining_collateral_amount: M::Num,
        is_output_token_long: bool,
        is_pnl_token_long: bool,
        prices: &Prices<M::Num>,
        is_insolvent_close_allowed: bool,
    ) -> Self {
        Self {
            market,
            state: State {
                remaining_collateral_amount,
                long_token_price: prices.long_token_price.clone(),
                short_token_price: prices.short_token_price.clone(),
                index_token_price: prices.index_token_price.clone(),
                is_pnl_token_long,
                is_output_token_long,
                output_amount: Zero::zero(),
                secondary_output_amount: Zero::zero(),
            },
            is_insolvent_close_allowed,
        }
    }

    fn add_pnl_token_amount(&mut self, deduction_amount_for_pool: M::Num) -> crate::Result<()> {
        if self.state.output_token_is_pnl_token() {
            self.state.output_amount = self
                .state
                .output_amount
                .checked_add(&deduction_amount_for_pool)
                .ok_or(crate::Error::Computation(
                    "overflow adding deduction amount to output_amount",
                ))?;
        } else {
            self.state.secondary_output_amount = self
                .state
                .secondary_output_amount
                .checked_add(&deduction_amount_for_pool)
                .ok_or(crate::Error::Computation(
                    "overflow adding deduction amount to secondary_output_amount",
                ))?;
        }
        Ok(())
    }

    fn into_report(self) -> ProcessReport<M::Num> {
        ProcessReport {
            output_amount: self.state.output_amount,
            remaining_collateral_amount: self.state.remaining_collateral_amount,
            secondary_output_amount: self.state.secondary_output_amount,
        }
    }

    pub(super) fn process(
        mut self,
        f: impl FnOnce(Context<'_, 'a, M, DECIMALS>) -> crate::Result<()>,
    ) -> crate::Result<ProcessReport<M::Num>> {
        let res = (f)(Context {
            processor: &mut self,
        });
        match res {
            Ok(()) => Ok(self.into_report()),
            Err(crate::Error::InsufficientFundsToPayForCosts)
                if self.is_insolvent_close_allowed =>
            {
                Ok(self.into_report())
            }
            Err(err) => Err(err),
        }
    }

    fn pay_for_cost(
        &mut self,
        mut cost: M::Num,
        receive: impl FnOnce(&mut Self, &M::Num, &M::Num, &M::Num) -> crate::Result<()>,
    ) -> crate::Result<()> {
        let (_paid_in_output_amount, paid_in_collateral_amount, paid_in_secondary_amount) =
            self.state.do_pay_for_cost(&mut cost)?;
        (receive)(
            self,
            &paid_in_collateral_amount,
            &paid_in_secondary_amount,
            &cost,
        )?;
        if !cost.is_zero() {
            return Err(crate::Error::InsufficientFundsToPayForCosts);
        }
        Ok(())
    }

    fn pay_with_primary_pool(
        &mut self,
        collateral_token_amount: &M::Signed,
        secondary_output_token_amount: &M::Signed,
    ) -> crate::Result<()> {
        if !collateral_token_amount.is_zero() {
            self.market
                .apply_delta(self.state.is_output_token_long, collateral_token_amount)?;
        }
        if !secondary_output_token_amount.is_zero() {
            self.market
                .apply_delta(self.state.is_pnl_token_long, secondary_output_token_amount)?;
        }
        Ok(())
    }
}

pub(super) struct Context<'a, 'market, M, const DECIMALS: u8>
where
    M: Market<DECIMALS>,
{
    processor: &'a mut CollateralProcessor<'market, M, DECIMALS>,
}

impl<'a, 'market, M, const DECIMALS: u8> Deref for Context<'a, 'market, M, DECIMALS>
where
    M: Market<DECIMALS>,
{
    type Target = CollateralProcessor<'market, M, DECIMALS>;

    fn deref(&self) -> &Self::Target {
        self.processor
    }
}

impl<'a, 'market, M, const DECIMALS: u8> DerefMut for Context<'a, 'market, M, DECIMALS>
where
    M: Market<DECIMALS>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.processor
    }
}

impl<'a, 'market, M, const DECIMALS: u8> Context<'a, 'market, M, DECIMALS>
where
    M: Market<DECIMALS>,
{
    pub(super) fn add_pnl_if_positive(&mut self, pnl: &M::Signed) -> crate::Result<&mut Self> {
        if pnl.is_positive() {
            // TODO: pick max pnl token price.
            let deduction_amount_for_pool =
                pnl.unsigned_abs() / self.state.pnl_token_price().clone();

            let is_pnl_token_long = self.state.is_pnl_token_long;
            self.market.apply_delta(
                is_pnl_token_long,
                &deduction_amount_for_pool.to_opposite_signed()?,
            )?;

            self.add_pnl_token_amount(deduction_amount_for_pool)?;
        }
        Ok(self)
    }

    pub(super) fn pay_for_pnl_if_negative(&mut self, pnl: &M::Signed) -> crate::Result<&mut Self> {
        if pnl.is_negative() {
            self.pay_for_cost(
                pnl.unsigned_abs(),
                |processor, paid_in_collateral_amount, paid_in_secondary_output_amount, _| {
                    processor.pay_with_primary_pool(
                        &paid_in_collateral_amount.to_signed()?,
                        &paid_in_secondary_output_amount.to_signed()?,
                    )
                },
            )?;
        }
        Ok(self)
    }

    pub(super) fn add_price_impact_if_positive(
        &mut self,
        price_impact: &M::Signed,
    ) -> crate::Result<&mut Self> {
        if price_impact.is_positive() {
            // TODO: use min price to maximize the amount to reduce.
            let min_amount = price_impact
                .unsigned_abs()
                .checked_round_up_div(&self.state.index_token_price)
                .ok_or(crate::Error::Computation(
                    "calculating positive price impact amount",
                ))?;
            self.market
                .apply_delta_to_position_impact_pool(&min_amount.to_opposite_signed()?)?;

            // TODO: use max price to minimize the amount to pay.
            // The price has been validated to be non-zero.
            let deduction_amount_for_pool =
                price_impact.unsigned_abs() / self.state.pnl_token_price().clone();
            let is_pnl_token_long = self.state.is_pnl_token_long;
            self.market.apply_delta(
                is_pnl_token_long,
                &deduction_amount_for_pool.to_opposite_signed()?,
            )?;
            self.add_pnl_token_amount(deduction_amount_for_pool)?;
        }
        Ok(self)
    }

    pub(super) fn pay_for_price_impact_if_negative(
        &mut self,
        price_impact: &M::Signed,
    ) -> crate::Result<&mut Self> {
        if price_impact.is_negative() {
            self.pay_for_cost(
                price_impact.unsigned_abs(),
                |processor, paid_in_collateral_amount, paid_in_secondary_output_amount, _| {
                    processor.pay_with_primary_pool(
                        &paid_in_collateral_amount.to_signed()?,
                        &paid_in_secondary_output_amount.to_signed()?,
                    )?;
                    if !paid_in_collateral_amount.is_zero() {
                        let delta = paid_in_collateral_amount
                            .checked_mul_div(
                                processor.state.output_token_price(),
                                &processor.state.index_token_price,
                            )
                            .ok_or(crate::Error::Computation(
                                "calculating price impact paied in collateral (output) token",
                            ))?
                            .to_signed()?;
                        processor
                            .market
                            .apply_delta_to_position_impact_pool(&delta)?;
                    }
                    if !paid_in_secondary_output_amount.is_zero() {
                        let delta = paid_in_secondary_output_amount
                            .checked_mul_div(
                                processor.state.secondary_output_token_price(),
                                &processor.state.index_token_price,
                            )
                            .ok_or(crate::Error::Computation(
                                "calculating price impact paied in secondary output token",
                            ))?
                            .to_signed()?;
                        processor
                            .market
                            .apply_delta_to_position_impact_pool(&delta)?;
                    }
                    Ok(())
                },
            )?;
        }
        Ok(self)
    }

    pub(super) fn pay_for_fees_excluding_funding(
        &mut self,
        fees: &mut PositionFees<M::Num>,
    ) -> crate::Result<&mut Self> {
        use num_traits::CheckedMul;

        let cost_amount = fees.total_cost_excluding_funding()?;
        if !cost_amount.is_zero() {
            // TODO: use min price.
            let min_price = self.state.output_token_price();
            let cost = cost_amount
                .checked_mul(min_price)
                .ok_or(crate::Error::Computation("calculating total fee cost"))?;
            self.pay_for_cost(
                cost,
                |processor,
                 paid_in_collateral_amount,
                 paid_in_secondary_output_amount,
                 remaining_cost| {
                    if remaining_cost.is_zero() && paid_in_secondary_output_amount.is_zero() {
                        let is_collateral_token_long = processor.state.is_output_token_long;
                        processor.market.apply_delta(
                            is_collateral_token_long,
                            &fees.for_pool()?.to_signed()?,
                        )?;
                        processor.market.apply_delta_to_claimable_fee_pool(
                            is_collateral_token_long,
                            &fees.for_receiver().to_signed()?,
                        )?;
                        // TODO: apply ui fee.
                    } else {
                        processor.pay_with_primary_pool(
                            &paid_in_collateral_amount.to_signed()?,
                            &paid_in_secondary_output_amount.to_signed()?,
                        )?;
                        fees.clear_fees_excluding_funding();
                    }
                    Ok(())
                },
            )?;
        }
        Ok(self)
    }

    pub(super) fn pay_for_price_impact_diff(
        &mut self,
        price_impact_diff: &M::Num,
    ) -> crate::Result<&mut Self> {
        if !price_impact_diff.is_zero() {
            // TODO: apply to the debt.
            // self.debt.add_claimable_collateral_debt(price_impact_diff)?;
        }
        Ok(self)
    }
}
