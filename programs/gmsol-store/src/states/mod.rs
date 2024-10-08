/// Data Store.
pub mod store;

/// Feature.
pub mod feature;

/// Roles.
pub mod roles;

/// Common types.
pub mod common;

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

pub use deposit::DepositV2;
pub use market::*;
pub use oracle::*;
pub use order::{OrderParamsV2, OrderV2, UpdateOrderParams};
pub use position::Position;
pub use roles::*;
pub use shift::*;
pub use store::*;
pub use token_config::*;
pub use withdrawal::WithdrawalV2;

pub type Amount = u64;
pub type Factor = u128;

use gmsol_utils::InitSpace;

use anchor_lang::{
    prelude::{borsh, AnchorDeserialize, AnchorSerialize, Pubkey, Result},
    Bump,
};
use gmsol_utils::to_seed;

/// Data type that has [`SEED`](Seed::SEED).
pub trait Seed {
    /// Prefix seed for program derived addresses.
    const SEED: &'static [u8];
}

/// Data type stored in data store.
pub trait Data: Bump + Seed {
    /// Verify the key.
    #[allow(unused_variables)]
    fn verify(&self, key: &str) -> Result<()> {
        Ok(())
    }

    /// Recreate the Program Derived Address.
    fn pda(&self, store: &Pubkey, key: &str) -> Result<Pubkey> {
        self.verify(key)?;
        let pda = Pubkey::create_program_address(
            &[Self::SEED, store.as_ref(), &to_seed(key), &[self.seed()]],
            &crate::ID,
        )
        .map_err(|_| crate::StoreError::InvalidPDA)?;
        Ok(pda)
    }
}

/// Nonce Bytes.
pub type NonceBytes = [u8; 32];

/// Action.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Initialize.
    Init,
    /// Change.
    Change,
    /// Remove.
    Remove,
}
