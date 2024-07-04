use std::{num::NonZeroU64, str::FromStr};

use anchor_lang::prelude::*;
use bytemuck::Zeroable;
use gmsol_utils::to_seed;

use crate::{constants, StoreError, StoreResult};

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
            "Store({}): authority={} roles={} members={} token_map={} treasury={}",
            self.key()
                .map(|s| if s.is_empty() { "*default*" } else { s })
                .unwrap_or("*failed to parse*"),
            self.authority,
            self.role.num_roles(),
            self.role.num_members(),
            self.token_map()
                .map(|pubkey| pubkey.to_string())
                .unwrap_or("*unset*".to_string()),
            self.treasury,
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

    /// Get amount.
    pub fn get_amount(&self, key: &str) -> Result<&Amount> {
        let key = AmountKey::from_str(key).map_err(|_| error!(StoreError::InvalidKey))?;
        Ok(self.get_amount_by_key(key))
    }

    /// Get amount by key.
    #[inline]
    pub fn get_amount_by_key(&self, key: AmountKey) -> &Amount {
        self.amount.get(&key)
    }

    /// Get amount mutably
    pub fn get_amount_mut(&mut self, key: &str) -> Result<&mut Amount> {
        let key = AmountKey::from_str(key).map_err(|_| error!(StoreError::InvalidKey))?;
        Ok(self.amount.get_mut(&key))
    }

    /// Get factor.
    pub fn get_factor(&self, key: &str) -> Result<&Factor> {
        let key = FactorKey::from_str(key).map_err(|_| error!(StoreError::InvalidKey))?;
        Ok(self.get_factor_by_key(key))
    }

    /// Get factor by key.
    #[inline]
    pub fn get_factor_by_key(&self, key: FactorKey) -> &Factor {
        self.factor.get(&key)
    }

    /// Get factor mutably
    pub fn get_factor_mut(&mut self, key: &str) -> Result<&mut Factor> {
        let key = FactorKey::from_str(key).map_err(|_| error!(StoreError::InvalidKey))?;
        Ok(self.factor.get_mut(&key))
    }

    /// Get address.
    pub fn get_address(&self, key: &str) -> Result<&Pubkey> {
        let key = AddressKey::from_str(key).map_err(|_| error!(StoreError::InvalidKey))?;
        Ok(self.get_address_by_key(key))
    }

    /// Get address by key.
    #[inline]
    pub fn get_address_by_key(&self, key: AddressKey) -> &Pubkey {
        self.address.get(&key)
    }

    /// Get address mutably
    pub fn get_address_mut(&mut self, key: &str) -> Result<&mut Pubkey> {
        let key = AddressKey::from_str(key).map_err(|_| error!(StoreError::InvalidKey))?;
        Ok(self.address.get_mut(&key))
    }

    /// Calculate the request expiration time.
    pub fn request_expiration_at(&self, start: i64) -> StoreResult<i64> {
        start
            .checked_add_unsigned(self.amount.request_expiration)
            .ok_or(StoreError::AmountOverflow)
    }

    /// Get claimable time window size.
    pub fn claimable_time_window(&self) -> Result<NonZeroU64> {
        NonZeroU64::new(self.amount.claimable_time_window).ok_or(error!(StoreError::CannotBeZero))
    }

    /// Get claimable time window index for the given timestamp.
    pub fn claimable_time_window_index(&self, timestamp: i64) -> Result<i64> {
        let window: i64 = self
            .claimable_time_window()?
            .get()
            .try_into()
            .map_err(|_| error!(StoreError::AmountOverflow))?;
        Ok(timestamp / window)
    }

    /// Get claimable time key for the given timestamp.
    pub fn claimable_time_key(&self, timestamp: i64) -> Result<[u8; 8]> {
        let index = self.claimable_time_window_index(timestamp)?;
        Ok(index.to_be_bytes())
    }

    /// Get holding address.
    pub fn holding(&self) -> &Pubkey {
        &self.address.holding
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

#[cfg(feature = "display")]
impl std::fmt::Display for Treasury {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{ receiver={}, treasury={}, treasury_factor={} }}",
            self.receiver, self.treasury, self.treasury_factor
        )
    }
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

/// Amount keys.
#[derive(strum::EnumString, strum::Display, Clone, Copy)]
#[strum(serialize_all = "snake_case")]
#[non_exhaustive]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "enum-iter", derive(strum::EnumIter))]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[cfg_attr(feature = "clap", clap(rename_all = "snake_case"))]
pub enum AmountKey {
    /// Claimable time window.
    ClaimableTimeWindow,
    /// Recent time window.
    RecentTimeWindow,
    /// Request expiration.
    RequestExpiration,
    /// Oracle max age.
    OracleMaxAge,
    /// Oracle max timestamp range.
    OracleMaxTimestampRange,
}

impl Amounts {
    fn init(&mut self) {
        self.claimable_time_window = constants::DEFAULT_CLAIMABLE_TIME_WINDOW;
        self.recent_time_window = constants::DEFAULT_RECENT_TIME_WINDOW;
        self.request_expiration = constants::DEFAULT_REQUEST_EXPIRATION;
        self.oracle_max_age = constants::DEFAULT_ORACLE_MAX_AGE;
        self.oracle_max_timestamp_range = constants::DEFAULT_ORACLE_MAX_TIMESTAMP_RANGE;
    }

    /// Get.
    fn get(&self, key: &AmountKey) -> &Amount {
        match key {
            AmountKey::ClaimableTimeWindow => &self.claimable_time_window,
            AmountKey::RecentTimeWindow => &self.recent_time_window,
            AmountKey::RequestExpiration => &self.request_expiration,
            AmountKey::OracleMaxAge => &self.oracle_max_age,
            AmountKey::OracleMaxTimestampRange => &self.oracle_max_timestamp_range,
        }
    }

    /// Get mutably.
    fn get_mut(&mut self, key: &AmountKey) -> &mut Amount {
        match key {
            AmountKey::ClaimableTimeWindow => &mut self.claimable_time_window,
            AmountKey::RecentTimeWindow => &mut self.recent_time_window,
            AmountKey::RequestExpiration => &mut self.request_expiration,
            AmountKey::OracleMaxAge => &mut self.oracle_max_age,
            AmountKey::OracleMaxTimestampRange => &mut self.oracle_max_timestamp_range,
        }
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

/// Factor keys.
#[derive(strum::EnumString, strum::Display, Clone, Copy)]
#[strum(serialize_all = "snake_case")]
#[non_exhaustive]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "enum-iter", derive(strum::EnumIter))]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[cfg_attr(feature = "clap", clap(rename_all = "snake_case"))]
pub enum FactorKey {
    /// Oracle Ref Price Deviation.
    OracleRefPriceDeviation,
}

impl Factors {
    fn init(&mut self) {
        self.oracle_ref_price_deviation = constants::DEFAULT_ORACLE_REF_PRICE_DEVIATION;
    }

    /// Get.
    fn get(&self, key: &FactorKey) -> &Factor {
        match key {
            FactorKey::OracleRefPriceDeviation => &self.oracle_ref_price_deviation,
        }
    }

    /// Get mutably.
    fn get_mut(&mut self, key: &FactorKey) -> &mut Factor {
        match key {
            FactorKey::OracleRefPriceDeviation => &mut self.oracle_ref_price_deviation,
        }
    }
}

/// Addresses.
#[account(zero_copy)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Addresses {
    pub(crate) holding: Pubkey,
    reserved: [Pubkey; 31],
}

/// Address keys.
#[derive(strum::EnumString, strum::Display, Clone, Copy)]
#[strum(serialize_all = "snake_case")]
#[non_exhaustive]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "enum-iter", derive(strum::EnumIter))]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[cfg_attr(feature = "clap", clap(rename_all = "snake_case"))]
pub enum AddressKey {
    /// Holding.
    Holding,
}

impl Addresses {
    fn init(&mut self, holding: Pubkey) {
        self.holding = holding;
    }

    /// Get.
    fn get(&self, key: &AddressKey) -> &Pubkey {
        match key {
            AddressKey::Holding => &self.holding,
        }
    }

    /// Get mutably.
    fn get_mut(&mut self, key: &AddressKey) -> &mut Pubkey {
        match key {
            AddressKey::Holding => &mut self.holding,
        }
    }
}

#[event]
pub struct DataStoreInitEvent {
    pub key: String,
    pub address: Pubkey,
}
