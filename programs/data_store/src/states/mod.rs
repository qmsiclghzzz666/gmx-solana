/// Data Store.
pub mod data_store;

/// Config.
pub mod config;

/// Common types.
pub mod common;

/// Nonce.
pub mod nonce;

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

pub use config::Config;
pub use data_store::*;
pub use deposit::Deposit;
pub use market::*;
pub use nonce::*;
pub use oracle::*;
pub use order::Order;
pub use position::Position;
pub use token_config::*;
pub use withdrawal::Withdrawal;

use anchor_lang::{
    prelude::{borsh, AnchorDeserialize, AnchorSerialize, Pubkey, Result},
    Bump,
};
use gmx_solana_utils::to_seed;

/// Data type that has [`SEED`].
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
        .map_err(|_| crate::DataStoreError::InvalidPDA)?;
        Ok(pda)
    }
}

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

/// Factor.
pub type Factor = u128;

/// Amount.
pub type Amount = u64;

/// Alias of [`Space`](anchor_lang::Space).
pub trait InitSpace {
    /// Init Space.
    const INIT_SPACE: usize;
}

impl InitSpace for u8 {
    const INIT_SPACE: usize = 1;
}

impl InitSpace for i64 {
    const INIT_SPACE: usize = 8;
}

impl InitSpace for Factor {
    const INIT_SPACE: usize = 16;
}

impl InitSpace for Amount {
    const INIT_SPACE: usize = 8;
}

impl InitSpace for Pubkey {
    const INIT_SPACE: usize = 32;
}

impl<T, const LEN: usize> InitSpace for [T; LEN]
where
    T: InitSpace,
{
    const INIT_SPACE: usize = T::INIT_SPACE * LEN;
}

impl<T> InitSpace for Option<T>
where
    T: InitSpace,
{
    const INIT_SPACE: usize = 1 + T::INIT_SPACE;
}
