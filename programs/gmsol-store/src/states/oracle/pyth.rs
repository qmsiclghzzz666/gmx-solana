use anchor_lang::prelude::*;
use gmsol_utils::price::{Decimal, Price};
use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;

use crate::{states::TokenConfig, CoreError};

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

    #[allow(clippy::manual_inspect)]
    pub(super) fn check_and_get_price<'info>(
        clock: &Clock,
        token_config: &TokenConfig,
        feed: &'info AccountInfo<'info>,
        feed_id: &Pubkey,
    ) -> Result<(u64, i64, Price)> {
        let feed = Account::<PriceUpdateV2>::try_from(feed)?;
        let feed_id = feed_id.to_bytes();
        let max_age = token_config.heartbeat_duration().into();
        let price = feed
            .get_price_no_older_than(clock, max_age, &feed_id)
            .map_err(|err| {
                let price_ts = feed.price_message.publish_time;
                msg!(
                    "[Pyth] get price error, clock={} price_ts={} max_age={}",
                    clock.unix_timestamp,
                    price_ts,
                    max_age,
                );
                err
            })?;
        let parsed_price = pyth_price_with_confidence_to_price(
            price.price,
            price.conf,
            price.exponent,
            token_config,
        )?;
        Ok((feed.posted_slot, price.publish_time, parsed_price))
    }
}

/// Convert pyth price value with confidence to [`Price`].
pub fn pyth_price_with_confidence_to_price(
    price: i64,
    confidence: u64,
    exponent: i32,
    token_config: &TokenConfig,
) -> Result<Price> {
    let mid_price: u64 = price
        .try_into()
        .map_err(|_| error!(CoreError::InvalidPriceFeedPrice))?;
    // Note: No validation of Pyth’s price volatility has been conducted yet.
    // Exercise caution when choosing Pyth as the primary oracle.
    let min_price = mid_price
        .checked_sub(confidence)
        .ok_or_else(|| error!(CoreError::InvalidPriceFeedPrice))?;
    let max_price = mid_price
        .checked_add(confidence)
        .ok_or_else(|| error!(CoreError::InvalidPriceFeedPrice))?;
    Ok(Price {
        min: pyth_price_value_to_decimal(min_price, exponent, token_config)?,
        max: pyth_price_value_to_decimal(max_price, exponent, token_config)?,
    })
}

/// Pyth price value to decimal.
pub fn pyth_price_value_to_decimal(
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
            .map_err(|_| CoreError::InvalidPriceFeedPrice)?
    } else {
        let factor = 10u64
            .checked_pow(exponent as u32)
            .ok_or_else(|| error!(CoreError::InvalidPriceFeedPrice))?;
        value = value
            .checked_mul(factor)
            .ok_or_else(|| error!(CoreError::InvalidPriceFeedPrice))?;
        0
    };
    let price = Decimal::try_from_price(
        value as u128,
        decimals,
        token_config.token_decimals(),
        token_config.precision(),
    )
    .map_err(|_| CoreError::InvalidPriceFeedPrice)?;
    Ok(price)
}
