use crate::{
    action::deposit::Deposit,
    num::{MulDiv, Num, UnsignedAbs},
    params::SwapImpactParams,
    pool::{Pool, PoolExt},
};
use num_traits::CheckedAdd;

/// A market.
pub trait Market {
    /// Unsigned number type used in the market.
    type Num: MulDiv<Signed = Self::Signed> + Num;

    /// Signed number type used in the market.
    type Signed: UnsignedAbs<Unsigned = Self::Num> + TryFrom<Self::Num> + Num;

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
    fn total_supply(&self) -> Self::Num;

    /// Perform mint.
    fn mint(&mut self, amount: &Self::Num) -> Result<(), crate::Error>;

    /// Usd value to market token amount divisor.
    ///
    /// One should make sure it is non-zero.
    fn usd_to_amount_divisor(&self) -> Self::Num;

    /// Unit value used in the market, i.e. the amount of `one`,
    /// not the amount unit of market token.
    fn unit(&self) -> Self::Num;

    /// Get the swap impact params.
    fn swap_impact_params(&self) -> SwapImpactParams<Self::Num>;
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

    fn total_supply(&self) -> Self::Num {
        (**self).total_supply()
    }

    fn mint(&mut self, amount: &Self::Num) -> Result<(), crate::Error> {
        (**self).mint(amount)
    }

    fn usd_to_amount_divisor(&self) -> Self::Num {
        (**self).usd_to_amount_divisor()
    }

    fn unit(&self) -> Self::Num {
        (**self).unit()
    }

    fn swap_impact_params(&self) -> SwapImpactParams<Self::Num> {
        (**self).swap_impact_params()
    }
}

/// Extension trait for [`Market`] with utils.
pub trait MarketExt: Market {
    /// Get the usd value of primary pool.
    fn pool_value(
        &self,
        long_token_price: &Self::Num,
        short_token_price: &Self::Num,
    ) -> Option<Self::Num> {
        let long_value = self.pool().long_token_usd_value(long_token_price)?;
        let short_value = self.pool().short_token_usd_value(short_token_price)?;
        long_value.checked_add(&short_value)
    }

    /// Create a [`Deposit`] action.
    fn deposit(
        &mut self,
        long_token_amount: Self::Num,
        short_token_amount: Self::Num,
        long_token_price: Self::Num,
        short_token_price: Self::Num,
    ) -> Result<Deposit<&mut Self>, crate::Error>
    where
        Self: Sized,
    {
        Deposit::try_new(
            self,
            long_token_amount,
            short_token_amount,
            long_token_price,
            short_token_price,
        )
    }
}

impl<M: Market> MarketExt for M {}
