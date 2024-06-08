use anchor_lang::prelude::*;
use dual_vec_map::DualVecMap;
use gmx_solana_utils::to_seed;

use crate::DataStoreError;

use super::{
    roles::{RoleKey, RoleMetadata, Roles, MAX_ROLES},
    Seed,
};

const MAX_LEN: usize = 32;

#[account]
#[derive(InitSpace)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Store {
    #[max_len(MAX_ROLES)]
    roles_metadata: Vec<RoleMetadata>,
    #[max_len(MAX_ROLES)]
    roles: Vec<RoleKey>,
    num_admins: u32,
    #[max_len(MAX_LEN)]
    key_seed: Vec<u8>,
    pub bump: [u8; 1],
    reserved: [u8; 64],
}

impl Seed for Store {
    const SEED: &'static [u8] = b"data_store";
}

impl Store {
    /// Maximum length of key.
    pub const MAX_LEN: usize = MAX_LEN;

    /// Init.
    /// # Warning
    /// The `roles` is assumed to be initialized with `is_admin == false`.
    pub fn init(&mut self, roles: &mut Roles, key: &str, bump: u8) -> Result<()> {
        // Init roles map.
        self.roles.clear();
        self.roles_metadata.clear();
        self.num_admins = 0;

        // Init others.
        self.key_seed = to_seed(key).into();
        self.bump = [bump];

        self.add_admin(roles)
    }

    pub(crate) fn pda_seeds(&self) -> [&[u8]; 3] {
        [Self::SEED, &self.key_seed, &self.bump]
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
        if let Some(metadata) = map.get_mut(role) {
            metadata.enabled = true;
        } else {
            let metadata = RoleMetadata {
                index: map
                    .len()
                    .try_into()
                    .map_err(|_| DataStoreError::ExceedMaxLengthLimit)?,
                enabled: true,
            };
            map.try_insert(role.into(), metadata)
                .map_err(|_| DataStoreError::InvalidDataStore)?;
        }
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

#[event]
pub struct DataStoreInitEvent {
    pub key: String,
    pub address: Pubkey,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_uninited_store() -> Store {
        Store {
            roles_metadata: vec![],
            roles: vec![],
            num_admins: 0,
            key_seed: vec![],
            bump: [0],
            reserved: [0; 64],
        }
    }

    fn new_uninited_roles() -> Roles {
        Roles {
            authority: Pubkey::default(),
            store: Pubkey::default(),
            is_admin: false,
            value: 0,
            bump: 0,
        }
    }

    fn new_store(roles: &mut Roles) -> Store {
        let mut store = new_uninited_store();
        store.init(roles, "hello", 255).unwrap();
        store
    }

    fn new_roles() -> Roles {
        let mut roles = new_uninited_roles();
        roles.init(Pubkey::default(), Pubkey::default(), 255);
        roles
    }

    #[test]
    fn test_admins() {
        let mut roles = new_roles();
        let mut store = new_store(&mut roles);
        assert_eq!(store.num_admins, 1);

        assert!(store.remove_admin(&mut roles).is_err());

        let mut other_roles = new_roles();
        store.add_admin(&mut other_roles).unwrap();
        assert_eq!(store.num_admins, 2);
        store.remove_admin(&mut other_roles).unwrap();

        assert!(store.add_admin(&mut roles).is_err());
        assert!(store.remove_admin(&mut other_roles).is_err());
        assert_eq!(store.num_admins, 1);
    }

    #[test]
    fn swap_admins() {
        let mut roles_1 = new_roles();
        let mut roles_2 = new_roles();
        let mut store = new_store(&mut roles_1);

        assert!(roles_1.is_admin);
        assert!(!roles_2.is_admin);
        assert_eq!(store.num_admins, 1);

        store.add_admin(&mut roles_2).unwrap();
        store.remove_admin(&mut roles_1).unwrap();
        assert!(!roles_1.is_admin);
        assert!(roles_2.is_admin);
        assert_eq!(store.num_admins, 1);
    }

    #[test]
    fn grant_and_revoke_roles() {
        let mut roles_1 = new_roles();
        let mut store = new_store(&mut roles_1);

        assert!(store.grant(&mut roles_1, RoleKey::CONTROLLER).is_err());
        assert_eq!(store.has_role(&roles_1, RoleKey::CONTROLLER), Ok(false));

        store.enable_role(RoleKey::CONTROLLER).unwrap();
        store.enable_role(RoleKey::MARKET_KEEPER).unwrap();

        store.grant(&mut roles_1, RoleKey::CONTROLLER).unwrap();
        assert_eq!(store.has_role(&roles_1, RoleKey::CONTROLLER), Ok(true));
        store.grant(&mut roles_1, RoleKey::MARKET_KEEPER).unwrap();
        assert_eq!(store.has_role(&roles_1, RoleKey::MARKET_KEEPER), Ok(true));
        assert_eq!(store.has_role(&roles_1, RoleKey::CONTROLLER), Ok(true));

        store.revoke(&mut roles_1, RoleKey::CONTROLLER).unwrap();
        assert_eq!(store.has_role(&roles_1, RoleKey::MARKET_KEEPER), Ok(true));
        assert_eq!(store.has_role(&roles_1, RoleKey::CONTROLLER), Ok(false));

        store.revoke(&mut roles_1, RoleKey::MARKET_KEEPER).unwrap();
        assert_eq!(store.has_role(&roles_1, RoleKey::MARKET_KEEPER), Ok(false));
        assert_eq!(store.has_role(&roles_1, RoleKey::CONTROLLER), Ok(false));

        store.disable_role(RoleKey::MARKET_KEEPER).unwrap();
        assert!(store.grant(&mut roles_1, RoleKey::MARKET_KEEPER).is_err());
        assert_eq!(store.has_role(&roles_1, RoleKey::MARKET_KEEPER), Ok(false));
        store.enable_role(RoleKey::MARKET_KEEPER).unwrap();
        store.grant(&mut roles_1, RoleKey::MARKET_KEEPER).unwrap();
        assert_eq!(store.has_role(&roles_1, RoleKey::MARKET_KEEPER), Ok(true));
    }

    #[test]
    fn enable_and_disable_role() {
        let mut roles_1 = new_roles();
        let mut store = new_store(&mut roles_1);

        store.enable_role(RoleKey::CONTROLLER).unwrap();
        store.grant(&mut roles_1, RoleKey::CONTROLLER).unwrap();
        assert_eq!(store.has_role(&roles_1, RoleKey::CONTROLLER), Ok(true));
        store.disable_role(RoleKey::CONTROLLER).unwrap();
        assert_eq!(store.has_role(&roles_1, RoleKey::CONTROLLER), Ok(false));
        store.enable_role(RoleKey::CONTROLLER).unwrap();
        assert_eq!(store.has_role(&roles_1, RoleKey::CONTROLLER), Ok(true));
    }
}
