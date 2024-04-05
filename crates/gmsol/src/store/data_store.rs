use std::ops::Deref;

use anchor_client::{
    anchor_lang::system_program,
    solana_sdk::{pubkey::Pubkey, signer::Signer},
    Program, RequestBuilder,
};
use data_store::{
    accounts, instruction,
    states::{DataStore, Market, Seed, TokenConfigMap},
};
use gmx_solana_utils::to_seed;

use super::roles::find_roles_address;

/// Find PDA for [`DataStore`] account.
pub fn find_store_address(key: &str) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[DataStore::SEED, &to_seed(key)], &data_store::id())
}

/// Find PDA for [`Market`] account.
pub fn find_market_address(store: &Pubkey, token: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[Market::SEED, store.as_ref(), &to_seed(&token.to_string())],
        &data_store::id(),
    )
}

/// Find PDA for market vault account.
pub fn find_market_vault_address(store: &Pubkey, token: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            data_store::constants::MARKET_VAULT_SEED,
            store.as_ref(),
            token.as_ref(),
            &[],
        ],
        &data_store::id(),
    )
}

/// Find PDA for [`TokenConfigMap`] account.
pub fn find_token_config_map(store: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[TokenConfigMap::SEED, store.as_ref()], &data_store::id())
}

/// Data Store management for GMSOL.
pub trait StoreOps<C> {
    /// Initialize [`DataStore`] account.
    fn initialize_store(&self, key: &str) -> RequestBuilder<C>;
}

impl<C, S> StoreOps<C> for Program<C>
where
    C: Deref<Target = S> + Clone,
    S: Signer,
{
    fn initialize_store(&self, key: &str) -> RequestBuilder<C> {
        let store = find_store_address(key).0;
        let roles = find_roles_address(&store, &self.payer()).0;
        self.request()
            .accounts(accounts::Initialize {
                authority: self.payer(),
                data_store: store,
                roles,
                system_program: system_program::ID,
            })
            .args(instruction::Initialize {
                key: key.to_string(),
            })
    }
}
