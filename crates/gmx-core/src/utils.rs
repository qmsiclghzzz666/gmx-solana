use std::cmp::Ordering;

use crate::num::{MulDiv, Num};

/// Usd value to market token amount.
///
/// Returns `None` if the computation cannot be done.
pub fn usd_to_market_token_amount<T>(
    usd_value: T,
    pool_value: T,
    supply: T,
    usd_to_amount_divisor: T,
) -> Option<T>
where
    T: MulDiv + Num,
{
    if usd_to_amount_divisor.is_zero() {
        return None;
    }
    if supply.is_zero() && pool_value.is_zero() {
        Some(usd_value / usd_to_amount_divisor)
    } else if supply.is_zero() && !pool_value.is_zero() {
        Some((pool_value.checked_add(&usd_value)?) / usd_to_amount_divisor)
    } else {
        supply.checked_mul_div(&usd_value, &pool_value)
    }
}

/// Apply factors using this formula: `A * x^E`.
///
/// Assuming that all values are "float"s with the same decimals.
pub fn apply_factors<T>(value: T, factor: T, exponent_factor: T, unit: T) -> Option<T>
where
    T: MulDiv + Num,
{
    apply_factor(
        apply_exponent_factor(value, exponent_factor, unit.clone())?,
        factor,
        unit,
    )
}

/// Apply exponent factor using this formula: `x^E`.
///
/// Assuming that all values are "float"s with the same decimals.
#[inline]
pub fn apply_exponent_factor<T>(value: T, exponent_factor: T, unit: T) -> Option<T>
where
    T: Num,
{
    if unit.is_zero() {
        return None;
    }
    match value.cmp(&unit) {
        Ordering::Less => Some(T::zero()),
        Ordering::Equal => Some(unit),
        Ordering::Greater => {
            if exponent_factor.is_zero() {
                Some(unit)
            } else if exponent_factor.is_one() {
                Some(value)
            } else {
                Some(T::zero())
            }
        }
    }
}

/// Apply factor using this formula: `A * x`.
///
/// Assuming that values are "float"s with the same decimals.
#[inline]
pub fn apply_factor<T>(value: T, factor: T, unit: T) -> Option<T>
where
    T: MulDiv,
{
    value.checked_mul_div(&factor, &unit)
}
