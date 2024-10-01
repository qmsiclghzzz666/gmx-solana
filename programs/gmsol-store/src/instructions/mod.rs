//! # Implementations of instructions
//! This crate contains the implementation of all core instructions for the Store Program.
//!
//! The instructions can be roughly categorized as follows:
//! - The creation and management instructions for [`Store`] account
//! are implemented in [`data_store`]. Although global configuration and permissions
//! management are also defined in the [`Store`] account, the related
//! instructions are implemented separately in [`config`] and [`roles`].
//! - The instructions related to token configuration management are implemented in
//! [`token_config`], which also includes the management instructions for the `TokenMap`
//! account (see [`token_config`](crate::states::token_config) for details) used to store these token configurations.
//! - The creation and removal instructions for [`Market`](crate::states::Market) account are implemented in
//! [`market`].
//! - The instrcutions for managing token accounts owned by the store account (such as
//! market vaults) are implemented in [`token`].
//! - The creation and management instructions for the [`Oracle`](crate::states::Oracle) account, used to parse
//! and cache oracle prices, are implemented in [`oracle`].
//! - The creation and removal instructions for the core actions for GMSOL are defined in
//! [`deposit`], [`withdrawal`], and [`order`] accordingly. The execution instructions for
//! these actions are defined in [`exchange`].
//! - Temporary instructions for fixing bugs are defined in [`bug_fix`].
//!
//! [`Store`]: crate::states::Store

/// Instructions for [`Store`](crate::states::Store) account.
pub mod data_store;

/// Instructions for config magament.
pub mod config;

/// Instructions for roles management.
pub mod roles;

/// Instructions for [`TokenConfig`](crate::states::TokenConfig) management.
pub mod token_config;

/// Instructions for [`Market`](crate::states::Market) account.
pub mod market;

/// Instructions for tokens and token accounts.
pub mod token;

/// Instructions for [`Oracle`](crate::states::Oracle) account.
pub mod oracle;

/// Instructions for [`Deposit`](crate::states::Deposit) account.
pub mod deposit;

/// Instructions for [`Withdrawal`](crate::states::Withdrawal) account.
pub mod withdrawal;

/// Instructions for [`Order`](crate::states::Order) account.
pub mod order;

/// Instructions for [`Position`](crate::states::Position) account.
pub mod position;

/// Instructions for the exchange funtionality.
pub mod exchange;

/// Instructions for GT.
pub mod gt;

/// Instructions for User accounts.
pub mod user;

/// Instructions for bug fixes.
#[cfg(not(feature = "no-bug-fix"))]
pub mod bug_fix;

pub use config::*;
pub use data_store::*;
pub use deposit::*;
pub use exchange::*;
pub use gt::*;
pub use market::*;
pub use oracle::*;
pub use order::*;
pub use position::*;
pub use roles::*;
pub use token::*;
pub use token_config::*;
pub use user::*;
pub use withdrawal::*;

#[cfg(not(feature = "no-bug-fix"))]
pub use bug_fix::*;
