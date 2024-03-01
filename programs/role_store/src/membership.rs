use crate::RoleStoreError;
use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Membership {
    // Length <= 32 bytes.
    #[max_len(32)]
    role: String,
    bump: u8,
    valid: bool,
    pub authority: Pubkey,
}

impl Membership {
    /// Seed.
    pub const SEED: &'static [u8] = b"membership";

    /// The ROLE_ADMIN role.
    pub const ROLE_ADMIN: &'static str = "ROLE_ADMIN";

    /// The CONTROLLER role.
    pub const CONTROLLER: &'static str = "CONTROLLER";

    pub(super) fn grant_role(&mut self, role: &str, bump: u8, authority: Pubkey) -> Result<()> {
        if role.len() > 32 {
            return Err(RoleStoreError::InvalidRoleName.into());
        }
        self.role = role.to_string();
        self.bump = bump;
        self.valid = true;
        self.authority = authority;
        Ok(())
    }

    pub(super) fn revoke_role(&mut self) {
        self.valid = false;
    }

    /// Check if it is a valid membership.
    pub fn is_valid(&self) -> bool {
        self.valid
    }

    /// Check if it is a role admin.
    pub fn is_admin(&self) -> bool {
        self.is_valid() && matches!(self.role.as_str(), Self::ROLE_ADMIN)
    }

    /// Check if it is a controller.
    pub fn is_controller(&self) -> bool {
        self.is_valid() && matches!(self.role.as_str(), Self::CONTROLLER)
    }

    /// Bump.
    pub fn bump(&self) -> u8 {
        self.bump
    }
}
