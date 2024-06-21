/// Revertible Balance.
pub mod balance;

/// Revertible Market.
pub mod market;

/// Revertible Swap Market.
pub mod swap_market;

/// Revertible Liquidity Market.
pub mod liquidity_market;

/// Revertible Perpetual Market.
pub mod perp_market;

pub use self::{
    balance::RevertibleBalance,
    liquidity_market::RevertibleLiquidityMarket,
    market::{RevertibleMarket, RevertiblePool},
    swap_market::RevertibleSwapMarket,
};

/// Revertible type.
pub trait Revertible {
    /// Commit the changes.
    ///
    /// ## Panic
    /// - Should panic if the commitment cannot be done.
    fn commit(self);
}
