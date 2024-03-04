use anchor_lang::prelude::{borsh, AnchorDeserialize, AnchorSerialize};

/// Decimal type for storing prices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AnchorSerialize, AnchorDeserialize)]
pub struct Decimal {
    /// Value.
    pub value: u32,
    /// Decimal multiplier.
    pub decimal_multiplier: u8,
}

impl Decimal {
    /// The Maximum Decimals.
    pub const MAX_DECIMALS: u8 = 30;

    /// The Maximum Decimal Multiplier,
    /// which satisfy `u32::MAX * 10^{MAX_DECIMAL_MULTIPLIER} <= u128::MAX`.
    pub const MAX_DECIMAL_MULTIPLIER: u8 = 28;

    /// Returns the price of one unit (with [`MAX_DECIMALS`](Self::MAX_DECIMALS)).
    pub fn to_unit_price(&self) -> u128 {
        self.value as u128 * 10u128.pow(self.decimal_multiplier as u32)
    }

    /// From the price of one asset.
    pub fn try_new(
        price: u128,
        decimals: u8,
        decimal_multiplier: u8,
    ) -> Result<Self, DecimalError> {
        if decimals > Self::MAX_DECIMALS {
            return Err(DecimalError::ExceedMaxDecimals);
        }
        if decimal_multiplier > Self::MAX_DECIMAL_MULTIPLIER {
            return Err(DecimalError::ExceedMaxDecimalMultipler);
        }
        // 2 * MAX_DECIMALS + MAX_DECIMAL_MULTIPLER <= u8::MAX
        let multiplier = (decimals << 1) + decimal_multiplier;
        let value = if Self::MAX_DECIMALS >= multiplier {
            price * 10u128.pow((Self::MAX_DECIMALS - multiplier) as u32)
        } else {
            // FIXME: should we require that `decimal + decimal_multiplier <= MAX_DECIMALS`,
            // than the `pow` will never overflow.
            price / 10u128.pow((multiplier - Self::MAX_DECIMALS) as u32)
        };
        Ok(Self {
            value: value as u32,
            decimal_multiplier,
        })
    }

    /// Calculate the decimal multiplier with the desired precision.
    /// # Warning
    /// One should check that `decimals + precision` is not greater than [`MAX_DECIMALS`](Self::MAX_DECIMALS),
    /// otherwise the result might be incorrect due to underflow.
    pub const fn decimal_multiplier_from_precision(decimals: u8, precision: u8) -> u8 {
        Self::MAX_DECIMALS - decimals - precision
    }

    /// Returns the max representable decimal with the same decimal multiplier.
    pub fn maximum(&self) -> Self {
        Self {
            value: u32::MAX,
            decimal_multiplier: self.decimal_multiplier,
        }
    }
}

/// Errors of decimals.
#[derive(Debug, thiserror::Error)]
pub enum DecimalError {
    /// Exceed the maximum decimals.
    #[error("exceeds the maximum decimals")]
    ExceedMaxDecimals,
    /// Invalid decimals.
    #[error("exceeds the maximum decimal multipler")]
    ExceedMaxDecimalMultipler,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_price_1() {
        // The price of ETH is 5,000 with 18 decimals and the decimal multipler is set to 8 (so that we have decimals of precision 4).
        let multiplier = Decimal::decimal_multiplier_from_precision(18, 4);
        assert_eq!(multiplier, 8);
        let price = Decimal::try_new(5_000_000_000_000_000_000_000, 18, multiplier).unwrap();
        assert_eq!(price.to_unit_price(), 5_000_000_000_000_000);
    }

    #[test]
    fn test_price_2() {
        // The price of BTC is 60,000 with 8 decimals and the decimal multipler is set to 20 (so that we have decimals of precision 2).
        let multiplier = Decimal::decimal_multiplier_from_precision(8, 2);
        assert_eq!(multiplier, 20);
        let price = Decimal::try_new(6_000_000_000_000, 8, multiplier).unwrap();
        assert_eq!(price.to_unit_price(), 600_000_000_000_000_000_000_000_000);
    }

    #[test]
    fn test_price_3() {
        // The price of USDC is 1 with 6 decimals and the decimal multipler is set to 18 (so that we have decimals of precision 6).
        let multiplier = Decimal::decimal_multiplier_from_precision(6, 6);
        assert_eq!(multiplier, 18);
        let price = Decimal::try_new(1_000_000, 6, multiplier).unwrap();
        assert_eq!(price.to_unit_price(), 1_000_000_000_000_000_000_000_000);
    }

    #[test]
    fn test_price_4() {
        // The price of DG is 0.00000001 with 18 decimals and the decimal multipler is set to 1 (so that we have decimals of precision 11).
        let multiplier = Decimal::decimal_multiplier_from_precision(18, 11);
        assert_eq!(multiplier, 1);
        let price = Decimal::try_new(10_000_000_000, 18, multiplier).unwrap();
        assert_eq!(price.to_unit_price(), 10_000);
    }
}
