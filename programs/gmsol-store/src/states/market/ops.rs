use anchor_lang::prelude::*;

use gmsol_model::BaseMarketExt;

use crate::{constants, ModelError, StoreError};

use super::HasMarketMeta;

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
        amount: u64,
    ) -> Result<()> {
        let side = self.market_meta().to_token_side(token)?;
        let balance = self
            .balance_excluding(token, &amount)
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
