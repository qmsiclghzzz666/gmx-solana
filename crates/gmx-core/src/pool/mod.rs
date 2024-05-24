use std::fmt;

use self::delta::PoolDelta;

/// Balance.
pub mod balance;

/// Delta.
pub mod delta;

pub use self::balance::{Balance, BalanceExt};

/// A pool for holding tokens.
pub trait Pool: Balance {
    /// Apply delta to long amount.
    fn apply_delta_to_long_amount(&mut self, delta: &Self::Signed) -> Result<(), crate::Error>;

    /// Apply delta to short amount.
    fn apply_delta_to_short_amount(&mut self, delta: &Self::Signed) -> Result<(), crate::Error>;
}

/// Extension trait for [`Pool`] with utils.
pub trait PoolExt: Pool {
    /// Apply delta.
    #[inline]
    fn apply_delta_amount(&mut self, is_long: bool, delta: &Self::Signed) -> crate::Result<()> {
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
    Debug,
    Clone,
    Copy,
    Default,
    num_enum::TryFromPrimitive,
    num_enum::IntoPrimitive,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
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
    /// Position impact.
    PositionImpact,
    /// Borrowing Factor.
    BorrowingFactor,
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
            Self::PositionImpact => "PositionImpact",
            Self::BorrowingFactor => "BorrowingFactor",
        };
        write!(f, "{name}")
    }
}
