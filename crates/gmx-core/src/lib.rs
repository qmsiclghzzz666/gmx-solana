#![deny(missing_docs)]
#![deny(unreachable_pub)]
// FIXME: enable this when we are ready.
// #![warn(clippy::arithmetic_side_effects)]

//! The core concepts of GMX.

/// Pool.
pub mod pool;

/// Market.
pub mod market;

/// Position.
pub mod position;

/// Market params.
pub mod params;

/// Actions.
pub mod action;

/// Error type.
pub mod error;

/// Number utils.
pub mod num;

/// Fixed-point decimal type.
pub mod fixed;

/// Utils.
pub mod utils;

/// Utils for testing.
#[cfg(any(test, feature = "test"))]
pub mod test;

pub use error::Error;
pub use market::{Market, MarketExt};
pub use pool::{Pool, PoolExt, PoolKind};

/// Alias for result.
pub type Result<T> = std::result::Result<T, Error>;
