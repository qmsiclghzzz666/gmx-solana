use anchor_lang::prelude::*;
use gmx_solana_utils::price::{Decimal, Price};
use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;

use crate::{states::TokenConfig, DataStoreError};

/// The Pyth receiver program.
pub struct Pyth;

impl Id for Pyth {
    fn id() -> Pubkey {
        pyth_solana_receiver_sdk::ID
    }
}

impl Pyth {
    /// Push Oracle ID.
    pub const PUSH_ORACLE_ID: Pubkey = pyth_solana_receiver_sdk::PYTH_PUSH_ORACLE_ID;

    pub(super) fn check_and_get_price<'info>(
        clock: &Clock,
        token_config: &TokenConfig,
        feed: &'info AccountInfo<'info>,
        feed_id: &Pubkey,
    ) -> Result<(i64, Price)> {
        let feed = Account::<PriceUpdateV2>::try_from(feed)?;
        let feed_id = feed_id.to_bytes();
        let price =
            feed.get_price_no_older_than(clock, token_config.heartbeat_duration.into(), &feed_id)?;
        let mid_price: u64 = price
            .price
            .try_into()
            .map_err(|_| DataStoreError::NegativePrice)?;
        // FIXME: use min and max price when ready.
        let _min_price = mid_price
            .checked_sub(price.conf)
            .ok_or(DataStoreError::NegativePrice)?;
        let _max_price = mid_price
            .checked_add(price.conf)
            .ok_or(DataStoreError::PriceOverflow)?;
        let parsed_price = Price {
            min: Self::price_value_to_decimal(mid_price, price.exponent, token_config)?,
            max: Self::price_value_to_decimal(mid_price, price.exponent, token_config)?,
        };
        Ok((price.publish_time, parsed_price))
    }

    fn price_value_to_decimal(
        mut value: u64,
        exponent: i32,
        token_config: &TokenConfig,
    ) -> Result<Decimal> {
        // actual price == value * 10^exponent
        // - If `exponent` is not positive, then the `decimals` is set to `-exponent`.
        // - Otherwise, we should use `value * 10^exponent` as `price` argument, and let `decimals` be `0`.
        let decimals: u8 = if exponent <= 0 {
            (-exponent)
                .try_into()
                .map_err(|_| DataStoreError::InvalidPriceFeedPrice)?
        } else {
            let factor = 10u64
                .checked_pow(exponent as u32)
                .ok_or(DataStoreError::InvalidPriceFeedPrice)?;
            value = value
                .checked_mul(factor)
                .ok_or(DataStoreError::PriceOverflow)?;
            0
        };
        let price = Decimal::try_from_price(
            value as u128,
            decimals,
            token_config.token_decimals,
            token_config.precision,
        )
        .map_err(|_| DataStoreError::InvalidPriceFeedPrice)?;
        Ok(price)
    }
}

/// The legacy Pyth program.
pub struct PythLegacy;

impl PythLegacy {
    pub(super) fn check_and_get_price<'info>(
        clock: &Clock,
        token_config: &TokenConfig,
        feed: &'info AccountInfo<'info>,
    ) -> Result<(i64, Price)> {
        use pyth_sdk_solana::state::SolanaPriceAccount;
        let feed = SolanaPriceAccount::account_info_to_feed(feed).map_err(|err| {
            msg!("Pyth Error: {}", err);
            DataStoreError::Unknown
        })?;
        let Some(price) = feed
            .get_price_no_older_than(clock.unix_timestamp, token_config.heartbeat_duration.into())
        else {
            return err!(DataStoreError::PriceFeedNotUpdated);
        };
        let mid_price: u64 = price
            .price
            .try_into()
            .map_err(|_| DataStoreError::NegativePrice)?;
        let parsed_price = Price {
            min: Pyth::price_value_to_decimal(mid_price, price.expo, token_config)?,
            max: Pyth::price_value_to_decimal(mid_price, price.expo, token_config)?,
        };
        Ok((price.publish_time, parsed_price))
    }
}

/// The address of legacy Pyth program.
#[cfg(not(feature = "devnet"))]
pub const PYTH_LEGACY_ID: Pubkey = Pubkey::new_from_array([
    220, 229, 235, 225, 228, 156, 59, 159, 17, 76, 181, 84, 76, 80, 169, 158, 192, 214, 146, 214,
    63, 86, 121, 90, 224, 41, 172, 131, 217, 234, 139, 226,
]);

#[cfg(feature = "devnet")]
pub const PYTH_LEGACY_ID: Pubkey = Pubkey::new_from_array([
    10, 26, 152, 51, 163, 118, 85, 43, 86, 183, 202, 13, 237, 25, 41, 23, 0, 87, 232, 39, 160, 198,
    39, 244, 182, 71, 185, 238, 144, 153, 175, 180,
]);

impl Id for PythLegacy {
    fn id() -> Pubkey {
        PYTH_LEGACY_ID
    }
}
