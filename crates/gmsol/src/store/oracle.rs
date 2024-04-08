use std::ops::Deref;

use anchor_client::{
    anchor_lang::system_program,
    solana_sdk::{pubkey::Pubkey, signer::Signer},
    Program, RequestBuilder,
};
use data_store::{
    accounts, instruction,
    states::{Oracle, Seed},
};

use super::roles::find_roles_address;

/// Find PDA for [`Oracle`] account.
pub fn find_oracle_address(store: &Pubkey, index: u8) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[Oracle::SEED, store.as_ref(), &[index]], &data_store::id())
}

/// Oracle management for GMSOL.
pub trait OracleOps<C> {
    /// Initialize [`Oracle`] account.
    fn initialize_oracle(&self, store: &Pubkey, index: u8) -> (RequestBuilder<C>, Pubkey);
}

impl<C, S> OracleOps<C> for Program<C>
where
    C: Deref<Target = S> + Clone,
    S: Signer,
{
    fn initialize_oracle(&self, store: &Pubkey, index: u8) -> (RequestBuilder<C>, Pubkey) {
        let authority = self.payer();
        let oracle = find_oracle_address(store, index).0;
        let builder = self
            .request()
            .accounts(accounts::InitializeOracle {
                authority,
                store: *store,
                only_controller: find_roles_address(store, &authority).0,
                oracle,
                system_program: system_program::ID,
            })
            .args(instruction::InitializeOracle { index });
        (builder, oracle)
    }
}
