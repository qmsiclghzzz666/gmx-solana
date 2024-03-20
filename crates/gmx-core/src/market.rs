use crate::{
    num::{MulDiv, UnsignedAbs},
    pool::{Pool, PoolExt},
};

/// A market.
pub trait Market {
    /// Unsigned number type used in the market.
    type Num: MulDiv<Signed = Self::Signed> + Clone;

    /// Signed number type used in the market.
    type Signed: UnsignedAbs<Self::Num> + TryFrom<Self::Num> + Clone;

    /// Pool type.
    type Pool: Pool<Num = Self::Num, Signed = Self::Signed>;

    /// Get the reference to the primary pool.
    fn pool(&self) -> &Self::Pool;

    /// Get the mutable reference to the primary pool.
    fn pool_mut(&mut self) -> &mut Self::Pool;

    /// Get the reference to the price impact pool.
    fn price_impact_pool(&self) -> &Self::Pool;

    /// Get the mutable reference to the price impact pool.
    fn price_impact_pool_mut(&mut self) -> &mut Self::Pool;

    /// Get total supply of the market token.
    fn total_supply(&self) -> &Self::Num;

    /// Perform mint.
    fn mint(&mut self, amount: Self::Num);
}

impl<'a, M: Market> Market for &'a mut M {
    type Num = M::Num;

    type Signed = M::Signed;

    type Pool = M::Pool;

    fn pool(&self) -> &Self::Pool {
        (**self).pool()
    }

    fn pool_mut(&mut self) -> &mut Self::Pool {
        (**self).pool_mut()
    }

    fn price_impact_pool(&self) -> &Self::Pool {
        (**self).price_impact_pool()
    }

    fn price_impact_pool_mut(&mut self) -> &mut Self::Pool {
        (**self).price_impact_pool_mut()
    }

    fn total_supply(&self) -> &Self::Num {
        (**self).total_supply()
    }

    fn mint(&mut self, amount: Self::Num) {
        (**self).mint(amount)
    }
}

/// Extension trait for [`Market`] with utils.
pub trait MarketExt: Market {
    /// Get the usd value of primary pool.
    fn pool_value(
        &self,
        long_token_price: Self::Num,
        short_token_price: Self::Num,
    ) -> Option<Self::Num> {
        let long_value = self.pool().long_token_usd_value(long_token_price);
        let short_value = self.pool().short_token_usd_value(short_token_price);
        Some(long_value + short_value)
    }
}

impl<M: Market> MarketExt for M {}
