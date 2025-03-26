/// Constants.
pub mod constants;

/// Error type.
pub mod error;

/// WASM support.
#[cfg(feature = "wasm")]
pub mod wasm;

/// Model support.
pub mod model {
    pub use gmsol_model::*;
    pub use gmsol_programs::model::*;
}

pub use error::Error;

/// Result type.
pub type Result<T> = std::result::Result<T, Error>;

pub use gmsol_programs as programs;
