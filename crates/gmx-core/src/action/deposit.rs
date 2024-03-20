use crate::market::Market;

/// A deposit.
#[must_use = "Action do nothing if not execute"]
pub struct Deposit<M: Market> {
    market: M,
    long_token_amount: M::Num,
    short_token_amount: M::Num,
}

impl<M: Market> Deposit<M> {
    /// Create a new deposit to the given market.
    pub fn new(market: M, long_token_amount: M::Num, short_token_amount: M::Num) -> Self {
        Self {
            market,
            long_token_amount,
            short_token_amount,
        }
    }

    /// Execute.
    pub fn execute(self) -> Result<(), crate::Error> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::TestMarket;

    #[test]
    fn basic() -> Result<(), crate::Error> {
        let mut market = TestMarket::default();
        Deposit::new(&mut market, 1000, 0).execute()?;
        Ok(())
    }
}
