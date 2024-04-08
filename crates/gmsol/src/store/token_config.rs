use std::ops::Deref;

use anchor_client::{
    anchor_lang::system_program,
    solana_sdk::{pubkey::Pubkey, signer::Signer},
    Program, RequestBuilder,
};
use data_store::{
    accounts, instruction,
    states::{Seed, TokenConfigMap},
};

use super::roles::find_roles_address;

/// Find PDA for [`TokenConfigMap`] account.
pub fn find_token_config_map(store: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[TokenConfigMap::SEED, store.as_ref()], &data_store::id())
}
/// Token config management for GMSOL.
pub trait TokenConfigOps<C> {
    /// Initialize [`TokenConfigMap`] account.
    fn initialize_token_config_map(&self, store: &Pubkey) -> (RequestBuilder<C>, Pubkey);
}

impl<C, S> TokenConfigOps<C> for Program<C>
where
    C: Deref<Target = S> + Clone,
    S: Signer,
{
    fn initialize_token_config_map(&self, store: &Pubkey) -> (RequestBuilder<C>, Pubkey) {
        let authority = self.payer();
        let map = find_token_config_map(store).0;
        let builder = self
            .request()
            .accounts(accounts::InitializeTokenConfigMap {
                authority,
                only_controller: find_roles_address(store, &authority).0,
                store: *store,
                map,
                system_program: system_program::ID,
            })
            .args(instruction::InitializeTokenConfigMap { len: 0 });
        (builder, map)
    }
}
