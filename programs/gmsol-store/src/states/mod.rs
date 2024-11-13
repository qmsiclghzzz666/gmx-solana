/// Common types.
pub mod common;

/// Store.
pub mod store;

/// Feature.
pub mod feature;

/// Roles.
pub mod roles;

/// Token Config.
pub mod token_config;

/// Market.
pub mod market;

/// Oracle.
pub mod oracle;

/// Deposit.
pub mod deposit;

/// Withdrawal.
pub mod withdrawal;

/// Order.
pub mod order;

/// Position.
pub mod position;

/// Shift.
pub mod shift;

/// User.
pub mod user;

/// GLV.
pub mod glv;

/// GT model.
pub mod gt;

pub use deposit::Deposit;
pub use glv::{Glv, GlvDeposit, GlvShift, GlvWithdrawal};
pub use market::{
    config::MarketConfigKey, pool::Pool, HasMarketMeta, Market, MarketMeta, OtherState,
};
pub use oracle::*;
pub use order::{Order, OrderParams, UpdateOrderParams};
pub use position::Position;
pub use roles::*;
pub use shift::*;
pub use store::*;
pub use token_config::*;
pub use withdrawal::Withdrawal;

pub type Amount = u64;
pub type Factor = u128;

use gmsol_utils::InitSpace;

/// Data type that has [`SEED`](Seed::SEED).
pub trait Seed {
    /// Prefix seed for program derived addresses.
    const SEED: &'static [u8];
}

/// Nonce Bytes.
pub type NonceBytes = [u8; 32];
