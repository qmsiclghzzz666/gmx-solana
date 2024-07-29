/// Program Derived Addresses for GMSOL Programs.
pub mod pda;

/// GMSOL Client.
pub mod client;

/// GMSOL resource discovery.
#[cfg(feature = "discover")]
pub mod discover;

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

/// Program IDs.
pub mod program_ids;

/// GMSOL constants.
pub mod constants {
    pub use gmsol_exchange::constants::*;
    pub use gmsol_store::constants::*;
}

/// Test Utils.
#[cfg(test)]
mod test;

pub use client::{Client, ClientOptions};
pub use error::Error;
pub use gmsol_model as model;

#[cfg(feature = "decode")]
pub use gmsol_decode as decode;

pub type Result<T> = std::result::Result<T, Error>;
