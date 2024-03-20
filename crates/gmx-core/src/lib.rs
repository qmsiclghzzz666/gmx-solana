#![deny(missing_docs)]
#![deny(unreachable_pub)]

//! The core concepts of GMX.

/// Pool.
pub mod pool;

/// Market.
pub mod market;

/// Actions.
pub mod action;

/// Error type.
pub mod error;

/// Utils for testing.
#[cfg(any(test, feature = "test"))]
pub mod test;

pub use error::Error;
