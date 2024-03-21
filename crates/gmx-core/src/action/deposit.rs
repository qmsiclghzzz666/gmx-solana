use num_traits::{CheckedAdd, CheckedMul, Zero};

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
        float_to_wei_divisor: M::Num,
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
            float_to_wei_divisor,
        })
    }

    fn price_impact(&self) -> Option<(M::Signed, M::Num, M::Num)> {
        let long_token_usd_value = self.long_token_amount.checked_mul(&self.long_token_price)?;
        let short_token_usd_value = self
            .short_token_amount
            .checked_mul(&self.short_token_price)?;
        // TODO: calculate the price impart.
        Some((Zero::zero(), long_token_usd_value, short_token_usd_value))
    }

    fn deposit(
        &mut self,
        is_long_token: bool,
        pool_value: M::Num,
        _price_impact: M::Signed,
    ) -> Result<M::Num, crate::Error> {
        let mut mint_amount: M::Num = Zero::zero();
        let supply = self.market.total_supply();
        if pool_value.is_zero() && !supply.is_zero() {
            return Err(crate::Error::InvalidPoolValueForDeposit);
        }
        let (amount, price) = if is_long_token {
            (&self.long_token_amount, &self.long_token_price)
        } else {
            (&self.short_token_amount, &self.short_token_price)
        };
        // TODO: handle fees.
        // TODO: apply price impact.
        mint_amount = mint_amount
            .checked_add(
                &utils::usd_to_market_token_amount(
                    amount.checked_mul(price).ok_or(crate::Error::Computation)?,
                    pool_value,
                    supply.clone(),
                    self.float_to_wei_divisor.clone(),
                )
                .ok_or(crate::Error::Computation)?,
            )
            .ok_or(crate::Error::Computation)?;
        if is_long_token {
            self.market.pool_mut().apply_delta_to_long_token_amount(
                amount
                    .clone()
                    .try_into()
                    .map_err(|_| crate::Error::Convert)?,
            )?;
        } else {
            self.market.pool_mut().apply_delta_to_short_token_amount(
                amount
                    .clone()
                    .try_into()
                    .map_err(|_| crate::Error::Convert)?,
            )?;
        }
        Ok(mint_amount)
    }

    /// Execute.
    pub fn execute(mut self) -> Result<(), crate::Error> {
        debug_assert!(
            !self.long_token_amount.is_zero() || !self.short_token_amount.is_zero(),
            "shouldn't be empty deposit"
        );
        let (price_impact, long_token_usd_value, short_token_usd_value) =
            self.price_impact().ok_or(crate::Error::Computation)?;
        let mut market_token_to_mint: M::Num = Zero::zero();
        let pool_value = self
            .market
            .pool_value(&self.long_token_price, &self.short_token_price)
            .ok_or(crate::Error::Computation)?;
        if !self.long_token_amount.is_zero() {
            let price_impact = long_token_usd_value
                .clone()
                .checked_mul_div_with_signed_numberator(
                    price_impact.clone(),
                    long_token_usd_value
                        .checked_add(&short_token_usd_value)
                        .ok_or(crate::Error::Computation)?,
                )
                .ok_or(crate::Error::Computation)?;
            market_token_to_mint = market_token_to_mint
                .checked_add(&self.deposit(true, pool_value.clone(), price_impact)?)
                .ok_or(crate::Error::Computation)?;
        }
        if !self.short_token_amount.is_zero() {
            let price_impact = short_token_usd_value
                .clone()
                .checked_mul_div_with_signed_numberator(
                    price_impact,
                    long_token_usd_value
                        .checked_add(&short_token_usd_value)
                        .ok_or(crate::Error::Computation)?,
                )
                .ok_or(crate::Error::Computation)?;
            market_token_to_mint = market_token_to_mint
                .checked_add(&self.deposit(false, pool_value, price_impact)?)
                .ok_or(crate::Error::Computation)?;
        }
        self.market.mint(&market_token_to_mint)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::TestMarket;

    #[test]
    fn basic() -> Result<(), crate::Error> {
        const FLOAT_TO_WEI_DIVISOR: u64 = 1;
        let mut market = TestMarket::default();
        Deposit::try_new(&mut market, 1000, 0, 120, 1, FLOAT_TO_WEI_DIVISOR)?.execute()?;
        Deposit::try_new(&mut market, 0, 2000, 120, 1, FLOAT_TO_WEI_DIVISOR)?.execute()?;
        Deposit::try_new(&mut market, 100, 0, 100, 1, FLOAT_TO_WEI_DIVISOR)?.execute()?;
        println!("{market:?}, {}", market.pool_value(&200, &1).unwrap());
        Deposit::try_new(&mut market, 100, 0, 200, 1, FLOAT_TO_WEI_DIVISOR)?.execute()?;
        println!("{market:?}, {}", market.pool_value(&200, &1).unwrap());
        Deposit::try_new(&mut market, 100, 0, 200, 1, FLOAT_TO_WEI_DIVISOR)?.execute()?;
        Ok(())
    }
}
