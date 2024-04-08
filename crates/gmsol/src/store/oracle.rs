use anchor_client::solana_sdk::pubkey::Pubkey;
use data_store::states::{Oracle, Seed};

/// Find PDA for [`Oracle`] account.
pub fn find_oracle_address(store: &Pubkey, index: u8) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[Oracle::SEED, store.as_ref(), &[index]], &data_store::id())
}
