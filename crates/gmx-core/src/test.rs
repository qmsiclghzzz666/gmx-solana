use crate::{market::Market, pool::Pool};

/// Test Pool.
#[derive(Debug, Default)]
pub struct TestPool {
    long_token_amount: u64,
    short_token_amount: u64,
}

impl Pool for TestPool {
    type Num = u64;

    fn long_token_amount_mut(&mut self) -> &mut Self::Num {
        &mut self.long_token_amount
    }

    fn short_token_amount_mut(&mut self) -> &mut Self::Num {
        &mut self.short_token_amount
    }
}

/// Test Market.
#[derive(Default)]
pub struct TestMarket {
    primary: TestPool,
    price_impact: TestPool,
    total_supply: u64,
}

impl Market for TestMarket {
    type Num = u64;

    type Pool = TestPool;

    fn pool_mut(&mut self) -> &mut Self::Pool {
        &mut self.primary
    }

    fn price_impact_pool_mut(&mut self) -> &mut Self::Pool {
        &mut self.price_impact
    }

    fn total_supply(&self) -> &Self::Num {
        &self.total_supply
    }
}
