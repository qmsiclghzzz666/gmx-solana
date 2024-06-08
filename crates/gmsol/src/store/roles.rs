use std::ops::Deref;

use anchor_client::{
    solana_sdk::{pubkey::Pubkey, signer::Signer},
    RequestBuilder,
};

use data_store::{accounts, instruction};

/// Roles management for GMSOL.
pub trait RolesOps<C> {
    /// Grant a role to user.
    fn grant_role(&self, store: &Pubkey, user: &Pubkey, role: &str) -> RequestBuilder<C>;

    /// Enable a role.
    fn enable_role(&self, store: &Pubkey, role: &str) -> RequestBuilder<C>;
}

impl<C, S> RolesOps<C> for crate::Client<C>
where
    C: Deref<Target = S> + Clone,
    S: Signer,
{
    fn grant_role(&self, store: &Pubkey, user: &Pubkey, role: &str) -> RequestBuilder<C> {
        let authority = self.payer();
        self.data_store()
            .request()
            .accounts(accounts::GrantRole {
                authority,
                store: *store,
            })
            .args(instruction::GrantRole {
                user: *user,
                role: role.to_string(),
            })
    }

    fn enable_role(&self, store: &Pubkey, role: &str) -> RequestBuilder<C> {
        let authority = self.payer();
        self.data_store()
            .request()
            .accounts(accounts::EnableRole {
                authority,
                store: *store,
            })
            .args(instruction::EnableRole {
                role: role.to_string(),
            })
    }
}
