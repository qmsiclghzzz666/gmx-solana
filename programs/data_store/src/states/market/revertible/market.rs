use std::{cell::RefMut, ops::Deref};

use anchor_lang::prelude::*;
use gmx_core::{Balance, PoolKind};

use crate::{
    constants,
    states::{Market, MarketConfig, Pool},
    DataStoreError, GmxCoreError,
};

use super::{swap_market::AsSwapMarket, RevertibleBalance};

/// Small Pool.
pub struct SmallPool {
    is_pure: bool,
    long_amount: u128,
    short_amount: u128,
}

impl<'a> From<&'a Pool> for SmallPool {
    fn from(pool: &'a Pool) -> Self {
        Self {
            is_pure: pool.is_pure(),
            long_amount: pool.long_token_amount,
            short_amount: pool.short_token_amount,
        }
    }
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
}

impl gmx_core::Balance for SmallPool {
    type Num = u128;

    type Signed = i128;

    /// Get the long token amount.
    fn long_amount(&self) -> gmx_core::Result<Self::Num> {
        if self.is_pure {
            debug_assert_eq!(self.short_amount, 0, "short token amount must be zero");
            Ok(self.long_amount / 2)
        } else {
            Ok(self.long_amount)
        }
    }

    /// Get the short token amount.
    fn short_amount(&self) -> gmx_core::Result<Self::Num> {
        if self.is_pure {
            debug_assert_eq!(self.short_amount, 0, "short token amount must be zero");
            Ok(self.long_amount / 2)
        } else {
            Ok(self.short_amount)
        }
    }
}

impl gmx_core::Pool for SmallPool {
    fn apply_delta_to_long_amount(&mut self, delta: &Self::Signed) -> gmx_core::Result<()> {
        self.long_amount = self
            .long_amount
            .checked_add_signed(*delta)
            .ok_or(gmx_core::Error::Computation("apply delta to long amount"))?;
        Ok(())
    }

    fn apply_delta_to_short_amount(&mut self, delta: &Self::Signed) -> gmx_core::Result<()> {
        let amount = if self.is_pure {
            &mut self.long_amount
        } else {
            &mut self.short_amount
        };
        *amount = amount
            .checked_add_signed(*delta)
            .ok_or(gmx_core::Error::Computation("apply delta to short amount"))?;
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

impl gmx_core::Balance for RevertiblePool {
    type Num = u128;

    type Signed = i128;

    fn long_amount(&self) -> gmx_core::Result<Self::Num> {
        match self {
            Self::SmallPool(pool) => pool.long_amount(),
            Self::Storage(long_amount, _) => Ok(*long_amount),
        }
    }

    fn short_amount(&self) -> gmx_core::Result<Self::Num> {
        match self {
            Self::SmallPool(pool) => pool.short_amount(),
            Self::Storage(_, short_amount) => Ok(*short_amount),
        }
    }
}

impl gmx_core::Pool for RevertiblePool {
    fn apply_delta_to_long_amount(&mut self, delta: &Self::Signed) -> gmx_core::Result<()> {
        let Self::SmallPool(pool) = self else {
            return Err(gmx_core::Error::invalid_argument(
                "Cannot modify pool from the storage",
            ));
        };
        pool.apply_delta_to_long_amount(delta)
    }

    fn apply_delta_to_short_amount(&mut self, delta: &Self::Signed) -> gmx_core::Result<()> {
        let Self::SmallPool(pool) = self else {
            return Err(gmx_core::Error::invalid_argument(
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

impl<'a, 'info> TryFrom<&'a AccountLoader<'info, Market>> for RevertibleMarket<'a> {
    type Error = Error;

    fn try_from(
        loader: &'a AccountLoader<'info, Market>,
    ) -> std::result::Result<Self, Self::Error> {
        let storage = loader.load_mut()?;
        let liquidity = storage
            .pools
            .get(PoolKind::Primary)
            .ok_or(error!(DataStoreError::RequiredResourceNotFound))?
            .into();
        let claimable_fee = storage
            .pools
            .get(PoolKind::ClaimableFee)
            .ok_or(error!(DataStoreError::RequiredResourceNotFound))?
            .into();
        let swap_impact = storage
            .pools
            .get(PoolKind::SwapImpact)
            .ok_or(error!(DataStoreError::RequiredResourceNotFound))?
            .into();
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
    pub fn commit(self) {
        self.commit_with(|_| ());
    }

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

    /// Get market config.
    pub fn config(&self) -> &MarketConfig {
        &self.storage.config
    }

    /// Get balance.
    pub fn balance(&self) -> &RevertibleBalance {
        &self.balance
    }

    /// Record transferred in by the given token.
    pub fn record_transferred_in_by_token(&mut self, token: &Pubkey, amount: u64) -> Result<()> {
        if self.storage.meta.long_token_mint == *token {
            self.balance.record_transferred_in(true, amount)
        } else if self.storage.meta.short_token_mint == *token {
            self.balance.record_transferred_in(false, amount)
        } else {
            Err(error!(DataStoreError::InvalidCollateralToken))
        }
    }

    /// Record transferred out by the given token.
    pub fn record_transferred_out_by_token(&mut self, token: &Pubkey, amount: u64) -> Result<()> {
        if self.storage.meta.long_token_mint == *token {
            self.balance.record_transferred_out(true, amount)
        } else if self.storage.meta.short_token_mint == *token {
            self.balance.record_transferred_out(false, amount)
        } else {
            Err(error!(DataStoreError::InvalidCollateralToken))
        }
    }

    /// Get pool from storage.
    pub fn get_pool_from_storage(&self, kind: PoolKind) -> Result<RevertiblePool> {
        let pool = self
            .storage
            .pools
            .get(kind)
            .ok_or(error!(DataStoreError::RequiredResourceNotFound))?;
        Ok(RevertiblePool::Storage(
            pool.long_amount().map_err(GmxCoreError::from)?,
            pool.short_amount().map_err(GmxCoreError::from)?,
        ))
    }

    /// As a [`SwapMarket`](gmx_core::SwapMarket).
    pub fn as_swap_market(&mut self) -> Result<AsSwapMarket<'_, 'a>> {
        AsSwapMarket::new(self)
    }
}

impl<'a> gmx_core::BaseMarket<{ constants::MARKET_DECIMALS }> for RevertibleMarket<'a> {
    type Num = u128;

    type Signed = i128;

    type Pool = RevertiblePool;

    fn liquidity_pool(&self) -> gmx_core::Result<&Self::Pool> {
        Ok(&self.liquidity)
    }

    fn liquidity_pool_mut(&mut self) -> gmx_core::Result<&mut Self::Pool> {
        Ok(&mut self.liquidity)
    }

    fn claimable_fee_pool(&self) -> gmx_core::Result<&Self::Pool> {
        Ok(&self.claimable_fee)
    }

    fn claimable_fee_pool_mut(&mut self) -> gmx_core::Result<&mut Self::Pool> {
        Ok(&mut self.claimable_fee)
    }

    fn swap_impact_pool(&self) -> gmx_core::Result<&Self::Pool> {
        Ok(&self.swap_impact)
    }

    fn open_interest_pool(&self, _is_long: bool) -> gmx_core::Result<&Self::Pool> {
        Err(gmx_core::Error::Unimplemented)
    }

    fn open_interest_in_tokens_pool(&self, _is_long: bool) -> gmx_core::Result<&Self::Pool> {
        Err(gmx_core::Error::Unimplemented)
    }

    fn usd_to_amount_divisor(&self) -> Self::Num {
        constants::MARKET_USD_TO_AMOUNT_DIVISOR
    }

    fn max_pool_amount(&self, is_long_token: bool) -> gmx_core::Result<Self::Num> {
        if is_long_token {
            Ok(self.config().max_pool_amount_for_long_token)
        } else {
            Ok(self.config().max_pool_amount_for_short_token)
        }
    }

    fn max_pnl_factor(
        &self,
        kind: gmx_core::PnlFactorKind,
        is_long: bool,
    ) -> gmx_core::Result<Self::Num> {
        use gmx_core::PnlFactorKind;

        match (kind, is_long) {
            (PnlFactorKind::Deposit, true) => Ok(self.config().max_pnl_factor_for_long_deposit),
            (PnlFactorKind::Deposit, false) => Ok(self.config().max_pnl_factor_for_short_deposit),
            (PnlFactorKind::Withdrawal, true) => {
                Ok(self.config().max_pnl_factor_for_long_withdrawal)
            }
            (PnlFactorKind::Withdrawal, false) => {
                Ok(self.config().max_pnl_factor_for_short_withdrawal)
            }
            _ => Err(error!(DataStoreError::RequiredResourceNotFound).into()),
        }
    }

    fn reserve_factor(&self) -> gmx_core::Result<Self::Num> {
        Ok(self.config().reserve_factor)
    }
}
