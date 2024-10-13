use anchor_lang::prelude::*;
use gmsol_model::{
    params::{
        fee::{BorrowingFeeParams, FundingFeeParams},
        position::PositionImpactDistributionParams,
        FeeParams, PositionParams, PriceImpactParams,
    },
    ClockKind, PoolKind,
};

use crate::{
    constants,
    states::{
        market::clock::{AsClock, AsClockMut},
        HasMarketMeta, Market,
    },
    CoreError,
};

use super::{Revertible, RevertibleMarket, RevertiblePool};

/// Convert a [`RevertibleMarket`] to a [`PerpMarket`](gmsol_model::PerpMarket).
pub struct RevertiblePerpMarket<'a> {
    market: RevertibleMarket<'a>,
    clocks: Clocks,
    pools: Box<Pools>,
    state: State,
    order_fee_discount_factor: u128,
}

impl<'a> RevertiblePerpMarket<'a> {
    /// Next trade id.
    ///
    /// This method is idempotent, meaning that multiple calls to it
    /// result in the same state changes as a single call.
    pub fn next_trade_id(&mut self) -> Result<u64> {
        let next_trade_id = self
            .state
            .trade_id
            .checked_add(1)
            .ok_or(error!(CoreError::TokenAmountOverflow))?;
        self.state.next_trade_id = Some(next_trade_id);
        Ok(next_trade_id)
    }
}

struct Clocks {
    position_impact_distribution_clock: i64,
    borrowing_clock: i64,
    funding_clock: i64,
}

impl<'a, 'market> TryFrom<&'a RevertibleMarket<'market>> for Clocks {
    type Error = Error;

    fn try_from(market: &'a RevertibleMarket<'market>) -> Result<Self> {
        Ok(Self {
            position_impact_distribution_clock: market
                .get_clock(ClockKind::PriceImpactDistribution)?,
            borrowing_clock: market.get_clock(ClockKind::Borrowing)?,
            funding_clock: market.get_clock(ClockKind::Funding)?,
        })
    }
}

impl Clocks {
    fn write_to_market(&self, market: &mut Market) {
        *market
            .clocks
            .get_mut(ClockKind::PriceImpactDistribution)
            .expect("must exist") = self.position_impact_distribution_clock;
        *market
            .clocks
            .get_mut(ClockKind::Borrowing)
            .expect("must exist") = self.borrowing_clock;
        *market
            .clocks
            .get_mut(ClockKind::Funding)
            .expect("must exist") = self.funding_clock;
    }
}

struct Pools {
    swap_impact: RevertiblePool,
    position_impact: RevertiblePool,
    open_interest: (RevertiblePool, RevertiblePool),
    open_interest_in_tokens: (RevertiblePool, RevertiblePool),
    borrowing_factor: RevertiblePool,
    funding_amount_per_size: (RevertiblePool, RevertiblePool),
    claimable_funding_amount_per_size: (RevertiblePool, RevertiblePool),
    collateral_sum: (RevertiblePool, RevertiblePool),
    total_borrowing: RevertiblePool,
}

impl<'a, 'market> TryFrom<&'a RevertibleMarket<'market>> for Pools {
    type Error = Error;

    fn try_from(market: &'a RevertibleMarket<'market>) -> Result<Self> {
        Ok(Self {
            swap_impact: market.create_revertible_pool(PoolKind::SwapImpact)?,
            position_impact: market.create_revertible_pool(PoolKind::PositionImpact)?,
            open_interest: (
                market.create_revertible_pool(PoolKind::OpenInterestForLong)?,
                market.create_revertible_pool(PoolKind::OpenInterestForShort)?,
            ),
            open_interest_in_tokens: (
                market.create_revertible_pool(PoolKind::OpenInterestInTokensForLong)?,
                market.create_revertible_pool(PoolKind::OpenInterestInTokensForShort)?,
            ),
            borrowing_factor: market.create_revertible_pool(PoolKind::BorrowingFactor)?,
            funding_amount_per_size: (
                market.create_revertible_pool(PoolKind::FundingAmountPerSizeForLong)?,
                market.create_revertible_pool(PoolKind::FundingAmountPerSizeForShort)?,
            ),
            claimable_funding_amount_per_size: (
                market.create_revertible_pool(PoolKind::ClaimableFundingAmountPerSizeForLong)?,
                market.create_revertible_pool(PoolKind::ClaimableFundingAmountPerSizeForShort)?,
            ),
            collateral_sum: (
                market.create_revertible_pool(PoolKind::CollateralSumForLong)?,
                market.create_revertible_pool(PoolKind::CollateralSumForShort)?,
            ),
            total_borrowing: market.create_revertible_pool(PoolKind::TotalBorrowing)?,
        })
    }
}

impl Pools {
    fn write_to_market(&self, market: &mut Market) {
        self.swap_impact.as_small_pool().write_to_pool(
            market
                .pools
                .get_mut(PoolKind::SwapImpact)
                .expect("must exist"),
        );

        self.position_impact.as_small_pool().write_to_pool(
            market
                .pools
                .get_mut(PoolKind::PositionImpact)
                .expect("must exist"),
        );

        self.open_interest.0.as_small_pool().write_to_pool(
            market
                .pools
                .get_mut(PoolKind::OpenInterestForLong)
                .expect("must exist"),
        );
        self.open_interest.1.as_small_pool().write_to_pool(
            market
                .pools
                .get_mut(PoolKind::OpenInterestForShort)
                .expect("must exist"),
        );

        self.open_interest_in_tokens
            .0
            .as_small_pool()
            .write_to_pool(
                market
                    .pools
                    .get_mut(PoolKind::OpenInterestInTokensForLong)
                    .expect("must exist"),
            );
        self.open_interest_in_tokens
            .1
            .as_small_pool()
            .write_to_pool(
                market
                    .pools
                    .get_mut(PoolKind::OpenInterestInTokensForShort)
                    .expect("must exist"),
            );

        self.borrowing_factor.as_small_pool().write_to_pool(
            market
                .pools
                .get_mut(PoolKind::BorrowingFactor)
                .expect("must exist"),
        );

        self.funding_amount_per_size
            .0
            .as_small_pool()
            .write_to_pool(
                market
                    .pools
                    .get_mut(PoolKind::FundingAmountPerSizeForLong)
                    .expect("must exist"),
            );
        self.funding_amount_per_size
            .1
            .as_small_pool()
            .write_to_pool(
                market
                    .pools
                    .get_mut(PoolKind::FundingAmountPerSizeForShort)
                    .expect("must exist"),
            );

        self.claimable_funding_amount_per_size
            .0
            .as_small_pool()
            .write_to_pool(
                market
                    .pools
                    .get_mut(PoolKind::ClaimableFundingAmountPerSizeForLong)
                    .expect("must exist"),
            );
        self.claimable_funding_amount_per_size
            .1
            .as_small_pool()
            .write_to_pool(
                market
                    .pools
                    .get_mut(PoolKind::ClaimableFundingAmountPerSizeForShort)
                    .expect("must exist"),
            );

        self.collateral_sum.0.as_small_pool().write_to_pool(
            market
                .pools
                .get_mut(PoolKind::CollateralSumForLong)
                .expect("must exist"),
        );
        self.collateral_sum.1.as_small_pool().write_to_pool(
            market
                .pools
                .get_mut(PoolKind::CollateralSumForShort)
                .expect("must exist"),
        );

        self.total_borrowing.as_small_pool().write_to_pool(
            market
                .pools
                .get_mut(PoolKind::TotalBorrowing)
                .expect("must exist"),
        );
    }
}

struct State {
    trade_id: u64,
    next_trade_id: Option<u64>,
    funding_factor_per_second: i128,
}

impl<'a, 'market> TryFrom<&'a RevertibleMarket<'market>> for State {
    type Error = Error;

    fn try_from(market: &'a RevertibleMarket<'market>) -> Result<Self> {
        let trade_id = market.state().trade_count();
        Ok(Self {
            trade_id,
            next_trade_id: None,
            funding_factor_per_second: market.state().funding_factor_per_second,
        })
    }
}

impl State {
    fn write_to_market(&self, market: &mut Market) {
        market.state.funding_factor_per_second = self.funding_factor_per_second;
        if let Some(next_trade_id) = self.next_trade_id {
            let trade_id = market.state.next_trade_id().expect("must success");
            assert_eq!(trade_id, next_trade_id);
        }
    }
}

impl<'a> Key for RevertiblePerpMarket<'a> {
    fn key(&self) -> anchor_lang::prelude::Pubkey {
        self.market.key()
    }
}

impl<'a> HasMarketMeta for RevertiblePerpMarket<'a> {
    fn is_pure(&self) -> bool {
        self.market.is_pure()
    }

    fn market_meta(&self) -> &crate::states::MarketMeta {
        self.market.market_meta()
    }
}

impl<'a> gmsol_model::Bank<Pubkey> for RevertiblePerpMarket<'a> {
    type Num = u64;

    fn record_transferred_in_by_token<Q: std::borrow::Borrow<Pubkey> + ?Sized>(
        &mut self,
        token: &Q,
        amount: &Self::Num,
    ) -> gmsol_model::Result<()> {
        self.market.record_transferred_in_by_token(token, amount)
    }

    fn record_transferred_out_by_token<Q: std::borrow::Borrow<Pubkey> + ?Sized>(
        &mut self,
        token: &Q,
        amount: &Self::Num,
    ) -> gmsol_model::Result<()> {
        self.market.record_transferred_out_by_token(token, amount)
    }

    fn balance<Q: std::borrow::Borrow<Pubkey> + ?Sized>(
        &self,
        token: &Q,
    ) -> gmsol_model::Result<Self::Num> {
        self.market.balance(token)
    }
}

impl<'a> RevertiblePerpMarket<'a> {
    pub(crate) fn new<'info>(
        loader: &'a AccountLoader<'info, Market>,
        order_fee_discount_factor: u128,
    ) -> Result<Self> {
        let market = loader.try_into()?;
        Self::from_market(market, order_fee_discount_factor)
    }

    pub(crate) fn from_market(
        market: RevertibleMarket<'a>,
        order_fee_discount_factor: u128,
    ) -> Result<Self> {
        Ok(Self {
            pools: Box::new((&market).try_into()?),
            clocks: (&market).try_into()?,
            state: (&market).try_into()?,
            market,
            order_fee_discount_factor,
        })
    }
}

impl<'a> Revertible for RevertiblePerpMarket<'a> {
    fn commit(self) {
        self.market.commit_with(|market| {
            self.clocks.write_to_market(market);
            self.pools.write_to_market(market);
            self.state.write_to_market(market);
        });
    }
}

impl<'a> gmsol_model::BaseMarket<{ constants::MARKET_DECIMALS }> for RevertiblePerpMarket<'a> {
    type Num = u128;

    type Signed = i128;

    type Pool = RevertiblePool;

    fn liquidity_pool(&self) -> gmsol_model::Result<&Self::Pool> {
        self.market.liquidity_pool()
    }

    fn claimable_fee_pool(&self) -> gmsol_model::Result<&Self::Pool> {
        self.market.claimable_fee_pool()
    }

    fn swap_impact_pool(&self) -> gmsol_model::Result<&Self::Pool> {
        Ok(&self.pools.swap_impact)
    }

    fn open_interest_pool(&self, is_long: bool) -> gmsol_model::Result<&Self::Pool> {
        if is_long {
            Ok(&self.pools.open_interest.0)
        } else {
            Ok(&self.pools.open_interest.1)
        }
    }

    fn open_interest_in_tokens_pool(&self, is_long: bool) -> gmsol_model::Result<&Self::Pool> {
        if is_long {
            Ok(&self.pools.open_interest_in_tokens.0)
        } else {
            Ok(&self.pools.open_interest_in_tokens.1)
        }
    }

    fn collateral_sum_pool(&self, is_long: bool) -> gmsol_model::Result<&Self::Pool> {
        if is_long {
            Ok(&self.pools.collateral_sum.0)
        } else {
            Ok(&self.pools.collateral_sum.1)
        }
    }

    fn usd_to_amount_divisor(&self) -> Self::Num {
        self.market.usd_to_amount_divisor()
    }

    fn max_pool_amount(&self, is_long_token: bool) -> gmsol_model::Result<Self::Num> {
        self.market.max_pool_amount(is_long_token)
    }

    fn pnl_factor_config(
        &self,
        kind: gmsol_model::PnlFactorKind,
        is_long: bool,
    ) -> gmsol_model::Result<Self::Num> {
        self.market.pnl_factor_config(kind, is_long)
    }

    fn reserve_factor(&self) -> gmsol_model::Result<Self::Num> {
        self.market.reserve_factor()
    }
}

impl<'a> gmsol_model::BaseMarketMut<{ constants::MARKET_DECIMALS }> for RevertiblePerpMarket<'a> {
    fn liquidity_pool_mut(&mut self) -> gmsol_model::Result<&mut Self::Pool> {
        self.market.liquidity_pool_mut()
    }

    fn claimable_fee_pool_mut(&mut self) -> gmsol_model::Result<&mut Self::Pool> {
        self.market.claimable_fee_pool_mut()
    }
}

impl<'a> gmsol_model::SwapMarket<{ constants::MARKET_DECIMALS }> for RevertiblePerpMarket<'a> {
    fn swap_impact_params(
        &self,
    ) -> gmsol_model::Result<gmsol_model::params::PriceImpactParams<Self::Num>> {
        self.market.swap_impact_params()
    }

    fn swap_fee_params(&self) -> gmsol_model::Result<gmsol_model::params::FeeParams<Self::Num>> {
        self.market.swap_fee_params()
    }
}

impl<'a> gmsol_model::SwapMarketMut<{ constants::MARKET_DECIMALS }> for RevertiblePerpMarket<'a> {
    fn swap_impact_pool_mut(&mut self) -> gmsol_model::Result<&mut Self::Pool> {
        Ok(&mut self.pools.swap_impact)
    }
}

impl<'a> gmsol_model::PositionImpactMarket<{ constants::MARKET_DECIMALS }>
    for RevertiblePerpMarket<'a>
{
    fn position_impact_pool(&self) -> gmsol_model::Result<&Self::Pool> {
        Ok(&self.pools.position_impact)
    }

    fn position_impact_params(&self) -> gmsol_model::Result<PriceImpactParams<Self::Num>> {
        self.market.position_impact_params()
    }

    fn position_impact_distribution_params(
        &self,
    ) -> gmsol_model::Result<PositionImpactDistributionParams<Self::Num>> {
        self.market.position_impact_distribution_params()
    }

    fn passed_in_seconds_for_position_impact_distribution(&self) -> gmsol_model::Result<u64> {
        self.market
            .passed_in_seconds_for_position_impact_distribution()
    }
}

impl<'a> gmsol_model::PositionImpactMarketMut<{ constants::MARKET_DECIMALS }>
    for RevertiblePerpMarket<'a>
{
    fn position_impact_pool_mut(&mut self) -> gmsol_model::Result<&mut Self::Pool> {
        Ok(&mut self.pools.position_impact)
    }

    fn just_passed_in_seconds_for_position_impact_distribution(
        &mut self,
    ) -> gmsol_model::Result<u64> {
        AsClockMut::from(&mut self.clocks.position_impact_distribution_clock)
            .just_passed_in_seconds()
    }
}

impl<'a> gmsol_model::BorrowingFeeMarket<{ constants::MARKET_DECIMALS }>
    for RevertiblePerpMarket<'a>
{
    fn borrowing_factor_pool(&self) -> gmsol_model::Result<&Self::Pool> {
        Ok(&self.pools.borrowing_factor)
    }

    fn total_borrowing_pool(&self) -> gmsol_model::Result<&Self::Pool> {
        Ok(&self.pools.total_borrowing)
    }

    fn borrowing_fee_params(&self) -> gmsol_model::Result<BorrowingFeeParams<Self::Num>> {
        self.market.borrowing_fee_params()
    }

    fn passed_in_seconds_for_borrowing(&self) -> gmsol_model::Result<u64> {
        AsClock::from(&self.clocks.borrowing_clock).passed_in_seconds()
    }
}

impl<'a> gmsol_model::PerpMarket<{ constants::MARKET_DECIMALS }> for RevertiblePerpMarket<'a> {
    fn funding_factor_per_second(&self) -> &Self::Signed {
        &self.state.funding_factor_per_second
    }

    fn funding_amount_per_size_pool(&self, is_long: bool) -> gmsol_model::Result<&Self::Pool> {
        if is_long {
            Ok(&self.pools.funding_amount_per_size.0)
        } else {
            Ok(&self.pools.funding_amount_per_size.1)
        }
    }

    fn claimable_funding_amount_per_size_pool(
        &self,
        is_long: bool,
    ) -> gmsol_model::Result<&Self::Pool> {
        if is_long {
            Ok(&self.pools.claimable_funding_amount_per_size.0)
        } else {
            Ok(&self.pools.claimable_funding_amount_per_size.1)
        }
    }

    fn funding_amount_per_size_adjustment(&self) -> Self::Num {
        self.market.funding_amount_per_size_adjustment()
    }

    fn funding_fee_params(&self) -> gmsol_model::Result<FundingFeeParams<Self::Num>> {
        self.market.funding_fee_params()
    }

    fn position_params(&self) -> gmsol_model::Result<PositionParams<Self::Num>> {
        self.market.position_params()
    }

    fn order_fee_params(&self) -> gmsol_model::Result<FeeParams<Self::Num>> {
        Ok(self
            .market
            .order_fee_params()?
            .with_discount_factor(self.order_fee_discount_factor))
    }

    fn open_interest_reserve_factor(&self) -> gmsol_model::Result<Self::Num> {
        self.market.open_interest_reserve_factor()
    }

    fn max_open_interest(&self, is_long: bool) -> gmsol_model::Result<Self::Num> {
        self.market.max_open_interest(is_long)
    }

    fn min_collateral_factor_for_open_interest_multiplier(
        &self,
        is_long: bool,
    ) -> gmsol_model::Result<Self::Num> {
        self.market
            .min_collateral_factor_for_open_interest_multiplier(is_long)
    }
}

impl<'a> gmsol_model::PerpMarketMut<{ constants::MARKET_DECIMALS }> for RevertiblePerpMarket<'a> {
    fn just_passed_in_seconds_for_borrowing(&mut self) -> gmsol_model::Result<u64> {
        AsClockMut::from(&mut self.clocks.borrowing_clock).just_passed_in_seconds()
    }

    fn just_passed_in_seconds_for_funding(&mut self) -> gmsol_model::Result<u64> {
        AsClockMut::from(&mut self.clocks.funding_clock).just_passed_in_seconds()
    }

    fn funding_factor_per_second_mut(&mut self) -> &mut Self::Signed {
        &mut self.state.funding_factor_per_second
    }

    fn open_interest_pool_mut(&mut self, is_long: bool) -> gmsol_model::Result<&mut Self::Pool> {
        if is_long {
            Ok(&mut self.pools.open_interest.0)
        } else {
            Ok(&mut self.pools.open_interest.1)
        }
    }

    fn open_interest_in_tokens_pool_mut(
        &mut self,
        is_long: bool,
    ) -> gmsol_model::Result<&mut Self::Pool> {
        if is_long {
            Ok(&mut self.pools.open_interest_in_tokens.0)
        } else {
            Ok(&mut self.pools.open_interest_in_tokens.1)
        }
    }

    fn borrowing_factor_pool_mut(&mut self) -> gmsol_model::Result<&mut Self::Pool> {
        Ok(&mut self.pools.borrowing_factor)
    }

    fn funding_amount_per_size_pool_mut(
        &mut self,
        is_long: bool,
    ) -> gmsol_model::Result<&mut Self::Pool> {
        if is_long {
            Ok(&mut self.pools.funding_amount_per_size.0)
        } else {
            Ok(&mut self.pools.funding_amount_per_size.1)
        }
    }

    fn claimable_funding_amount_per_size_pool_mut(
        &mut self,
        is_long: bool,
    ) -> gmsol_model::Result<&mut Self::Pool> {
        if is_long {
            Ok(&mut self.pools.claimable_funding_amount_per_size.0)
        } else {
            Ok(&mut self.pools.claimable_funding_amount_per_size.1)
        }
    }

    fn collateral_sum_pool_mut(&mut self, is_long: bool) -> gmsol_model::Result<&mut Self::Pool> {
        if is_long {
            Ok(&mut self.pools.collateral_sum.0)
        } else {
            Ok(&mut self.pools.collateral_sum.1)
        }
    }

    fn total_borrowing_pool_mut(&mut self) -> gmsol_model::Result<&mut Self::Pool> {
        Ok(&mut self.pools.total_borrowing)
    }

    fn insufficient_funding_fee_payment(
        &mut self,
        paid_in_collateral_amount: &Self::Num,
        cost_amount: &Self::Num,
    ) -> gmsol_model::Result<()> {
        msg!(
            "insufficient funding fee payment: paid={}, cost={}",
            paid_in_collateral_amount,
            cost_amount
        );
        Ok(())
    }
}
