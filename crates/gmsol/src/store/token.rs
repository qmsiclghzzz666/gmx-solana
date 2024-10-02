use std::ops::Deref;

use anchor_client::{
    anchor_lang::{
        system_program::{self, System},
        Id,
    },
    solana_sdk::{pubkey::Pubkey, signer::Signer},
};
use anchor_spl::{associated_token::get_associated_token_address_with_program_id, token::Token};
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

    /// Prepare associated token account.
    fn prepare_associated_token_account(
        &self,
        mint: &Pubkey,
        token_program_id: &Pubkey,
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
        owner: &Pubkey,
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
                owner: *owner,
                account: *account,
                system_program: System::id(),
                token_program: Token::id(),
            })
    }

    fn close_empty_claimable_account(
        &self,
        store: &Pubkey,
        mint: &Pubkey,
        owner: &Pubkey,
        timestamp: i64,
        account: &Pubkey,
    ) -> RpcBuilder<C> {
        let authority = self.payer();
        self.data_store_rpc()
            .args(instruction::CloseEmptyClaimableAccount { timestamp })
            .accounts(accounts::CloseEmptyClaimableAccount {
                authority,
                store: *store,
                mint: *mint,
                owner: *owner,
                account: *account,
                system_program: System::id(),
                token_program: Token::id(),
            })
    }

    fn prepare_associated_token_account(
        &self,
        mint: &Pubkey,
        token_program_id: &Pubkey,
    ) -> RpcBuilder<C> {
        let account =
            get_associated_token_address_with_program_id(&self.payer(), mint, token_program_id);
        self.data_store_rpc()
            .accounts(accounts::PrepareAssociatedTokenAccount {
                payer: self.payer(),
                owner: self.payer(),
                mint: *mint,
                account,
                system_program: system_program::ID,
                token_program: *token_program_id,
                associated_token_program: anchor_spl::associated_token::ID,
            })
            .args(instruction::PrepareAssociatedTokenAccount {})
    }
}
