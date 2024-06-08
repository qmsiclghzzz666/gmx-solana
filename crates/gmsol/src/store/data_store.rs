use std::ops::Deref;

use anchor_client::{anchor_lang::system_program, solana_sdk::signer::Signer, RequestBuilder};
use data_store::{accounts, instruction};

/// Data Store management for GMSOL.
pub trait StoreOps<C> {
    /// Initialize [`DataStore`] account.
    fn initialize_store(&self, key: &str) -> RequestBuilder<C>;
}

impl<C, S> StoreOps<C> for crate::Client<C>
where
    C: Deref<Target = S> + Clone,
    S: Signer,
{
    fn initialize_store(&self, key: &str) -> RequestBuilder<C> {
        let store = self.find_store_address(key);
        self.data_store()
            .request()
            .accounts(accounts::Initialize {
                authority: self.payer(),
                data_store: store,
                system_program: system_program::ID,
            })
            .args(instruction::Initialize {
                key: key.to_string(),
            })
    }
}
