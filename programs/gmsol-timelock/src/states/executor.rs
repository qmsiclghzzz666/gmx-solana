use anchor_lang::prelude::*;
use gmsol_store::{
    states::{Seed, MAX_ROLE_NAME_LEN},
    utils::fixed_str::{bytes_to_fixed_str, fixed_str_to_bytes},
};

/// Executor.
#[account(zero_copy)]
pub struct Executor {
    pub(crate) bump: u8,
    padding: [u8; 15],
    pub(crate) store: Pubkey,
    role_name: [u8; MAX_ROLE_NAME_LEN],
    reserved: [u8; 256],
}

impl Executor {
    /// Wallet Seed.
    pub const WALLET_SEED: &'static [u8] = b"wallet";

    /// Get role name.
    pub fn role_name(&self) -> Result<&str> {
        bytes_to_fixed_str(&self.role_name)
    }

    pub(crate) fn try_init(&mut self, bump: u8, store: Pubkey, role_name: &str) -> Result<()> {
        let role_name = fixed_str_to_bytes(role_name)?;
        self.bump = bump;
        self.store = store;
        self.role_name = role_name;
        Ok(())
    }
}

impl Seed for Executor {
    const SEED: &'static [u8] = b"timelock_executor";
}

impl gmsol_utils::InitSpace for Executor {
    const INIT_SPACE: usize = std::mem::size_of::<Self>();
}

/// Executor Wallet Signer.
pub struct ExecutorWalletSigner {
    executor: Pubkey,
    bump_bytes: [u8; 1],
}

impl ExecutorWalletSigner {
    pub(crate) fn new(executor: Pubkey, bump: u8) -> Self {
        Self {
            executor,
            bump_bytes: [bump],
        }
    }

    pub(crate) fn as_seeds(&self) -> [&[u8]; 3] {
        [
            Executor::WALLET_SEED,
            self.executor.as_ref(),
            &self.bump_bytes,
        ]
    }
}
