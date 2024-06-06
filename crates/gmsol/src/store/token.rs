use std::ops::Deref;

use anchor_client::{
    anchor_lang::{system_program::System, Id},
    solana_sdk::{pubkey::Pubkey, signer::Signer},
    Program,
};
use anchor_spl::token::Token;
use data_store::{accounts, constants, instruction};

use crate::utils::RpcBuilder;

use super::{config::find_config_pda, roles::find_roles_address};

/// Find PDA for claimable account.
pub fn find_claimable_account_pda(
    store: &Pubkey,
    mint: &Pubkey,
    user: &Pubkey,
    time_key: &[u8],
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            constants::CLAIMABLE_ACCOUNT_SEED,
            store.as_ref(),
            mint.as_ref(),
            user.as_ref(),
            time_key,
        ],
        &data_store::id(),
    )
}

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

impl<C, S> TokenAccountOps<C> for Program<C>
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
        RpcBuilder::new(self)
            .args(instruction::UseClaimableAccount { timestamp, amount })
            .accounts(accounts::UseClaimableAccount {
                authority,
                only_controller: find_roles_address(store, &authority).0,
                store: *store,
                config: find_config_pda(store).0,
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
        RpcBuilder::new(self)
            .args(instruction::CloseEmptyClaimableAccount {
                user: *user,
                timestamp,
            })
            .accounts(accounts::CloseEmptyClaimableAccount {
                authority,
                only_controller: find_roles_address(store, &authority).0,
                store: *store,
                config: find_config_pda(store).0,
                mint: *mint,
                account: *account,
                system_program: System::id(),
                token_program: Token::id(),
            })
    }
}
