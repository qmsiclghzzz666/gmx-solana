/// Controller instructions.
pub(crate) mod controller;

/// Market Management.
pub(crate) mod market;

/// Instructions for deposit.
pub(crate) mod deposit;

/// Instructions for withdrawal.
pub(crate) mod withdrawal;

/// Instructions for order.
pub(crate) mod order;

/// Instrcutions for features.
pub(crate) mod feature;

/// Instructions for the treasury.
pub(crate) mod treasury;

pub use controller::*;
pub use deposit::*;
pub use feature::*;
pub use market::*;
pub use order::*;
pub use treasury::*;
pub use withdrawal::*;
