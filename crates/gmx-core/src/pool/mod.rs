use std::fmt;

use crate::num::{Num, Unsigned};
use num_traits::CheckedMul;

use self::delta::PoolDelta;

/// Delta.
pub mod delta;

/// Balanced amounts.
pub trait Balance {
    /// Unsigned number type of the pool.
    type Num: Num + Unsigned<Signed = Self::Signed>;

    /// Signed number type of the pool.
    type Signed;

    /// Get the long token amount (when this is a token pool), or long usd value (when this is a usd value pool).
    fn long_amount(&self) -> crate::Result<Self::Num>;

    /// Get the short token amount (when this is a token pool), or short usd value (when this is a usd value pool).
    fn short_amount(&self) -> crate::Result<Self::Num>;
}

/// A pool for holding tokens.
pub trait Pool: Balance {
    /// Apply delta to long amount.
    fn apply_delta_to_long_amount(&mut self, delta: &Self::Signed) -> Result<(), crate::Error>;

    /// Apply delta to short amount.
    fn apply_delta_to_short_amount(&mut self, delta: &Self::Signed) -> Result<(), crate::Error>;
}

/// Extension trait for [`Balance`] with utils.
pub trait BalanceExt: Balance {
    /// Get the long amount value in USD.
    fn long_usd_value(&self, price: &Self::Num) -> crate::Result<Self::Num> {
        // FIXME: should we use MulDiv?
        self.long_amount()?
            .checked_mul(price)
            .ok_or(crate::Error::Overflow)
    }

    /// Get the short amount value in USD.
    fn short_usd_value(&self, price: &Self::Num) -> crate::Result<Self::Num> {
        // FIXME: should we use MulDiv?
        self.short_amount()?
            .checked_mul(price)
            .ok_or(crate::Error::Overflow)
    }

    /// Get pool value information after applying delta.
    fn pool_delta_with_amounts(
        &self,
        long_token_delta_amount: &Self::Signed,
        short_token_delta_amount: &Self::Signed,
        long_token_price: &Self::Num,
        short_token_price: &Self::Num,
    ) -> crate::Result<PoolDelta<Self::Num>> {
        PoolDelta::try_from_delta_amounts(
            self,
            long_token_delta_amount,
            short_token_delta_amount,
            long_token_price,
            short_token_price,
        )
    }

    /// Get pool value information after applying delta.
    fn pool_delta_with_values(
        &self,
        delta_long_token_usd_value: Self::Signed,
        delta_short_token_usd_value: Self::Signed,
        long_token_price: &Self::Num,
        short_token_price: &Self::Num,
    ) -> crate::Result<PoolDelta<Self::Num>> {
        PoolDelta::try_new(
            self,
            delta_long_token_usd_value,
            delta_short_token_usd_value,
            long_token_price,
            short_token_price,
        )
    }
}

impl<P: Balance + ?Sized> BalanceExt for P {}

/// Extension trait for [`Pool`] with utils.
pub trait PoolExt: Pool {
    /// Apply delta.
    fn apply_delta_amount(
        &mut self,
        is_long: bool,
        delta: &Self::Signed,
    ) -> Result<(), crate::Error> {
        if is_long {
            self.apply_delta_to_long_amount(delta)
        } else {
            self.apply_delta_to_short_amount(delta)
        }
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
    /// Open Interest for long collateral.
    OpenInterestForLongCollateral,
    /// Open Interest for short collateral.
    OpenInterestForShortCollateral,
    /// Open Interest in tokens for long collateral.
    OpenInterestInTokensForLongCollateral,
    /// Open Interest in tokens for short collateral.
    OpenInterestInTokensForShortCollateral,
}

impl fmt::Display for PoolKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Primary => "Primary",
            Self::SwapImpact => "SwapImpact",
            Self::ClaimableFee => "ClaimableFee",
            Self::OpenInterestForLongCollateral => "OpenInterestForLongCollateral",
            Self::OpenInterestForShortCollateral => "OpenInterestForShortCollateral",
            Self::OpenInterestInTokensForLongCollateral => "OpenInterestInTokensForLongCollateral",
            Self::OpenInterestInTokensForShortCollateral => {
                "OpenInterestInTokensForShortCollateral"
            }
        };
        write!(f, "{name}")
    }
}
