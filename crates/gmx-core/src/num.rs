use num_traits::{CheckedAdd, CheckedMul, CheckedSub, One, Signed, Zero};

/// Num trait used in GMX.
pub trait Num: num_traits::Num + CheckedAdd + CheckedMul + CheckedSub + Clone + Ord {}

impl<T: num_traits::Num + CheckedAdd + CheckedMul + CheckedSub + Clone + Ord> Num for T {}

/// Unsigned value that cannot be negative.
pub trait Unsigned: num_traits::Unsigned {
    /// The signed type.
    type Signed: TryFrom<Self> + UnsignedAbs<Unsigned = Self>;

    /// Convert to signed.
    fn to_signed(&self) -> crate::Result<Self::Signed>
    where
        Self: Clone,
    {
        self.clone().try_into().map_err(|_| crate::Error::Convert)
    }

    /// Convert to opposite signed.
    fn to_opposite_signed(&self) -> crate::Result<Self::Signed>
    where
        Self: Clone,
        Self::Signed: CheckedSub,
    {
        let value = self.to_signed()?;
        Self::Signed::zero()
            .checked_sub(&value)
            .ok_or(crate::Error::Underflow)
    }

    /// Compute the absolute difference of two values.
    fn diff(self, other: Self) -> Self;

    /// Checked signed add.
    fn checked_add_with_signed(&self, other: &Self::Signed) -> Option<Self>
    where
        Self: CheckedAdd + CheckedSub,
    {
        let value = other.unsigned_abs();
        if other.is_positive() {
            self.checked_add(&value)
        } else {
            self.checked_sub(&value)
        }
    }

    /// Checked signed sub.
    fn checked_sub_with_signed(&self, other: &Self::Signed) -> Option<Self>
    where
        Self: CheckedAdd + CheckedSub,
    {
        let value = other.unsigned_abs();
        if other.is_positive() {
            self.checked_sub(&value)
        } else {
            self.checked_add(&value)
        }
    }

    /// Checked signed mul.
    fn checked_mul_with_signed(&self, other: &Self::Signed) -> Option<Self::Signed>
    where
        Self: CheckedMul,
    {
        let value = other.unsigned_abs();
        if other.is_negative() {
            Some(-self.checked_mul(&value)?.try_into().ok()?)
        } else {
            self.checked_mul(&value)?.try_into().ok()
        }
    }

    /// As divisor to checked divde other and round up magnitude.
    fn as_divisor_to_round_up_magnitude_div(&self, dividend: &Self::Signed) -> Option<Self::Signed>
    where
        Self: Clone,
        Self::Signed: CheckedSub + CheckedAdd,
    {
        if self.is_zero() {
            return None;
        }
        let divisor: Self::Signed = self.clone().try_into().ok()?;
        if dividend.is_negative() {
            Some(dividend.checked_sub(&divisor)?.checked_add(&One::one())? / divisor)
        } else {
            Some(dividend.checked_add(&divisor)?.checked_sub(&One::one())? / divisor)
        }
    }

    /// Checked round up division.
    fn checked_round_up_div(&self, divisor: &Self) -> Option<Self>
    where
        Self: CheckedAdd + CheckedSub + Clone,
    {
        if divisor.is_zero() {
            return None;
        }
        Some(self.checked_add(divisor)?.checked_sub(&One::one())? / divisor.clone())
    }
}

/// Convert signed value to unsigned.
pub trait UnsignedAbs: Signed {
    /// Unsigned type.
    type Unsigned;

    /// Computes the absolute value and returns as a unsigned value.
    fn unsigned_abs(&self) -> Self::Unsigned;
}

/// Perform Mul-Div calculation with bigger range num type.
pub trait MulDiv: Unsigned {
    /// Calculates floor(self * numerator / denominator) with full precision.
    ///
    /// Returns `None` if the `denominator` is zero or overflow.
    fn checked_mul_div(&self, numerator: &Self, denominator: &Self) -> Option<Self>;

    /// Calculates ceil(self * numerator / denominator) with full precision.
    ///
    /// Returns `None` if the `denominator` is zero or overflow.
    fn checked_mul_div_ceil(&self, numerator: &Self, denominator: &Self) -> Option<Self>;

    /// Calculates floor(self * numerator / denominator) with full precision,
    /// where `numerator` is signed.
    ///
    /// Returns `None` if the `denominator` is zero or overflow.
    fn checked_mul_div_with_signed_numberator(
        &self,
        numerator: &Self::Signed,
        denominator: &Self,
    ) -> Option<Self::Signed> {
        let ans = self
            .checked_mul_div(&numerator.unsigned_abs(), denominator)?
            .try_into()
            .ok()?;
        if numerator.is_positive() {
            Some(ans)
        } else {
            Some(-ans)
        }
    }
}

impl Unsigned for u64 {
    type Signed = i64;

    fn diff(self, other: Self) -> Self {
        self.abs_diff(other)
    }
}

impl MulDiv for u64 {
    fn checked_mul_div(&self, numerator: &Self, denominator: &Self) -> Option<Self> {
        if *denominator == 0 {
            return None;
        }
        let x = *self as u128;
        let numerator = *numerator as u128;
        let denominator = *denominator as u128;
        let ans = x * numerator / denominator;
        ans.try_into().ok()
    }

    fn checked_mul_div_ceil(&self, numerator: &Self, denominator: &Self) -> Option<Self> {
        if *denominator == 0 {
            return None;
        }
        let x = *self as u128;
        let numerator = *numerator as u128;
        let denominator = *denominator as u128;
        let ans = (x * numerator + denominator - 1) / denominator;
        ans.try_into().ok()
    }
}

impl UnsignedAbs for i64 {
    type Unsigned = u64;

    fn unsigned_abs(&self) -> u64 {
        (*self).unsigned_abs()
    }
}

#[cfg(feature = "u128")]
/// Add support to `u128`.
mod u128 {
    use super::{MulDiv, Unsigned, UnsignedAbs};
    use ruint::aliases::U256;

    impl Unsigned for u128 {
        type Signed = i128;

        fn diff(self, other: Self) -> Self {
            self.abs_diff(other)
        }
    }

    impl UnsignedAbs for i128 {
        type Unsigned = u128;

        fn unsigned_abs(&self) -> u128 {
            (*self).unsigned_abs()
        }
    }

    impl MulDiv for u128 {
        fn checked_mul_div(&self, numerator: &Self, denominator: &Self) -> Option<Self> {
            if *denominator == 0 {
                return None;
            }
            let x = U256::from(*self);
            let numerator = U256::from(*numerator);
            let denominator = U256::from(*denominator);
            let ans = x * numerator / denominator;
            ans.try_into().ok()
        }

        fn checked_mul_div_ceil(&self, numerator: &Self, denominator: &Self) -> Option<Self> {
            if *denominator == 0 {
                return None;
            }
            let x = U256::from(*self);
            let numerator = U256::from(*numerator);
            let denominator = U256::from(*denominator);
            let ans = (x * numerator).div_ceil(denominator);
            ans.try_into().ok()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_up_magnitude_division() {
        let b = 3u64;
        let positive = 1i64;
        let negative = -1i64;

        assert_eq!(b.as_divisor_to_round_up_magnitude_div(&positive), Some(1));
        assert_eq!(b.as_divisor_to_round_up_magnitude_div(&negative), Some(-1));
    }

    #[test]
    fn round_up_division() {
        let b = 3u64;
        let a = 1u64;
        assert_eq!(a.checked_round_up_div(&b), Some(1));
    }

    #[test]
    fn mul_div_ceil() {
        let a = 650_406_504u64;
        let a2 = 650_406_505u64;
        let b = 40_000_000_000u64;
        let c = 80_000_000_000u64;
        assert_eq!(a.checked_mul_div_ceil(&b, &c).unwrap(), 325_203_252);
        assert_eq!(a2.checked_mul_div_ceil(&b, &c).unwrap(), 325_203_253);
    }

    #[cfg(feature = "u128")]
    #[test]
    fn mul_div_ceil_u128() {
        let a = 650_406_504u128;
        let a2 = 650_406_505u128;
        let b = 40_000_000_000u128;
        let c = 80_000_000_000u128;
        assert_eq!(a.checked_mul_div_ceil(&b, &c).unwrap(), 325_203_252);
        assert_eq!(a2.checked_mul_div_ceil(&b, &c).unwrap(), 325_203_253);
    }
}
