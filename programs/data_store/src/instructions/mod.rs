/// Instructions for Data Store.
pub mod data_store;

/// Instructions for roles management.
pub mod roles;

/// Instructions for Token Config.
pub mod token_config;

/// Instructions for Market.
pub mod market;

/// Instructions for Oracle.
pub mod oracle;

pub use data_store::*;
pub use market::*;
pub use oracle::*;
pub use roles::*;
pub use token_config::*;
