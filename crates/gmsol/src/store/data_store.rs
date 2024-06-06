use std::ops::Deref;

use anchor_client::{
    anchor_lang::system_program,
    solana_sdk::{pubkey::Pubkey, signer::Signer},
    Program, RequestBuilder,
};
use data_store::{
    accounts, instruction,
    states::{DataStore, Seed},
};
use gmx_solana_utils::to_seed;

use super::roles::find_roles_address;

/// Find PDA for [`DataStore`] account.
pub fn find_store_address(key: &str) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[DataStore::SEED, &to_seed(key)], &data_store::id())
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

impl<C, S> StoreOps<C> for crate::Client<C>
where
    C: Deref<Target = S> + Clone,
    S: Signer,
{
    fn initialize_store(&self, key: &str) -> RequestBuilder<C> {
        let store = self.find_store_address(key);
        let roles = self.payer_roles_address(&store);
        self.data_store()
            .request()
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
