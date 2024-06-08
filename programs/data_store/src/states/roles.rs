use std::borrow::Borrow;

use anchor_lang::prelude::*;
use bitmaps::Bitmap;

use crate::DataStoreError;

use super::{InitSpace, Seed};

const MAX_LEN: usize = 32;

/// Max number of roles.
pub const MAX_ROLES: usize = 32;

/// Max number of members.
pub const MAX_MEMBERS: usize = 16;

/// The key of a Role.
#[derive(Clone, AnchorSerialize, AnchorDeserialize, InitSpace, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct RoleKey {
    #[max_len(MAX_LEN)]
    name: String,
}

impl RoleKey {
    /// CONTROLLER.
    pub const CONTROLLER: &'static str = "CONTROLLER";

    /// MARKET KEEPER.
    pub const MARKET_KEEPER: &'static str = "MARKET_KEEPER";

    /// ORDER KEEPER.
    pub const ORDER_KEEPER: &'static str = "ORDER_KEEPER";
}

/// Metadata of a role.
#[derive(Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct RoleMetadata {
    pub(super) enabled: bool,
    pub(super) index: u8,
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
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Roles {
    /// Authority.
    pub authority: Pubkey,
    /// Store.
    pub store: Pubkey,
    /// Is admin.
    pub(super) is_admin: bool,
    /// Roles value (a bitmap).
    pub(super) value: u32,
    pub bump: u8,
}

type RoleBitmap = Bitmap<MAX_ROLES>;
type RoleBitmapValue = u32;

impl Roles {
    /// Initialize the [`Roles`]
    pub fn init(&mut self, authority: Pubkey, store: Pubkey, bump: u8) {
        self.is_admin = false;
        self.value = RoleBitmap::new().into_value();
        self.authority = authority;
        self.store = store;
        self.bump = bump;
    }

    pub(super) fn get(&self, index: u8) -> bool {
        let map = RoleBitmap::from_value(self.value);
        map.get(index as usize)
    }

    pub(super) fn set(&mut self, index: u8, enable: bool) {
        let mut map = RoleBitmap::from_value(self.value);
        map.set(index as usize, enable);
        self.value = map.into_value();
    }

    /// Returns whether it is an admin.
    pub fn is_admin(&self) -> bool {
        self.is_admin
    }
}

impl Seed for Roles {
    const SEED: &'static [u8] = b"roles";
}

/// Role Metadata.
#[zero_copy]
#[derive(Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct RoleMetadataV2 {
    name: [u8; MAX_LEN],
    enabled: u8,
    index: u8,
}

impl InitSpace for RoleMetadataV2 {
    const INIT_SPACE: usize = 32 + 2;
}

#[cfg(test)]
const_assert_eq!(
    std::mem::size_of::<RoleMetadataV2>(),
    RoleMetadataV2::INIT_SPACE
);

impl RoleMetadataV2 {
    /// A `u8` value indicates that this role is enabled.
    pub const ROLE_ENABLED: u8 = u8::MAX;

    fn name_to_bytes(name: &str) -> Result<[u8; MAX_LEN]> {
        let bytes = name.as_bytes();
        require!(
            bytes.len() <= MAX_LEN,
            DataStoreError::ExceedMaxStringLengthLimit
        );
        let mut buffer = [0; 32];
        buffer[..bytes.len()].copy_from_slice(bytes);
        Ok(buffer)
    }

    fn bytes_to_name(bytes: &[u8; 32]) -> Result<&str> {
        let Some(end) = bytes.iter().position(|&x| x == 0) else {
            return err!(DataStoreError::InvalidRole);
        };
        let valid_bytes = &bytes[..end];
        std::str::from_utf8(valid_bytes).map_err(|_| error!(DataStoreError::InvalidRole))
    }

    /// Create a new role metadata.
    pub fn new(name: &str, index: u8) -> Result<Self> {
        Ok(Self {
            name: Self::name_to_bytes(name)?,
            enabled: Self::ROLE_ENABLED,
            index,
        })
    }

    /// Get the name of this role.
    pub fn name(&self) -> Result<&str> {
        Self::bytes_to_name(&self.name)
    }

    /// Enable this role.
    pub fn enable(&mut self) {
        self.enabled = Self::ROLE_ENABLED;
    }

    /// Disable this role.
    pub fn disable(&mut self) {
        self.enabled = 0;
    }

    /// Is enbaled.
    pub fn is_enabled(&self) -> bool {
        self.enabled == Self::ROLE_ENABLED
    }
}

crate::fixed_map!(RoleMap, RoleMetadataV2, MAX_ROLES, 0);

crate::fixed_map!(
    Members,
    Pubkey,
    crate::utils::pubkey::to_bytes,
    RoleBitmapValue,
    MAX_MEMBERS,
    0
);

/// Role Store.
#[zero_copy]
pub struct RoleStore {
    roles: RoleMap,
    members: Members,
}

impl InitSpace for RoleStore {
    const INIT_SPACE: usize = std::mem::size_of::<RoleStore>();
}

impl RoleStore {
    /// Enable a role.
    pub fn enable_role(&mut self, role: &str) -> Result<()> {
        match self.roles.get_mut(role) {
            Some(metadata) => {
                require_eq!(metadata.name()?, role, DataStoreError::InvalidArgument);
                metadata.enable();
            }
            None => {
                let index = self
                    .roles
                    .len()
                    .try_into()
                    .map_err(|_| error!(DataStoreError::ExceedMaxLengthLimit))?;
                self.roles
                    .insert_with_options(role, RoleMetadataV2::new(role, index)?, true)?;
            }
        }
        Ok(())
    }

    /// Disable a role.
    pub fn disable_role(&mut self, role: &str) -> Result<()> {
        if let Some(metadata) = self.roles.get_mut(role) {
            require_eq!(metadata.name()?, role, DataStoreError::InvalidArgument);
            metadata.disable();
        }
        Ok(())
    }

    /// Get the index of a enabled role.
    pub fn enabled_role_index(&self, role: &str) -> Result<Option<u8>> {
        if let Some(metadata) = self.roles.get(role) {
            require_eq!(metadata.name()?, role, DataStoreError::InvalidArgument);
            require!(metadata.is_enabled(), DataStoreError::DisabledRole);
            Ok(Some(metadata.index))
        } else {
            Ok(None)
        }
    }

    /// Check if the given role is granted to the pubkey.
    pub fn has_role(&self, authority: &Pubkey, role: &str) -> Result<bool> {
        let Some(value) = self.members.get(authority) else {
            return err!(DataStoreError::PermissionDenied);
        };
        let Some(index) = self.enabled_role_index(role)? else {
            return err!(DataStoreError::NoSuchRole);
        };
        let bitmap = RoleBitmap::from_value(*value);
        Ok(bitmap.get(index as usize))
    }

    /// Grant a role to the pubkey.
    pub fn grant(&mut self, authority: &Pubkey, role: &str) -> Result<()> {
        let Some(index) = self.enabled_role_index(role)? else {
            return err!(DataStoreError::NoSuchRole);
        };
        match self.members.get_mut(authority) {
            Some(value) => {
                let mut bitmap = RoleBitmap::from_value(*value);
                bitmap.set(index as usize, true);
                *value = bitmap.into_value();
            }
            None => {
                let mut bitmap = RoleBitmap::new();
                bitmap.set(index as usize, true);
                self.members
                    .insert_with_options(authority, bitmap.into_value(), true)?;
            }
        }
        Ok(())
    }

    /// Revoke a role from the pubkey.
    pub fn revoke(&mut self, authority: &Pubkey, role: &str) -> Result<()> {
        let Some(index) = self.enabled_role_index(role)? else {
            return err!(DataStoreError::NoSuchRole);
        };
        let Some(value) = self.members.get_mut(authority) else {
            return err!(DataStoreError::PermissionDenied);
        };
        let mut bitmap = RoleBitmap::from_value(*value);
        bitmap.set(index as usize, false);
        *value = bitmap.into_value();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use bytemuck::Zeroable;

    use super::*;

    #[test]
    fn grant_and_revoke_roles() {
        let mut store = RoleStore::zeroed();
        let authority = Pubkey::new_unique();

        assert!(store.grant(&authority, RoleKey::CONTROLLER).is_err());
        assert!(store.has_role(&authority, RoleKey::CONTROLLER).is_err());

        store.enable_role(RoleKey::CONTROLLER).unwrap();
        store.enable_role(RoleKey::MARKET_KEEPER).unwrap();

        store.grant(&authority, RoleKey::CONTROLLER).unwrap();
        assert_eq!(store.has_role(&authority, RoleKey::CONTROLLER), Ok(true));
        store.grant(&authority, RoleKey::MARKET_KEEPER).unwrap();
        assert_eq!(store.has_role(&authority, RoleKey::MARKET_KEEPER), Ok(true));
        assert_eq!(store.has_role(&authority, RoleKey::CONTROLLER), Ok(true));

        store.revoke(&authority, RoleKey::CONTROLLER).unwrap();
        assert_eq!(store.has_role(&authority, RoleKey::MARKET_KEEPER), Ok(true));
        assert_eq!(store.has_role(&authority, RoleKey::CONTROLLER), Ok(false));

        store.revoke(&authority, RoleKey::MARKET_KEEPER).unwrap();
        assert_eq!(
            store.has_role(&authority, RoleKey::MARKET_KEEPER),
            Ok(false)
        );
        assert_eq!(store.has_role(&authority, RoleKey::CONTROLLER), Ok(false));

        store.disable_role(RoleKey::MARKET_KEEPER).unwrap();
        assert!(store.grant(&authority, RoleKey::MARKET_KEEPER).is_err());
        assert!(store.has_role(&authority, RoleKey::MARKET_KEEPER).is_err());
        store.enable_role(RoleKey::MARKET_KEEPER).unwrap();
        store.grant(&authority, RoleKey::MARKET_KEEPER).unwrap();
        assert_eq!(store.has_role(&authority, RoleKey::MARKET_KEEPER), Ok(true));
    }

    #[test]
    fn enable_and_disable_role() {
        let mut store = RoleStore::zeroed();
        let authority = Pubkey::new_unique();

        store.enable_role(RoleKey::CONTROLLER).unwrap();
        store.grant(&authority, RoleKey::CONTROLLER).unwrap();
        assert_eq!(store.has_role(&authority, RoleKey::CONTROLLER), Ok(true));
        store.disable_role(RoleKey::CONTROLLER).unwrap();
        assert!(store.has_role(&authority, RoleKey::CONTROLLER).is_err());
        store.enable_role(RoleKey::CONTROLLER).unwrap();
        assert_eq!(store.has_role(&authority, RoleKey::CONTROLLER), Ok(true));
    }
}
