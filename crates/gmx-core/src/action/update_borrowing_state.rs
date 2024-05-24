use crate::{num::Unsigned, ClockKind, Market, MarketExt};

use super::Prices;

/// Update Borrowing State.
#[must_use]
pub struct UpdateBorrowingState<M: Market<DECIMALS>, const DECIMALS: u8> {
    market: M,
    prices: Prices<M::Num>,
}

impl<M: Market<DECIMALS>, const DECIMALS: u8> UpdateBorrowingState<M, DECIMALS> {
    /// Create a new [`UpdateBorrowingState`] action.
    pub fn try_new(market: M, prices: &Prices<M::Num>) -> crate::Result<Self> {
        prices.validate()?;
        Ok(Self {
            market,
            prices: prices.clone(),
        })
    }

    fn execute_one_side(
        &mut self,
        is_long: bool,
        duration_in_seconds: u64,
    ) -> crate::Result<M::Num> {
        let (next_cumulative_borrowing_factor, delta) = self
            .market
            .calc_next_cumulative_borrowing_factor(is_long, &self.prices, duration_in_seconds)?;
        self.market
            .apply_delta_to_borrowing_factor(is_long, &delta.to_signed()?)?;
        Ok(next_cumulative_borrowing_factor)
    }

    /// Execute.
    pub fn execute(mut self) -> crate::Result<UpdateBorrowingReport<M::Num>> {
        let duration_in_seconds = self.market.just_passed_in_seconds(ClockKind::Borrowing)?;
        let next_cumulative_borrowing_factor_for_long =
            self.execute_one_side(true, duration_in_seconds)?;
        let next_cumulative_borrowing_factor_for_short =
            self.execute_one_side(false, duration_in_seconds)?;
        Ok(UpdateBorrowingReport {
            duration_in_seconds,
            next_cumulative_borrowing_factor_for_long,
            next_cumulative_borrowing_factor_for_short,
        })
    }
}

/// Update Borrowing Report.
#[derive(Debug)]
pub struct UpdateBorrowingReport<T> {
    duration_in_seconds: u64,
    next_cumulative_borrowing_factor_for_long: T,
    next_cumulative_borrowing_factor_for_short: T,
}

impl<T> UpdateBorrowingReport<T> {
    /// Get considered duration in seconds.
    pub fn duration_in_seconds(&self) -> u64 {
        self.duration_in_seconds
    }

    /// Get next cumulative borrowing factor for long.
    pub fn next_cumulative_borrowing_factor_for_long(&self) -> &T {
        &self.next_cumulative_borrowing_factor_for_long
    }

    /// Get next cumulative borrowing factor for short.
    pub fn next_cumulative_borrowing_factor_for_short(&self) -> &T {
        &self.next_cumulative_borrowing_factor_for_short
    }
}

#[cfg(test)]
mod tests {
    use std::{thread::sleep, time::Duration};

    use crate::{
        test::{TestMarket, TestPosition},
        PositionExt,
    };

    use super::*;

    #[test]
    fn test_update_borrowing_state() -> crate::Result<()> {
        let mut market = TestMarket::<u64, 9>::default();
        market
            .deposit(1_000_000_000_000, 100_000_000_000_000, 120, 1)?
            .execute()?;
        println!("{market:#?}");
        let mut position = TestPosition::long(true);
        let prices = Prices {
            index_token_price: 123,
            long_token_price: 123,
            short_token_price: 1,
        };
        let report = position
            .ops(&mut market)
            .increase(prices, 1_000_000_000_000, 50_000_000_000_000, None)?
            .execute()?;
        println!("{report:#?}");
        println!("{market:#?}");
        sleep(Duration::from_secs(2));
        let report = position
            .ops(&mut market)
            .decrease(prices, 50_000_000_000_000, None, 0, false, false)?
            .execute()?;
        println!("{report:#?}");
        println!("{market:#?}");
        Ok(())
    }
}
