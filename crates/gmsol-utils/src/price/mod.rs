/// Decimal type for price.
pub mod decimal;

pub use self::decimal::{Decimal, DecimalError};
use anchor_lang::prelude::*;

/// Price type.
#[derive(Debug, AnchorSerialize, AnchorDeserialize, Clone, Copy, InitSpace)]
pub struct Price {
    /// Min Price.
    pub min: Decimal,
    /// Max Price.
    pub max: Decimal,
}
