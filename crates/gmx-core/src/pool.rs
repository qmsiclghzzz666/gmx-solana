use crate::num::Num;
use num_traits::CheckedMul;

/// A pool for holding tokens.
pub trait Pool {
    /// Unsigned number type of the pool.
    type Num: Num;

    /// Signed number type of the pool.
    type Signed;

    // /// Signed number type of the pool.
    // type Signed: Signed;

    /// Get the long token amount.
    fn long_token_amount(&self) -> Self::Num;

    /// Get the short token amount.
    fn short_token_amount(&self) -> Self::Num;

    /// Apply delta to long token pool amount.
    fn apply_delta_to_long_token_amount(
        &mut self,
        delta: &Self::Signed,
    ) -> Result<(), crate::Error>;

    /// Apply delta to short token pool amount.
    fn apply_delta_to_short_token_amount(
        &mut self,
        delta: &Self::Signed,
    ) -> Result<(), crate::Error>;
}

/// Extension trait for [`Pool`] with utils.
pub trait PoolExt: Pool {
    /// Get the long token value in USD.
    fn long_token_usd_value(&self, price: &Self::Num) -> Option<Self::Num> {
        self.long_token_amount().checked_mul(price)
    }

    /// Get the short token value in USD.
    fn short_token_usd_value(&self, price: &Self::Num) -> Option<Self::Num> {
        self.short_token_amount().checked_mul(price)
    }
}

impl<P: Pool> PoolExt for P {}
