use anchor_lang::prelude::*;

use crate::{states::Market, DataStoreError};

/// Balance.
pub struct RevertibleBalance {
    is_pure: bool,
    long_token_balance: u64,
    short_token_balance: u64,
}

impl RevertibleBalance {
    /// Record transferred in.
    pub fn record_transferred_in(&mut self, is_long_token: bool, amount: u64) -> Result<()> {
        if self.is_pure || is_long_token {
            self.long_token_balance = self
                .long_token_balance
                .checked_add(amount)
                .ok_or(error!(DataStoreError::AmountOverflow))?;
        } else {
            self.short_token_balance = self
                .short_token_balance
                .checked_add(amount)
                .ok_or(error!(DataStoreError::AmountOverflow))?;
        }
        Ok(())
    }

    /// Record transferred out.
    pub fn record_transferred_out(&mut self, is_long_token: bool, amount: u64) -> Result<()> {
        if self.is_pure || is_long_token {
            self.long_token_balance = self
                .long_token_balance
                .checked_sub(amount)
                .ok_or(error!(DataStoreError::AmountOverflow))?;
        } else {
            self.short_token_balance = self
                .short_token_balance
                .checked_sub(amount)
                .ok_or(error!(DataStoreError::AmountOverflow))?;
        }
        Ok(())
    }

    /// Get balance for one side.
    pub fn balance_for_one_side(&self, is_long: bool) -> u64 {
        if is_long || self.is_pure {
            if self.is_pure {
                self.long_token_balance / 2
            } else {
                self.long_token_balance
            }
        } else {
            self.short_token_balance
        }
    }

    /// Write to market.
    ///
    /// ## Panic
    /// Panic if the pure falg is not matched.
    pub(crate) fn write_to_market(&self, market: &mut Market) {
        assert_eq!(market.is_pure(), self.is_pure);
        market.state.long_token_balance = self.long_token_balance;
        market.state.short_token_balance = self.short_token_balance;
        msg!(
            "{}: {},{}",
            market.meta.market_token_mint,
            market.state.long_token_balance,
            market.state.short_token_balance
        );
    }
}

impl<'a> From<&'a Market> for RevertibleBalance {
    fn from(market: &'a Market) -> Self {
        Self {
            is_pure: market.is_pure(),
            long_token_balance: market.state.long_token_balance,
            short_token_balance: market.state.short_token_balance,
        }
    }
}
