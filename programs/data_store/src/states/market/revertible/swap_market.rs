use anchor_lang::prelude::*;
use gmx_core::{
    params::{FeeParams, PriceImpactParams},
    Bank, PoolKind, SwapMarketExt,
};
use indexmap::{map::Entry, IndexMap};

use crate::{
    constants,
    states::{
        common::SwapParams, ops::ValidateMarketBalances, HasMarketMeta, Market, MarketMeta, Oracle,
    },
    DataStoreError, GmxCoreError,
};

use super::{RevertibleMarket, RevertiblePool};

/// A map of markets used for swaps where the key is the market token mint address.
pub struct SwapMarkets<'a>(IndexMap<Pubkey, RevertibleMarket<'a>>);

impl<'a> SwapMarkets<'a> {
    /// Create a new [`SwapMarkets`] from loders.
    pub fn new<'info>(
        loaders: &'a [AccountLoader<'info, Market>],
        current_market: Option<&Pubkey>,
    ) -> Result<Self> {
        let mut map = IndexMap::with_capacity(loaders.len());
        for loader in loaders {
            let key = loader.load()?.meta().market_token_mint;
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

    /// Get market mutably.
    pub fn get_mut(&mut self, token: &Pubkey) -> Option<&mut RevertibleMarket<'a>> {
        self.0.get_mut(token)
    }

    /// Revertible Swap.
    pub(crate) fn revertible_swap<M>(
        &mut self,
        mut direction: SwapDirection<M>,
        oracle: &Oracle,
        params: &SwapParams,
        expected_token_outs: (Pubkey, Pubkey),
        token_ins: (Option<Pubkey>, Option<Pubkey>),
        token_in_amounts: (u64, u64),
    ) -> Result<(u64, u64)>
    where
        M: Key
            + HasMarketMeta
            + gmx_core::Bank<Pubkey, Num = u64>
            + gmx_core::SwapMarket<{ constants::MARKET_DECIMALS }, Num = u128>,
    {
        let long_path = params.validated_long_path()?;
        let long_output_amount = token_ins
            .0
            .map(|token_in| {
                self.revertible_swap_for_one_side(
                    &mut direction,
                    oracle,
                    long_path,
                    expected_token_outs.0,
                    token_in,
                    token_in_amounts.0,
                )
            })
            .transpose()?
            .unwrap_or_default();
        let short_path = params.validated_short_path()?;
        let short_output_amount = token_ins
            .1
            .map(|token_in| {
                self.revertible_swap_for_one_side(
                    &mut direction,
                    oracle,
                    short_path,
                    expected_token_outs.1,
                    token_in,
                    token_in_amounts.1,
                )
            })
            .transpose()?
            .unwrap_or_default();
        Ok((long_output_amount, short_output_amount))
    }

    /// Swap along the path.
    ///
    /// ## Assumptions
    /// - The input amount is already deposited in the first market.
    /// - The path consists of the mint addresses of unique market tokens.
    ///
    /// ## Notes
    /// - The output amount will also remain deposited in the last market.
    fn swap_along_the_path(
        &mut self,
        oracle: &Oracle,
        path: &[Pubkey],
        token_in: &mut Pubkey,
        token_in_amount: &mut u64,
    ) -> Result<()> {
        if path.is_empty() {
            return Ok(());
        }
        let last_idx = path.len().saturating_sub(1);
        for (idx, market_token) in path.iter().enumerate() {
            let mut market = self
                .get_mut(market_token)
                .ok_or_else(|| {
                    msg!("Swap Error: missing market account for {}", market_token);
                    error!(DataStoreError::MissingMarketAccount)
                })?
                .as_swap_market()?;
            if idx != 0 {
                market
                    .record_transferred_in_by_token(token_in, token_in_amount)
                    .map_err(GmxCoreError::from)?;
            }
            let side = market.market_meta().to_token_side(token_in)?;
            let prices = oracle.market_prices(&market)?;
            let report = market
                .swap(side, (*token_in_amount).into(), prices)
                .map_err(GmxCoreError::from)?
                .execute()
                .map_err(GmxCoreError::from)?;
            msg!("swap along the path: {:?}", report);
            *token_in = *market.market_meta().opposite_token(token_in)?;
            *token_in_amount = (*report.token_out_amount())
                .try_into()
                .map_err(|_| error!(DataStoreError::AmountOverflow))?;
            if idx != last_idx {
                market
                    .record_transferred_out_by_token(token_in, token_in_amount)
                    .map_err(GmxCoreError::from)?;
                market.validate_market_balances(0, 0)?;
            } else {
                market
                    .validate_market_balances_excluding_token_amount(token_in, *token_in_amount)?;
            }
        }
        Ok(())
    }

    /// Swap for one side.
    ///
    /// ## Assumption
    /// - The market tokens in the path must be unique.
    fn revertible_swap_for_one_side<M>(
        &mut self,
        direction: &mut SwapDirection<M>,
        oracle: &Oracle,
        mut path: &[Pubkey],
        expected_token_out: Pubkey,
        mut token_in: Pubkey,
        mut token_in_amount: u64,
    ) -> Result<u64>
    where
        M: Key
            + gmx_core::Bank<Pubkey, Num = u64>
            + gmx_core::SwapMarket<{ constants::MARKET_DECIMALS }, Num = u128>
            + HasMarketMeta,
    {
        require!(
            self.get_mut(&direction.current()).is_none(),
            DataStoreError::InvalidSwapPath
        );
        if !path.is_empty() {
            let current = direction.current();

            let first_market_token = path.first().unwrap();
            if let SwapDirection::From(from_market) = direction {
                if *first_market_token != current {
                    let first_market = self.get_mut(first_market_token).ok_or_else(|| {
                        msg!(
                            "Swap Error: missing market account for {}",
                            first_market_token
                        );
                        error!(DataStoreError::MissingMarketAccount)
                    })?;
                    // We are assuming that they are sharing the same vault of `token_in`.
                    from_market
                        .record_transferred_out_by_token(&token_in, &token_in_amount)
                        .map_err(GmxCoreError::from)?;
                    first_market
                        .record_transferred_in_by_token(&token_in, &token_in_amount)
                        .map_err(GmxCoreError::from)?;
                }
            }

            if *first_market_token == direction.current() {
                direction.swap_with_current(oracle, &mut token_in, &mut token_in_amount)?;
                path = &path[1..];
            }

            if !path.is_empty() {
                let mut should_swap_with_current = false;
                let last_market_token = path.last().unwrap();

                if *last_market_token == direction.current() {
                    should_swap_with_current = true;
                    path = path.split_last().unwrap().1;
                }

                self.swap_along_the_path(oracle, path, &mut token_in, &mut token_in_amount)?;

                if should_swap_with_current {
                    direction.swap_with_current(oracle, &mut token_in, &mut token_in_amount)?;
                }

                if let SwapDirection::Into(into_market) = direction {
                    if *last_market_token != current {
                        let last_market = self.get_mut(last_market_token).ok_or_else(|| {
                            msg!(
                                "Swap Error: missing market account for {}",
                                last_market_token
                            );
                            error!(DataStoreError::MissingMarketAccount)
                        })?;
                        // We are assuming that they are sharing the same vault of `token_in`.
                        last_market
                            .record_transferred_out_by_token(&token_in, &token_in_amount)
                            .map_err(GmxCoreError::from)?;
                        into_market
                            .record_transferred_in_by_token(&token_in, &token_in_amount)
                            .map_err(GmxCoreError::from)?;
                    }
                }
            }
        }
        require_eq!(
            token_in,
            expected_token_out,
            DataStoreError::InvalidSwapPath
        );
        Ok(token_in_amount)
    }
}

pub(crate) enum SwapDirection<M> {
    From(M),
    Into(M),
}

impl<M> SwapDirection<M>
where
    M: Key,
{
    fn current(&self) -> Pubkey {
        match self {
            Self::From(p) | Self::Into(p) => p.key(),
        }
    }
}

impl<M> SwapDirection<M>
where
    M: HasMarketMeta + gmx_core::SwapMarket<{ constants::MARKET_DECIMALS }, Num = u128>,
{
    fn swap_with_current(
        &mut self,
        oracle: &Oracle,
        token_in: &mut Pubkey,
        token_in_amount: &mut u64,
    ) -> Result<()> {
        let current = match self {
            Self::From(m) | Self::Into(m) => m,
        };
        let side = current.market_meta().to_token_side(token_in)?;
        let prices = oracle.market_prices(current)?;
        let report = current
            .swap(side, (*token_in_amount).into(), prices)
            .map_err(GmxCoreError::from)?
            .execute()
            .map_err(GmxCoreError::from)?;
        msg!("swap in current market: {:?}", report);
        *token_in_amount = (*report.token_out_amount())
            .try_into()
            .map_err(|_| error!(DataStoreError::AmountOverflow))?;
        *token_in = *current.market_meta().opposite_token(token_in)?;
        Ok(())
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

impl<'a, 'market> HasMarketMeta for AsSwapMarket<'a, 'market> {
    fn is_pure(&self) -> bool {
        self.market.is_pure()
    }

    fn market_meta(&self) -> &MarketMeta {
        self.market.market_meta()
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

impl<'a, 'market> gmx_core::Bank<Pubkey> for AsSwapMarket<'a, 'market> {
    type Num = u64;

    fn record_transferred_in_by_token<Q: std::borrow::Borrow<Pubkey> + ?Sized>(
        &mut self,
        token: &Q,
        amount: &Self::Num,
    ) -> gmx_core::Result<()> {
        self.market
            .record_transferred_in_by_token(token.borrow(), amount)
    }

    fn record_transferred_out_by_token<Q: std::borrow::Borrow<Pubkey> + ?Sized>(
        &mut self,
        token: &Q,
        amount: &Self::Num,
    ) -> gmx_core::Result<()> {
        self.market
            .record_transferred_out_by_token(token.borrow(), amount)
    }

    fn balance<Q: std::borrow::Borrow<Pubkey> + ?Sized>(
        &self,
        token: &Q,
    ) -> gmx_core::Result<Self::Num> {
        self.market.balance(token)
    }
}

impl<'a, 'market> Key for AsSwapMarket<'a, 'market> {
    fn key(&self) -> Pubkey {
        self.market.key()
    }
}
