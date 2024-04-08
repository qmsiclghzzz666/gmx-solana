/// Error type for `gmsol`.
pub mod error;

/// Actions for `DataStore` program.
pub mod store;

/// Actions for `Exchange` program.`
pub mod exchange;

/// Utils.
pub mod utils;

pub use error::Error;

pub type Result<T> = std::result::Result<T, Error>;
