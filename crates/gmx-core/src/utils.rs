use crate::num::MulDiv;

/// Usd value to market token amount.
///
/// Returns `None` if the computation cannot be done.
pub fn usd_to_market_token_amount<T>(
    usd_value: T,
    pool_value: T,
    supply: T,
    float_to_wei_divisor: T,
) -> Option<T>
where
    T: MulDiv,
{
    if float_to_wei_divisor.is_zero() {
        return None;
    }
    if supply.is_zero() && pool_value.is_zero() {
        Some(usd_value / float_to_wei_divisor)
    } else if supply.is_zero() && !pool_value.is_zero() {
        Some((pool_value + usd_value) / float_to_wei_divisor)
    } else {
        supply.checked_mul_div(usd_value, pool_value)
    }
}
