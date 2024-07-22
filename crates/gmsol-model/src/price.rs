use num_traits::{CheckedAdd, CheckedDiv};

/// Price.
#[derive(Debug, Clone, Copy)]
pub struct Price<T> {
    /// Minimum Price.
    pub min: T,
    /// Maximum Price.
    pub max: T,
}

impl<T> Price<T>
where
    T: CheckedAdd + CheckedDiv + num_traits::One,
{
    /// Get mid price checked.
    pub fn checked_mid(&self) -> Option<T> {
        self.min
            .checked_add(&self.max)
            .and_then(|p| p.checked_div(&(T::one() + T::one())))
    }

    /// Get mid price.
    ///
    /// # Panic
    /// Panic if cannot calculate the mid price.
    pub fn mid(&self) -> T {
        self.checked_mid().expect("cannot calculate the mid price")
    }
}

impl<T> Price<T>
where
    T: num_traits::Zero + CheckedAdd + CheckedDiv + num_traits::One,
{
    fn is_valid(&self) -> bool {
        !self.min.is_zero() && !self.max.is_zero() && self.checked_mid().is_some()
    }
}

/// Prices for execution.
#[derive(Debug, Clone, Copy)]
pub struct Prices<T> {
    /// Index token price.
    pub index_token_price: Price<T>,
    /// Long token price.
    pub long_token_price: Price<T>,
    /// Short token price.
    pub short_token_price: Price<T>,
}

impl<T> Prices<T> {
    /// Get collateral token price.
    pub fn collateral_token_price(&self, is_long: bool) -> &Price<T> {
        if is_long {
            &self.long_token_price
        } else {
            &self.short_token_price
        }
    }
}

impl<T> Prices<T>
where
    T: num_traits::Zero + CheckedAdd + CheckedDiv + num_traits::One,
{
    /// Check if the prices is valid.
    pub fn is_valid(&self) -> bool {
        self.index_token_price.is_valid()
            && self.long_token_price.is_valid()
            && self.short_token_price.is_valid()
    }

    /// Validate the prices.
    pub fn validate(&self) -> crate::Result<()> {
        if self.is_valid() {
            Ok(())
        } else {
            Err(crate::Error::invalid_argument("invalid prices"))
        }
    }
}
