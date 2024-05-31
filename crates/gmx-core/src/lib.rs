#![deny(missing_docs)]
#![deny(unreachable_pub)]
// FIXME: enable this when we are ready.
// #![warn(clippy::arithmetic_side_effects)]

//! The core concepts of GMX.

/// Pool.
pub mod pool;

/// Market.
pub mod market;

/// Clock.
pub mod clock;

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

pub use clock::ClockKind;
pub use error::Error;
pub use market::{Market, MarketExt, PnlFactorKind};
pub use pool::{Balance, BalanceExt, Pool, PoolExt, PoolKind};
pub use position::{Position, PositionExt};

/// Alias for result.
pub type Result<T> = std::result::Result<T, Error>;
