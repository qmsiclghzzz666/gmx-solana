use crate::{market::Market, pool::Pool};

/// Test Pool.
#[derive(Debug, Default)]
pub struct TestPool {
    long_token_amount: u64,
    short_token_amount: u64,
}

impl Pool for TestPool {
    type Num = u64;

    type Signed = i64;

    fn long_token_amount(&self) -> Self::Num {
        self.long_token_amount
    }

    fn short_token_amount(&self) -> Self::Num {
        self.short_token_amount
    }

    fn apply_delta_to_long_token_amount(
        &mut self,
        delta: Self::Signed,
    ) -> Result<(), crate::Error> {
        if delta > 0 {
            self.long_token_amount += delta.unsigned_abs();
        } else {
            self.long_token_amount -= delta.unsigned_abs();
        }
        Ok(())
    }

    fn apply_delta_to_short_token_amount(
        &mut self,
        delta: Self::Signed,
    ) -> Result<(), crate::Error> {
        if delta > 0 {
            self.short_token_amount += delta.unsigned_abs();
        } else {
            self.short_token_amount -= delta.unsigned_abs();
        }
        Ok(())
    }
}

/// Test Market.
#[derive(Debug, Default)]
pub struct TestMarket {
    primary: TestPool,
    price_impact: TestPool,
    total_supply: u64,
}

impl Market for TestMarket {
    type Num = u64;

    type Signed = i64;

    type Pool = TestPool;

    fn pool(&self) -> &Self::Pool {
        &self.primary
    }

    fn pool_mut(&mut self) -> &mut Self::Pool {
        &mut self.primary
    }

    fn price_impact_pool(&self) -> &Self::Pool {
        &self.price_impact
    }

    fn price_impact_pool_mut(&mut self) -> &mut Self::Pool {
        &mut self.price_impact
    }

    fn total_supply(&self) -> &Self::Num {
        &self.total_supply
    }

    fn mint(&mut self, amount: &Self::Num) -> Result<(), crate::Error> {
        self.total_supply = self
            .total_supply
            .checked_add(*amount)
            .ok_or(crate::Error::Overflow)?;
        println!("minted: {amount}");
        Ok(())
    }
}
