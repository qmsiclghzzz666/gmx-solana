#[cfg(feature = "cpi")]
mod cpi;

#[cfg(feature = "cpi")]
pub use self::cpi::*;

pub(crate) mod internal;

/// Chunk by.
pub mod chunk_by;

/// Pubkey utils.
pub mod pubkey;

/// Fixed-size string.
pub mod fixed_str;

/// Dynamic Access.
pub mod dynamic_access;

/// Utils for deserializing "zero-copy" account.
#[cfg(feature = "utils")]
pub mod de;
