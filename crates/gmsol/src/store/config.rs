use anchor_client::solana_sdk::pubkey::Pubkey;
use data_store::states::{Config, Seed};

/// Find PDA for `Config` account.
pub fn find_config_pda(store: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[Config::SEED, store.as_ref()], &data_store::id())
}
