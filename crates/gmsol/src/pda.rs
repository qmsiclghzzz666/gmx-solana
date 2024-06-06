use anchor_client::solana_sdk::pubkey::Pubkey;
use data_store::states::{Config, DataStore, Oracle, Roles, Seed, TokenConfigMap};
use gmx_solana_utils::to_seed;

/// Find PDA for [`DataStore`] account.
pub fn find_store_address(key: &str, store_program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[DataStore::SEED, &to_seed(key)], store_program_id)
}

/// Find PDA for [`Roles`] account.
pub fn find_roles_address(
    store: &Pubkey,
    authority: &Pubkey,
    store_program_id: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[Roles::SEED, store.as_ref(), authority.as_ref()],
        store_program_id,
    )
}

/// Find PDA for the controller address of exchange program.
pub fn find_controller_address(store: &Pubkey, exchange_program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[exchange::constants::CONTROLLER_SEED, store.as_ref()],
        exchange_program_id,
    )
}

/// Find PDA for [`Oracle`] account.
pub fn find_oracle_address(store: &Pubkey, index: u8, store_program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[Oracle::SEED, store.as_ref(), &[index]], store_program_id)
}

/// Find PDA for [`TokenConfigMap`] account.
pub fn find_token_config_map(store: &Pubkey, store_program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[TokenConfigMap::SEED, store.as_ref()], store_program_id)
}

/// Find PDA for [`Config`] account.
pub fn find_config_pda(store: &Pubkey, store_program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[Config::SEED, store.as_ref()], store_program_id)
}
