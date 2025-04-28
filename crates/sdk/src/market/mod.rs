/// Value.
pub mod value;

/// Market Status.
pub mod status;

use gmsol_model::{
    num::Unsigned, price::Prices, Balance, BaseMarket, BaseMarketExt, BorrowingFeeMarketExt,
    PerpMarketMutExt,
};
use gmsol_programs::model::MarketModel;

use crate::constants;

pub use self::{
    status::MarketStatus,
    value::{SignedValue, Value},
};

/// Market Calculations.
pub trait MarketCalculations {
    /// Calculate market status.
    fn status(&self, prices: &Prices<u128>) -> crate::Result<MarketStatus>;
}

impl MarketCalculations for MarketModel {
    fn status(&self, prices: &Prices<u128>) -> crate::Result<MarketStatus> {
        // Calculate open interests.
        let open_interest = self.open_interest()?;
        let open_interest_for_long = open_interest.long_amount()?;
        let open_interest_for_short = open_interest.short_amount()?;
        let open_interest_in_tokens = self.open_interest_in_tokens()?;

        // Calculate funding rates.
        let (funding_factor_per_second, longs_pay_shorts, _) = self
            .clone()
            .update_funding(prices)?
            .next_funding_factor_per_second(0, &open_interest_for_long, &open_interest_for_short)?;
        let size_for_larger_side = open_interest_for_long.max(open_interest_for_short);
        let funding_value = funding_factor_per_second
            .checked_mul(size_for_larger_side)
            .ok_or_else(|| crate::Error::unknown("failed to calculate funding value"))?;
        let funding_rate_per_second_for_long = if longs_pay_shorts {
            funding_value
                .checked_round_up_div(&open_interest_for_long)
                .ok_or_else(|| crate::Error::unknown("failed to calculate funding rate for long"))?
                .to_signed()?
        } else {
            funding_value
                .checked_div(open_interest_for_long)
                .ok_or_else(|| crate::Error::unknown("failed to calculate funding rate for long"))?
                .to_opposite_signed()?
        };
        let funding_rate_per_second_for_short = if !longs_pay_shorts {
            funding_value
                .checked_round_up_div(&open_interest_for_short)
                .ok_or_else(|| crate::Error::unknown("failed to calculate funding rate for short"))?
                .to_signed()?
        } else {
            funding_value
                .checked_div(open_interest_for_short)
                .ok_or_else(|| crate::Error::unknown("failed to calculate funding rate for short"))?
                .to_opposite_signed()?
        };

        // Calculate liquidities.
        let reserved_value_for_long = self.reserved_value(&prices.index_token_price, true)?;
        let reserved_value_for_short = self.reserved_value(&prices.index_token_price, false)?;
        let pool_value_without_pnl_for_long = Value {
            min: self.pool_value_without_pnl_for_one_side(prices, true, false)?,
            max: self.pool_value_without_pnl_for_one_side(prices, true, true)?,
        };
        let pool_value_without_pnl_for_short = Value {
            min: self.pool_value_without_pnl_for_one_side(prices, false, false)?,
            max: self.pool_value_without_pnl_for_one_side(prices, false, true)?,
        };
        let reserve_factor = self
            .reserve_factor()?
            .min(self.open_interest_reserve_factor()?);
        let max_reserved_value_for_long = gmsol_model::utils::apply_factor::<
            _,
            { constants::MARKET_DECIMALS },
        >(
            &pool_value_without_pnl_for_long.min, &reserve_factor
        )
        .ok_or_else(|| crate::Error::unknown("failed to calculate max reserved value for long"))?;
        let max_reserved_value_for_short = gmsol_model::utils::apply_factor::<
            _,
            { constants::MARKET_DECIMALS },
        >(
            &pool_value_without_pnl_for_short.min, &reserve_factor
        )
        .ok_or_else(|| crate::Error::unknown("failed to calculate max reserved value for short"))?;

        Ok(MarketStatus {
            funding_rate_per_second_for_long,
            funding_rate_per_second_for_short,
            borrowing_rate_per_second_for_long: self.borrowing_factor_per_second(true, prices)?,
            borrowing_rate_per_second_for_short: self.borrowing_factor_per_second(false, prices)?,
            pending_pnl_for_long: SignedValue {
                min: self.pnl(&prices.index_token_price, true, false)?,
                max: self.pnl(&prices.index_token_price, true, true)?,
            },
            pending_pnl_for_short: SignedValue {
                min: self.pnl(&prices.index_token_price, false, false)?,
                max: self.pnl(&prices.index_token_price, false, true)?,
            },
            reserved_value_for_long,
            reserved_value_for_short,
            pool_value_without_pnl_for_long,
            pool_value_without_pnl_for_short,
            liquidity_for_long: max_reserved_value_for_long.saturating_sub(reserved_value_for_long),
            liquidity_for_short: max_reserved_value_for_short
                .saturating_sub(reserved_value_for_short),
            open_interest_for_long,
            open_interest_for_short,
            open_interest_in_tokens_for_long: open_interest_in_tokens.long_amount()?,
            open_interest_in_tokens_for_short: open_interest_in_tokens.short_amount()?,
        })
    }
}
