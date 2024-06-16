use crate::action::{deposit::Deposit, withdraw::Withdrawal, Prices};

use super::{get_msg_by_side, BaseMarketExt, SwapMarket};

/// A market for providing liquidity.
pub trait LiquidityMarket<const DECIMALS: u8>: SwapMarket<DECIMALS> {
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
        let pool_value = self.pool_value_for_one_side(prices, is_long_token, true)?;
        let max_pool_value = self.max_pool_value_for_deposit(is_long_token)?;
        if pool_value > max_pool_value {
            Err(crate::Error::MaxPoolAmountExceeded(get_msg_by_side(
                is_long_token,
            )))
        } else {
            Ok(())
        }
    }
}

impl<M: LiquidityMarket<DECIMALS>, const DECIMALS: u8> LiquidityMarketExt<DECIMALS> for M {}
