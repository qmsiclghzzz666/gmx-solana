use anchor_lang::prelude::*;
use gmx_core::{
    params::{FeeParams, PriceImpactParams},
    PoolKind,
};
use indexmap::{map::Entry, IndexMap};

use crate::{constants, states::Market, DataStoreError};

use super::{RevertibleMarket, RevertiblePool};

/// A collection of markets for swap.
pub struct SwapMarkets<'a>(IndexMap<Pubkey, RevertibleMarket<'a>>);

impl<'a> SwapMarkets<'a> {
    /// Create a new [`SwapMarkets`] from loders.
    pub fn new<'info>(
        loaders: &'a [AccountLoader<'info, Market>],
        current_market: Option<&Pubkey>,
    ) -> Result<Self> {
        let mut map = IndexMap::with_capacity(loaders.len());
        for loader in loaders {
            let key = loader.key();
            if let Some(market) = current_market {
                require!(key != *market, DataStoreError::InvalidSwapPath);
            }
            match map.entry(key) {
                // Cannot have duplicated markets.
                Entry::Occupied(_) => return err!(DataStoreError::InvalidSwapPath),
                Entry::Vacant(e) => {
                    e.insert(RevertibleMarket::try_from(loader)?);
                }
            }
        }
        Ok(Self(map))
    }

    /// Commit the swap.
    /// ## Panic
    /// Panic if one of the commitments panics.
    pub fn commit(self) {
        for market in self.0.into_values() {
            market.commit();
        }
    }
}

/// Convert a [`RevertibleMarket`] to a [`SwapMarket`](gmx_core::SwapMarket).
pub struct AsSwapMarket<'a, 'market> {
    market: &'a mut RevertibleMarket<'market>,
    open_interest: (RevertiblePool, RevertiblePool),
    open_interest_in_tokens: (RevertiblePool, RevertiblePool),
}

impl<'a, 'market> AsSwapMarket<'a, 'market> {
    pub(crate) fn new(market: &'a mut RevertibleMarket<'market>) -> Result<Self> {
        let open_interest = (
            market.get_pool_from_storage(PoolKind::OpenInterestForLong)?,
            market.get_pool_from_storage(PoolKind::OpenInterestForShort)?,
        );
        let open_interest_in_tokens = (
            market.get_pool_from_storage(PoolKind::OpenInterestInTokensForLong)?,
            market.get_pool_from_storage(PoolKind::OpenInterestInTokensForShort)?,
        );
        Ok(Self {
            market,
            open_interest,
            open_interest_in_tokens,
        })
    }
}

impl<'a, 'market> gmx_core::BaseMarket<{ constants::MARKET_DECIMALS }>
    for AsSwapMarket<'a, 'market>
{
    type Num = u128;

    type Signed = i128;

    type Pool = RevertiblePool;

    fn liquidity_pool(&self) -> gmx_core::Result<&Self::Pool> {
        self.market.liquidity_pool()
    }

    fn liquidity_pool_mut(&mut self) -> gmx_core::Result<&mut Self::Pool> {
        self.market.liquidity_pool_mut()
    }

    fn claimable_fee_pool(&self) -> gmx_core::Result<&Self::Pool> {
        self.market.claimable_fee_pool()
    }

    fn claimable_fee_pool_mut(&mut self) -> gmx_core::Result<&mut Self::Pool> {
        self.market.claimable_fee_pool_mut()
    }

    fn swap_impact_pool(&self) -> gmx_core::Result<&Self::Pool> {
        self.market.swap_impact_pool()
    }

    fn open_interest_pool(&self, is_long: bool) -> gmx_core::Result<&Self::Pool> {
        Ok(if is_long {
            &self.open_interest.0
        } else {
            &self.open_interest.1
        })
    }

    fn open_interest_in_tokens_pool(&self, is_long: bool) -> gmx_core::Result<&Self::Pool> {
        Ok(if is_long {
            &self.open_interest_in_tokens.0
        } else {
            &self.open_interest_in_tokens.1
        })
    }

    fn usd_to_amount_divisor(&self) -> Self::Num {
        self.market.usd_to_amount_divisor()
    }

    fn max_pool_amount(&self, is_long_token: bool) -> gmx_core::Result<Self::Num> {
        self.market.max_pool_amount(is_long_token)
    }

    fn max_pnl_factor(
        &self,
        kind: gmx_core::PnlFactorKind,
        is_long: bool,
    ) -> gmx_core::Result<Self::Num> {
        self.market.max_pnl_factor(kind, is_long)
    }

    fn reserve_factor(&self) -> gmx_core::Result<Self::Num> {
        self.market.reserve_factor()
    }
}

impl<'a, 'market> gmx_core::SwapMarket<{ constants::MARKET_DECIMALS }>
    for AsSwapMarket<'a, 'market>
{
    fn swap_impact_params(&self) -> gmx_core::Result<PriceImpactParams<Self::Num>> {
        PriceImpactParams::builder()
            .with_exponent(self.market.config().swap_impact_exponent)
            .with_positive_factor(self.market.config().swap_impact_positive_factor)
            .with_negative_factor(self.market.config().swap_impact_negative_factor)
            .build()
    }

    fn swap_fee_params(&self) -> gmx_core::Result<FeeParams<Self::Num>> {
        Ok(FeeParams::builder()
            .with_fee_receiver_factor(self.market.config().swap_fee_receiver_factor)
            .with_positive_impact_fee_factor(
                self.market.config().swap_fee_factor_for_positive_impact,
            )
            .with_negative_impact_fee_factor(
                self.market.config().swap_fee_factor_for_positive_impact,
            )
            .build())
    }

    fn swap_impact_pool_mut(&mut self) -> gmx_core::Result<&mut Self::Pool> {
        Ok(&mut self.market.swap_impact)
    }
}
