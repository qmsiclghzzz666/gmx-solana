/// Program Derived Addresses for GMSOL Programs.
pub mod pda;

/// GMSOL Client.
pub mod client;

/// Error type for `gmsol`.
pub mod error;

/// Actions for `DataStore` program.
pub mod store;

/// Actions for `Exchange` program.`
pub mod exchange;

/// Pyth integration.
pub mod pyth;

/// Utils.
pub mod utils;

/// GMSOL types.
pub mod types;

pub use client::{Client, ClientOptions};
pub use error::Error;
pub use gmx_core as core;

pub type Result<T> = std::result::Result<T, Error>;
