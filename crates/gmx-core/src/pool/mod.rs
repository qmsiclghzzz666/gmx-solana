use std::fmt;

use crate::num::{Num, Unsigned};
use num_traits::CheckedMul;

use self::delta::PoolDelta;

/// Delta.
pub mod delta;

/// A pool for holding tokens.
pub trait Pool {
    /// Unsigned number type of the pool.
    type Num: Num + Unsigned<Signed = Self::Signed>;

    /// Signed number type of the pool.
    type Signed;

    // /// Signed number type of the pool.
    // type Signed: Signed;

    /// Get the long token amount.
    fn long_token_amount(&self) -> crate::Result<Self::Num>;

    /// Get the short token amount.
    fn short_token_amount(&self) -> crate::Result<Self::Num>;

    // /// Get long token amount after applying delta.
    // fn long_token_amount_after_delta(&self, delta: &Self::Signed) -> crate::Result<Self::Num>;

    // /// Get short token amount after applying delta.
    // fn short_token_amount_after_delta(&self, delta: &Self::Signed) -> crate::Result<Self::Num>;

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
    fn long_token_usd_value(&self, price: &Self::Num) -> crate::Result<Self::Num> {
        // FIXME: should we use MulDiv?
        self.long_token_amount()?
            .checked_mul(price)
            .ok_or(crate::Error::Computation)
    }

    /// Get the short token value in USD.
    fn short_token_usd_value(&self, price: &Self::Num) -> crate::Result<Self::Num> {
        // FIXME: should we use MulDiv?
        self.short_token_amount()?
            .checked_mul(price)
            .ok_or(crate::Error::Computation)
    }

    /// Apply delta.
    fn apply_delta_amount(
        &mut self,
        is_long_token: bool,
        delta: &Self::Signed,
    ) -> Result<(), crate::Error> {
        if is_long_token {
            self.apply_delta_to_long_token_amount(delta)
        } else {
            self.apply_delta_to_short_token_amount(delta)
        }
    }

    /// Get pool value information after applying delta.
    fn pool_delta(
        &self,
        long_token_delta_amount: &Self::Signed,
        short_token_delta_amount: &Self::Signed,
        long_token_price: &Self::Num,
        short_token_price: &Self::Num,
    ) -> crate::Result<PoolDelta<Self::Num>> {
        PoolDelta::try_new(
            self,
            long_token_delta_amount,
            short_token_delta_amount,
            long_token_price,
            short_token_price,
        )
    }
}

impl<P: Pool + ?Sized> PoolExt for P {}

/// Pool kind.
#[derive(
    Debug, Clone, Copy, Default, num_enum::TryFromPrimitive, PartialEq, Eq, PartialOrd, Ord,
)]
#[repr(u8)]
#[non_exhaustive]
pub enum PoolKind {
    /// Primary.
    #[default]
    Primary,
    /// Swap impact.
    SwapImpact,
    /// Claimable fee.
    ClaimableFee,
}

impl fmt::Display for PoolKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Primary => "Primary",
            Self::SwapImpact => "SwapImpact",
            Self::ClaimableFee => "ClaimableFee",
        };
        write!(f, "{name}")
    }
}
