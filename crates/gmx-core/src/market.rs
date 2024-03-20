use crate::pool::Pool;

/// A market.
pub trait Market {
    /// The number type used in the market.
    type Num;

    /// Pool type.
    type Pool: Pool<Num = Self::Num>;

    /// Get the mutable reference to the primary pool.
    fn pool_mut(&mut self) -> &mut Self::Pool;

    /// Get the mutable reference to the price impact pool.
    fn price_impact_pool_mut(&mut self) -> &mut Self::Pool;

    /// Get total supply of the market token.
    fn total_supply(&self) -> &Self::Num;
}

impl<'a, M: Market> Market for &'a mut M {
    type Num = M::Num;

    type Pool = M::Pool;

    fn pool_mut(&mut self) -> &mut Self::Pool {
        (**self).pool_mut()
    }

    fn price_impact_pool_mut(&mut self) -> &mut Self::Pool {
        (**self).price_impact_pool_mut()
    }

    fn total_supply(&self) -> &Self::Num {
        (**self).total_supply()
    }
}
