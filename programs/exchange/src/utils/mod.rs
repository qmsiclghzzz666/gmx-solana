use anchor_lang::solana_program::pubkey::Pubkey;

use crate::constants;

/// Utils for market.
pub mod market;

/// Controller Seeds.
pub struct ControllerSeeds<'a> {
    store: &'a Pubkey,
    bump_bytes: [u8; 1],
}

impl<'a> ControllerSeeds<'a> {
    /// Create a controller seeds with the given store and bump.
    pub fn new(store: &'a Pubkey, bump: u8) -> Self {
        Self {
            store,
            bump_bytes: [bump],
        }
    }

    /// Create a controller seeds with only store address.
    pub fn find(store: &'a Pubkey) -> Self {
        let bump = Pubkey::find_program_address(
            &[&constants::CONTROLLER_SEED, store.as_ref()],
            &crate::ID,
        )
        .1;
        Self::new(store, bump)
    }

    /// Convert to seeds slice.
    pub fn as_seeds(&self) -> [&[u8]; 3] {
        [
            &constants::CONTROLLER_SEED,
            self.store.as_ref(),
            &self.bump_bytes,
        ]
    }
}
