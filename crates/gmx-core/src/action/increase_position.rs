use crate::position::Position;

use super::Prices;

/// Increase the position.
#[must_use]
pub struct IncreasePosition<P: Position<DECIMALS>, const DECIMALS: u8> {
    position: P,
    params: IncreasePositionParams<P::Num>,
}

/// Increase Position Params.
#[derive(Debug, Clone, Copy)]
pub struct IncreasePositionParams<T> {
    collateral_increment_amount: T,
    size_delta_usd: T,
    acceptable_price: Option<T>,
    prices: Prices<T>,
}

/// Report of the execution of position increasing.
#[derive(Debug, Clone, Copy)]
pub struct IncreasePositionReport<T> {
    params: IncreasePositionParams<T>,
}

impl<T> IncreasePositionReport<T> {
    fn new(params: IncreasePositionParams<T>) -> Self {
        Self { params }
    }

    /// Get params.
    pub fn params(&self) -> &IncreasePositionParams<T> {
        &self.params
    }
}

impl<const DECIMALS: u8, P: Position<DECIMALS>> IncreasePosition<P, DECIMALS> {
    /// Create a new action to increase the given position.
    pub fn try_new(
        position: P,
        prices: Prices<P::Num>,
        collateral_increment_amount: P::Num,
        size_delta_usd: P::Num,
        acceptable_price: Option<P::Num>,
    ) -> crate::Result<Self> {
        Ok(Self {
            position,
            params: IncreasePositionParams {
                collateral_increment_amount,
                size_delta_usd,
                acceptable_price,
                prices,
            },
        })
    }

    /// Execute.
    pub fn execute(self) -> crate::Result<IncreasePositionReport<P::Num>> {
        Ok(IncreasePositionReport::new(self.params))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        position::PositionExt,
        test::{TestMarket, TestPosition},
        MarketExt,
    };

    use super::*;

    #[test]
    fn basic() -> crate::Result<()> {
        let mut market = TestMarket::<u64, 9>::default();
        market.deposit(1_000_000_000, 0, 120, 1)?.execute()?;
        market.deposit(0, 1_000_000_000, 120, 1)?.execute()?;
        println!("{market:#?}");
        let mut position = TestPosition::default();
        let report = position
            .ops(&mut market)
            .increase(
                Prices {
                    index_token_price: 123,
                    long_token_price: 123,
                    short_token_price: 1,
                },
                1_000_000,
                80_000_000,
                None,
            )?
            .execute()?;
        println!("{report:#?}");
        Ok(())
    }
}
