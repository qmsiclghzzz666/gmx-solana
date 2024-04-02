use num_traits::{CheckedAdd, CheckedMul, CheckedSub, Signed};

/// Num trait used in GMX.
pub trait Num: num_traits::Num + CheckedAdd + CheckedMul + CheckedSub + Clone + Ord {}

impl<T: num_traits::Num + CheckedAdd + CheckedMul + CheckedSub + Clone + Ord> Num for T {}

/// Unsigned value that cannot be negative.
pub trait Unsigned: num_traits::Unsigned {
    /// The signed type.
    type Signed: TryFrom<Self> + UnsignedAbs<Unsigned = Self>;

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
    }
}
