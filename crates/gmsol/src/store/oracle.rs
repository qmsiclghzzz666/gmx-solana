use std::ops::Deref;

use anchor_client::{
    anchor_lang::system_program,
    solana_sdk::{pubkey::Pubkey, signer::Signer},
    RequestBuilder,
};
use data_store::{accounts, instruction};

/// Oracle management for GMSOL.
pub trait OracleOps<C> {
    /// Initialize [`Oracle`] account.
    fn initialize_oracle(&self, store: &Pubkey, index: u8) -> (RequestBuilder<C>, Pubkey);
}

impl<C, S> OracleOps<C> for crate::Client<C>
where
    C: Deref<Target = S> + Clone,
    S: Signer,
{
    fn initialize_oracle(&self, store: &Pubkey, index: u8) -> (RequestBuilder<C>, Pubkey) {
        let authority = self.payer();
        let oracle = self.find_oracle_address(store, index);
        let builder = self
            .data_store()
            .request()
            .accounts(accounts::InitializeOracle {
                authority,
                store: *store,
                oracle,
                system_program: system_program::ID,
            })
            .args(instruction::InitializeOracle { index });
        (builder, oracle)
    }
}
