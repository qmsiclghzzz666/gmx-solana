use anchor_lang::prelude::*;
use gmx_solana_utils::to_seed;

use super::{InitSpace, RoleStore, Seed};

const MAX_LEN: usize = 32;

/// Data Store.
#[account(zero_copy)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Store {
    bump: [u8; 1],
    authority: Pubkey,
    key_seed: [u8; 32],
    key: [u8; MAX_LEN],
    padding: [u8; 7],
    role: RoleStore,
}

impl InitSpace for Store {
    const INIT_SPACE: usize = std::mem::size_of::<Self>();
}

impl Seed for Store {
    const SEED: &'static [u8] = b"data_store";
}

#[cfg(feature = "display")]
impl std::fmt::Display for Store {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Store({}): authority={} roles={} members={}",
            self.key().unwrap_or("*failed to parse*"),
            self.authority,
            self.role.num_roles(),
            self.role.num_members(),
        )
    }
}

impl Store {
    /// Maximum length of key.
    pub const MAX_LEN: usize = MAX_LEN;

    /// Init.
    /// # Warning
    /// The `roles` is assumed to be initialized with `is_admin == false`.
    pub fn init(&mut self, authority: Pubkey, key: &str, bump: u8) -> Result<()> {
        self.key = crate::utils::fixed_str::fixed_str_to_bytes(key)?;
        self.key_seed = to_seed(key);
        self.bump = [bump];
        self.authority = authority;
        Ok(())
    }

    pub(crate) fn pda_seeds(&self) -> [&[u8]; 3] {
        [Self::SEED, &self.key_seed, &self.bump]
    }

    /// Get the role store.
    pub fn role(&self) -> &RoleStore {
        &self.role
    }

    /// Get the key of the store.
    pub fn key(&self) -> Result<&str> {
        crate::utils::fixed_str::bytes_to_fixed_str(&self.key)
    }

    /// Enable a role.
    pub fn enable_role(&mut self, role: &str) -> Result<()> {
        self.role.enable_role(role)
    }

    /// Disable a role.
    pub fn disable_role(&mut self, role: &str) -> Result<()> {
        self.role.disable_role(role)
    }

    /// Check if the roles has the given enabled role.
    /// Returns `true` only when the `role` is enabled and the `roles` has that role.
    pub fn has_role(&self, authority: &Pubkey, role: &str) -> Result<bool> {
        self.role.has_role(authority, role)
    }

    /// Grant a role.
    pub fn grant(&mut self, authority: &Pubkey, role: &str) -> Result<()> {
        self.role.grant(authority, role)
    }

    /// Revoke a role.
    pub fn revoke(&mut self, authority: &Pubkey, role: &str) -> Result<()> {
        self.role.revoke(authority, role)
    }

    /// Check if the given pubkey is the authority of the store.
    pub fn is_authority(&self, authority: &Pubkey) -> bool {
        self.authority == *authority
    }
}

#[event]
pub struct DataStoreInitEvent {
    pub key: String,
    pub address: Pubkey,
}
