/// Deposit.
pub mod deposit;

/// Withdraw.
pub mod withdraw;

/// Swap.
pub mod swap;

/// Increase Position.
pub mod increase_position;

/// Decrease Position.
pub mod decrease_position;

/// Prices of a market.
#[derive(Debug, Clone, Copy)]
pub struct Prices<T> {
    /// Index token price.
    pub index_token_price: T,
    /// Long token price.
    pub long_token_price: T,
    /// Short token price.
    pub short_token_price: T,
}

impl<T> Prices<T> {
    /// Get collateral token price.
    pub fn collateral_token_price(&self, is_long: bool) -> &T {
        if is_long {
            &self.long_token_price
        } else {
            &self.short_token_price
        }
    }
}

impl<T> Prices<T>
where
    T: num_traits::Zero,
{
    /// Check if the prices is valid.
    pub fn is_valid(&self) -> bool {
        !self.index_token_price.is_zero()
            && !self.long_token_price.is_zero()
            && !self.short_token_price.is_zero()
    }
}
