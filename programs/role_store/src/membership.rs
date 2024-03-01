use crate::RoleStoreError;
use anchor_lang::prelude::*;

#[account]
pub struct Membership {
    // Length <= 32 bytes.
    role: String,
    bump: u8,
    valid: bool,
}

impl Membership {
    /// Maximum size.
    pub const MAX_SIZE: usize = (4 + 32) + 1 + 1;

    /// Seed.
    pub const SEED: &'static [u8] = b"membership";

    /// The ROLE_ADMIN role.
    pub const ROLE_ADMIN: &'static str = "ROLE_ADMIN";

    pub(super) fn grant_role(&mut self, role: &str, bump: u8) -> Result<()> {
        if role.len() > 32 {
            return Err(RoleStoreError::InvalidRoleName.into());
        }
        self.role = role.to_string();
        self.bump = bump;
        self.valid = true;
        Ok(())
    }

    pub(super) fn is_admin(&self) -> bool {
        matches!(self.role.as_str(), Self::ROLE_ADMIN)
    }

    pub(super) fn bump(&self) -> u8 {
        self.bump
    }
}
