/// Revertible Market.
pub mod market;

/// Revertible Swap Market.
pub mod swap_market;

/// Revertible Liquidity Market.
pub mod liquidity_market;

/// Revertible Balance.
pub mod balance;

pub use self::{
    balance::RevertibleBalance,
    market::{RevertibleMarket, RevertiblePool},
};

/// Revertible type.
pub trait Revertible {
    /// Commit the changes.
    ///
    /// ## Panic
    /// - Should panic if the commitment cannot be done.
    fn commit(self);
}
