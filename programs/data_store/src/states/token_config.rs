use anchor_lang::prelude::*;
use dual_vec_map::DualVecMap;

use super::Seed;

#[derive(AnchorSerialize, AnchorDeserialize, InitSpace, Clone)]
pub struct TokenConfig2 {
    /// Enabled.
    pub enabled: bool,
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
pub struct TokenConfigMap {
    pub(crate) bump: u8,
    tokens: Vec<Pubkey>,
    configs: Vec<TokenConfig2>,
}

impl TokenConfigMap {
    /// Get init space.
    pub const fn init_space(len: usize) -> usize {
        1 + (4 + TokenConfig2::INIT_SPACE * len) + (4 + 32 * len)
    }

    pub(crate) fn as_map(&self) -> DualVecMap<&Vec<Pubkey>, &Vec<TokenConfig2>> {
        DualVecMap::from_sorted_stores_unchecked(&self.tokens, &self.configs)
    }

    pub(crate) fn as_map_mut(&mut self) -> DualVecMap<&mut Vec<Pubkey>, &mut Vec<TokenConfig2>> {
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
