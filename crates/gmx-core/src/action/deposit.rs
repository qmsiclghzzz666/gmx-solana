use num_traits::{CheckedAdd, CheckedMul, CheckedSub, Signed, Zero};

use crate::{
    fixed::FixedPointOps,
    market::{Market, MarketExt},
    num::{MulDiv, Num},
    params::{Fees, SwapImpactParams},
    pool::PoolKind,
    utils, PoolExt,
};

/// A deposit.
#[must_use]
pub struct Deposit<M: Market<DECIMALS>, const DECIMALS: u8> {
    market: M,
    params: DepositParams<M::Num>,
}

/// Deposit params.
#[derive(Debug, Clone, Copy)]
pub struct DepositParams<T> {
    long_token_amount: T,
    short_token_amount: T,
    long_token_price: T,
    short_token_price: T,
}

impl<T> DepositParams<T> {
    /// Get long token amount.
    pub fn long_token_amount(&self) -> &T {
        &self.long_token_amount
    }

    /// Get short token amount.
    pub fn short_token_amount(&self) -> &T {
        &self.short_token_amount
    }

    /// Get long token price.
    pub fn long_token_price(&self) -> &T {
        &self.long_token_price
    }

    /// Get short token price.
    pub fn short_token_price(&self) -> &T {
        &self.short_token_price
    }
}

/// Report the execution of deposit.
#[derive(Debug, Clone, Copy)]
pub struct DepositReport<T>
where
    T: MulDiv,
{
    params: DepositParams<T>,
    minted: T,
    price_impact: T::Signed,
    fees: [Fees<T>; 2],
}

impl<T> DepositReport<T>
where
    T: MulDiv,
{
    fn new(
        params: DepositParams<T>,
        price_impact: T::Signed,
        minted: T,
        fees: [Fees<T>; 2],
    ) -> Self {
        Self {
            params,
            minted,
            price_impact,
            fees,
        }
    }

    /// Get minted.
    pub fn minted(&self) -> &T {
        &self.minted
    }

    /// Get price impact.
    pub fn price_impact(&self) -> &T::Signed {
        &self.price_impact
    }

    /// Get the deposit params.
    pub fn params(&self) -> &DepositParams<T> {
        &self.params
    }

    /// Get long token fees.
    pub fn long_token_fees(&self) -> &Fees<T> {
        &self.fees[0]
    }

    /// Get short token fees.
    pub fn short_token_fees(&self) -> &Fees<T> {
        &self.fees[1]
    }
}

struct PoolParams<T> {
    long_token_usd_value: T,
    short_token_usd_value: T,
    delta_long_token_usd_value: T,
    delta_short_token_usd_value: T,
    next_long_token_usd_value: T,
    next_short_token_usd_value: T,
}

impl<T> PoolParams<T>
where
    T: MulDiv + Num,
{
    #[inline]
    fn initial_diff_usd(&self) -> T {
        self.long_token_usd_value
            .clone()
            .diff(self.short_token_usd_value.clone())
    }

    #[inline]
    fn next_diff_usd(&self) -> T {
        self.next_long_token_usd_value
            .clone()
            .diff(self.next_short_token_usd_value.clone())
    }

    #[inline]
    fn is_same_side_rebalance(&self) -> bool {
        (self.long_token_usd_value <= self.short_token_usd_value)
            == (self.next_long_token_usd_value <= self.next_short_token_usd_value)
    }

    fn price_impact<const DECIMALS: u8>(&self, params: &SwapImpactParams<T>) -> Option<T::Signed>
    where
        T: FixedPointOps<DECIMALS>,
    {
        if self.is_same_side_rebalance() {
            self.price_impact_for_same_side_rebalance(params)
        } else {
            self.price_impact_for_cross_over_rebalance(params)
        }
    }

    #[inline]
    fn price_impact_for_same_side_rebalance<const DECIMALS: u8>(
        &self,
        params: &SwapImpactParams<T>,
    ) -> Option<T::Signed>
    where
        T: FixedPointOps<DECIMALS>,
    {
        let initial = self.initial_diff_usd();
        let next = self.next_diff_usd();
        let has_positive_impact = next < initial;
        let (positive_factor, negative_factor) = params.adjusted_factors();

        let factor = if has_positive_impact {
            positive_factor
        } else {
            negative_factor
        };
        let exponent_factor = params.exponent();

        let initial = utils::apply_factors(initial, factor.clone(), exponent_factor.clone())?;
        let next = utils::apply_factors(next, factor.clone(), exponent_factor.clone())?;
        let delta: T::Signed = initial.diff(next).try_into().ok()?;
        Some(if has_positive_impact { delta } else { -delta })
    }

    #[inline]
    fn price_impact_for_cross_over_rebalance<const DECIMALS: u8>(
        &self,
        params: &SwapImpactParams<T>,
    ) -> Option<T::Signed>
    where
        T: FixedPointOps<DECIMALS>,
    {
        let initial = self.initial_diff_usd();
        let next = self.next_diff_usd();
        let (positive_factor, negative_factor) = params.adjusted_factors();
        let exponent_factor = params.exponent();
        let positive_impact =
            utils::apply_factors(initial, positive_factor.clone(), exponent_factor.clone())?;
        let negative_impact =
            utils::apply_factors(next, negative_factor.clone(), exponent_factor.clone())?;
        let has_positive_impact = positive_impact > negative_impact;
        let delta: T::Signed = positive_impact.diff(negative_impact).try_into().ok()?;
        Some(if has_positive_impact { delta } else { -delta })
    }
}

impl<const DECIMALS: u8, M: Market<DECIMALS>> Deposit<M, DECIMALS> {
    /// Create a new deposit to the given market.
    pub fn try_new(
        market: M,
        long_token_amount: M::Num,
        short_token_amount: M::Num,
        long_token_price: M::Num,
        short_token_price: M::Num,
    ) -> Result<Self, crate::Error> {
        if long_token_amount.is_zero() && short_token_amount.is_zero() {
            return Err(crate::Error::EmptyDeposit);
        }
        Ok(Self {
            market,
            params: DepositParams {
                long_token_amount,
                short_token_amount,
                long_token_price,
                short_token_price,
            },
        })
    }

    fn pool_params(&self) -> Option<PoolParams<M::Num>> {
        let long_token_usd_value = self
            .market
            .pool(PoolKind::Primary)
            .ok()??
            .long_token_usd_value(&self.params.long_token_price)
            .ok()??;
        let short_token_usd_value = self
            .market
            .pool(PoolKind::Primary)
            .ok()??
            .short_token_usd_value(&self.params.short_token_price)
            .ok()??;
        let delta_long_token_usd_value = self
            .params
            .long_token_amount
            .checked_mul(&self.params.long_token_price)?;
        let delta_short_token_usd_value = self
            .params
            .short_token_amount
            .checked_mul(&self.params.short_token_price)?;
        Some(PoolParams {
            next_long_token_usd_value: long_token_usd_value
                .checked_add(&delta_long_token_usd_value)?,
            next_short_token_usd_value: short_token_usd_value
                .checked_add(&delta_short_token_usd_value)?,
            long_token_usd_value,
            short_token_usd_value,
            delta_long_token_usd_value,
            delta_short_token_usd_value,
        })
    }

    /// Get the price impact USD value.
    fn price_impact(&self) -> Option<(M::Signed, M::Num, M::Num)> {
        let params = self.pool_params()?;
        let price_impact = params.price_impact(&self.market.swap_impact_params())?;
        Some((
            price_impact,
            params.delta_long_token_usd_value,
            params.delta_short_token_usd_value,
        ))
    }

    /// Charge swap fees.
    ///
    /// The `amount` will become the amount after fees.
    fn charge_fees(
        &self,
        is_positive_impact: bool,
        amount: &mut M::Num,
    ) -> crate::Result<Fees<M::Num>> {
        let (amount_after_fees, fees) = self
            .market
            .swap_fee_params()
            .apply_fees(is_positive_impact, amount)
            .ok_or(crate::Error::Computation)?;
        *amount = amount_after_fees;
        Ok(fees)
    }

    fn execute_deposit(
        &mut self,
        is_long_token: bool,
        pool_value: M::Num,
        mut price_impact: M::Signed,
    ) -> Result<(M::Num, Fees<M::Num>), crate::Error> {
        let mut mint_amount: M::Num = Zero::zero();
        let supply = self.market.total_supply();
        if pool_value.is_zero() && !supply.is_zero() {
            return Err(crate::Error::InvalidPoolValueForDeposit);
        }
        let (mut amount, price, opposite_price) = if is_long_token {
            (
                self.params.long_token_amount.clone(),
                &self.params.long_token_price,
                &self.params.short_token_price,
            )
        } else {
            (
                self.params.short_token_amount.clone(),
                &self.params.short_token_price,
                &self.params.long_token_price,
            )
        };
        let fees = self.charge_fees(price_impact.is_positive(), &mut amount)?;
        self.market
            .pool_mut(PoolKind::ClaimableFee)?
            .ok_or(crate::Error::MissingPoolKind(PoolKind::ClaimableFee))?
            .apply_delta_amount(
                is_long_token,
                &fees
                    .fee_receiver_amount()
                    .clone()
                    .try_into()
                    .map_err(|_| crate::Error::Convert)?,
            )?;
        // FIXME: will this case happend in our implementation?
        if price_impact.is_positive() && supply.is_zero() {
            price_impact = Zero::zero();
        }
        if price_impact.is_positive() {
            let positive_impact_amount = self.market.apply_swap_impact_value_with_cap(
                !is_long_token,
                opposite_price,
                &price_impact,
            )?;
            mint_amount = mint_amount
                .checked_add(
                    &utils::usd_to_market_token_amount(
                        positive_impact_amount
                            .checked_mul(opposite_price)
                            .ok_or(crate::Error::Computation)?,
                        pool_value.clone(),
                        supply.clone(),
                        self.market.usd_to_amount_divisor(),
                    )
                    .ok_or(crate::Error::Computation)?,
                )
                .ok_or(crate::Error::Computation)?;
            self.market.apply_delta(
                !is_long_token,
                &positive_impact_amount
                    .try_into()
                    .map_err(|_| crate::Error::Convert)?,
            )?;
            // TODO: validate the amounts.
        } else if price_impact.is_negative() {
            let negative_impact_amount = self.market.apply_swap_impact_value_with_cap(
                is_long_token,
                price,
                &price_impact,
            )?;
            amount = amount
                .checked_sub(&negative_impact_amount)
                .ok_or(crate::Error::Underflow)?;
        }
        mint_amount = mint_amount
            .checked_add(
                &utils::usd_to_market_token_amount(
                    amount.checked_mul(price).ok_or(crate::Error::Computation)?,
                    pool_value,
                    supply.clone(),
                    self.market.usd_to_amount_divisor(),
                )
                .ok_or(crate::Error::Computation)?,
            )
            .ok_or(crate::Error::Computation)?;
        self.market.apply_delta(
            is_long_token,
            &(amount
                .checked_add(fees.fee_amount_for_pool())
                .ok_or(crate::Error::Overflow)?)
            .clone()
            .try_into()
            .map_err(|_| crate::Error::Convert)?,
        )?;
        // TODO: validate the amounts.
        Ok((mint_amount, fees))
    }

    /// Execute.
    pub fn execute(mut self) -> Result<DepositReport<M::Num>, crate::Error> {
        debug_assert!(
            !self.params.long_token_amount.is_zero() || !self.params.short_token_amount.is_zero(),
            "shouldn't be empty deposit"
        );
        // TODO: validate first deposit.
        let (price_impact, long_token_usd_value, short_token_usd_value) =
            self.price_impact().ok_or(crate::Error::Computation)?;
        let mut market_token_to_mint: M::Num = Zero::zero();
        let pool_value = self
            .market
            .pool_value(
                &self.params.long_token_price,
                &self.params.short_token_price,
            )?
            .ok_or(crate::Error::Computation)?;
        let mut all_fees = [Default::default(), Default::default()];
        if !self.params.long_token_amount.is_zero() {
            let price_impact = long_token_usd_value
                .clone()
                .checked_mul_div_with_signed_numberator(
                    &price_impact,
                    &long_token_usd_value
                        .checked_add(&short_token_usd_value)
                        .ok_or(crate::Error::Computation)?,
                )
                .ok_or(crate::Error::Computation)?;
            let (mint_amount, fees) =
                self.execute_deposit(true, pool_value.clone(), price_impact)?;
            market_token_to_mint = market_token_to_mint
                .checked_add(&mint_amount)
                .ok_or(crate::Error::Computation)?;
            all_fees[0] = fees;
        }
        if !self.params.short_token_amount.is_zero() {
            let price_impact = short_token_usd_value
                .clone()
                .checked_mul_div_with_signed_numberator(
                    &price_impact,
                    &long_token_usd_value
                        .checked_add(&short_token_usd_value)
                        .ok_or(crate::Error::Computation)?,
                )
                .ok_or(crate::Error::Computation)?;
            let (mint_amount, fees) = self.execute_deposit(false, pool_value, price_impact)?;
            market_token_to_mint = market_token_to_mint
                .checked_add(&mint_amount)
                .ok_or(crate::Error::Computation)?;
            all_fees[1] = fees;
        }
        self.market.mint(&market_token_to_mint)?;
        Ok(DepositReport::new(
            self.params,
            price_impact,
            market_token_to_mint,
            all_fees,
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::{market::MarketExt, test::TestMarket};

    #[test]
    fn basic() -> Result<(), crate::Error> {
        let mut market = TestMarket::<u64, 8>::default();
        println!("{:#?}", market.deposit(100_000_000, 0, 120, 1)?.execute()?);
        println!("{:#?}", market.deposit(100_000_000, 0, 120, 1)?.execute()?);
        println!("{:#?}", market.deposit(0, 100_000_000, 120, 1)?.execute()?);
        println!("{market:#?}");
        Ok(())
    }

    #[test]
    fn sequence() -> crate::Result<()> {
        let mut market_1 = TestMarket::<u64, 8>::default();
        println!(
            "{:#?}",
            market_1.deposit(1_000_000_000, 0, 120, 1)?.execute()?
        );
        println!(
            "{:#?}",
            market_1.deposit(1_000_000_000, 0, 120, 1)?.execute()?
        );
        let mut market_2 = TestMarket::<u64, 8>::default();
        println!(
            "{:#?}",
            market_2.deposit(2_000_000_000, 0, 120, 1)?.execute()?
        );
        Ok(())
    }

    #[cfg(feature = "u128")]
    #[test]
    fn basic_u128() -> Result<(), crate::Error> {
        let mut market = TestMarket::<u128, 20>::default();
        println!(
            "{:#?}",
            market
                .deposit(100_000_000, 0, 120_000_000_000_000, 1)?
                .execute()?
        );
        println!(
            "{:#?}",
            market
                .deposit(100_000_000, 0, 120_000_000_000_000, 1)?
                .execute()?
        );
        println!(
            "{:#?}",
            market
                .deposit(0, 100_000_000, 120_000_000_000_000, 1_000_000_000_000)?
                .execute()?
        );
        println!("{market:#?}");
        Ok(())
    }
}
