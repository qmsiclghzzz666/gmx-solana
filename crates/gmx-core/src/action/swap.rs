use crate::{params::Fees, Market};

use num_traits::Zero;

/// A swap.
#[must_use]
pub struct Swap<M: Market<DECIMALS>, const DECIMALS: u8> {
    market: M,
    params: SwapParams<M::Num>,
}

/// Swap params.
#[derive(Debug, Clone, Copy)]
pub struct SwapParams<T> {
    is_token_in_long: bool,
    token_in_amount: T,
    long_token_price: T,
    short_token_price: T,
}

impl<T> SwapParams<T> {
    /// Get long token price.
    pub fn long_token_price(&self) -> &T {
        &self.long_token_price
    }

    /// Get short token price.
    pub fn short_token_price(&self) -> &T {
        &self.short_token_price
    }

    /// Whether the in token is long token.
    pub fn is_token_in_long(&self) -> bool {
        self.is_token_in_long
    }

    /// Get the amount of in token.
    pub fn token_in_amount(&self) -> &T {
        &self.token_in_amount
    }
}

/// Report of the execution of swap.
#[must_use = "`token_out_amount` must use"]
#[derive(Debug, Clone, Copy)]
pub struct SwapReport<T> {
    params: SwapParams<T>,
    token_in_fees: Fees<T>,
    token_out_amount: T,
}

impl<T> SwapReport<T> {
    /// Get swap params.
    pub fn params(&self) -> &SwapParams<T> {
        &self.params
    }

    /// Get token in fees.
    pub fn token_in_fees(&self) -> &Fees<T> {
        &self.token_in_fees
    }

    /// Get the amount of out token.
    pub fn token_out_amount(&self) -> &T {
        &self.token_out_amount
    }
}

impl<const DECIMALS: u8, M: Market<DECIMALS>> Swap<M, DECIMALS> {
    /// Create a new swap in the given market.
    pub fn try_new(
        market: M,
        is_token_in_long: bool,
        token_in_amount: M::Num,
        long_token_price: M::Num,
        short_token_price: M::Num,
    ) -> crate::Result<Self> {
        if token_in_amount.is_zero() {
            return Err(crate::Error::EmptySwap);
        }
        if long_token_price.is_zero() || short_token_price.is_zero() {
            return Err(crate::Error::InvalidPrices);
        }
        Ok(Self {
            market,
            params: SwapParams {
                is_token_in_long,
                token_in_amount,
                long_token_price,
                short_token_price,
            },
        })
    }

    /// Execute the swap.
    pub fn execute(self) -> crate::Result<SwapReport<M::Num>> {
        self.market.swap_impact_params();
        Ok(SwapReport {
            params: self.params,
            token_in_fees: Fees::default(),
            token_out_amount: Zero::zero(),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{test::TestMarket, MarketExt};

    #[test]
    fn basic() -> crate::Result<()> {
        let mut market = TestMarket::<u64, 9>::default();
        market.deposit(1_000_000_000, 0, 120, 1)?.execute()?;
        market.deposit(1_000_000_000, 0, 120, 1)?.execute()?;
        market.deposit(0, 1_000_000_000, 120, 1)?.execute()?;
        println!("{market:#?}");
        let report = market.swap(true, 100_000_000, 120, 1)?.execute()?;
        println!("{report:#?}");
        println!("{market:#?}");
        Ok(())
    }
}
