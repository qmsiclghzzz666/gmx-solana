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
    states::{
        market::{
            clock::{AsClock, AsClockMut},
            Clocks,
        },
        Factor, HasMarketMeta, Market, MarketMeta, OtherState, Pool,
    },
    CoreError, ModelError,
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
            .pool(PoolKind::Primary)
            .ok_or(error!(CoreError::NotFound))?
            .create_small(PoolKind::Primary);
        let claimable_fee = storage
            .pool(PoolKind::ClaimableFee)
            .ok_or(error!(CoreError::NotFound))?
            .create_small(PoolKind::ClaimableFee);
        let swap_impact = storage
            .pool(PoolKind::SwapImpact)
            .ok_or(error!(CoreError::NotFound))?
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
            .pool_mut(PoolKind::Primary)
            .expect("must be exist");
        self.liquidity.as_small_pool().write_to_pool(liquidity);

        let claimable_fee = self
            .storage
            .pool_mut(PoolKind::ClaimableFee)
            .expect("must be exist");
        self.claimable_fee
            .as_small_pool()
            .write_to_pool(claimable_fee);

        let swap_impact = self
            .storage
            .pool_mut(PoolKind::SwapImpact)
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
        let pool = self.storage.pool(kind).ok_or(error!(CoreError::NotFound))?;
        Ok(RevertiblePool::Storage(
            pool.long_amount().map_err(ModelError::from)?,
            pool.short_amount().map_err(ModelError::from)?,
        ))
    }

    /// Create a revertible pool from the storage.
    pub fn create_revertible_pool(&self, kind: PoolKind) -> Result<RevertiblePool> {
        let pool = self.storage.pool(kind).ok_or(error!(CoreError::NotFound))?;
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
    pub fn state(&self) -> &OtherState {
        &self.storage.state.other
    }

    /// Get clock.
    pub fn get_clock(&self, kind: ClockKind) -> Result<i64> {
        let clock = self
            .storage
            .clock(kind)
            .ok_or(error!(CoreError::NotFound))?;
        Ok(clock)
    }

    /// Set clock.
    pub fn set_clock(&mut self, kind: ClockKind, last: i64) -> Result<()> {
        let clock = self
            .storage
            .state
            .clocks
            .get_mut(kind)
            .ok_or(error!(CoreError::NotFound))?;
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

    pub(super) fn passed_in_seconds_for_borrowing(&self) -> gmsol_model::Result<u64> {
        self.storage.passed_in_seconds_for_borrowing()
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

    pub(super) fn min_collateral_factor_for_open_interest_multiplier(
        &self,
        is_long: bool,
    ) -> gmsol_model::Result<Factor> {
        self.storage
            .min_collateral_factor_for_open_interest_multiplier(is_long)
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

/// Revertible Market.
pub struct RevertibleMarket2<'a> {
    pub(super) market: RefMut<'a, Market>,
}

impl<'a, 'info> TryFrom<&'a AccountLoader<'info, Market>> for RevertibleMarket2<'a> {
    type Error = Error;

    fn try_from(value: &'a AccountLoader<'info, Market>) -> std::result::Result<Self, Self::Error> {
        Ok(Self {
            market: value.load_mut()?,
        })
    }
}

impl<'a, 'info> Key for RevertibleMarket2<'a> {
    fn key(&self) -> Pubkey {
        self.market.meta.market_token_mint
    }
}

impl<'a> RevertibleMarket2<'a> {
    fn pool(&self, kind: PoolKind) -> gmsol_model::Result<&Pool> {
        let Market { state, buffer, .. } = &*self.market;
        buffer
            .pool(kind, state)
            .ok_or_else(|| gmsol_model::Error::MissingPoolKind(kind))
    }

    fn pool_mut(&mut self, kind: PoolKind) -> gmsol_model::Result<&mut Pool> {
        let Market { state, buffer, .. } = &mut *self.market;
        buffer
            .pool_mut(kind, state)
            .ok_or_else(|| gmsol_model::Error::MissingPoolKind(kind))
    }

    fn other(&self) -> &OtherState {
        let Market { state, buffer, .. } = &*self.market;
        buffer.other(state)
    }

    fn other_mut(&mut self) -> &mut OtherState {
        let Market { state, buffer, .. } = &mut *self.market;
        buffer.other_mut(state)
    }

    fn clocks(&self) -> &Clocks {
        let Market { state, buffer, .. } = &*self.market;
        buffer.clocks(state)
    }

    fn clocks_mut(&mut self) -> &mut Clocks {
        let Market { state, buffer, .. } = &mut *self.market;
        buffer.clocks_mut(state)
    }

    fn balance_for_one_side(&self, is_long: bool) -> u64 {
        let other = self.other();
        if is_long || self.market.is_pure() {
            if self.market.is_pure() {
                other.long_token_balance / 2
            } else {
                other.long_token_balance
            }
        } else {
            other.short_token_balance
        }
    }

    /// Record transferred in.
    fn record_transferred_in(&mut self, is_long_token: bool, amount: u64) -> Result<()> {
        let mint = self.market.meta.market_token_mint;
        let is_pure = self.market.is_pure();
        let other = self.other_mut();

        msg!(
            "[Balance to be committed] {}: {},{}(+{} {is_long_token})",
            mint,
            other.long_token_balance,
            other.short_token_balance,
            amount,
        );

        if is_pure || is_long_token {
            other.long_token_balance = other
                .long_token_balance
                .checked_add(amount)
                .ok_or(error!(CoreError::TokenAmountOverflow))?;
        } else {
            other.short_token_balance = other
                .short_token_balance
                .checked_add(amount)
                .ok_or(error!(CoreError::TokenAmountOverflow))?;
        }
        Ok(())
    }

    /// Record transferred out.
    fn record_transferred_out(&mut self, is_long_token: bool, amount: u64) -> Result<()> {
        let mint = self.market.meta.market_token_mint;
        let is_pure = self.market.is_pure();
        let other = self.other_mut();

        msg!(
            "[Balance to be committed] {}: {},{}(-{} {is_long_token})",
            mint,
            other.long_token_balance,
            other.short_token_balance,
            amount,
        );

        if is_pure || is_long_token {
            other.long_token_balance = other
                .long_token_balance
                .checked_sub(amount)
                .ok_or(error!(CoreError::TokenAmountOverflow))?;
        } else {
            other.short_token_balance = other
                .short_token_balance
                .checked_sub(amount)
                .ok_or(error!(CoreError::TokenAmountOverflow))?;
        }
        Ok(())
    }
}

impl<'a> Revertible for RevertibleMarket2<'a> {
    fn commit(mut self) {
        let Market {
            meta,
            state,
            buffer,
            ..
        } = &mut *self.market;
        buffer.commit_to_storage(state);
        msg!(
            "[Balance committed] {}: {},{}",
            meta.market_token_mint,
            state.other.long_token_balance,
            state.other.short_token_balance
        );
    }
}

impl<'a> HasMarketMeta for RevertibleMarket2<'a> {
    fn is_pure(&self) -> bool {
        self.market.is_pure()
    }
    fn market_meta(&self) -> &MarketMeta {
        self.market.market_meta()
    }
}

impl<'a> gmsol_model::Bank<Pubkey> for RevertibleMarket2<'a> {
    type Num = u64;

    fn record_transferred_in_by_token<Q: ?Sized + Borrow<Pubkey>>(
        &mut self,
        token: &Q,
        amount: &Self::Num,
    ) -> gmsol_model::Result<()> {
        let is_long_token = self.market.meta.to_token_side(token.borrow())?;
        self.record_transferred_in(is_long_token, *amount)?;
        Ok(())
    }

    fn record_transferred_out_by_token<Q: ?Sized + Borrow<Pubkey>>(
        &mut self,
        token: &Q,
        amount: &Self::Num,
    ) -> gmsol_model::Result<()> {
        let is_long_token = self.market.meta.to_token_side(token.borrow())?;
        self.record_transferred_out(is_long_token, *amount)?;
        Ok(())
    }

    fn balance<Q: Borrow<Pubkey> + ?Sized>(&self, token: &Q) -> gmsol_model::Result<Self::Num> {
        let side = self.market.meta.to_token_side(token.borrow())?;
        Ok(self.balance_for_one_side(side))
    }
}

impl<'a> gmsol_model::BaseMarket<{ constants::MARKET_DECIMALS }> for RevertibleMarket2<'a> {
    type Num = u128;

    type Signed = i128;

    type Pool = Pool;

    fn liquidity_pool(&self) -> gmsol_model::Result<&Self::Pool> {
        self.pool(PoolKind::Primary)
    }

    fn claimable_fee_pool(&self) -> gmsol_model::Result<&Self::Pool> {
        self.pool(PoolKind::ClaimableFee)
    }

    fn swap_impact_pool(&self) -> gmsol_model::Result<&Self::Pool> {
        self.pool(PoolKind::SwapImpact)
    }

    fn open_interest_pool(&self, is_long: bool) -> gmsol_model::Result<&Self::Pool> {
        self.pool(if is_long {
            PoolKind::OpenInterestForLong
        } else {
            PoolKind::OpenInterestForShort
        })
    }

    fn open_interest_in_tokens_pool(&self, is_long: bool) -> gmsol_model::Result<&Self::Pool> {
        self.pool(if is_long {
            PoolKind::OpenInterestInTokensForLong
        } else {
            PoolKind::OpenInterestInTokensForShort
        })
    }

    fn collateral_sum_pool(&self, is_long: bool) -> gmsol_model::Result<&Self::Pool> {
        self.pool(if is_long {
            PoolKind::CollateralSumForLong
        } else {
            PoolKind::CollateralSumForShort
        })
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

impl<'a> gmsol_model::BaseMarketMut<{ constants::MARKET_DECIMALS }> for RevertibleMarket2<'a> {
    fn liquidity_pool_mut(&mut self) -> gmsol_model::Result<&mut Self::Pool> {
        self.pool_mut(PoolKind::Primary)
    }

    fn claimable_fee_pool_mut(&mut self) -> gmsol_model::Result<&mut Self::Pool> {
        self.pool_mut(PoolKind::ClaimableFee)
    }
}

impl<'a> gmsol_model::SwapMarket<{ constants::MARKET_DECIMALS }> for RevertibleMarket2<'a> {
    fn swap_impact_params(&self) -> gmsol_model::Result<PriceImpactParams<Factor>> {
        self.market.swap_impact_params()
    }

    fn swap_fee_params(&self) -> gmsol_model::Result<FeeParams<Factor>> {
        self.market.swap_fee_params()
    }
}

impl<'a> gmsol_model::SwapMarketMut<{ constants::MARKET_DECIMALS }> for RevertibleMarket2<'a> {
    fn swap_impact_pool_mut(&mut self) -> gmsol_model::Result<&mut Self::Pool> {
        self.pool_mut(PoolKind::SwapImpact)
    }
}

impl<'a> gmsol_model::PositionImpactMarket<{ constants::MARKET_DECIMALS }>
    for RevertibleMarket2<'a>
{
    fn position_impact_pool(&self) -> gmsol_model::Result<&Self::Pool> {
        self.pool(PoolKind::PositionImpact)
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
        AsClock::from(&self.clocks().price_impact_distribution).passed_in_seconds()
    }
}

impl<'a> gmsol_model::PositionImpactMarketMut<{ constants::MARKET_DECIMALS }>
    for RevertibleMarket2<'a>
{
    fn position_impact_pool_mut(&mut self) -> gmsol_model::Result<&mut Self::Pool> {
        self.pool_mut(PoolKind::PositionImpact)
    }

    fn just_passed_in_seconds_for_position_impact_distribution(
        &mut self,
    ) -> gmsol_model::Result<u64> {
        AsClockMut::from(&mut self.clocks_mut().price_impact_distribution).just_passed_in_seconds()
    }
}

impl<'a> gmsol_model::BorrowingFeeMarket<{ constants::MARKET_DECIMALS }> for RevertibleMarket2<'a> {
    fn borrowing_factor_pool(&self) -> gmsol_model::Result<&Self::Pool> {
        self.pool(PoolKind::BorrowingFactor)
    }

    fn total_borrowing_pool(&self) -> gmsol_model::Result<&Self::Pool> {
        self.pool(PoolKind::TotalBorrowing)
    }

    fn borrowing_fee_params(&self) -> gmsol_model::Result<BorrowingFeeParams<Self::Num>> {
        self.market.borrowing_fee_params()
    }

    fn passed_in_seconds_for_borrowing(&self) -> gmsol_model::Result<u64> {
        AsClock::from(&self.clocks().borrowing).passed_in_seconds()
    }
}

impl<'a> gmsol_model::PerpMarket<{ constants::MARKET_DECIMALS }> for RevertibleMarket2<'a> {
    fn funding_factor_per_second(&self) -> &Self::Signed {
        &self.other().funding_factor_per_second
    }

    fn funding_amount_per_size_pool(&self, is_long: bool) -> gmsol_model::Result<&Self::Pool> {
        self.pool(if is_long {
            PoolKind::FundingAmountPerSizeForLong
        } else {
            PoolKind::FundingAmountPerSizeForShort
        })
    }

    fn claimable_funding_amount_per_size_pool(
        &self,
        is_long: bool,
    ) -> gmsol_model::Result<&Self::Pool> {
        self.pool(if is_long {
            PoolKind::ClaimableFundingAmountPerSizeForLong
        } else {
            PoolKind::ClaimableFundingAmountPerSizeForShort
        })
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
        self.market.order_fee_params()
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

impl<'a> gmsol_model::PerpMarketMut<{ constants::MARKET_DECIMALS }> for RevertibleMarket2<'a> {
    fn just_passed_in_seconds_for_borrowing(&mut self) -> gmsol_model::Result<u64> {
        AsClockMut::from(&mut self.clocks_mut().borrowing).just_passed_in_seconds()
    }

    fn just_passed_in_seconds_for_funding(&mut self) -> gmsol_model::Result<u64> {
        AsClockMut::from(&mut self.clocks_mut().funding).just_passed_in_seconds()
    }

    fn funding_factor_per_second_mut(&mut self) -> &mut Self::Signed {
        &mut self.other_mut().funding_factor_per_second
    }

    fn open_interest_pool_mut(&mut self, is_long: bool) -> gmsol_model::Result<&mut Self::Pool> {
        self.pool_mut(if is_long {
            PoolKind::OpenInterestForLong
        } else {
            PoolKind::OpenInterestForShort
        })
    }

    fn open_interest_in_tokens_pool_mut(
        &mut self,
        is_long: bool,
    ) -> gmsol_model::Result<&mut Self::Pool> {
        self.pool_mut(if is_long {
            PoolKind::OpenInterestInTokensForLong
        } else {
            PoolKind::OpenInterestInTokensForShort
        })
    }

    fn borrowing_factor_pool_mut(&mut self) -> gmsol_model::Result<&mut Self::Pool> {
        self.pool_mut(PoolKind::BorrowingFactor)
    }

    fn funding_amount_per_size_pool_mut(
        &mut self,
        is_long: bool,
    ) -> gmsol_model::Result<&mut Self::Pool> {
        self.pool_mut(if is_long {
            PoolKind::FundingAmountPerSizeForLong
        } else {
            PoolKind::FundingAmountPerSizeForShort
        })
    }

    fn claimable_funding_amount_per_size_pool_mut(
        &mut self,
        is_long: bool,
    ) -> gmsol_model::Result<&mut Self::Pool> {
        self.pool_mut(if is_long {
            PoolKind::ClaimableFundingAmountPerSizeForLong
        } else {
            PoolKind::ClaimableFundingAmountPerSizeForShort
        })
    }

    fn collateral_sum_pool_mut(&mut self, is_long: bool) -> gmsol_model::Result<&mut Self::Pool> {
        self.pool_mut(if is_long {
            PoolKind::CollateralSumForLong
        } else {
            PoolKind::CollateralSumForShort
        })
    }

    fn total_borrowing_pool_mut(&mut self) -> gmsol_model::Result<&mut Self::Pool> {
        self.pool_mut(PoolKind::TotalBorrowing)
    }
}
