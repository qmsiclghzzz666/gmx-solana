use std::ops::Deref;

use anchor_client::{
    anchor_lang::{system_program::System, Id},
    solana_sdk::{pubkey::Pubkey, signer::Signer},
};
use data_store::{
    accounts, instruction,
    states::{Amount, Factor},
};

use crate::utils::RpcBuilder;

/// Config Operations.
pub trait ConfigOps<C> {
    /// Initialize config account.
    fn initialize_config(&self, store: &Pubkey) -> RpcBuilder<C>;

    /// Insert a global amount.
    fn insert_global_amount(
        &self,
        store: &Pubkey,
        key: &str,
        amount: Amount,
        new: bool,
    ) -> RpcBuilder<C>;

    /// Insert a global factor.
    fn insert_global_factor(
        &self,
        store: &Pubkey,
        key: &str,
        factor: Factor,
        new: bool,
    ) -> RpcBuilder<C>;

    /// Insert a global address.
    fn insert_global_address(
        &self,
        store: &Pubkey,
        key: &str,
        address: &Pubkey,
        new: bool,
    ) -> RpcBuilder<C>;
}

impl<C, S> ConfigOps<C> for crate::Client<C>
where
    C: Deref<Target = S> + Clone,
    S: Signer,
{
    fn initialize_config(&self, store: &Pubkey) -> RpcBuilder<C> {
        let authority = self.payer();
        let config = self.find_config_address(store);
        self.data_store_request()
            .args(instruction::InitializeConfig {})
            .accounts(accounts::InitializeConfig {
                authority,
                store: *store,
                config,
                system_program: System::id(),
            })
    }

    fn insert_global_amount(
        &self,
        store: &Pubkey,
        key: &str,
        amount: Amount,
        new: bool,
    ) -> RpcBuilder<C> {
        let authority = self.payer();
        let config = self.find_config_address(store);
        self.data_store_request()
            .args(instruction::InsertAmount {
                key: key.to_string(),
                amount,
                new,
            })
            .accounts(accounts::InsertAmount {
                authority,
                store: *store,
                config,
            })
    }

    fn insert_global_factor(
        &self,
        store: &Pubkey,
        key: &str,
        factor: Factor,
        new: bool,
    ) -> RpcBuilder<C> {
        let authority = self.payer();
        let config = self.find_config_address(store);
        self.data_store_request()
            .args(instruction::InsertFactor {
                key: key.to_string(),
                amount: factor,
                new,
            })
            .accounts(accounts::InsertFactor {
                authority,
                store: *store,
                config,
            })
    }

    fn insert_global_address(
        &self,
        store: &Pubkey,
        key: &str,
        address: &Pubkey,
        new: bool,
    ) -> RpcBuilder<C> {
        let authority = self.payer();
        let config = self.find_config_address(store);
        self.data_store_request()
            .args(instruction::InsertAddress {
                key: key.to_string(),
                address: *address,
                new,
            })
            .accounts(accounts::InsertAddress {
                authority,
                store: *store,
                config,
            })
    }
}
