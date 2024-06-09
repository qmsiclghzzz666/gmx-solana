use std::{
    cell::{Ref, RefMut},
    mem::size_of,
};

use anchor_lang::prelude::*;
use bitmaps::Bitmap;
use dual_vec_map::DualVecMap;

use crate::{
    constants::keys::token::{TIMESTAMP_ADJUSTMENT, TOKEN},
    DataStoreError,
};

use super::{common::MapStore, InitSpace, PriceProviderKind, Seed};

/// Default heartbeat duration for price updates.
pub const DEFAULT_HEARTBEAT_DURATION: u32 = 30;

/// Default precision for price.
pub const DEFAULT_PRECISION: u8 = 4;

/// Default timestamp adjustment.
pub const DEFAULT_TIMESTAMP_ADJUSTMENT: u32 = 0;

const MAX_FEEDS: usize = 4;
const MAX_FLAGS: usize = 8;
const MAX_TOKENS: usize = 256;

#[derive(AnchorSerialize, AnchorDeserialize, InitSpace, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct TokenConfig {
    /// Enabled.
    pub enabled: bool,
    /// Synthetic.
    pub synthetic: bool,
    /// Heartbeat duration.
    pub heartbeat_duration: u32,
    /// Token decimals.
    pub token_decimals: u8,
    /// Precision.
    pub precision: u8,
    /// Price Feeds.
    #[max_len(MAX_FEEDS)]
    pub feeds: Vec<Pubkey>,
    /// Expected provider.
    pub expected_provider: u8,
    /// Amounts config.
    pub amounts: MapStore<[u8; 32], u64, 1>,
}

impl TokenConfig {
    /// Get the corresponding price feed address.
    pub fn get_feed(&self, kind: &PriceProviderKind) -> Result<Pubkey> {
        let index = *kind as usize;
        let feed = self
            .feeds
            .get(index)
            .ok_or(DataStoreError::PriceFeedNotSet)?;
        if *feed == Pubkey::default() {
            err!(DataStoreError::PriceFeedNotSet)
        } else {
            Ok(*feed)
        }
    }

    /// Get expected price provider kind.
    pub fn expected_provider(&self) -> Result<PriceProviderKind> {
        let kind = PriceProviderKind::try_from(self.expected_provider)
            .map_err(|_| DataStoreError::InvalidProviderKindIndex)?;
        Ok(kind)
    }

    /// Get price feed address for the expected provider.
    pub fn get_expected_feed(&self) -> Result<Pubkey> {
        self.get_feed(&self.expected_provider()?)
    }

    /// Create a new token config from builder.
    pub fn new(
        synthetic: bool,
        token_decimals: u8,
        builder: TokenConfigBuilder,
        enable: bool,
    ) -> Self {
        Self {
            enabled: enable,
            synthetic,
            token_decimals,
            heartbeat_duration: builder.heartbeat_duration,
            precision: builder.precision,
            feeds: builder.feeds,
            expected_provider: builder
                .expected_provider
                .unwrap_or(PriceProviderKind::default() as u8),
            amounts: Default::default(),
        }
    }

    /// Get timestamp adjustment.
    pub fn timestamp_adjustment(&self) -> Option<u64> {
        self.amounts
            .get_with(TOKEN, TIMESTAMP_ADJUSTMENT, |amount| amount.copied())
    }
}

/// Token Flags.
#[repr(u8)]
#[non_exhaustive]
pub enum Flag {
    /// Is initialized.
    Initialized,
    /// Enabled.
    Enabled,
    /// Is a synthetic asset.
    Synthetic,
    // WARNING: Cannot have more than `MAX_FLAGS` flags.
}

type TokenFlags = Bitmap<MAX_FLAGS>;
type TokenFlagsValue = u8;

#[zero_copy]
#[derive(PartialEq, Eq)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct TokenConfigV2 {
    /// Flags.
    flags: TokenFlagsValue,
    /// Token decimals.
    token_decimals: u8,
    /// Precision.
    precision: u8,
    /// Expected provider.
    expected_provider: u8,
    /// Price Feeds.
    feeds: [Pubkey; MAX_FEEDS],
    /// Heartbeat duration.
    heartbeat_duration: u32,
    /// Timestamp adjustment.
    timestamp_adjustment: u32,
    reserved: [u8; 32],
}

impl TokenConfigV2 {
    /// Get the corresponding price feed address.
    pub fn get_feed(&self, kind: &PriceProviderKind) -> Result<Pubkey> {
        let index = *kind as usize;
        let feed = self
            .feeds
            .get(index)
            .ok_or(DataStoreError::PriceFeedNotSet)?;
        if *feed == Pubkey::default() {
            err!(DataStoreError::PriceFeedNotSet)
        } else {
            Ok(*feed)
        }
    }

    /// Get expected price provider kind.
    pub fn expected_provider(&self) -> Result<PriceProviderKind> {
        let kind = PriceProviderKind::try_from(self.expected_provider)
            .map_err(|_| DataStoreError::InvalidProviderKindIndex)?;
        Ok(kind)
    }

    /// Get price feed address for the expected provider.
    pub fn get_expected_feed(&self) -> Result<Pubkey> {
        self.get_feed(&self.expected_provider()?)
    }

    /// Set enabled.
    pub fn set_enabled(&mut self, enable: bool) {
        self.set_flag(Flag::Enabled, enable)
    }

    /// Set synthetic.
    pub fn set_synthetic(&mut self, is_synthetic: bool) {
        self.set_flag(Flag::Synthetic, is_synthetic)
    }

    /// Is enabled.
    pub fn is_enabled(&self) -> bool {
        self.flag(Flag::Enabled)
    }

    /// Is synthetic.
    pub fn is_synthetic(&self) -> bool {
        self.flag(Flag::Synthetic)
    }

    /// Set flag
    pub fn set_flag(&mut self, flag: Flag, value: bool) {
        let mut bitmap = TokenFlags::from_value(self.flags);
        let index = flag as usize;
        bitmap.set(index, value);
        self.flags = bitmap.into_value();
    }

    /// Get flag.
    pub fn flag(&self, flag: Flag) -> bool {
        let bitmap = TokenFlags::from_value(self.flags);
        let index = flag as usize;
        bitmap.get(index)
    }

    /// Create a new token config from builder.
    pub fn init(
        &mut self,
        synthetic: bool,
        token_decimals: u8,
        builder: TokenConfigBuilder,
        enable: bool,
    ) -> Result<()> {
        require!(
            !self.flag(Flag::Initialized),
            DataStoreError::InvalidArgument
        );
        self.set_flag(Flag::Initialized, true);
        let TokenConfigBuilder {
            heartbeat_duration,
            precision,
            feeds,
            expected_provider,
        } = builder;
        self.set_synthetic(synthetic);
        self.set_enabled(enable);
        self.token_decimals = token_decimals;
        self.precision = precision;
        self.feeds = feeds
            .try_into()
            .map_err(|_| error!(DataStoreError::InvalidArgument))?;
        self.expected_provider = expected_provider.unwrap_or(PriceProviderKind::default() as u8);
        self.heartbeat_duration = heartbeat_duration;
        self.timestamp_adjustment = DEFAULT_TIMESTAMP_ADJUSTMENT;
        Ok(())
    }

    /// Token decimals.
    pub fn token_decimals(&self) -> u8 {
        self.token_decimals
    }

    /// Price Precision.
    pub fn precision(&self) -> u8 {
        self.precision
    }

    /// Get timestamp adjustment.
    pub fn timestamp_adjustment(&self) -> u32 {
        self.timestamp_adjustment
    }

    /// Heartbeat duration.
    pub fn heartbeat_duration(&self) -> u32 {
        self.heartbeat_duration
    }
}

impl InitSpace for TokenConfigV2 {
    const INIT_SPACE: usize = std::mem::size_of::<Self>();
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct TokenConfigBuilder {
    heartbeat_duration: u32,
    precision: u8,
    feeds: Vec<Pubkey>,
    expected_provider: Option<u8>,
}

impl Default for TokenConfigBuilder {
    fn default() -> Self {
        Self {
            heartbeat_duration: DEFAULT_HEARTBEAT_DURATION,
            precision: DEFAULT_PRECISION,
            feeds: vec![Pubkey::default(); MAX_FEEDS],
            expected_provider: None,
        }
    }
}

impl TokenConfigBuilder {
    /// Update the feed address for the given price provider.
    /// Return error when the feed was not set before.
    pub fn update_price_feed(mut self, kind: &PriceProviderKind, new_feed: Pubkey) -> Result<Self> {
        let index = *kind as usize;
        let feed = self
            .feeds
            .get_mut(index)
            .ok_or(DataStoreError::PriceFeedNotSet)?;
        *feed = new_feed;
        Ok(self)
    }

    /// Set heartbeat duration.
    pub fn with_heartbeat_duration(mut self, duration: u32) -> Self {
        self.heartbeat_duration = duration;
        self
    }

    /// Set precision.
    pub fn with_precision(mut self, precision: u8) -> Self {
        self.precision = precision;
        self
    }

    /// Set expected provider.
    pub fn with_expected_provider(mut self, provider: PriceProviderKind) -> Self {
        self.expected_provider = Some(provider as u8);
        self
    }
}

#[account]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct TokenConfigMap {
    pub(crate) bump: u8,
    pub(crate) store: Pubkey,
    tokens: Vec<Pubkey>,
    configs: Vec<TokenConfig>,
}

impl TokenConfigMap {
    /// Get init space.
    pub const fn init_space(len: usize) -> usize {
        1 + 32 + (4 + TokenConfig::INIT_SPACE * len) + (4 + 32 * len)
    }

    pub(crate) fn as_map(&self) -> DualVecMap<&Vec<Pubkey>, &Vec<TokenConfig>> {
        DualVecMap::from_sorted_stores_unchecked(&self.tokens, &self.configs)
    }

    pub(crate) fn length_after_insert(&self, token: &Pubkey) -> usize {
        let map = self.as_map();
        match map.get(token) {
            None => map.len() + 1,
            Some(_) => map.len(),
        }
    }

    /// Check if the synthetic flag is the same as `expected` if exists.
    /// Always returns `true` if the config does not exist.
    fn check_synthetic_or_does_not_exist(&self, key: &Pubkey, expected: bool) -> bool {
        match self.as_map().get(key) {
            Some(config) => config.synthetic == expected,
            None => true,
        }
    }

    pub(crate) fn checked_insert(&mut self, key: Pubkey, config: TokenConfig) -> Result<()> {
        require!(
            self.check_synthetic_or_does_not_exist(&key, config.synthetic),
            DataStoreError::InvalidSynthetic
        );
        self.as_map_mut().insert(key, config);
        Ok(())
    }

    pub(crate) fn toggle_token_config(&mut self, key: &Pubkey, enable: bool) -> Result<()> {
        self.as_map_mut()
            .get_mut(key)
            .ok_or(DataStoreError::RequiredResourceNotFound)?
            .enabled = enable;
        Ok(())
    }

    fn as_map_mut(&mut self) -> DualVecMap<&mut Vec<Pubkey>, &mut Vec<TokenConfig>> {
        DualVecMap::from_sorted_stores_unchecked(&mut self.tokens, &mut self.configs)
    }

    pub(crate) fn set_expected_provider(
        &mut self,
        key: &Pubkey,
        kind: PriceProviderKind,
    ) -> Result<()> {
        self.as_map_mut()
            .get_mut(key)
            .ok_or(DataStoreError::RequiredResourceNotFound)?
            .expected_provider = kind as u8;
        Ok(())
    }

    pub(crate) fn init(&mut self, bump: u8, store: Pubkey) {
        self.bump = bump;
        self.store = store;
        self.configs.clear();
        self.tokens.clear();
    }

    pub(crate) fn insert_amount(&mut self, token: &Pubkey, key: &str, amount: u64) -> Result<()> {
        self.as_map_mut()
            .get_mut(token)
            .ok_or(DataStoreError::RequiredResourceNotFound)?
            .amounts
            .insert(TOKEN, key, amount);
        Ok(())
    }
}

impl Seed for TokenConfigMap {
    const SEED: &'static [u8] = b"token_config_map";
}

crate::fixed_map!(
    Tokens,
    Pubkey,
    crate::utils::pubkey::to_bytes,
    u8,
    MAX_TOKENS,
    0
);

/// Header of `TokenMap`.
#[account(zero_copy)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct TokenMapHeader {
    pub(crate) store: Pubkey,
    tokens: Tokens,
    reserved: [u8; 64],
}

impl InitSpace for TokenMapHeader {
    const INIT_SPACE: usize = std::mem::size_of::<TokenMapHeader>();
}

impl TokenMapHeader {
    /// Get the space of the whole `TokenMap` required, excluding discriminator.
    pub fn space(num_configs: u8) -> usize {
        TokenMapHeader::INIT_SPACE + (usize::from(num_configs) * TokenConfigV2::INIT_SPACE)
    }

    /// Get the space after push.
    pub fn space_after_push(&self) -> Result<usize> {
        let num_configs: u8 = self
            .tokens
            .len()
            .checked_add(1)
            .ok_or(error!(DataStoreError::ExceedMaxLengthLimit))?
            .try_into()
            .map_err(|_| error!(DataStoreError::AmountOverflow))?;
        Ok(Self::space(num_configs))
    }
}

/// Reference to Token Map.
pub struct TokenMapRef<'a> {
    header: Ref<'a, TokenMapHeader>,
    configs: Ref<'a, [u8]>,
}

/// Mutable Reference to Token Map.
pub struct TokenMapMut<'a> {
    header: RefMut<'a, TokenMapHeader>,
    configs: RefMut<'a, [u8]>,
}

/// Token Map Loader.
pub trait TokenMapLoader<'info> {
    fn load_token_map(&self) -> Result<TokenMapRef>;
    fn load_token_map_mut(&self) -> Result<TokenMapMut>;
}

impl<'info> TokenMapLoader<'info> for AccountLoader<'info, TokenMapHeader> {
    fn load_token_map(&self) -> Result<TokenMapRef> {
        // Check the account.
        self.load()?;

        let data = self.as_ref().try_borrow_data()?;
        let (_disc, data) = Ref::map_split(data, |d| d.split_at(8));
        let (header, configs) = Ref::map_split(data, |d| d.split_at(size_of::<TokenMapHeader>()));

        Ok(TokenMapRef {
            header: Ref::map(header, bytemuck::from_bytes),
            configs,
        })
    }

    fn load_token_map_mut(&self) -> Result<TokenMapMut> {
        // Check the account for mutablely access.
        self.load_mut()?;

        let data = self.as_ref().try_borrow_mut_data()?;
        let (_disc, data) = RefMut::map_split(data, |d| d.split_at_mut(8));
        let (header, configs) =
            RefMut::map_split(data, |d| d.split_at_mut(size_of::<TokenMapHeader>()));
        Ok(TokenMapMut {
            header: RefMut::map(header, bytemuck::from_bytes_mut),
            configs,
        })
    }
}

/// Read Token Map.
pub trait TokenMapAccess {
    /// Get the config of the given token.
    fn get(&self, token: &Pubkey) -> Option<&TokenConfigV2>;
}

impl<'a> TokenMapAccess for TokenMapRef<'a> {
    fn get(&self, token: &Pubkey) -> Option<&TokenConfigV2> {
        let index = usize::from(*self.header.tokens.get(token)?);
        msg!("{}", index);
        crate::utils::dynamic_access::get(&self.configs, index)
    }
}

/// Token Map Operations.
///
/// The token map is append-only.
pub trait TokenMapMutAccess {
    /// Get mutably the config of the given token.
    fn get_mut(&mut self, token: &Pubkey) -> Option<&mut TokenConfigV2>;

    /// Push a new token config.
    fn push_with(
        &mut self,
        token: &Pubkey,
        f: impl FnOnce(&mut TokenConfigV2) -> Result<()>,
    ) -> Result<()>;
}

impl<'a> TokenMapMutAccess for TokenMapMut<'a> {
    fn get_mut(&mut self, token: &Pubkey) -> Option<&mut TokenConfigV2> {
        let index = usize::from(*self.header.tokens.get(token)?);
        crate::utils::dynamic_access::get_mut(&mut self.configs, index)
    }

    fn push_with(
        &mut self,
        token: &Pubkey,
        f: impl FnOnce(&mut TokenConfigV2) -> Result<()>,
    ) -> Result<()> {
        let next_index = self.header.tokens.len();
        require!(
            next_index < MAX_TOKENS,
            DataStoreError::ExceedMaxLengthLimit
        );
        let index = next_index
            .try_into()
            .map_err(|_| error!(DataStoreError::AmountOverflow))?;
        self.header.tokens.insert_with_options(token, index, true)?;
        let Some(dst) =
            crate::utils::dynamic_access::get_mut::<TokenConfigV2>(&mut self.configs, next_index)
        else {
            return err!(DataStoreError::NoSpaceForNewData);
        };
        (f)(dst)
    }
}
