use std::ops::Deref;

use anchor_client::{
    anchor_lang::system_program,
    solana_sdk::{pubkey::Pubkey, signer::Signer},
    Program, RequestBuilder,
};

use data_store::{
    accounts, instruction,
    states::{Roles, Seed},
};

/// Roles management for GMSOL.
pub trait RolesOps<C> {
    /// Find the derived address of [`Roles`] account.
    fn find_roles_address(&self, store: &Pubkey, authority: &Pubkey) -> (Pubkey, u8);

    /// Create a request to initialize a new [`Roles`] account.
    fn initialize_roles<'a>(
        &'a self,
        store: &Pubkey,
        authority: Option<&'a dyn Signer>,
    ) -> RequestBuilder<'a, C>;

    /// Grant a role to user.
    fn grant_role(&self, store: &Pubkey, user: &Pubkey, role: &str) -> RequestBuilder<C>;
}

impl<C, S> RolesOps<C> for Program<C>
where
    C: Deref<Target = S> + Clone,
    S: Signer,
{
    fn find_roles_address(&self, store: &Pubkey, authority: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[Roles::SEED, store.as_ref(), authority.as_ref()],
            &self.id(),
        )
    }

    fn initialize_roles<'a>(
        &'a self,
        store: &Pubkey,
        authority: Option<&'a dyn Signer>,
    ) -> RequestBuilder<'a, C> {
        let authority_pubkey = authority.map(|s| s.pubkey()).unwrap_or(self.payer());
        let roles = self.find_roles_address(store, &authority_pubkey).0;
        let builder = self
            .request()
            .accounts(accounts::InitializeRoles {
                authority: authority_pubkey,
                store: *store,
                roles,
                system_program: system_program::ID,
            })
            .args(instruction::InitializeRoles {});
        match authority {
            Some(signer) => builder.signer(signer),
            None => builder,
        }
    }

    fn grant_role(&self, store: &Pubkey, user: &Pubkey, role: &str) -> RequestBuilder<C> {
        let authority = self.payer();
        let only_admin = self.find_roles_address(store, &authority).0;
        let user_roles = self.find_roles_address(store, user).0;
        self.request()
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
}
