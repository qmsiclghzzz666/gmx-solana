use std::ops::{Add, Mul};

use num_traits::{CheckedAdd, CheckedMul};

use crate::num::MulDiv;

/// Integer type used in [`Decimal`].
pub trait Integer<const DECIMALS: u8> {
    /// Ten.
    const TEN: Self;
    /// The unit with value pow(TEN, DECIMALS).
    const UNIT: Self;
}

impl<const DECIMALS: u8> Integer<DECIMALS> for u64 {
    const TEN: Self = 10u64;
    const UNIT: Self = 10u64.pow(DECIMALS as u32);
}

#[cfg(feature = "u128")]
impl<const DECIMALS: u8> Integer<DECIMALS> for u128 {
    const TEN: Self = 10u128;
    const UNIT: Self = 10u128.pow(DECIMALS as u32);
}

/// Decimal type with fixed decimals.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Decimal<T, const DECIMALS: u8>(T);

impl<T, const DECIMALS: u8> Decimal<T, DECIMALS> {
    /// Get the internal integer representation.
    pub fn get(&self) -> &T {
        &self.0
    }

    /// Create a new decimal from the inner representation.
    pub fn from_inner(inner: T) -> Self {
        Self(inner)
    }
}

impl<T: Integer<DECIMALS>, const DECIMALS: u8> Decimal<T, DECIMALS> {
    /// The unit value.
    pub const ONE: Decimal<T, DECIMALS> = Decimal(Integer::UNIT);
}

impl<T: Add<Output = T>, const DECIMALS: u8> Add for Decimal<T, DECIMALS> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl<T: CheckedAdd, const DECIMALS: u8> CheckedAdd for Decimal<T, DECIMALS> {
    fn checked_add(&self, v: &Self) -> Option<Self> {
        Some(Self(self.0.checked_add(&v.0)?))
    }
}

impl<T: MulDiv + Integer<DECIMALS>, const DECIMALS: u8> Mul for Decimal<T, DECIMALS> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        self.checked_mul(&rhs).expect("invalid mulplication")
    }
}

impl<T: MulDiv + Integer<DECIMALS>, const DECIMALS: u8> CheckedMul for Decimal<T, DECIMALS> {
    fn checked_mul(&self, v: &Self) -> Option<Self> {
        Some(Self(self.0.checked_mul_div(&v.0, &Self::ONE.0)?))
    }
}

/// Decimal type with `8` decimals and backed by [`u64`]
pub type U64D8 = Decimal<u64, 8>;

#[cfg(feature = "u128")]
/// Decimal type with `20` decimals and backed by [`u128`]
pub type U128D20 = Decimal<u128, 20>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let x = U64D8::from_inner(1_280_000_000);
        let y = U64D8::from_inner(2_560_000_001);
        assert_eq!(x * y, U64D8::from_inner(32_768_000_012));
    }

    #[cfg(feature = "u128")]
    #[test]
    fn basic_u128() {
        let x = U128D20::from_inner(128 * U128D20::ONE.0);
        let y = U128D20::from_inner(256 * U128D20::ONE.0 + 1);
        assert_eq!(
            x * y,
            U128D20::from_inner(3_276_800_000_000_000_000_000_128)
        );
    }
}
