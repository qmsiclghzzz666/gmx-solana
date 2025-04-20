#![deny(missing_docs)]
#![deny(unreachable_pub)]

//! # GMSOL Solana Utils

/// Error type.
pub mod error;

/// Cluster.
pub mod cluster;

/// Signer.
pub mod signer;

/// Program.
pub mod program;

/// Compute budget.
pub mod compute_budget;

/// Transaction builder.
pub mod transaction_builder;

/// Transaction bundle builder.
#[cfg(client)]
pub mod bundle_builder;

/// RPC client extension.
#[cfg(client)]
pub(crate) mod client;

/// Utils.
pub mod utils;

pub use crate::error::Error;

/// Result type.
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(anchor)]
pub use anchor_lang;
#[cfg(client)]
pub use solana_client;
pub use solana_sdk;
