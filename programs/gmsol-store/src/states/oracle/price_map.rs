use anchor_lang::prelude::*;
use gmsol_utils::price::Decimal;

use crate::{utils::pubkey::to_bytes, CoreError};

/// Zero-copy price structure for storing min max prices.
#[zero_copy]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct SmallPrices {
    decimal_multipler: u8,
    padding_0: [u8; 3],
    min: u32,
    max: u32,
}

impl Default for SmallPrices {
    fn default() -> Self {
        bytemuck::Zeroable::zeroed()
    }
}

impl SmallPrices {
    /// Get min price.
    pub fn min(&self) -> Decimal {
        Decimal {
            value: self.min,
            decimal_multiplier: self.decimal_multipler,
        }
    }

    /// Get max price.
    pub fn max(&self) -> Decimal {
        Decimal {
            value: self.max,
            decimal_multiplier: self.decimal_multipler,
        }
    }

    pub(crate) fn from_price(price: &gmsol_utils::Price) -> Result<Self> {
        // Validate price data.
        require_eq!(
            price.min.decimal_multiplier,
            price.max.decimal_multiplier,
            CoreError::InvalidArgument
        );
        require_neq!(price.min.value, 0, CoreError::InvalidArgument);
        require_gt!(price.max.value, price.min.value, CoreError::InvalidArgument);

        Ok(SmallPrices {
            decimal_multipler: price.min.decimal_multiplier,
            padding_0: [0; 3],
            min: price.min.value,
            max: price.max.value,
        })
    }
}

const MAX_TOKENS: usize = 512;

gmsol_utils::fixed_map!(PriceMap, Pubkey, to_bytes, SmallPrices, MAX_TOKENS, 0);

impl PriceMap {
    /// Max tokens.
    pub const MAX_TOKENS: usize = MAX_TOKENS;

    pub(super) fn set(&mut self, token: &Pubkey, price: gmsol_utils::Price) -> Result<()> {
        self.insert(token, SmallPrices::from_price(&price)?);
        Ok(())
    }
}
