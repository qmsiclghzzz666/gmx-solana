use anchor_lang::prelude::*;
use gmx_solana_utils::to_seed;

const MAX_LEN: usize = 32;

#[account]
#[derive(InitSpace)]
pub struct DataStore {
    pub role_store: Pubkey,
    #[max_len(MAX_LEN)]
    pub key: Vec<u8>,
    pub bump: u8,
}

impl DataStore {
    /// Seed.
    pub const SEED: &'static [u8] = b"data_store";

    /// Maximum length of key.
    pub const MAX_LEN: usize = MAX_LEN;

    pub fn init(&mut self, role_store: Pubkey, key: &str, bump: u8) {
        self.role_store = role_store;
        self.key = to_seed(key).into();
        self.bump = bump;
    }

    /// Get the role store key.
    pub fn role_store(&self) -> &Pubkey {
        &self.role_store
    }
}

#[event]
pub struct DataStoreInitEvent {
    pub key: String,
    pub address: Pubkey,
    pub role_store: Pubkey,
}
