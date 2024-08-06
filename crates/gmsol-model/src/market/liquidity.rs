use num_traits::{CheckedAdd, CheckedSub};

use crate::{
    action::{deposit::Deposit, withdraw::Withdrawal, Prices},
    num::Unsigned,
    PnlFactorKind, PositionImpactMarket,
};

use super::{get_msg_by_side, BaseMarketExt, PositionImpactMarketExt, SwapMarketMut};

/// A market for providing liquidity.
pub trait LiquidityMarket<const DECIMALS: u8>:
    SwapMarketMut<DECIMALS> + PositionImpactMarket<DECIMALS>
{
    /// Get total supply of the market token.
    fn total_supply(&self) -> Self::Num;

    /// Get max pool value for deposit.
    fn max_pool_value_for_deposit(&self, is_long_token: bool) -> crate::Result<Self::Num>;

    /// Perform mint.
    fn mint(&mut self, amount: &Self::Num) -> Result<(), crate::Error>;

    /// Perform burn.
    fn burn(&mut self, amount: &Self::Num) -> crate::Result<()>;
}

impl<'a, M: LiquidityMarket<DECIMALS>, const DECIMALS: u8> LiquidityMarket<DECIMALS> for &'a mut M {
    fn total_supply(&self) -> Self::Num {
        (**self).total_supply()
    }

    fn max_pool_value_for_deposit(&self, is_long_token: bool) -> crate::Result<Self::Num> {
        (**self).max_pool_value_for_deposit(is_long_token)
    }

    fn mint(&mut self, amount: &Self::Num) -> Result<(), crate::Error> {
        (**self).mint(amount)
    }

    fn burn(&mut self, amount: &Self::Num) -> crate::Result<()> {
        (**self).burn(amount)
    }
}

/// Extension trait of [`LiquidityMarket`].
pub trait LiquidityMarketExt<const DECIMALS: u8>: LiquidityMarket<DECIMALS> {
    /// Create a [`Deposit`] action.
    fn deposit(
        &mut self,
        long_token_amount: Self::Num,
        short_token_amount: Self::Num,
        prices: Prices<Self::Num>,
    ) -> Result<Deposit<&mut Self, DECIMALS>, crate::Error>
    where
        Self: Sized,
    {
        Deposit::try_new(self, long_token_amount, short_token_amount, prices)
    }

    /// Create a [`Withdrawal`].
    fn withdraw(
        &mut self,
        market_token_amount: Self::Num,
        prices: Prices<Self::Num>,
    ) -> crate::Result<Withdrawal<&mut Self, DECIMALS>>
    where
        Self: Sized,
    {
        Withdrawal::try_new(self, market_token_amount, prices)
    }

    /// Validate (primary) pool value for deposit.
    fn validate_pool_value_for_deposit(
        &self,
        prices: &Prices<Self::Num>,
        is_long_token: bool,
    ) -> crate::Result<()> {
        let pool_value = self.pool_value_without_pnl_for_one_side(prices, is_long_token, true)?;
        let max_pool_value = self.max_pool_value_for_deposit(is_long_token)?;
        if pool_value > max_pool_value {
            Err(crate::Error::MaxPoolValueExceeded(get_msg_by_side(
                is_long_token,
            )))
        } else {
            Ok(())
        }
    }

    /// Get the usd value of primary pool.
    fn pool_value(
        &self,
        prices: &Prices<Self::Num>,
        pnl_factor: PnlFactorKind,
        maximize: bool,
    ) -> crate::Result<Self::Num> {
        // TODO: All pending values should be taken into consideration.
        let mut pool_value = {
            let long_value = self.pool_value_without_pnl_for_one_side(prices, true, maximize)?;
            let short_value = self.pool_value_without_pnl_for_one_side(prices, false, maximize)?;
            long_value
                .checked_add(&short_value)
                .ok_or(crate::Error::Overflow)?
        };

        // TODO: add total pending borrowing fees.

        // Deduct net pnl.
        let long_pnl = {
            let pnl = self.pnl(&prices.index_token_price, true, !maximize)?;
            self.cap_pnl(prices, true, &pnl, pnl_factor)?
        };
        let short_pnl = {
            let pnl = self.pnl(&prices.index_token_price, false, !maximize)?;
            self.cap_pnl(prices, false, &pnl, pnl_factor)?
        };
        let net_pnl = long_pnl
            .checked_add(&short_pnl)
            .ok_or(crate::Error::Computation("calculating net pnl"))?;
        pool_value = pool_value
            .checked_sub_with_signed(&net_pnl)
            .ok_or(crate::Error::Computation("deducting net pnl"))?;

        // Deduct impact pool value.
        let impact_pool_value = {
            let duration = self.passed_in_seconds_for_position_impact_distribution()?;
            self.pending_position_impact_pool_distribution_amount(duration)?
                .1
        };
        pool_value = pool_value
            .checked_sub(&impact_pool_value)
            .ok_or(crate::Error::Computation("deducting impact pool value"))?;

        Ok(pool_value)
    }
}

impl<M: LiquidityMarket<DECIMALS>, const DECIMALS: u8> LiquidityMarketExt<DECIMALS> for M {}
