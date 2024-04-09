use std::fmt;

use crate::{num::Unsigned, position::Position};

use super::Prices;

/// Decrease the position.
#[must_use]
pub struct DecreasePosition<P: Position<DECIMALS>, const DECIMALS: u8> {
    position: P,
    params: DecreasePositionParams<P::Num>,
}

/// Decrease Position Params.
#[derive(Debug, Clone, Copy)]
pub struct DecreasePositionParams<T> {
    acceptable_price: Option<T>,
    prices: Prices<T>,
}

/// Report of the execution of posiiton decreasing.
pub struct DecreasePositionReport<T: Unsigned> {
    params: DecreasePositionParams<T>,
    execution: ExecutionParams<T>,
}

impl<T: Unsigned + fmt::Debug> fmt::Debug for DecreasePositionReport<T>
where
    T::Signed: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DecreasePositionReport")
            .field("params", &self.params)
            .finish()
    }
}

impl<T: Unsigned> DecreasePositionReport<T> {
    fn new(params: DecreasePositionParams<T>, execution: ExecutionParams<T>) -> Self {
        Self { params, execution }
    }

    /// Get params.
    pub fn params(&self) -> &DecreasePositionParams<T> {
        &self.params
    }

    /// Get execution params.
    pub fn execution_params(&self) -> &ExecutionParams<T> {
        &self.execution
    }
}

/// Exeuction Params for decreasing position.
#[derive(Debug, Clone, Copy)]
pub struct ExecutionParams<T: Unsigned> {
    execution_price: T,
}

impl<T: Unsigned> ExecutionParams<T> {
    /// Get execution price.
    pub fn execution_price(&self) -> &T {
        &self.execution_price
    }
}

impl<const DECIMALS: u8, P: Position<DECIMALS>> DecreasePosition<P, DECIMALS> {
    /// Create a new action to decrease the given position.
    pub fn try_new(
        position: P,
        prices: Prices<P::Num>,
        acceptable_price: Option<P::Num>,
    ) -> crate::Result<Self> {
        if !prices.is_valid() {
            return Err(crate::Error::invalid_argument("invalid prices"));
        }
        Ok(Self {
            position,
            params: DecreasePositionParams {
                acceptable_price,
                prices,
            },
        })
    }

    /// Execute.
    pub fn execute(self) -> crate::Result<DecreasePositionReport<P::Num>> {
        let execution = self.get_execution_params()?;
        Ok(DecreasePositionReport::new(self.params, execution))
    }

    fn get_execution_params(&self) -> crate::Result<ExecutionParams<P::Num>> {
        Ok(ExecutionParams {
            execution_price: self.params.prices.index_token_price.clone(),
        })
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
        let mut position = TestPosition::long();
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
        println!("{position:#?}");
        let report = position
            .ops(&mut market)
            .decrease(
                Prices {
                    index_token_price: 125,
                    long_token_price: 125,
                    short_token_price: 1,
                },
                None,
            )?
            .execute()?;
        println!("{report:#?}");
        println!("{position:#?}");
        Ok(())
    }
}
