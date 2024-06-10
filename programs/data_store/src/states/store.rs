use anchor_lang::prelude::*;
use bytemuck::Zeroable;
use gmx_solana_utils::to_seed;

use crate::constants;

use super::{Amount, Factor, InitSpace, RoleStore, Seed};

const MAX_LEN: usize = 32;

/// Data Store.
#[account(zero_copy)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Store {
    bump: [u8; 1],
    key_seed: [u8; 32],
    key: [u8; MAX_LEN],
    padding: [u8; 7],
    role: RoleStore,
    /// Store authority.
    pub authority: Pubkey,
    /// The token map to used.
    pub token_map: Pubkey,
    /// Treasury Config.
    treasury: Treasury,
    /// Amounts.
    pub(crate) amount: Amounts,
    /// Factors.
    pub(crate) factor: Factors,
    /// Addresses.
    pub(crate) address: Addresses,
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
            "Store({}): authority={} roles={} members={} token_map={}",
            self.key().unwrap_or("*failed to parse*"),
            self.authority,
            self.role.num_roles(),
            self.role.num_members(),
            self.token_map()
                .map(|pubkey| pubkey.to_string())
                .unwrap_or("*unset*".to_string()),
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
        self.treasury.init(authority, authority);
        self.amount.init();
        self.factor.init();
        self.address.init(authority);
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

    /// Get token map address.
    pub fn token_map(&self) -> Option<&Pubkey> {
        if self.token_map == Pubkey::zeroed() {
            None
        } else {
            Some(&self.token_map)
        }
    }
}

/// Treasury.
#[account(zero_copy)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Treasury {
    /// Receiver.
    receiver: Pubkey,
    /// Treasury.
    treasury: Pubkey,
    /// Treasury claim factor.
    treasury_factor: u128,
    /// Next treasury claim factor.
    next_treasury_factor: u128,
}

impl Treasury {
    fn init(&mut self, receiver: Pubkey, treasury: Pubkey) {
        self.receiver = receiver;
        self.treasury = treasury;
    }
}

/// Amounts.
#[account(zero_copy)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Amounts {
    pub(crate) claimable_time_window: Amount,
    pub(crate) recent_time_window: Amount,
    pub(crate) request_expiration: Amount,
    pub(crate) oracle_max_age: Amount,
    pub(crate) oracle_max_timestamp_range: Amount,
    reserved_1: [Amount; 27],
    reserved_2: [Amount; 96],
}

impl Amounts {
    fn init(&mut self) {
        self.claimable_time_window = constants::DEFAULT_CLAIMABLE_TIME_WINDOW;
        self.recent_time_window = constants::DEFAULT_RECENT_TIME_WINDOW;
        self.request_expiration = constants::DEFAULT_REQUEST_EXPIRATION;
        self.oracle_max_age = constants::DEFAULT_ORACLE_MAX_AGE;
        self.oracle_max_timestamp_range = constants::DEFAULT_ORACLE_MAX_TIMESTAMP_RANGE;
    }
}

/// Factors.
#[account(zero_copy)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Factors {
    pub(crate) oracle_ref_price_deviation: Factor,
    reserved_1: [Factor; 31],
    reserved_2: [Factor; 32],
}

impl Factors {
    fn init(&mut self) {
        self.oracle_ref_price_deviation = constants::DEFAULT_ORACLE_REF_PRICE_DEVIATION;
    }
}

/// Addresses.
#[account(zero_copy)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Addresses {
    pub(crate) holding: Pubkey,
    reserved: [Pubkey; 31],
}

impl Addresses {
    fn init(&mut self, holding: Pubkey) {
        self.holding = holding;
    }
}

#[event]
pub struct DataStoreInitEvent {
    pub key: String,
    pub address: Pubkey,
}
