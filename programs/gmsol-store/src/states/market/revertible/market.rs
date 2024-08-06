use std::{borrow::Borrow, cell::RefMut, ops::Deref};

use anchor_lang::prelude::*;
use gmsol_model::{
    params::{
        fee::{BorrowingFeeParams, FundingFeeParams},
        position::PositionImpactDistributionParams,
        FeeParams, PositionParams, PriceImpactParams,
    },
    Balance, BorrowingFeeMarket, ClockKind, PerpMarket, PoolKind, PositionImpactMarket,
};

use crate::{
    constants,
    states::{Factor, HasMarketMeta, Market, MarketMeta, MarketState, Pool},
    ModelError, StoreError,
};

use super::{swap_market::RevertibleSwapMarket, Revertible, RevertibleBalance};

/// Small Pool.
pub struct SmallPool {
    kind: PoolKind,
    is_pure: bool,
    long_amount: u128,
    short_amount: u128,
}

impl SmallPool {
    /// Write the data to target pool.
    ///
    /// ## Panic
    /// Panic if the pure flag is not matched.
    pub(crate) fn write_to_pool(&self, pool: &mut Pool) {
        assert_eq!(self.is_pure, pool.is_pure());
        pool.long_token_amount = self.long_amount;
        pool.short_token_amount = self.short_amount;
    }

    /// Create a new small pool from [`Pool`]
    pub fn new(kind: PoolKind, pool: &Pool) -> Self {
        Self {
            kind,
            is_pure: pool.is_pure(),
            long_amount: pool.long_token_amount,
            short_amount: pool.short_token_amount,
        }
    }
}

impl gmsol_model::Balance for SmallPool {
    type Num = u128;

    type Signed = i128;

    /// Get the long token amount.
    fn long_amount(&self) -> gmsol_model::Result<Self::Num> {
        if self.is_pure {
            debug_assert_eq!(self.short_amount, 0, "short token amount must be zero");
            Ok(self.long_amount / 2)
        } else {
            Ok(self.long_amount)
        }
    }

    /// Get the short token amount.
    fn short_amount(&self) -> gmsol_model::Result<Self::Num> {
        if self.is_pure {
            debug_assert_eq!(self.short_amount, 0, "short token amount must be zero");
            Ok(self.long_amount / 2)
        } else {
            Ok(self.short_amount)
        }
    }
}

impl gmsol_model::Pool for SmallPool {
    fn apply_delta_to_long_amount(&mut self, delta: &Self::Signed) -> gmsol_model::Result<()> {
        self.long_amount = self.long_amount.checked_add_signed(*delta).ok_or(
            gmsol_model::Error::PoolComputation(self.kind, "apply delta to long amount"),
        )?;
        Ok(())
    }

    fn apply_delta_to_short_amount(&mut self, delta: &Self::Signed) -> gmsol_model::Result<()> {
        let amount = if self.is_pure {
            &mut self.long_amount
        } else {
            &mut self.short_amount
        };
        *amount = amount
            .checked_add_signed(*delta)
            .ok_or(gmsol_model::Error::PoolComputation(
                self.kind,
                "apply delta to short amount",
            ))?;
        Ok(())
    }
}

/// Pool for revertible markets.
pub enum RevertiblePool {
    /// Small Pool
    SmallPool(SmallPool),
    /// From storage.
    Storage(u128, u128),
}

impl gmsol_model::Balance for RevertiblePool {
    type Num = u128;

    type Signed = i128;

    fn long_amount(&self) -> gmsol_model::Result<Self::Num> {
        match self {
            Self::SmallPool(pool) => pool.long_amount(),
            Self::Storage(long_amount, _) => Ok(*long_amount),
        }
    }

    fn short_amount(&self) -> gmsol_model::Result<Self::Num> {
        match self {
            Self::SmallPool(pool) => pool.short_amount(),
            Self::Storage(_, short_amount) => Ok(*short_amount),
        }
    }
}

impl gmsol_model::Pool for RevertiblePool {
    fn apply_delta_to_long_amount(&mut self, delta: &Self::Signed) -> gmsol_model::Result<()> {
        let Self::SmallPool(pool) = self else {
            return Err(gmsol_model::Error::invalid_argument(
                "Cannot modify pool from the storage",
            ));
        };
        pool.apply_delta_to_long_amount(delta)
    }

    fn apply_delta_to_short_amount(&mut self, delta: &Self::Signed) -> gmsol_model::Result<()> {
        let Self::SmallPool(pool) = self else {
            return Err(gmsol_model::Error::invalid_argument(
                "Cannot modify pool from the storage",
            ));
        };
        pool.apply_delta_to_short_amount(delta)
    }
}

impl RevertiblePool {
    /// As small pool.
    /// ## Panic
    /// Panic if this is a small pool.
    pub(crate) fn as_small_pool(&self) -> &SmallPool {
        let Self::SmallPool(pool) = self else {
            panic!("not a small pool");
        };
        pool
    }
}

/// Revertible Market.
pub struct RevertibleMarket<'a> {
    storage: RefMut<'a, Market>,
    balance: RevertibleBalance,
    pub(crate) liquidity: RevertiblePool,
    pub(crate) claimable_fee: RevertiblePool,
    pub(crate) swap_impact: RevertiblePool,
}

impl<'a> Key for RevertibleMarket<'a> {
    fn key(&self) -> Pubkey {
        self.storage.meta().market_token_mint
    }
}

impl<'a, 'info> TryFrom<&'a AccountLoader<'info, Market>> for RevertibleMarket<'a> {
    type Error = Error;

    fn try_from(
        loader: &'a AccountLoader<'info, Market>,
    ) -> std::result::Result<Self, Self::Error> {
        let storage = loader.load_mut()?;
        let liquidity = storage
            .pools
            .get(PoolKind::Primary)
            .ok_or(error!(StoreError::RequiredResourceNotFound))?
            .create_small(PoolKind::Primary);
        let claimable_fee = storage
            .pools
            .get(PoolKind::ClaimableFee)
            .ok_or(error!(StoreError::RequiredResourceNotFound))?
            .create_small(PoolKind::ClaimableFee);
        let swap_impact = storage
            .pools
            .get(PoolKind::SwapImpact)
            .ok_or(error!(StoreError::RequiredResourceNotFound))?
            .create_small(PoolKind::SwapImpact);
        let balance = RevertibleBalance::from(storage.deref());
        Ok(Self {
            storage,
            balance,
            liquidity: RevertiblePool::SmallPool(liquidity),
            claimable_fee: RevertiblePool::SmallPool(claimable_fee),
            swap_impact: RevertiblePool::SmallPool(swap_impact),
        })
    }
}

impl<'a> RevertibleMarket<'a> {
    /// Commit the changes.
    /// ## Panic
    /// - Panic if the storage doesn't have the requried pools.
    /// - Panic if `f` decides to do so.
    pub fn commit_with(mut self, f: impl FnOnce(&mut Market)) {
        let liquidity = self
            .storage
            .pools
            .get_mut(PoolKind::Primary)
            .expect("must be exist");
        self.liquidity.as_small_pool().write_to_pool(liquidity);

        let claimable_fee = self
            .storage
            .pools
            .get_mut(PoolKind::ClaimableFee)
            .expect("must be exist");
        self.claimable_fee
            .as_small_pool()
            .write_to_pool(claimable_fee);

        let swap_impact = self
            .storage
            .pools
            .get_mut(PoolKind::SwapImpact)
            .expect("must be exist");
        self.swap_impact.as_small_pool().write_to_pool(swap_impact);

        self.balance.write_to_market(&mut self.storage);

        (f)(&mut self.storage)
    }

    pub(super) fn max_pool_value_for_deposit(
        &self,
        is_long_token: bool,
    ) -> gmsol_model::Result<Factor> {
        self.storage.max_pool_value_for_deposit(is_long_token)
    }

    /// Get pool from storage.
    pub fn get_pool_from_storage(&self, kind: PoolKind) -> Result<RevertiblePool> {
        let pool = self
            .storage
            .pools
            .get(kind)
            .ok_or(error!(StoreError::RequiredResourceNotFound))?;
        Ok(RevertiblePool::Storage(
            pool.long_amount().map_err(ModelError::from)?,
            pool.short_amount().map_err(ModelError::from)?,
        ))
    }

    /// Create a revertible pool from the storage.
    pub fn create_revertible_pool(&self, kind: PoolKind) -> Result<RevertiblePool> {
        let pool = self
            .storage
            .pools
            .get(kind)
            .ok_or(error!(StoreError::RequiredResourceNotFound))?;
        Ok(RevertiblePool::SmallPool(pool.create_small(kind)))
    }

    /// As a [`SwapMarket`](gmsol_model::SwapMarket).
    pub fn into_swap_market(self) -> Result<RevertibleSwapMarket<'a>> {
        RevertibleSwapMarket::from_market(self)
    }

    /// Get the balance field.
    pub fn revertible_balance(&self) -> &RevertibleBalance {
        &self.balance
    }

    /// Get market state.
    pub fn state(&self) -> &MarketState {
        &self.storage.state
    }

    /// Get clock.
    pub fn get_clock(&self, kind: ClockKind) -> Result<i64> {
        let clock = self
            .storage
            .clocks
            .get(kind)
            .ok_or(error!(StoreError::RequiredResourceNotFound))?;
        Ok(*clock)
    }

    /// Set clock.
    pub fn set_clock(&mut self, kind: ClockKind, last: i64) -> Result<()> {
        let clock = self
            .storage
            .clocks
            .get_mut(kind)
            .ok_or(error!(StoreError::RequiredResourceNotFound))?;
        *clock = last;
        Ok(())
    }

    pub(super) fn position_impact_params(&self) -> gmsol_model::Result<PriceImpactParams<Factor>> {
        self.storage.position_impact_params()
    }

    pub(super) fn passed_in_seconds_for_position_impact_distribution(
        &self,
    ) -> gmsol_model::Result<u64> {
        self.storage
            .passed_in_seconds_for_position_impact_distribution()
    }

    pub(super) fn position_impact_distribution_params(
        &self,
    ) -> gmsol_model::Result<PositionImpactDistributionParams<Factor>> {
        self.storage.position_impact_distribution_params()
    }

    pub(super) fn borrowing_fee_params(&self) -> gmsol_model::Result<BorrowingFeeParams<Factor>> {
        self.storage.borrowing_fee_params()
    }

    pub(super) fn funding_fee_params(&self) -> gmsol_model::Result<FundingFeeParams<Factor>> {
        self.storage.funding_fee_params()
    }

    pub(super) fn position_params(&self) -> gmsol_model::Result<PositionParams<Factor>> {
        self.storage.position_params()
    }

    pub(super) fn order_fee_params(&self) -> gmsol_model::Result<FeeParams<Factor>> {
        self.storage.order_fee_params()
    }

    pub(super) fn funding_amount_per_size_adjustment(&self) -> Factor {
        self.storage.funding_amount_per_size_adjustment()
    }

    pub(super) fn open_interest_reserve_factor(&self) -> gmsol_model::Result<Factor> {
        self.storage.open_interest_reserve_factor()
    }

    pub(super) fn max_open_interest(&self, is_long: bool) -> gmsol_model::Result<Factor> {
        self.storage.max_open_interest(is_long)
    }
}

impl<'a> HasMarketMeta for RevertibleMarket<'a> {
    fn market_meta(&self) -> &MarketMeta {
        self.storage.market_meta()
    }

    fn is_pure(&self) -> bool {
        self.storage.is_pure()
    }
}

impl<'a> gmsol_model::Bank<Pubkey> for RevertibleMarket<'a> {
    type Num = u64;

    fn record_transferred_in_by_token<Q: ?Sized + Borrow<Pubkey>>(
        &mut self,
        token: &Q,
        amount: &Self::Num,
    ) -> gmsol_model::Result<()> {
        let is_long_token = self.storage.meta.to_token_side(token.borrow())?;
        // TODO: use event
        msg!(
            "[Balance Not committed] {}: {},{}(+{} {is_long_token})",
            self.storage.meta.market_token_mint,
            self.balance.long_token_balance,
            self.balance.short_token_balance,
            amount,
        );
        self.balance.record_transferred_in(is_long_token, *amount)?;
        Ok(())
    }

    fn record_transferred_out_by_token<Q: ?Sized + Borrow<Pubkey>>(
        &mut self,
        token: &Q,
        amount: &Self::Num,
    ) -> gmsol_model::Result<()> {
        let is_long_token = self.storage.meta.to_token_side(token.borrow())?;
        // TODO: use event
        msg!(
            "[Balance Not committed] {}: {},{}(-{} {is_long_token})",
            self.storage.meta.market_token_mint,
            self.balance.long_token_balance,
            self.balance.short_token_balance,
            amount,
        );
        self.balance
            .record_transferred_out(is_long_token, *amount)?;
        Ok(())
    }

    fn balance<Q: Borrow<Pubkey> + ?Sized>(&self, token: &Q) -> gmsol_model::Result<Self::Num> {
        let side = self.market_meta().to_token_side(token.borrow())?;
        Ok(self.revertible_balance().balance_for_one_side(side))
    }
}

impl<'a> gmsol_model::BaseMarket<{ constants::MARKET_DECIMALS }> for RevertibleMarket<'a> {
    type Num = u128;

    type Signed = i128;

    type Pool = RevertiblePool;

    fn liquidity_pool(&self) -> gmsol_model::Result<&Self::Pool> {
        Ok(&self.liquidity)
    }

    fn claimable_fee_pool(&self) -> gmsol_model::Result<&Self::Pool> {
        Ok(&self.claimable_fee)
    }

    fn swap_impact_pool(&self) -> gmsol_model::Result<&Self::Pool> {
        Ok(&self.swap_impact)
    }

    fn open_interest_pool(&self, _is_long: bool) -> gmsol_model::Result<&Self::Pool> {
        Err(gmsol_model::Error::Unimplemented)
    }

    fn open_interest_in_tokens_pool(&self, _is_long: bool) -> gmsol_model::Result<&Self::Pool> {
        Err(gmsol_model::Error::Unimplemented)
    }

    fn collateral_sum_pool(&self, _is_long: bool) -> gmsol_model::Result<&Self::Pool> {
        Err(gmsol_model::Error::Unimplemented)
    }

    fn usd_to_amount_divisor(&self) -> Self::Num {
        self.storage.usd_to_amount_divisor()
    }

    fn max_pool_amount(&self, is_long_token: bool) -> gmsol_model::Result<Self::Num> {
        self.storage.max_pool_amount(is_long_token)
    }

    fn pnl_factor_config(
        &self,
        kind: gmsol_model::PnlFactorKind,
        is_long: bool,
    ) -> gmsol_model::Result<Self::Num> {
        self.storage.pnl_factor_config(kind, is_long)
    }

    fn reserve_factor(&self) -> gmsol_model::Result<Self::Num> {
        self.storage.reserve_factor()
    }
}

impl<'a> gmsol_model::BaseMarketMut<{ constants::MARKET_DECIMALS }> for RevertibleMarket<'a> {
    fn liquidity_pool_mut(&mut self) -> gmsol_model::Result<&mut Self::Pool> {
        Ok(&mut self.liquidity)
    }

    fn claimable_fee_pool_mut(&mut self) -> gmsol_model::Result<&mut Self::Pool> {
        Ok(&mut self.claimable_fee)
    }
}

impl<'a> gmsol_model::SwapMarket<{ constants::MARKET_DECIMALS }> for RevertibleMarket<'a> {
    fn swap_impact_params(&self) -> gmsol_model::Result<PriceImpactParams<Factor>> {
        self.storage.swap_impact_params()
    }

    fn swap_fee_params(&self) -> gmsol_model::Result<FeeParams<Factor>> {
        self.storage.swap_fee_params()
    }
}

impl<'a> Revertible for RevertibleMarket<'a> {
    /// Commit the changes.
    /// ## Panic
    /// - Panic if the storage doesn't have the requried pools.
    fn commit(self) {
        self.commit_with(|_| ());
    }
}
