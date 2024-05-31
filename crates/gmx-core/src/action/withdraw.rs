use crate::{
    num::MulDiv, params::Fees, utils, BalanceExt, Market, MarketExt, PnlFactorKind, PoolExt,
    PoolKind,
};
use num_traits::{CheckedAdd, Zero};

use super::Prices;

/// A withdrawal.
#[must_use]
pub struct Withdrawal<M: Market<DECIMALS>, const DECIMALS: u8> {
    market: M,
    params: WithdrawParams<M::Num>,
}

/// Withdraw params.
#[derive(Debug, Clone, Copy)]
pub struct WithdrawParams<T> {
    market_token_amount: T,
    prices: Prices<T>,
}

impl<T> WithdrawParams<T> {
    /// Get market token amount to burn.
    pub fn market_token_amount(&self) -> &T {
        &self.market_token_amount
    }

    /// Get long token price.
    pub fn long_token_price(&self) -> &T {
        &self.prices.long_token_price
    }

    /// Get short token price.
    pub fn short_token_price(&self) -> &T {
        &self.prices.short_token_price
    }
}

/// Report of the execution of withdrawal.
#[must_use = "`long_token_output` and `short_token_output` must use"]
#[derive(Debug, Clone, Copy)]
pub struct WithdrawReport<T> {
    params: WithdrawParams<T>,
    long_token_fees: Fees<T>,
    short_token_fees: Fees<T>,
    long_token_output: T,
    short_token_output: T,
}

impl<T> WithdrawReport<T> {
    /// Get withdraw params.
    pub fn params(&self) -> &WithdrawParams<T> {
        &self.params
    }

    /// Get long token fees.
    pub fn long_token_fees(&self) -> &Fees<T> {
        &self.long_token_fees
    }

    /// Get short token fees.
    pub fn short_token_fees(&self) -> &Fees<T> {
        &self.short_token_fees
    }

    /// Get the output amount of long tokens.
    pub fn long_token_output(&self) -> &T {
        &self.long_token_output
    }

    /// Get the output amount of short tokens.
    pub fn short_token_output(&self) -> &T {
        &self.short_token_output
    }
}

impl<const DECIMALS: u8, M: Market<DECIMALS>> Withdrawal<M, DECIMALS> {
    /// Create a new withdrawal from the given market.
    pub fn try_new(
        market: M,
        market_token_amount: M::Num,
        prices: Prices<M::Num>,
    ) -> crate::Result<Self> {
        if market_token_amount.is_zero() {
            return Err(crate::Error::EmptyWithdrawal);
        }
        prices.validate()?;
        Ok(Self {
            market,
            params: WithdrawParams {
                market_token_amount,
                prices,
            },
        })
    }

    /// Execute the withdrawal.
    pub fn execute(mut self) -> crate::Result<WithdrawReport<M::Num>> {
        let (mut long_token_amount, mut short_token_amount) = self.output_amounts()?;
        let long_token_fees = self.charge_fees(&mut long_token_amount)?;
        let short_token_fees = self.charge_fees(&mut short_token_amount)?;
        // Apply claimable fees delta.
        let pool = self
            .market
            .pool_mut(PoolKind::ClaimableFee)?
            .ok_or(crate::Error::MissingPoolKind(PoolKind::ClaimableFee))?;
        pool.apply_delta_amount(
            true,
            &long_token_fees
                .fee_receiver_amount()
                .clone()
                .try_into()
                .map_err(|_| crate::Error::Convert)?,
        )?;
        pool.apply_delta_amount(
            false,
            &short_token_fees
                .fee_receiver_amount()
                .clone()
                .try_into()
                .map_err(|_| crate::Error::Convert)?,
        )?;
        // Apply pool delta.
        // The delta must be the amount leaves the pool: -(amount_after_fees + fee_receiver_amount)
        let pool = self
            .market
            .pool_mut(PoolKind::Primary)?
            .ok_or(crate::Error::MissingPoolKind(PoolKind::Primary))?;

        let delta = long_token_fees
            .fee_receiver_amount()
            .checked_add(&long_token_amount)
            .ok_or(crate::Error::Overflow)?;
        pool.apply_delta_amount(true, &-delta.try_into().map_err(|_| crate::Error::Convert)?)?;

        let delta = short_token_fees
            .fee_receiver_amount()
            .checked_add(&short_token_amount)
            .ok_or(crate::Error::Overflow)?;
        pool.apply_delta_amount(
            false,
            &-delta.try_into().map_err(|_| crate::Error::Convert)?,
        )?;

        self.market.validate_reserve(&self.params.prices, true)?;
        self.market.validate_reserve(&self.params.prices, false)?;
        self.market.validate_max_pnl(
            &self.params.prices,
            PnlFactorKind::Deposit,
            PnlFactorKind::Deposit,
        )?;

        self.market.burn(&self.params.market_token_amount)?;

        Ok(WithdrawReport {
            params: self.params,
            long_token_fees,
            short_token_fees,
            long_token_output: long_token_amount,
            short_token_output: short_token_amount,
        })
    }

    fn output_amounts(&self) -> crate::Result<(M::Num, M::Num)> {
        let pool_value = self.market.pool_value(
            self.params.long_token_price(),
            self.params.short_token_price(),
        )?;
        if pool_value.is_zero() {
            return Err(crate::Error::invalid_pool_value("withdrawal"));
        }
        let total_supply = self.market.total_supply();
        let pool = self.market.primary_pool()?;
        let long_token_value = pool.long_usd_value(self.params.long_token_price())?;
        let short_token_value = pool.short_usd_value(self.params.short_token_price())?;
        let market_token_value = utils::market_token_amount_to_usd(
            &self.params.market_token_amount,
            &pool_value,
            &total_supply,
        )
        .ok_or(crate::Error::Computation("amount to usd"))?;
        let long_token_amount = market_token_value
            .checked_mul_div(&long_token_value, &pool_value)
            .ok_or(crate::Error::Computation("long token amount"))?
            / self.params.long_token_price().clone();
        let short_token_amount = market_token_value
            .checked_mul_div(&short_token_value, &pool_value)
            .ok_or(crate::Error::Computation("short token amount"))?
            / self.params.short_token_price().clone();
        Ok((long_token_amount, short_token_amount))
    }

    fn charge_fees(&self, amount: &mut M::Num) -> crate::Result<Fees<M::Num>> {
        let (amount_after_fees, fees) = self
            .market
            .swap_fee_params()
            .apply_fees(false, amount)
            .ok_or(crate::Error::Computation("apply fees"))?;
        *amount = amount_after_fees;
        Ok(fees)
    }
}

#[cfg(test)]
mod tests {
    use crate::{action::Prices, pool::Balance, test::TestMarket, Market, MarketExt};

    #[test]
    fn basic() -> crate::Result<()> {
        let mut market = TestMarket::<u64, 9>::default();
        let prices = Prices {
            index_token_price: 120,
            long_token_price: 120,
            short_token_price: 1,
        };
        market.deposit(1_000_000_000, 0, prices)?.execute()?;
        market.deposit(1_000_000_000, 0, prices)?.execute()?;
        market.deposit(0, 1_000_000_000, prices)?.execute()?;
        println!("{market:#?}");
        let before_supply = market.total_supply();
        let before_long_amount = market.primary_pool()?.long_amount()?;
        let before_short_amount = market.primary_pool()?.short_amount()?;
        let prices = Prices {
            index_token_price: 120,
            long_token_price: 120,
            short_token_price: 1,
        };
        let report = market.withdraw(1_000_000_000, prices)?.execute()?;
        println!("{report:#?}");
        println!("{market:#?}");
        assert_eq!(
            market.total_supply() + report.params.market_token_amount,
            before_supply
        );
        assert_eq!(
            market.primary_pool()?.long_amount()?
                + report.long_token_fees.fee_receiver_amount()
                + report.long_token_output,
            before_long_amount
        );
        assert_eq!(
            market.primary_pool()?.short_amount()?
                + report.short_token_fees.fee_receiver_amount()
                + report.short_token_output,
            before_short_amount
        );
        Ok(())
    }
}
