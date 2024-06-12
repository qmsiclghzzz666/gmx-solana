use std::ops::Deref;

use anchor_client::solana_sdk::{pubkey::Pubkey, signer::Signer};
use data_store::{
    accounts, instruction,
    states::{AddressKey, Amount, AmountKey, Factor, FactorKey},
};

use crate::utils::RpcBuilder;

/// Config Operations.
pub trait ConfigOps<C> {
    /// Insert a global amount.
    fn insert_global_amount(&self, store: &Pubkey, key: &str, amount: &Amount) -> RpcBuilder<C>;

    /// Insert a global factor.
    fn insert_global_factor(&self, store: &Pubkey, key: &str, factor: &Factor) -> RpcBuilder<C>;

    /// Insert a global address.
    fn insert_global_address(&self, store: &Pubkey, key: &str, address: &Pubkey) -> RpcBuilder<C>;

    /// Insert a global amount by key.
    fn insert_global_amount_by_key(
        &self,
        store: &Pubkey,
        key: AmountKey,
        amount: &Amount,
    ) -> RpcBuilder<C> {
        let key = key.to_string();
        self.insert_global_amount(store, &key, amount)
    }

    /// Insert a global factor by key.
    fn insert_global_factor_by_key(
        &self,
        store: &Pubkey,
        key: FactorKey,
        factor: &Factor,
    ) -> RpcBuilder<C> {
        let key = key.to_string();
        self.insert_global_factor(store, &key, factor)
    }

    /// Insert a global address by key.
    fn insert_global_address_by_key(
        &self,
        store: &Pubkey,
        key: AddressKey,
        address: &Pubkey,
    ) -> RpcBuilder<C> {
        let key = key.to_string();
        self.insert_global_address(store, &key, address)
    }
}

impl<C, S> ConfigOps<C> for crate::Client<C>
where
    C: Deref<Target = S> + Clone,
    S: Signer,
{
    fn insert_global_amount(&self, store: &Pubkey, key: &str, amount: &Amount) -> RpcBuilder<C> {
        let authority = self.payer();
        self.data_store_request()
            .args(instruction::InsertAmount {
                key: key.to_string(),
                amount: *amount,
            })
            .accounts(accounts::InsertAmount {
                authority,
                store: *store,
            })
    }

    fn insert_global_factor(&self, store: &Pubkey, key: &str, factor: &Factor) -> RpcBuilder<C> {
        let authority = self.payer();
        self.data_store_request()
            .args(instruction::InsertFactor {
                key: key.to_string(),
                factor: *factor,
            })
            .accounts(accounts::InsertFactor {
                authority,
                store: *store,
            })
    }

    fn insert_global_address(&self, store: &Pubkey, key: &str, address: &Pubkey) -> RpcBuilder<C> {
        let authority = self.payer();
        self.data_store_request()
            .args(instruction::InsertAddress {
                key: key.to_string(),
                address: *address,
            })
            .accounts(accounts::InsertAddress {
                authority,
                store: *store,
            })
    }
}
