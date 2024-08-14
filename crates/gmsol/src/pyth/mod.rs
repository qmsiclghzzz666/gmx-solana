/// Pyth Pull Oracle.
#[cfg(feature = "pyth-pull-oracle")]
pub mod pull_oracle;

/// Push Oracle.
pub mod push_oracle;

use anchor_client::solana_sdk::pubkey::Pubkey;
#[cfg(feature = "pyth-pull-oracle")]
pub use pull_oracle::{
    hermes::{EncodingType, Hermes},
    PythPullOracle, PythPullOracleContext, PythPullOracleOps,
};

pub use push_oracle::find_pyth_feed_account;
use pyth_sdk::Identifier;

/// Convert a pubkey to feed id.
pub fn pubkey_to_identifier(feed: &Pubkey) -> Identifier {
    Identifier::new(feed.to_bytes())
}
