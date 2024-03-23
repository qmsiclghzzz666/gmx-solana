#![deny(missing_docs)]
#![deny(unreachable_pub)]

//! The core concepts of GMX.

/// Pool.
pub mod pool;

/// Market.
pub mod market;

/// Market params.
pub mod params;

/// Actions.
pub mod action;

/// Error type.
pub mod error;

/// Number utils.
pub mod num;

/// Utils.
pub mod utils;

/// Utils for testing.
#[cfg(any(test, feature = "test"))]
pub mod test;

pub use error::Error;
pub use market::{Market, MarketExt};
pub use pool::{Pool, PoolExt};

/// Alias for result.
pub type Result<T> = std::result::Result<T, Error>;
