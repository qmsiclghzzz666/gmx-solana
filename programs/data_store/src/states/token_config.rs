use anchor_lang::prelude::*;
use gmx_solana_utils::to_seed;

use crate::DataStoreError;

#[account]
#[derive(InitSpace)]
pub struct TokenConfig {
    /// Bump.
    pub bump: u8,
    /// The address of the price feed.
    pub price_feed: Pubkey,
    /// Heartbeat duration.
    pub heartbeat_duration: u32,
    /// Token decimals.
    pub token_decimals: u8,
    /// Precision.
    pub precision: u8,
}

impl TokenConfig {
    /// Seed for [`TokenConfig`].
    pub const SEED: &'static [u8] = b"token_config";

    /// Init.
    pub fn init(
        &mut self,
        bump: u8,
        price_feed: Pubkey,
        heartbeat_duration: u32,
        token_decimals: u8,
        precision: u8,
    ) {
        self.bump = bump;
        self.price_feed = price_feed;
        self.heartbeat_duration = heartbeat_duration;
        self.token_decimals = token_decimals;
        self.precision = precision;
    }

    /// Update.
    pub fn update(
        &mut self,
        price_feed: Option<Pubkey>,
        token_decimals: Option<u8>,
        precision: Option<u8>,
    ) {
        if let Some(price_feed) = price_feed {
            self.price_feed = price_feed;
        }
        if let Some(token_decimals) = token_decimals {
            self.token_decimals = token_decimals;
        }
        if let Some(precision) = precision {
            self.precision = precision;
        }
    }

    /// Recreate the PDA.
    pub fn pda(&self, store: &Pubkey, key: &str) -> Result<Pubkey> {
        let pda = Pubkey::create_program_address(
            &[Self::SEED, store.as_ref(), &to_seed(key), &[self.bump]],
            &crate::ID,
        )
        .map_err(|_| DataStoreError::InvalidPDA)?;
        Ok(pda)
    }
}

#[event]
pub struct TokenConfigChangeEvent {
    pub key: String,
    pub address: Pubkey,
    pub init: bool,
    pub config: TokenConfig,
}
