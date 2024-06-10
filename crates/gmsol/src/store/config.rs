use std::ops::Deref;

use anchor_client::solana_sdk::{pubkey::Pubkey, signer::Signer};
use data_store::{
    accounts, instruction,
    states::{Amount, Factor},
};

use crate::utils::RpcBuilder;

/// Config Operations.
pub trait ConfigOps<C> {
    /// Insert a global amount.
    fn insert_global_amount(&self, store: &Pubkey, key: &str, amount: Amount) -> RpcBuilder<C>;

    /// Insert a global factor.
    fn insert_global_factor(&self, store: &Pubkey, key: &str, factor: Factor) -> RpcBuilder<C>;

    /// Insert a global address.
    fn insert_global_address(&self, store: &Pubkey, key: &str, address: &Pubkey) -> RpcBuilder<C>;
}

impl<C, S> ConfigOps<C> for crate::Client<C>
where
    C: Deref<Target = S> + Clone,
    S: Signer,
{
    fn insert_global_amount(&self, store: &Pubkey, key: &str, amount: Amount) -> RpcBuilder<C> {
        let authority = self.payer();
        self.data_store_request()
            .args(instruction::InsertAmount {
                key: key.to_string(),
                amount,
            })
            .accounts(accounts::InsertAmount {
                authority,
                store: *store,
            })
    }

    fn insert_global_factor(&self, store: &Pubkey, key: &str, factor: Factor) -> RpcBuilder<C> {
        let authority = self.payer();
        self.data_store_request()
            .args(instruction::InsertFactor {
                key: key.to_string(),
                amount: factor,
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
