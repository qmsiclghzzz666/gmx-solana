use std::ops::Deref;

use anchor_client::{
    anchor_lang::system_program,
    solana_sdk::{pubkey::Pubkey, signer::Signer},
    RequestBuilder,
};

use data_store::{accounts, instruction};

/// Roles management for GMSOL.
pub trait RolesOps<C> {
    /// Create a request to initialize a new [`Roles`] account.
    fn initialize_roles<'a>(&'a self, store: &Pubkey, authority: &Pubkey) -> RequestBuilder<'a, C>;

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
    fn initialize_roles<'a>(&'a self, store: &Pubkey, authority: &Pubkey) -> RequestBuilder<'a, C> {
        let roles = self.find_roles_address(store, authority);
        let builder = self
            .data_store()
            .request()
            .accounts(accounts::InitializeRoles {
                payer: self.payer(),
                store: *store,
                roles,
                system_program: system_program::ID,
            })
            .args(instruction::InitializeRoles {
                authority: *authority,
            });
        builder
    }

    fn grant_role(&self, store: &Pubkey, user: &Pubkey, role: &str) -> RequestBuilder<C> {
        let authority = self.payer();
        let only_admin = self.payer_roles_address(store);
        let user_roles = self.find_roles_address(store, user);
        self.data_store()
            .request()
            .accounts(accounts::GrantRole {
                authority,
                store: *store,
                only_admin,
                user_roles,
            })
            .args(instruction::GrantRole {
                user: *user,
                role: role.to_string(),
            })
    }

    fn enable_role(&self, store: &Pubkey, role: &str) -> RequestBuilder<C> {
        let authority = self.payer();
        let only_admin = self.payer_roles_address(store);
        self.data_store()
            .request()
            .accounts(accounts::EnableRole {
                authority,
                store: *store,
                only_admin,
            })
            .args(instruction::EnableRole {
                role: role.to_string(),
            })
    }
}
