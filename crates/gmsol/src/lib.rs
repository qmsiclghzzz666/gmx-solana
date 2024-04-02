/// Error type for `gmsol`.
pub mod error;

/// Actions for `DataStore` program.
pub mod store;

pub use error::Error;

pub type Result<T> = std::result::Result<T, Error>;
