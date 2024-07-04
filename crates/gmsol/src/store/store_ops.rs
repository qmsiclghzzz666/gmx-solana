use std::ops::Deref;

use anchor_client::{
    anchor_lang::system_program,
    solana_sdk::{pubkey::Pubkey, signer::Signer},
};
use data_store::{accounts, instruction};

use crate::utils::RpcBuilder;

/// Data Store management for GMSOL.
pub trait StoreOps<C> {
    /// Initialize [`Store`](data_store::states::Store) account.
    fn initialize_store(&self, key: &str, authority: Option<&Pubkey>) -> RpcBuilder<C>;

    /// Transfer Store authority.
    fn transfer_store_authority(&self, store: &Pubkey, new_authority: &Pubkey) -> RpcBuilder<C>;

    /// Set new token map.
    fn set_token_map(&self, store: &Pubkey, token_map: &Pubkey) -> RpcBuilder<C>;
}

impl<C, S> StoreOps<C> for crate::Client<C>
where
    C: Deref<Target = S> + Clone,
    S: Signer,
{
    fn initialize_store(&self, key: &str, authority: Option<&Pubkey>) -> RpcBuilder<C> {
        let store = self.find_store_address(key);
        self.data_store_rpc()
            .accounts(accounts::Initialize {
                payer: self.payer(),
                store,
                system_program: system_program::ID,
            })
            .args(instruction::Initialize {
                key: key.to_string(),
                authority: authority.copied(),
            })
    }

    fn transfer_store_authority(&self, store: &Pubkey, new_authority: &Pubkey) -> RpcBuilder<C> {
        self.data_store_rpc()
            .args(instruction::TransferStoreAuthority {
                new_authority: *new_authority,
            })
            .accounts(accounts::TransferStoreAuthority {
                authority: self.payer(),
                store: *store,
            })
    }

    fn set_token_map(&self, store: &Pubkey, token_map: &Pubkey) -> RpcBuilder<C> {
        self.data_store_rpc()
            .args(instruction::SetTokenMap {})
            .accounts(accounts::SetTokenMap {
                authority: self.payer(),
                store: *store,
                token_map: *token_map,
            })
    }
}
