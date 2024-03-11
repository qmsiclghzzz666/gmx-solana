/// Data Store.
pub mod data_store;

/// Token Config.
pub mod token_config;

/// Market.
pub mod market;

pub use data_store::*;
pub use market::*;
pub use token_config::*;

use anchor_lang::{
    prelude::{Pubkey, Result},
    Bump,
};
use gmx_solana_utils::to_seed;

/// Data type stored in data store.
pub trait Data: Bump {
    /// Prefix seed for program derived addresses.
    const SEED: &'static [u8];

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
