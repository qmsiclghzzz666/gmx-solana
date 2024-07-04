use std::ops::Deref;

use anchor_client::{
    anchor_lang::{system_program::System, Id},
    solana_sdk::{pubkey::Pubkey, signer::Signer},
};
use anchor_spl::token::Token;
use gmsol_store::{accounts, instruction};

use crate::utils::RpcBuilder;
/// Token accounts management for GMSOL.
pub trait TokenAccountOps<C> {
    /// Prepare a claimable account.
    fn use_claimable_account(
        &self,
        store: &Pubkey,
        mint: &Pubkey,
        user: &Pubkey,
        timestamp: i64,
        account: &Pubkey,
        amount: u64,
    ) -> RpcBuilder<C>;

    /// Close a claimable account if it is emtpy.
    fn close_empty_claimable_account(
        &self,
        store: &Pubkey,
        mint: &Pubkey,
        user: &Pubkey,
        timestamp: i64,
        account: &Pubkey,
    ) -> RpcBuilder<C>;
}

impl<C, S> TokenAccountOps<C> for crate::Client<C>
where
    C: Deref<Target = S> + Clone,
    S: Signer,
{
    fn use_claimable_account(
        &self,
        store: &Pubkey,
        mint: &Pubkey,
        user: &Pubkey,
        timestamp: i64,
        account: &Pubkey,
        amount: u64,
    ) -> RpcBuilder<C> {
        let authority = self.payer();
        self.data_store_rpc()
            .args(instruction::UseClaimableAccount { timestamp, amount })
            .accounts(accounts::UseClaimableAccount {
                authority,
                store: *store,
                mint: *mint,
                user: *user,
                account: *account,
                system_program: System::id(),
                token_program: Token::id(),
            })
    }

    fn close_empty_claimable_account(
        &self,
        store: &Pubkey,
        mint: &Pubkey,
        user: &Pubkey,
        timestamp: i64,
        account: &Pubkey,
    ) -> RpcBuilder<C> {
        let authority = self.payer();
        self.data_store_rpc()
            .args(instruction::CloseEmptyClaimableAccount {
                user: *user,
                timestamp,
            })
            .accounts(accounts::CloseEmptyClaimableAccount {
                authority,
                store: *store,
                mint: *mint,
                account: *account,
                system_program: System::id(),
                token_program: Token::id(),
            })
    }
}
