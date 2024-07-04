use anchor_lang::prelude::*;
use gmsol_store::states::{InitSpace, Seed};

use crate::{constants::CONTROLLER_SEED, utils::ControllerSeeds};

use super::ReferralRoot;

/// Controller.
#[account(zero_copy)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Controller {
    /// Bump Seed.
    bump: u8,
    /// Padding.
    padding_0: [u8; 15],
    /// Store.
    store: Pubkey,
    /// Referral root.
    root: ReferralRoot,
    /// Reserved.
    reserved: [u8; 256],
}

impl Seed for Controller {
    const SEED: &'static [u8] = CONTROLLER_SEED;
}

impl InitSpace for Controller {
    const INIT_SPACE: usize = std::mem::size_of::<Self>();
}

impl Controller {
    /// As a [`ControllerSeeds`].
    pub fn as_controller_seeds(&self) -> ControllerSeeds<'_> {
        ControllerSeeds::new(&self.store, self.bump)
    }

    /// Initialize.
    pub fn init(&mut self, store: Pubkey, bump: u8) {
        self.store = store;
        self.bump = bump;
    }
}
