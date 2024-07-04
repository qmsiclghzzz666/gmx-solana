use anchor_lang::prelude::*;
use gmsol_model::{
    params::{FeeParams, PriceImpactParams},
    Bank, PoolKind, SwapMarketExt,
};
use indexmap::{map::Entry, IndexMap};

use crate::{
    constants,
    states::{
        common::SwapParams, ops::ValidateMarketBalances, HasMarketMeta, Market, MarketMeta, Oracle,
    },
    StoreError, ModelError,
};

use super::{Revertible, RevertibleMarket, RevertiblePool};

/// A map of markets used for swaps where the key is the market token mint address.
pub struct SwapMarkets<'a>(IndexMap<Pubkey, RevertibleSwapMarket<'a>>);

impl<'a> SwapMarkets<'a> {
    /// Create a new [`SwapMarkets`] from loders.
    pub fn new<'info>(
        store: &Pubkey,
        loaders: &'a [AccountLoader<'info, Market>],
        current_market_token: Option<&Pubkey>,
    ) -> Result<Self> {
        let mut map = IndexMap::with_capacity(loaders.len());
        for loader in loaders {
            let key = loader.load()?.meta().market_token_mint;
            if let Some(market_token) = current_market_token {
                require!(key != *market_token, StoreError::InvalidSwapPath);
            }
            match map.entry(key) {
                // Cannot have duplicated markets.
                Entry::Occupied(_) => return err!(StoreError::InvalidSwapPath),
                Entry::Vacant(e) => {
                    loader.load()?.validate(store)?;
                    let market =
                        RevertibleSwapMarket::from_market(RevertibleMarket::try_from(loader)?)?;
                    e.insert(market);
                }
            }
        }
        Ok(Self(map))
    }

    /// Get market mutably.
    pub fn get_mut(&mut self, token: &Pubkey) -> Option<&mut RevertibleSwapMarket<'a>> {
        self.0.get_mut(token)
    }

    /// Get market.
    pub fn get(&self, token: &Pubkey) -> Option<&RevertibleSwapMarket<'a>> {
        self.0.get(token)
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
            + gmsol_model::Bank<Pubkey, Num = u64>
            + gmsol_model::SwapMarket<{ constants::MARKET_DECIMALS }, Num = u128>,
    {
        let long_path = params.validated_long_path()?;
        let long_output_amount = token_ins
            .0
            .and_then(|token| (token_in_amounts.0 != 0).then_some(token))
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
            .and_then(|token| (token_in_amounts.1 != 0).then_some(token))
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

        // Validate token balances of output markets and current market.
        let current = direction.current();
        let mut long_output_market_token = long_path.last().unwrap_or(&current);
        let mut short_output_market_token = short_path.last().unwrap_or(&current);

        // When the swap direction is swapping into current market,
        // the balances should have been transferred into current market.
        if direction.is_into() {
            long_output_market_token = &current;
            short_output_market_token = &current;
        }

        let mut current_validated = false;
        if *long_output_market_token == *short_output_market_token {
            if *long_output_market_token != current {
                self.get(long_output_market_token)
                    .expect("must exist")
                    .validate_market_balances_excluding_the_given_token_amounts(
                        &expected_token_outs.0,
                        &expected_token_outs.1,
                        long_output_amount,
                        short_output_amount,
                    )?;
            } else {
                direction
                    .current_market()
                    .validate_market_balances_excluding_the_given_token_amounts(
                        &expected_token_outs.0,
                        &expected_token_outs.1,
                        long_output_amount,
                        short_output_amount,
                    )?;
                current_validated = true;
            }
        } else {
            for (market_token, amount, token) in [
                (
                    long_output_market_token,
                    long_output_amount,
                    &expected_token_outs.0,
                ),
                (
                    short_output_market_token,
                    short_output_amount,
                    &expected_token_outs.1,
                ),
            ] {
                if *market_token != current {
                    self.get(market_token)
                        .expect("must exist")
                        .validate_market_balances_excluding_the_given_token_amounts(
                            token, token, amount, 0,
                        )?;
                } else {
                    direction
                        .current_market()
                        .validate_market_balances_excluding_the_given_token_amounts(
                            token, token, amount, 0,
                        )?;
                    current_validated = true;
                }
            }
        }
        if !current_validated {
            direction.current_market().validate_market_balances(0, 0)?;
        }
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
            let market = self.get_mut(market_token).ok_or_else(|| {
                msg!("Swap Error: missing market account for {}", market_token);
                error!(StoreError::MissingMarketAccount)
            })?;
            if idx != 0 {
                market
                    .record_transferred_in_by_token(token_in, token_in_amount)
                    .map_err(ModelError::from)?;
            }
            let side = market.market_meta().to_token_side(token_in)?;
            let prices = oracle.market_prices(market)?;
            let report = market
                .swap(side, (*token_in_amount).into(), prices)
                .map_err(ModelError::from)?
                .execute()
                .map_err(ModelError::from)?;
            msg!("[Swap] swap along the path: {:?}", report);
            *token_in = *market.market_meta().opposite_token(token_in)?;
            *token_in_amount = (*report.token_out_amount())
                .try_into()
                .map_err(|_| error!(StoreError::AmountOverflow))?;
            // Only validate the market without extra balances.
            if idx != last_idx {
                market
                    .record_transferred_out_by_token(token_in, token_in_amount)
                    .map_err(ModelError::from)?;
                market.validate_market_balances(0, 0)?;
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
            + gmsol_model::Bank<Pubkey, Num = u64>
            + gmsol_model::SwapMarket<{ constants::MARKET_DECIMALS }, Num = u128>
            + HasMarketMeta,
    {
        require!(
            self.get_mut(&direction.current()).is_none(),
            StoreError::InvalidSwapPath
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
                        error!(StoreError::MissingMarketAccount)
                    })?;
                    // We are assuming that they are sharing the same vault of `token_in`.
                    from_market
                        .record_transferred_out_by_token(&token_in, &token_in_amount)
                        .map_err(ModelError::from)?;
                    // FIXME: is this validation needed?
                    from_market.validate_market_balance_for_the_given_token(&token_in, 0)?;
                    first_market
                        .record_transferred_in_by_token(&token_in, &token_in_amount)
                        .map_err(ModelError::from)?;
                }
            }

            if *first_market_token == current {
                // The validation of current market is delayed.
                direction.swap_with_current(oracle, &mut token_in, &mut token_in_amount)?;
                path = &path[1..];
                if let Some(first_market_token_to_swap_at) = path.first() {
                    debug_assert!(*first_market_token_to_swap_at != current);
                    let first_market =
                        self.get_mut(first_market_token_to_swap_at).ok_or_else(|| {
                            msg!(
                                "Swap Error: missing market account for {}",
                                first_market_token
                            );
                            error!(StoreError::MissingMarketAccount)
                        })?;
                    // We are assuming that they are sharing the same vault of `token_in`.
                    direction
                        .current_market_mut()
                        .record_transferred_out_by_token(&token_in, &token_in_amount)
                        .map_err(ModelError::from)?;
                    // The validation of current market is delayed.
                    first_market
                        .record_transferred_in_by_token(&token_in, &token_in_amount)
                        .map_err(ModelError::from)?;
                }
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
                    if let Some(last_swapped_market_token) = path.last() {
                        debug_assert!(*last_swapped_market_token != current);
                        let last_market =
                            self.get_mut(last_swapped_market_token).expect("must exist");
                        // We are assuming that they are sharing the same vault of `token_in`.
                        last_market
                            .record_transferred_out_by_token(&token_in, &token_in_amount)
                            .map_err(ModelError::from)?;
                        last_market.validate_market_balances(0, 0)?;
                        direction
                            .current_market_mut()
                            .record_transferred_in_by_token(&token_in, &token_in_amount)
                            .map_err(ModelError::from)?;
                    }
                    // The validation of current market is delayed.
                    direction.swap_with_current(oracle, &mut token_in, &mut token_in_amount)?;
                }

                if let SwapDirection::Into(into_market) = direction {
                    if *last_market_token != current {
                        let last_market = self.get_mut(last_market_token).expect("must exist");
                        // We are assuming that they are sharing the same vault of `token_in`.
                        last_market
                            .record_transferred_out_by_token(&token_in, &token_in_amount)
                            .map_err(ModelError::from)?;
                        last_market.validate_market_balances(0, 0)?;
                        into_market
                            .record_transferred_in_by_token(&token_in, &token_in_amount)
                            .map_err(ModelError::from)?;
                    }
                }
            }
        }
        require_eq!(
            token_in,
            expected_token_out,
            StoreError::InvalidSwapPath
        );
        Ok(token_in_amount)
    }
}

impl<'a> Revertible for SwapMarkets<'a> {
    /// Commit the swap.
    /// ## Panic
    /// Panic if one of the commitments panics.
    fn commit(self) {
        for market in self.0.into_values() {
            market.commit();
        }
    }
}

pub(crate) enum SwapDirection<'a, M> {
    From(&'a mut M),
    Into(&'a mut M),
}

impl<'a, M> SwapDirection<'a, M>
where
    M: Key,
{
    fn current(&self) -> Pubkey {
        match self {
            Self::From(p) | Self::Into(p) => p.key(),
        }
    }

    fn current_market_mut(&mut self) -> &mut M {
        match self {
            Self::From(m) | Self::Into(m) => m,
        }
    }

    fn current_market(&self) -> &M {
        match self {
            Self::From(m) | Self::Into(m) => m,
        }
    }

    fn is_into(&self) -> bool {
        matches!(self, Self::Into(_))
    }
}

impl<'a, M> SwapDirection<'a, M>
where
    M: HasMarketMeta + gmsol_model::SwapMarket<{ constants::MARKET_DECIMALS }, Num = u128>,
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
        let prices = oracle.market_prices(*current)?;
        let report = current
            .swap(side, (*token_in_amount).into(), prices)
            .map_err(ModelError::from)?
            .execute()
            .map_err(ModelError::from)?;
        msg!("[Swap] in current market: {:?}", report);
        *token_in_amount = (*report.token_out_amount())
            .try_into()
            .map_err(|_| error!(StoreError::AmountOverflow))?;
        *token_in = *current.market_meta().opposite_token(token_in)?;
        Ok(())
    }
}

/// Convert a [`RevertibleMarket`] to a [`SwapMarket`](gmsol_model::SwapMarket).
pub struct RevertibleSwapMarket<'a> {
    pub(super) market: RevertibleMarket<'a>,
    open_interest: (RevertiblePool, RevertiblePool),
    open_interest_in_tokens: (RevertiblePool, RevertiblePool),
}

impl<'a> RevertibleSwapMarket<'a> {
    pub(crate) fn from_market(market: RevertibleMarket<'a>) -> Result<Self> {
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

impl<'a> HasMarketMeta for RevertibleSwapMarket<'a> {
    fn is_pure(&self) -> bool {
        self.market.is_pure()
    }

    fn market_meta(&self) -> &MarketMeta {
        self.market.market_meta()
    }
}

impl<'a> Revertible for RevertibleSwapMarket<'a> {
    fn commit(self) {
        self.market.commit();
    }
}

impl<'a> RevertibleSwapMarket<'a> {
    /// Commit with the given function.
    ///
    /// ## Panic
    /// Panic if the commitment cannot be done.
    pub(crate) fn commit_with(self, f: impl FnOnce(&mut Market)) {
        self.market.commit_with(f)
    }
}

impl<'a> gmsol_model::BaseMarket<{ constants::MARKET_DECIMALS }> for RevertibleSwapMarket<'a> {
    type Num = u128;

    type Signed = i128;

    type Pool = RevertiblePool;

    fn liquidity_pool(&self) -> gmsol_model::Result<&Self::Pool> {
        self.market.liquidity_pool()
    }

    fn liquidity_pool_mut(&mut self) -> gmsol_model::Result<&mut Self::Pool> {
        self.market.liquidity_pool_mut()
    }

    fn claimable_fee_pool(&self) -> gmsol_model::Result<&Self::Pool> {
        self.market.claimable_fee_pool()
    }

    fn claimable_fee_pool_mut(&mut self) -> gmsol_model::Result<&mut Self::Pool> {
        self.market.claimable_fee_pool_mut()
    }

    fn swap_impact_pool(&self) -> gmsol_model::Result<&Self::Pool> {
        self.market.swap_impact_pool()
    }

    fn open_interest_pool(&self, is_long: bool) -> gmsol_model::Result<&Self::Pool> {
        Ok(if is_long {
            &self.open_interest.0
        } else {
            &self.open_interest.1
        })
    }

    fn open_interest_in_tokens_pool(&self, is_long: bool) -> gmsol_model::Result<&Self::Pool> {
        Ok(if is_long {
            &self.open_interest_in_tokens.0
        } else {
            &self.open_interest_in_tokens.1
        })
    }

    fn usd_to_amount_divisor(&self) -> Self::Num {
        self.market.usd_to_amount_divisor()
    }

    fn max_pool_amount(&self, is_long_token: bool) -> gmsol_model::Result<Self::Num> {
        self.market.max_pool_amount(is_long_token)
    }

    fn max_pnl_factor(
        &self,
        kind: gmsol_model::PnlFactorKind,
        is_long: bool,
    ) -> gmsol_model::Result<Self::Num> {
        self.market.max_pnl_factor(kind, is_long)
    }

    fn reserve_factor(&self) -> gmsol_model::Result<Self::Num> {
        self.market.reserve_factor()
    }
}

impl<'a> gmsol_model::SwapMarket<{ constants::MARKET_DECIMALS }> for RevertibleSwapMarket<'a> {
    fn swap_impact_params(&self) -> gmsol_model::Result<PriceImpactParams<Self::Num>> {
        self.market.swap_impact_params()
    }

    fn swap_fee_params(&self) -> gmsol_model::Result<FeeParams<Self::Num>> {
        self.market.swap_fee_params()
    }

    fn swap_impact_pool_mut(&mut self) -> gmsol_model::Result<&mut Self::Pool> {
        Ok(&mut self.market.swap_impact)
    }
}

impl<'a> gmsol_model::Bank<Pubkey> for RevertibleSwapMarket<'a> {
    type Num = u64;

    fn record_transferred_in_by_token<Q: std::borrow::Borrow<Pubkey> + ?Sized>(
        &mut self,
        token: &Q,
        amount: &Self::Num,
    ) -> gmsol_model::Result<()> {
        self.market
            .record_transferred_in_by_token(token.borrow(), amount)
    }

    fn record_transferred_out_by_token<Q: std::borrow::Borrow<Pubkey> + ?Sized>(
        &mut self,
        token: &Q,
        amount: &Self::Num,
    ) -> gmsol_model::Result<()> {
        self.market
            .record_transferred_out_by_token(token.borrow(), amount)
    }

    fn balance<Q: std::borrow::Borrow<Pubkey> + ?Sized>(
        &self,
        token: &Q,
    ) -> gmsol_model::Result<Self::Num> {
        self.market.balance(token)
    }
}

impl<'a> Key for RevertibleSwapMarket<'a> {
    fn key(&self) -> Pubkey {
        self.market.key()
    }
}
