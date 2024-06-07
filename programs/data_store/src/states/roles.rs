use std::borrow::Borrow;

use anchor_lang::prelude::*;
use bitmaps::Bitmap;

use super::Seed;

const MAX_LEN: usize = 32;

/// Max number of roles.
pub const MAX_ROLES: usize = 32;

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

type RolesMap = Bitmap<MAX_ROLES>;

impl Roles {
    /// Initialize the [`Roles`]
    pub fn init(&mut self, authority: Pubkey, store: Pubkey, bump: u8) {
        self.is_admin = false;
        self.value = RolesMap::new().into_value();
        self.authority = authority;
        self.store = store;
        self.bump = bump;
    }

    pub(super) fn get(&self, index: u8) -> bool {
        let map = RolesMap::from_value(self.value);
        map.get(index as usize)
    }

    pub(super) fn set(&mut self, index: u8, enable: bool) {
        let mut map = RolesMap::from_value(self.value);
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
