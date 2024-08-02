use num_traits::{CheckedSub, FromPrimitive, Zero};

use crate::{
    market::{BaseMarket, PositionImpactMarket, PositionImpactMarketExt},
    num::Unsigned,
};

/// Distribute Position Impact.
#[must_use]
pub struct DistributePositionImpact<M: BaseMarket<DECIMALS>, const DECIMALS: u8> {
    market: M,
}

/// Distribute Position Impact Report.
#[derive(Debug)]
pub struct DistributePositionImpactReport<T> {
    duration_in_seconds: u64,
    distribution_amount: T,
    next_position_impact_pool_amount: T,
}

impl<T> DistributePositionImpactReport<T> {
    /// Get considered duration in seconds.
    pub fn duration_in_seconds(&self) -> u64 {
        self.duration_in_seconds
    }

    /// Get distribution amount.
    pub fn distribution_amount(&self) -> &T {
        &self.distribution_amount
    }

    /// Get Next position impact pool amount.
    pub fn next_position_impact_pool_amount(&self) -> &T {
        &self.next_position_impact_pool_amount
    }
}

impl<M: PositionImpactMarket<DECIMALS>, const DECIMALS: u8> DistributePositionImpact<M, DECIMALS> {
    /// Execute.
    pub fn execute(mut self) -> crate::Result<DistributePositionImpactReport<M::Num>> {
        let duration_in_seconds = self
            .market
            .just_passed_in_seconds_for_position_impact_distribution()?;

        let (distribution_amount, next_position_impact_pool_amount) =
            self.pending_position_impact_pool_distribution_amount(duration_in_seconds)?;

        self.market
            .apply_delta_to_position_impact_pool(&distribution_amount.to_opposite_signed()?)?;

        Ok(DistributePositionImpactReport {
            duration_in_seconds,
            distribution_amount,
            next_position_impact_pool_amount,
        })
    }

    /// Get pending position impact pool distribution amount.
    fn pending_position_impact_pool_distribution_amount(
        &self,
        duration_in_secs: u64,
    ) -> crate::Result<(M::Num, M::Num)> {
        use crate::utils;

        let current_amount = self.market.position_impact_pool_amount()?;
        let params = self.market.position_impact_distribution_params()?;
        if params.distribute_factor().is_zero()
            || current_amount <= *params.min_position_impact_pool_amount()
        {
            return Ok((Zero::zero(), current_amount));
        }
        let max_distribution_amount = current_amount
            .checked_sub(params.min_position_impact_pool_amount())
            .ok_or(crate::Error::Computation(
                "calculating max distribution amount",
            ))?;

        let duration_value = M::Num::from_u64(duration_in_secs).ok_or(crate::Error::Convert)?;
        let mut distribution_amount =
            utils::apply_factor(&duration_value, params.distribute_factor())
                .ok_or(crate::Error::Computation("calculating distribution amount"))?;
        if distribution_amount > max_distribution_amount {
            distribution_amount = max_distribution_amount;
        }
        let next_amount =
            current_amount
                .checked_sub(&distribution_amount)
                .ok_or(crate::Error::Computation(
                    "calculating next position impact amount",
                ))?;
        Ok((distribution_amount, next_amount))
    }
}

impl<M: BaseMarket<DECIMALS>, const DECIMALS: u8> From<M>
    for DistributePositionImpact<M, DECIMALS>
{
    fn from(market: M) -> Self {
        Self { market }
    }
}

#[cfg(test)]
mod tests {
    use std::{thread::sleep, time::Duration};

    use crate::{
        action::Prices,
        market::LiquidityMarketExt,
        test::{TestMarket, TestPosition},
        PositionMutExt,
    };

    use super::*;

    #[test]
    fn test_distribute_position_impact() -> crate::Result<()> {
        let mut market = TestMarket::<u64, 9>::default();
        market.distribute_position_impact()?.execute()?;
        let prices = Prices {
            index_token_price: 120,
            long_token_price: 120,
            short_token_price: 1,
        };
        market
            .deposit(1_000_000_000_000, 100_000_000_000_000, prices)?
            .execute()?;
        println!("{market:#?}");
        let mut position = TestPosition::long(true);
        for _ in 0..100 {
            market.distribute_position_impact()?.execute()?;
            _ = position
                .ops(&mut market)
                .increase(
                    Prices {
                        index_token_price: 123,
                        long_token_price: 123,
                        short_token_price: 1,
                    },
                    1_000_000_000_000,
                    50_000_000_000_000,
                    None,
                )?
                .execute()?;
            market.distribute_position_impact()?.execute()?;
            _ = position
                .ops(&mut market)
                .decrease(
                    Prices {
                        index_token_price: 123,
                        long_token_price: 123,
                        short_token_price: 1,
                    },
                    50_000_000_000_000,
                    None,
                    0,
                    false,
                    false,
                )?
                .execute()?;
        }
        println!("{market:#?}");
        sleep(Duration::from_secs(1));
        let report = market.distribute_position_impact()?.execute()?;
        println!("{report:#?}");
        println!("{market:#?}");
        Ok(())
    }
}
