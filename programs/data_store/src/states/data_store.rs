use std::borrow::Borrow;

use anchor_lang::prelude::*;
use bitmaps::Bitmap;
use dual_vec_map::DualVecMap;
use gmx_solana_utils::to_seed;

use crate::DataStoreError;

const MAX_LEN: usize = 32;

const MAX_ROLES: usize = 32;

#[account]
#[derive(InitSpace)]
pub struct DataStore {
    #[max_len(MAX_ROLES)]
    roles_metadata: Vec<RoleMetadata>,
    #[max_len(MAX_ROLES)]
    roles: Vec<RoleKey>,
    num_admins: u32,
    pub role_store: Pubkey,
    #[max_len(MAX_LEN)]
    key_seed: Vec<u8>,
    bump: u8,
}

impl DataStore {
    /// Seed.
    pub const SEED: &'static [u8] = b"data_store";

    /// Maximum length of key.
    pub const MAX_LEN: usize = MAX_LEN;

    /// Init.
    pub fn init(&mut self, role_store: Pubkey, key: &str, bump: u8) {
        // Init roles map.
        self.roles.clear();
        self.roles_metadata.clear();
        self.num_admins = 1;

        // Init others.
        self.role_store = role_store;
        self.key_seed = to_seed(key).into();
        self.bump = bump;
    }

    fn as_map_mut(&mut self) -> Result<DualVecMap<&mut Vec<RoleKey>, &mut Vec<RoleMetadata>>> {
        DualVecMap::try_from_stores(&mut self.roles, &mut self.roles_metadata)
            .map_err(|_| DataStoreError::InvalidDataStore.into())
    }

    fn as_map(&self) -> Result<DualVecMap<&Vec<RoleKey>, &Vec<RoleMetadata>>> {
        DualVecMap::try_from_stores(&self.roles, &self.roles_metadata)
            .map_err(|_| DataStoreError::InvalidDataStore.into())
    }

    /// Enable a role.
    pub fn enable_role(&mut self, role: &str) -> Result<()> {
        require!(
            role.len() <= MAX_LEN,
            DataStoreError::ExceedMaxStringLengthLimit
        );
        let mut map = self.as_map_mut()?;
        require!(
            map.len() < MAX_ROLES || map.get(role).is_none(),
            DataStoreError::ExceedMaxLengthLimit
        );
        let metadata = RoleMetadata {
            index: map
                .len()
                .try_into()
                .map_err(|_| DataStoreError::ExceedMaxLengthLimit)?,
            enabled: true,
        };
        map.insert(role.into(), metadata);
        Ok(())
    }

    /// Disable a role.
    pub fn disable_role(&mut self, role: &str) -> Result<()> {
        let mut map = self.as_map_mut()?;
        if let Some(metadata) = map.get_mut(role) {
            metadata.enabled = false;
        }
        Ok(())
    }

    /// Get the index of a role.
    /// Returns `None` if the role is not exist or disabled.
    pub fn get_index(&self, role: &str) -> Result<Option<u8>> {
        Ok(self
            .as_map()?
            .get(role)
            .and_then(|metadata| metadata.enabled.then_some(metadata.index)))
    }

    /// Get the role store key.
    pub fn role_store(&self) -> &Pubkey {
        &self.role_store
    }

    /// Add admin.
    pub fn add_admin(&mut self, roles: &mut Roles) -> Result<()> {
        require!(!roles.is_admin, DataStoreError::AlreadyBeAnAdmin);
        self.num_admins = self
            .num_admins
            .checked_add(1)
            .ok_or(DataStoreError::TooManyAdmins)?;
        roles.is_admin = true;
        Ok(())
    }

    /// Remove admin.
    pub fn remove_admin(&mut self, roles: &mut Roles) -> Result<()> {
        require!(roles.is_admin, DataStoreError::NotAnAdmin);
        require!(self.num_admins > 1, DataStoreError::AtLeastOneAdmin);
        self.num_admins -= 1;
        roles.is_admin = false;
        Ok(())
    }

    /// Check if the roles has the given enabled role.
    /// Returns `true` only when the `role` is enabled and the `roles` has that role.
    pub fn has_role(&self, roles: &Roles, role: &str) -> Result<bool> {
        let Some(index) = self.get_index(role)? else {
            return Ok(false);
        };
        Ok(roles.get(index))
    }

    /// Grant a role.
    pub fn grant(&self, roles: &mut Roles, role: &str) -> Result<()> {
        let Some(index) = self.get_index(role)? else {
            return Err(DataStoreError::InvalidRole.into());
        };
        roles.set(index, true);
        Ok(())
    }

    /// Revoke a role.
    pub fn revoke(&self, roles: &mut Roles, role: &str) -> Result<()> {
        let Some(index) = self.get_index(role)? else {
            return Err(DataStoreError::InvalidRole.into());
        };
        roles.set(index, false);
        Ok(())
    }
}

/// The key of a Role.
#[derive(Clone, AnchorSerialize, AnchorDeserialize, InitSpace, PartialEq, Eq, PartialOrd, Ord)]
pub struct RoleKey {
    #[max_len(MAX_LEN)]
    name: String,
}

/// Metadata of a role.
#[derive(Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub struct RoleMetadata {
    enabled: bool,
    index: u8,
}

impl Borrow<str> for RoleKey {
    fn borrow(&self) -> &str {
        &self.name
    }
}

impl<'a> From<&'a str> for RoleKey {
    fn from(value: &'a str) -> Self {
        Self {
            name: value.to_owned(),
        }
    }
}

/// Account that stores the roles of an address.
#[account]
#[derive(InitSpace)]
pub struct Roles {
    /// Is admin.
    is_admin: bool,
    /// Roles value (a bitmap).
    value: u32,
}

type RolesMap = Bitmap<MAX_ROLES>;

impl Roles {
    /// Initialize the [`Roles`]
    pub fn init(&mut self) {
        self.is_admin = false;
        self.value = RolesMap::new().into_value();
    }

    fn get(&self, index: u8) -> bool {
        let map = RolesMap::from_value(self.value);
        map.get(index as usize)
    }

    fn set(&mut self, index: u8, enable: bool) {
        let mut map = RolesMap::from_value(self.value);
        map.set(index as usize, enable);
        self.value = map.into_value();
    }
}

#[event]
pub struct DataStoreInitEvent {
    pub key: String,
    pub address: Pubkey,
    pub role_store: Pubkey,
}
