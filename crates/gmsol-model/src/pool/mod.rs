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
#[cfg_attr(
    feature = "strum",
    derive(strum::EnumIter, strum::EnumString, strum::Display)
)]
#[cfg_attr(feature = "strum", strum(serialize_all = "snake_case"))]
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
    /// Open Interest for long.
    OpenInterestForLong,
    /// Open Interest for short.
    OpenInterestForShort,
    /// Open Interest in tokens for long.
    OpenInterestInTokensForLong,
    /// Open Interest in tokens for short.
    OpenInterestInTokensForShort,
    /// Position impact.
    PositionImpact,
    /// Borrowing Factor.
    BorrowingFactor,
    /// Funding Amount Per Size for long.
    FundingAmountPerSizeForLong,
    /// Funding Amount Per Size for short.
    FundingAmountPerSizeForShort,
    /// Claimable Funding Amount Per Size for long.
    ClaimableFundingAmountPerSizeForLong,
    /// Claimable Funding Amount Per Size for short.
    ClaimableFundingAmountPerSizeForShort,
}
