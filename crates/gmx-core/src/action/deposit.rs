use num_traits::{One, Zero};

use crate::{
    market::{Market, MarketExt},
    num::MulDiv,
    pool::Pool,
    utils,
};

/// A deposit.
#[must_use = "Action do nothing if not execute"]
pub struct Deposit<M: Market> {
    market: M,
    long_token_amount: M::Num,
    short_token_amount: M::Num,
    long_token_price: M::Num,
    short_token_price: M::Num,
    float_to_wei_divisor: M::Num,
}

impl<M: Market> Deposit<M> {
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
            long_token_amount,
            short_token_amount,
            long_token_price,
            short_token_price,
            // TODO: pass from args.
            float_to_wei_divisor: One::one(),
        })
    }

    fn price_impact(&self) -> (M::Signed, M::Num, M::Num) {
        let long_token_usd_value = self.long_token_amount.clone() * self.long_token_price.clone();
        let short_token_usd_value =
            self.short_token_amount.clone() * self.short_token_price.clone();
        // TODO: calculate the price impart.
        (Zero::zero(), long_token_usd_value, short_token_usd_value)
    }

    fn deposit(
        &mut self,
        is_long_token: bool,
        pool_value: M::Num,
        _price_impact: M::Signed,
    ) -> Result<M::Num, crate::Error> {
        let mut mint_amount = Zero::zero();
        let supply = self.market.total_supply();
        if pool_value.is_zero() && !supply.is_zero() {
            return Err(crate::Error::InvalidPoolValueForDeposit);
        }
        let (amount, price) = if is_long_token {
            (
                self.long_token_amount.clone(),
                self.long_token_price.clone(),
            )
        } else {
            (
                self.short_token_amount.clone(),
                self.short_token_price.clone(),
            )
        };
        // TODO: handle fees.
        // TODO: apply price impact.
        mint_amount = mint_amount
            + utils::usd_to_market_token_amount(
                amount.clone() * price,
                pool_value,
                supply.clone(),
                self.float_to_wei_divisor.clone(),
            )
            .ok_or(crate::Error::Computation)?;
        if is_long_token {
            self.market.pool_mut().apply_delta_to_long_token_amount(
                amount.try_into().map_err(|_| crate::Error::Convert)?,
            );
        } else {
            self.market.pool_mut().apply_delta_to_short_token_amount(
                amount.try_into().map_err(|_| crate::Error::Convert)?,
            );
        }
        Ok(mint_amount)
    }

    /// Execute.
    pub fn execute(mut self) -> Result<M::Num, crate::Error> {
        debug_assert!(
            !self.long_token_amount.is_zero() || !self.short_token_amount.is_zero(),
            "shouldn't be empty deposit"
        );
        let (price_impact, long_token_usd_value, short_token_usd_value) = self.price_impact();
        let mut market_token_to_mint = Zero::zero();
        let pool_value = self
            .market
            .pool_value(
                self.long_token_price.clone(),
                self.short_token_price.clone(),
            )
            .ok_or(crate::Error::Computation)?;
        if !self.long_token_amount.is_zero() {
            let price_impact = long_token_usd_value
                .clone()
                .checked_mul_div_with_signed_numberator(
                    price_impact.clone(),
                    long_token_usd_value.clone() + short_token_usd_value.clone(),
                )
                .ok_or(crate::Error::Computation)?;
            market_token_to_mint =
                market_token_to_mint + self.deposit(true, pool_value.clone(), price_impact)?;
        }
        if !self.short_token_amount.is_zero() {
            let price_impact = short_token_usd_value
                .clone()
                .checked_mul_div_with_signed_numberator(
                    price_impact,
                    long_token_usd_value + short_token_usd_value,
                )
                .ok_or(crate::Error::Computation)?;
            market_token_to_mint =
                market_token_to_mint + self.deposit(false, pool_value, price_impact)?;
        }
        Ok(market_token_to_mint)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::TestMarket;

    #[test]
    fn basic() -> Result<(), crate::Error> {
        let mut market = TestMarket::default();
        let amount = Deposit::try_new(&mut market, 1000, 0, 120, 1)?.execute()?;
        println!("{amount}");
        Ok(())
    }
}
