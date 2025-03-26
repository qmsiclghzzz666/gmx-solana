/// Constants.
pub mod constants;

/// WASM support.
#[cfg(feature = "wasm")]
pub mod wasm;

pub use gmsol_model as model;
pub use gmsol_programs as programs;
