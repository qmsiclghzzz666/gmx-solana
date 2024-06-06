use std::ops::Deref;

use anchor_client::{
    anchor_lang::{system_program::System, Id},
    solana_sdk::{pubkey::Pubkey, signer::Signer},
    Program,
};
use data_store::{
    accounts, instruction,
    states::{Amount, Config, Factor, Seed},
};

use crate::utils::RpcBuilder;

use super::roles::find_roles_address;

/// Find PDA for `Config` account.
pub fn find_config_pda(store: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[Config::SEED, store.as_ref()], &data_store::id())
}

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

impl<C, S> ConfigOps<C> for Program<C>
where
    C: Deref<Target = S> + Clone,
    S: Signer,
{
    fn initialize_config(&self, store: &Pubkey) -> RpcBuilder<C> {
        let authority = self.payer();
        let only_controller = find_roles_address(store, &authority).0;
        let config = find_config_pda(store).0;
        RpcBuilder::new(self)
            .args(instruction::InitializeConfig {})
            .accounts(accounts::InitializeConfig {
                authority,
                only_controller,
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
        let only_controller = find_roles_address(store, &authority).0;
        let config = find_config_pda(store).0;
        RpcBuilder::new(self)
            .args(instruction::InsertAmount {
                key: key.to_string(),
                amount,
                new,
            })
            .accounts(accounts::InsertAmount {
                authority,
                only_controller,
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
        let only_controller = find_roles_address(store, &authority).0;
        let config = find_config_pda(store).0;
        RpcBuilder::new(self)
            .args(instruction::InsertFactor {
                key: key.to_string(),
                amount: factor,
                new,
            })
            .accounts(accounts::InsertFactor {
                authority,
                only_controller,
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
        let only_controller = find_roles_address(store, &authority).0;
        let config = find_config_pda(store).0;
        RpcBuilder::new(self)
            .args(instruction::InsertAddress {
                key: key.to_string(),
                address: *address,
                new,
            })
            .accounts(accounts::InsertAddress {
                authority,
                only_controller,
                store: *store,
                config,
            })
    }
}

impl<C, S> ConfigOps<C> for crate::Client<C>
where
    C: Deref<Target = S> + Clone,
    S: Signer,
{
    fn initialize_config(&self, store: &Pubkey) -> RpcBuilder<C> {
        let authority = self.payer();
        let only_controller = self.payer_roles_address(store);
        let config = self.find_config_address(store);
        self.data_store_request()
            .args(instruction::InitializeConfig {})
            .accounts(accounts::InitializeConfig {
                authority,
                only_controller,
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
        let only_controller = self.payer_roles_address(store);
        let config = self.find_config_address(store);
        self.data_store_request()
            .args(instruction::InsertAmount {
                key: key.to_string(),
                amount,
                new,
            })
            .accounts(accounts::InsertAmount {
                authority,
                only_controller,
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
        let only_controller = self.payer_roles_address(store);
        let config = self.find_config_address(store);
        self.data_store_request()
            .args(instruction::InsertFactor {
                key: key.to_string(),
                amount: factor,
                new,
            })
            .accounts(accounts::InsertFactor {
                authority,
                only_controller,
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
        let only_controller = self.payer_roles_address(store);
        let config = self.find_config_address(store);
        self.data_store_request()
            .args(instruction::InsertAddress {
                key: key.to_string(),
                address: *address,
                new,
            })
            .accounts(accounts::InsertAddress {
                authority,
                only_controller,
                store: *store,
                config,
            })
    }
}
