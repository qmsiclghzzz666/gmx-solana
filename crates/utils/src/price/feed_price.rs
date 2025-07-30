use anchor_lang::prelude::zero_copy;

use crate::{
    price::{
        decimal::{Decimal, DecimalError},
        Price, PriceFlag, MAX_PRICE_FLAG,
    },
    token_config::TokenConfig,
};

crate::flags!(PriceFlag, MAX_PRICE_FLAG, u8);

const NANOS_PER_SECOND: i64 = 1_000_000_000;

/// Price structure for Price Feed.
#[zero_copy]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct PriceFeedPrice {
    decimals: u8,
    flags: PriceFlagContainer,
    padding: [u8; 2],
    last_update_diff_nanos: u32,
    ts: i64,
    price: u128,
    min_price: u128,
    max_price: u128,
}

impl PriceFeedPrice {
    /// Create a new [`PriceFeedPrice`].
    pub fn new(
        decimals: u8,
        ts: i64,
        price: u128,
        min_price: u128,
        max_price: u128,
        last_update_diff_nanos: u32,
    ) -> Self {
        Self {
            decimals,
            flags: Default::default(),
            padding: [0; 2],
            last_update_diff_nanos,
            ts,
            price,
            min_price,
            max_price,
        }
    }

    /// Set flag.
    pub fn set_flag(&mut self, flag: PriceFlag, value: bool) -> bool {
        self.flags.set_flag(flag, value)
    }

    /// Get ts.
    pub fn ts(&self) -> i64 {
        self.ts
    }

    /// Returns the nanoseconds the last update time is earlier than the report time.
    /// `None` means the field is disabled.
    pub fn last_update_diff_nanos(&self) -> Option<u32> {
        if self.flags.get_flag(PriceFlag::LastUpdateDiffEnabled) {
            Some(self.last_update_diff_nanos)
        } else {
            None
        }
    }

    /// Get min price.
    pub fn min_price(&self) -> &u128 {
        &self.min_price
    }

    /// Get max price.
    pub fn max_price(&self) -> &u128 {
        &self.max_price
    }

    /// Get price.
    pub fn price(&self) -> &u128 {
        &self.price
    }

    /// Returns whether the market is open.
    pub fn is_market_open(&self, current_timestamp: i64, heartbeat_duration: u32) -> bool {
        if !self.flags.get_flag(PriceFlag::Open) {
            return false;
        }

        let Some(last_update_diff_nanos) = self.last_update_diff_nanos() else {
            return true;
        };

        let last_update_diff_nanos = i64::from(last_update_diff_nanos);

        // The use of `saturating_sub` here is valid because:
        //   - In the case of overflow, the function returns `false`,
        //     and since `current_timestamp >= ts + i64::MAX`, it must
        //     also hold that `current_timestamp >= ts + heartbeat_duration`,
        //     and thus `current_timestamp > last_update + heartbeat_duration`.
        //   - In the case of underflow, the function returns `true`,
        //     and since `current_timestamp <= ts + i64::MIN`, it follows that
        //     `current_timestamp <= ts - last_update_diff_secs`, and thus
        //     `current_timestamp - last_update <= heartbeat_duration`.
        // Therefore, we only need to check the case where no overflow or underflow occurs.
        let current_diff = current_timestamp.saturating_sub(self.ts);
        let heartbeat_duration = heartbeat_duration.into();
        if current_diff > heartbeat_duration {
            return false;
        }

        last_update_diff_nanos
            <= heartbeat_duration
                // The use of `saturating_sub` is valid because:
                //   - Underflow is impossible because of the check above, and in the case of
                //     overflow, the function returns `true`, and since
                //     `heartbeat_duration >= current_diff + i64::MAX`, it must also hold that
                //     `heartbeat_duration >= current_diff + last_update_diff_secs`, and thus
                //     `current_timestamp - last_update <= heartbeat_duration`.
                .saturating_sub(current_diff)
                // The use of `saturating_mul` is valid because:
                //   - Underflow is impossible because `heartbeat_duration >= current_diff`,
                //     and in the case of overflow, the function returns `true`, and since
                //     `(heartbeat_duration - current_diff) * NANOS_PER_SECOND >= i64::MAX`, it must
                //     hold that `(heartbeat_duration - current_diff) * NANOS_PER_SECOND >= last_update_diff_nanos`.
                .saturating_mul(NANOS_PER_SECOND)
    }

    /// Try converting to [`Price`].
    pub fn try_to_price(&self, token_config: &TokenConfig) -> Result<Price, DecimalError> {
        let token_decimals = token_config.token_decimals();
        let precision = token_config.precision();

        let min =
            Decimal::try_from_price(self.min_price, self.decimals, token_decimals, precision)?;

        let max =
            Decimal::try_from_price(self.max_price, self.decimals, token_decimals, precision)?;

        Ok(Price { min, max })
    }

    /// Returns reference price in [`Decimal`].
    pub fn try_to_ref_price(&self, token_config: &TokenConfig) -> Result<Decimal, DecimalError> {
        let token_decimals = token_config.token_decimals();
        let precision = token_config.precision();
        Decimal::try_from_price(self.price, self.decimals, token_decimals, precision)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_market_open() {
        let mut price = PriceFeedPrice::new(0, 0, 1, 1, 1, 0);
        assert!(!price.is_market_open(i64::MAX, 0));
        price.set_flag(PriceFlag::Open, true);
        assert!(price.is_market_open(i64::MAX, 0));

        price.set_flag(PriceFlag::LastUpdateDiffEnabled, true);
        price.last_update_diff_nanos = u32::MAX;

        price.ts = i64::MIN;
        assert!(!price.is_market_open(i64::MAX, u32::MAX));

        price.ts = i64::MAX;
        assert!(price.is_market_open(i64::MIN, 0));

        let delay = 10i64;
        price.ts = i64::MAX - delay;
        assert!(!price.is_market_open(i64::MAX, u32::MAX / NANOS_PER_SECOND as u32 + delay as u32));
        assert!(price.is_market_open(
            i64::MAX,
            u32::MAX.div_ceil(NANOS_PER_SECOND as u32) + delay as u32
        ));

        let diff = 1i64;
        price.ts = i64::MAX;
        let current = i64::MAX - diff;
        assert!(!price.is_market_open(current, u32::MAX / NANOS_PER_SECOND as u32 - diff as u32));
        assert!(price.is_market_open(
            current,
            u32::MAX.div_ceil(NANOS_PER_SECOND as u32) - diff as u32
        ));
    }
}
