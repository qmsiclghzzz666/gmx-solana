use std::ops::Deref;

use anchor_client::solana_sdk::{pubkey::Pubkey, signer::Signer};

use gmsol_store::{accounts, instruction};

use crate::utils::RpcBuilder;

/// Roles management for GMSOL.
pub trait RolesOps<C> {
    /// Enable a role.
    fn enable_role(&self, store: &Pubkey, role: &str) -> RpcBuilder<C>;

    /// Disable a role.
    fn disable_role(&self, store: &Pubkey, role: &str) -> RpcBuilder<C>;

    /// Grant a role to user.
    fn grant_role(&self, store: &Pubkey, user: &Pubkey, role: &str) -> RpcBuilder<C>;

    /// Revoke a role from the user.
    fn revoke_role(&self, store: &Pubkey, user: &Pubkey, role: &str) -> RpcBuilder<C>;
}

impl<C, S> RolesOps<C> for crate::Client<C>
where
    C: Deref<Target = S> + Clone,
    S: Signer,
{
    fn enable_role(&self, store: &Pubkey, role: &str) -> RpcBuilder<C> {
        let authority = self.payer();
        self.store_rpc()
            .accounts(accounts::EnableRole {
                authority,
                store: *store,
            })
            .args(instruction::EnableRole {
                role: role.to_string(),
            })
    }

    fn disable_role(&self, store: &Pubkey, role: &str) -> RpcBuilder<C> {
        self.store_rpc()
            .accounts(accounts::DisableRole {
                authority: self.payer(),
                store: *store,
            })
            .args(instruction::DisableRole {
                role: role.to_string(),
            })
    }

    fn grant_role(&self, store: &Pubkey, user: &Pubkey, role: &str) -> RpcBuilder<C> {
        let authority = self.payer();
        self.store_rpc()
            .accounts(accounts::GrantRole {
                authority,
                store: *store,
            })
            .args(instruction::GrantRole {
                user: *user,
                role: role.to_string(),
            })
    }

    fn revoke_role(&self, store: &Pubkey, user: &Pubkey, role: &str) -> RpcBuilder<C> {
        self.store_rpc()
            .args(instruction::RevokeRole {
                user: *user,
                role: role.to_string(),
            })
            .accounts(accounts::RevokeRole {
                authority: self.payer(),
                store: *store,
            })
    }
}
