use anchor_client::solana_sdk::pubkey::Pubkey;
use data_store::states::{DataStore, Roles, Seed};
use gmx_solana_utils::to_seed;

/// Find PDA for [`DataStore`] account.
pub fn find_store_address(key: &str, program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[DataStore::SEED, &to_seed(key)], program_id)
}

/// Find PDA for [`Roles`] account.
pub fn find_roles_address(store: &Pubkey, authority: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[Roles::SEED, store.as_ref(), authority.as_ref()],
        program_id,
    )
}

/// Find PDA for the controller address of exchange program.
pub fn find_controller_address(store: &Pubkey, exchange_program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[exchange::constants::CONTROLLER_SEED, store.as_ref()],
        exchange_program_id,
    )
}
