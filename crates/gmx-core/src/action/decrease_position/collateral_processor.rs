use crate::{
    action::Prices,
    num::{MulDiv, Num, Unsigned, UnsignedAbs},
    params::fee::PositionFees,
    Market, MarketExt,
};

use num_traits::{CheckedAdd, Signed, Zero};

use super::debt::Debt;

/// Collateral Processor.
#[must_use]
pub(super) struct CollateralProcessor<'a, M: Market<DECIMALS>, const DECIMALS: u8> {
    market: &'a mut M,
    state: State<M::Num>,
    debt: Debt<M::Num>,
    price_impact_debt: Debt<M::Num>,
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
    fn pay_for_cost(&mut self, cost: &mut T) -> crate::Result<(T, T, T)> {
        let mut paid_in_output_token = Zero::zero();
        let mut paid_in_collateral_token = Zero::zero();
        let mut paid_in_secondary_output_token = Zero::zero();

        if cost.is_zero() {
            return Ok((
                paid_in_output_token,
                paid_in_collateral_token,
                paid_in_secondary_output_token,
            ));
        }

        let mut remaining_cost_in_output_token =
            cost.checked_round_up_div(self.output_token_price()).ok_or(
                crate::Error::Computation("initializing cost in output tokens"),
            )?;

        if !self.output_amount.is_zero() {
            if self.output_amount > remaining_cost_in_output_token {
                paid_in_output_token = paid_in_output_token
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
                paid_in_output_token = paid_in_output_token
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
                paid_in_output_token,
                paid_in_collateral_token,
                paid_in_secondary_output_token,
            ));
        }

        if !self.remaining_collateral_amount.is_zero() {
            if self.remaining_collateral_amount > remaining_cost_in_output_token {
                paid_in_collateral_token = paid_in_collateral_token
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
                paid_in_collateral_token = paid_in_collateral_token
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
                paid_in_output_token,
                paid_in_collateral_token,
                paid_in_secondary_output_token,
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
                paid_in_secondary_output_token = paid_in_secondary_output_token
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
                paid_in_secondary_output_token = paid_in_secondary_output_token
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
            paid_in_output_token,
            paid_in_collateral_token,
            paid_in_secondary_output_token,
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
            debt: Default::default(),
            price_impact_debt: Default::default(),
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

    pub(super) fn apply_pnl(&mut self, pnl: &M::Signed) -> crate::Result<&mut Self> {
        if pnl.is_positive() {
            // TODO: pick max pnl token price.
            let deduction_amount_for_pool =
                pnl.unsigned_abs() / self.state.pnl_token_price().clone();

            self.market.apply_delta(
                self.state.is_pnl_token_long,
                &deduction_amount_for_pool.to_opposite_signed()?,
            )?;

            self.add_pnl_token_amount(deduction_amount_for_pool)?;
        } else {
            self.debt.add_pool_debt(&pnl.unsigned_abs())?;
        }
        Ok(self)
    }

    pub(super) fn apply_price_impact(
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
            self.market.apply_delta(
                self.state.is_pnl_token_long,
                &deduction_amount_for_pool.to_opposite_signed()?,
            )?;
            self.add_pnl_token_amount(deduction_amount_for_pool)?;
        } else if price_impact.is_negative() {
            self.price_impact_debt
                .add_pool_debt(&price_impact.unsigned_abs())?;
        }
        Ok(self)
    }

    pub(super) fn apply_price_impact_diff(
        &mut self,
        price_impact_diff: &M::Num,
    ) -> crate::Result<&mut Self> {
        if !price_impact_diff.is_zero() {
            // TODO: apply to the debt.
            // self.debt.add_claimable_collateral_debt(price_impact_diff)?;
        }
        Ok(self)
    }

    pub(super) fn apply_fees(&mut self, fees: &PositionFees<M::Num>) -> crate::Result<&mut Self> {
        use num_traits::CheckedMul;

        let cost_amount = fees.total_cost_excluding_funding()?;
        if !cost_amount.is_zero() {
            // TODO: use min price.
            let min_price = self.state.output_token_price();
            let cost = cost_amount
                .checked_mul(min_price)
                .ok_or(crate::Error::Computation("calculating total fee cost"))?;
            self.debt.add_pool_debt(&cost)?;
        }
        // TODO: apply funding fees.
        Ok(self)
    }

    fn pay_for_debt_for_pool(&mut self) -> crate::Result<()> {
        let (paid_in_collateral_token, paid_in_secondary_output_token) =
            self.debt.pay_for_pool_debt(
                |cost| {
                    let (_, paid_in_collateral_token, paid_in_secondary_output_token) =
                        self.state.pay_for_cost(cost)?;
                    Ok((paid_in_collateral_token, paid_in_secondary_output_token))
                },
                self.is_insolvent_close_allowed,
            )?;
        if !paid_in_collateral_token.is_zero() {
            self.market.apply_delta(
                self.state.is_output_token_long,
                &paid_in_collateral_token.to_signed()?,
            )?;
        }
        if !paid_in_secondary_output_token.is_zero() {
            self.market.apply_delta(
                self.state.is_pnl_token_long,
                &paid_in_secondary_output_token.to_signed()?,
            )?;
        }
        Ok(())
    }

    fn pay_for_debt_for_claimable_collateral(&mut self) -> crate::Result<()> {
        let (paid_in_collateral_token, paid_in_secondary_output_token) =
            self.debt.pay_for_claimable_collateral_debt(
                |cost| {
                    let (_, paid_in_collateral_token, paid_in_secondary_output_token) =
                        self.state.pay_for_cost(cost)?;
                    Ok((paid_in_collateral_token, paid_in_secondary_output_token))
                },
                self.is_insolvent_close_allowed,
            )?;
        if !paid_in_collateral_token.is_zero() {
            // TODO: pay to claimable collateral pool.
            // self.market.apply_delta(
            //     self.state.is_output_token_long,
            //     &paid_in_collateral_token.to_signed()?,
            // )?;
        }
        if !paid_in_secondary_output_token.is_zero() {
            // TODO: pay to claimable collateral pool.
            // self.market.apply_delta(
            //     self.state.is_pnl_token_long,
            //     &paid_in_secondary_output_token.to_signed()?,
            // )?;
        }
        Ok(())
    }

    fn pay_for_price_impact_debt_for_pool(&mut self) -> crate::Result<()> {
        let (paid_in_collateral_token, paid_in_secondary_output_token) =
            self.price_impact_debt.pay_for_pool_debt(
                |cost| {
                    let (_, paid_in_collateral_token, paid_in_secondary_output_token) =
                        self.state.pay_for_cost(cost)?;
                    Ok((paid_in_collateral_token, paid_in_secondary_output_token))
                },
                self.is_insolvent_close_allowed,
            )?;
        if !paid_in_collateral_token.is_zero() {
            self.market.apply_delta(
                self.state.is_output_token_long,
                &paid_in_collateral_token.to_signed()?,
            )?;
            let delta = paid_in_collateral_token
                .checked_mul_div(
                    self.state.output_token_price(),
                    &self.state.index_token_price,
                )
                .ok_or(crate::Error::Computation(
                    "calculating price impact paied in collateral (output) token",
                ))?
                .to_signed()?;
            self.market.apply_delta_to_position_impact_pool(&delta)?;
        }
        if !paid_in_secondary_output_token.is_zero() {
            self.market.apply_delta(
                self.state.is_pnl_token_long,
                &paid_in_secondary_output_token.to_signed()?,
            )?;
            let delta = paid_in_collateral_token
                .checked_mul_div(
                    self.state.secondary_output_token_price(),
                    &self.state.index_token_price,
                )
                .ok_or(crate::Error::Computation(
                    "calculating price impact paied in secondary output token",
                ))?
                .to_signed()?;
            self.market.apply_delta_to_position_impact_pool(&delta)?;
        }
        Ok(())
    }

    fn pay_for_debt(&mut self) -> crate::Result<&mut Self> {
        self.pay_for_debt_for_pool()?;
        self.pay_for_debt_for_claimable_collateral()?;
        self.pay_for_price_impact_debt_for_pool()?;
        Ok(self)
    }

    pub(super) fn process(mut self) -> crate::Result<ProcessReport<M::Num>> {
        self.pay_for_debt()?;
        if self.state.is_output_token_long == self.state.is_pnl_token_long {
            self.state.output_amount = self
                .state
                .output_amount
                .checked_add(&self.state.secondary_output_amount)
                .ok_or(crate::Error::Computation(
                    "merge amounts when tokens are the same",
                ))?;
            self.state.secondary_output_amount = Zero::zero();
        }
        Ok(ProcessReport {
            output_amount: self.state.output_amount,
            remaining_collateral_amount: self.state.remaining_collateral_amount,
            secondary_output_amount: self.state.secondary_output_amount,
        })
    }
}
