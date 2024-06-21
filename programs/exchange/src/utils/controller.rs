use anchor_lang::solana_program::pubkey::Pubkey;

use crate::constants;

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
    pub fn find_with_address(store: &'a Pubkey) -> (Self, Pubkey) {
        let (address, bump) =
            Pubkey::find_program_address(&[constants::CONTROLLER_SEED, store.as_ref()], &crate::ID);
        (Self::new(store, bump), address)
    }

    /// Create a controller seeds with only store address.
    pub fn find(store: &'a Pubkey) -> Self {
        Self::find_with_address(store).0
    }

    /// Convert to seeds slice.
    pub fn as_seeds(&self) -> [&[u8]; 3] {
        [
            constants::CONTROLLER_SEED,
            self.store.as_ref(),
            &self.bump_bytes,
        ]
    }
}
