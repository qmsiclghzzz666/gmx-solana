use anchor_lang::prelude::*;

use gmsol_model::{BaseMarketExt, ClockKind, PnlFactorKind};

use crate::{constants, states::Oracle, ModelError, StoreError, StoreResult};

use super::{HasMarketMeta, Market};

/// Extension trait for validating market balances.
pub trait ValidateMarketBalances:
    gmsol_model::BaseMarket<{ constants::MARKET_DECIMALS }, Num = u128>
    + gmsol_model::Bank<Pubkey, Num = u64>
    + HasMarketMeta
{
    /// Validate market balances.
    fn validate_market_balances(
        &self,
        mut long_excluding_amount: u64,
        mut short_excluding_amount: u64,
    ) -> Result<()> {
        if self.is_pure() {
            let total = long_excluding_amount
                .checked_add(short_excluding_amount)
                .ok_or(error!(StoreError::AmountOverflow))?;
            (long_excluding_amount, short_excluding_amount) = (total / 2, total / 2);
        }
        let meta = self.market_meta();
        let long_token_balance = self
            .balance_excluding(&meta.long_token_mint, &long_excluding_amount)
            .map_err(ModelError::from)?
            .into();
        self.validate_token_balance_for_one_side(&long_token_balance, true)
            .map_err(ModelError::from)?;
        let short_token_balance = self
            .balance_excluding(&meta.short_token_mint, &short_excluding_amount)
            .map_err(ModelError::from)?
            .into();
        self.validate_token_balance_for_one_side(&short_token_balance, false)
            .map_err(ModelError::from)?;
        Ok(())
    }

    /// Validate market balances excluding the given token amounts.
    fn validate_market_balances_excluding_the_given_token_amounts(
        &self,
        first_token: &Pubkey,
        second_token: &Pubkey,
        first_excluding_amount: u64,
        second_excluding_amount: u64,
    ) -> Result<()> {
        let mut long_excluding_amount = 0u64;
        let mut short_excluding_amount = 0u64;

        for (token, amount) in [
            (first_token, first_excluding_amount),
            (second_token, second_excluding_amount),
        ] {
            if amount == 0 {
                continue;
            }
            let is_long = self.market_meta().to_token_side(token)?;
            if is_long {
                long_excluding_amount = long_excluding_amount
                    .checked_add(amount)
                    .ok_or(error!(StoreError::AmountOverflow))?;
            } else {
                short_excluding_amount = short_excluding_amount
                    .checked_add(amount)
                    .ok_or(error!(StoreError::AmountOverflow))?;
            }
        }
        self.validate_market_balances(long_excluding_amount, short_excluding_amount)
    }

    /// Validate market balance for the given token.
    fn validate_market_balance_for_the_given_token(
        &self,
        token: &Pubkey,
        excluded: u64,
    ) -> Result<()> {
        let side = self.market_meta().to_token_side(token)?;
        let balance = self
            .balance_excluding(token, &excluded)
            .map_err(ModelError::from)?
            .into();
        self.validate_token_balance_for_one_side(&balance, side)
            .map_err(ModelError::from)?;
        Ok(())
    }
}

impl<
        M: gmsol_model::BaseMarket<{ constants::MARKET_DECIMALS }, Num = u128>
            + gmsol_model::Bank<Pubkey, Num = u64>
            + HasMarketMeta,
    > ValidateMarketBalances for M
{
}

/// Trait for auto-deleveraging utils.
pub trait AdlOps {
    /// Validate if the ADL can be executed.
    fn validate_adl(&self, oracle: &Oracle, is_long: bool) -> StoreResult<()>;

    /// Latest ADL time.
    fn latest_adl_time(&self, is_long: bool) -> StoreResult<i64>;

    fn update_adl_state(&mut self, oracle: &Oracle, is_long: bool) -> Result<()>;
}

impl AdlOps for Market {
    fn latest_adl_time(&self, is_long: bool) -> StoreResult<i64> {
        let clock = if is_long {
            ClockKind::AdlForLong
        } else {
            ClockKind::AdlForShort
        };
        self.clock(clock)
            .ok_or(StoreError::RequiredResourceNotFound)
    }

    fn validate_adl(&self, oracle: &Oracle, is_long: bool) -> StoreResult<()> {
        if !self.is_adl_enabled(is_long) {
            return Err(StoreError::AdlNotEnabled);
        }
        if oracle.max_oracle_ts < self.latest_adl_time(is_long)? {
            return Err(StoreError::OracleTimestampsAreSmallerThanRequired);
        }
        Ok(())
    }

    fn update_adl_state(&mut self, oracle: &Oracle, is_long: bool) -> Result<()> {
        if oracle.max_oracle_ts < self.latest_adl_time(is_long)? {
            return err!(StoreError::OracleTimestampsAreSmallerThanRequired);
        }
        require!(self.is_enabled(), StoreError::DisabledMarket);
        let prices = self.prices(oracle)?;
        let is_exceeded = self
            .pnl_factor_exceeded(&prices, PnlFactorKind::ForAdl, is_long)
            .map_err(ModelError::from)?
            .is_some();
        self.set_adl_enabled(is_long, is_exceeded);
        let kind = if is_long {
            ClockKind::AdlForLong
        } else {
            ClockKind::AdlForShort
        };
        let clock = self
            .clocks
            .get_mut(kind)
            .ok_or(error!(StoreError::RequiredResourceNotFound))?;
        *clock = Clock::get()?.unix_timestamp;
        Ok(())
    }
}
