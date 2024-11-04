/// Chainlink Pull Oracle (Data Streams).
#[cfg(feature = "chainlink-pull-oracle")]
pub mod pull_oracle;

#[cfg(feature = "chainlink-pull-oracle")]
pub use pull_oracle::Client;
