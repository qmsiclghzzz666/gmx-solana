use anchor_lang::prelude::*;

use super::Seed;

/// Nonce bytes type.
pub type NonceBytes = [u8; 32];

/// Incrementing nonce value.
#[account]
#[derive(InitSpace)]
pub struct Nonce {
    /// The bump seed.
    pub bump: u8,
    /// High value.
    pub hi: u128,
    /// Low value.
    pub lo: u128,
}

impl Seed for Nonce {
    const SEED: &'static [u8] = b"nonce";
}

impl Nonce {
    pub(crate) fn init(&mut self, bump: u8) {
        self.bump = bump;
        self.hi = 0;
        self.lo = 0;
    }

    pub(crate) fn inc(&mut self) {
        match self.lo.checked_add(1) {
            Some(lo) => self.lo = lo,
            None => {
                // FIXME: should we also handle the overflow here?
                self.hi += 1;
                self.lo = 0;
            }
        }
    }

    /// Get the current nonce in bytes.
    pub fn nonce(&self) -> NonceBytes {
        let hi = self.hi.to_be_bytes();
        let lo = self.lo.to_be_bytes();
        let mut ans = [0u8; 32];
        ans[0..16].copy_from_slice(&hi);
        ans[16..32].copy_from_slice(&lo);
        ans
    }
}
