use anchor_lang::prelude::*;
use dual_vec_map::DualVecMap;

use crate::DataStoreError;

use super::Seed;

#[derive(AnchorSerialize, AnchorDeserialize, InitSpace, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct TokenConfig {
    /// Enabled.
    pub enabled: bool,
    /// Synthetic.
    pub synthetic: bool,
    /// The address of the price feed.
    pub price_feed: Pubkey,
    /// Heartbeat duration.
    pub heartbeat_duration: u32,
    /// Token decimals.
    pub token_decimals: u8,
    /// Precision.
    pub precision: u8,
}

#[account]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct TokenConfigMap {
    pub(crate) bump: u8,
    tokens: Vec<Pubkey>,
    configs: Vec<TokenConfig>,
}

impl TokenConfigMap {
    /// Get init space.
    pub const fn init_space(len: usize) -> usize {
        1 + (4 + TokenConfig::INIT_SPACE * len) + (4 + 32 * len)
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

    pub(crate) fn init(&mut self, bump: u8) {
        self.bump = bump;
        self.configs.clear();
        self.tokens.clear();
    }
}

impl Seed for TokenConfigMap {
    const SEED: &'static [u8] = b"token_config_map";
}
