/// Deposit.
pub mod deposit;

/// Withdrawal.
pub mod withdrawal;

use std::ops::Deref;

use anchor_client::{
    anchor_lang::system_program,
    solana_sdk::{pubkey::Pubkey, signer::Signer},
    Program, RequestBuilder,
};
use data_store::states::{DataStore, NonceBytes, Seed};
use exchange::{accounts, instruction};
use gmx_solana_utils::to_seed;
use rand::{distributions::Standard, Rng};

use crate::store::{
    market::{find_market_address, find_market_token_address, find_market_vault_address},
    roles::find_roles_address,
};

use self::{
    deposit::{CancelDepositBuilder, CreateDepositBuilder, ExecuteDepositBuilder},
    withdrawal::{CancelWithdrawalBuilder, CreateWithdrawalBuilder, ExecuteWithdrawalBuilder},
};

/// Find PDA for `DataStore` account.
pub fn find_store_address(key: &str) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[DataStore::SEED, &to_seed(key)], &data_store::id())
}

/// Exchange instructions for GMSOL.
pub trait ExchangeOps<C> {
    /// Create a deposit.
    fn create_deposit(&self, store: &Pubkey, market_token: &Pubkey) -> CreateDepositBuilder<C>;

    /// Cancel a deposit.
    fn cancel_deposit(&self, store: &Pubkey, deposit: &Pubkey) -> CancelDepositBuilder<C>;

    /// Execute a deposit.
    fn execute_deposit(
        &self,
        store: &Pubkey,
        oracle: &Pubkey,
        deposit: &Pubkey,
    ) -> ExecuteDepositBuilder<C>;

    /// Create a withdrawal.
    fn create_withdrawal(
        &self,
        store: &Pubkey,
        market_token: &Pubkey,
        amount: u64,
    ) -> CreateWithdrawalBuilder<C>;

    /// Cancel a withdrawal.
    fn cancel_withdrawal(&self, store: &Pubkey, withdrawal: &Pubkey) -> CancelWithdrawalBuilder<C>;

    /// Execute a withdrawal.
    fn execute_withdrawal(
        &self,
        store: &Pubkey,
        oracle: &Pubkey,
        withdrawal: &Pubkey,
    ) -> ExecuteWithdrawalBuilder<C>;

    /// Create a new market and return its token mint address.
    fn create_market(
        &self,
        store: &Pubkey,
        index_token: &Pubkey,
        long_token: &Pubkey,
        short_token: &Pubkey,
    ) -> (RequestBuilder<C>, Pubkey);
}

impl<S, C> ExchangeOps<C> for Program<C>
where
    C: Deref<Target = S> + Clone,
    S: Signer,
{
    fn create_deposit(&self, store: &Pubkey, market_token: &Pubkey) -> CreateDepositBuilder<C> {
        CreateDepositBuilder::new(self, *store, *market_token)
    }

    fn cancel_deposit(&self, store: &Pubkey, deposit: &Pubkey) -> CancelDepositBuilder<C> {
        CancelDepositBuilder::new(self, store, deposit)
    }

    fn execute_deposit(
        &self,
        store: &Pubkey,
        oracle: &Pubkey,
        deposit: &Pubkey,
    ) -> ExecuteDepositBuilder<C> {
        ExecuteDepositBuilder::new(self, store, oracle, deposit)
    }

    fn create_withdrawal(
        &self,
        store: &Pubkey,
        market_token: &Pubkey,
        amount: u64,
    ) -> CreateWithdrawalBuilder<C> {
        CreateWithdrawalBuilder::new(self, *store, *market_token, amount)
    }

    fn cancel_withdrawal(&self, store: &Pubkey, withdrawal: &Pubkey) -> CancelWithdrawalBuilder<C> {
        CancelWithdrawalBuilder::new(self, store, withdrawal)
    }

    fn execute_withdrawal(
        &self,
        store: &Pubkey,
        oracle: &Pubkey,
        withdrawal: &Pubkey,
    ) -> ExecuteWithdrawalBuilder<C> {
        ExecuteWithdrawalBuilder::new(self, store, oracle, withdrawal)
    }

    fn create_market(
        &self,
        store: &Pubkey,
        index_token: &Pubkey,
        long_token: &Pubkey,
        short_token: &Pubkey,
    ) -> (RequestBuilder<C>, Pubkey) {
        let authority = self.payer();
        let market_token = find_market_token_address(store, index_token, long_token, short_token).0;
        let builder = self
            .request()
            .accounts(accounts::CreateMarket {
                authority,
                only_market_keeper: find_roles_address(store, &authority).0,
                data_store: *store,
                market: find_market_address(store, &market_token).0,
                market_token_mint: market_token,
                long_token_mint: *long_token,
                short_token_mint: *short_token,
                market_token_vault: find_market_vault_address(store, &market_token).0,
                data_store_program: data_store::id(),
                system_program: system_program::ID,
                token_program: anchor_spl::token::ID,
            })
            .args(instruction::CreateMarket {
                index_token_mint: *index_token,
            });
        (builder, market_token)
    }
}

fn generate_nonce() -> NonceBytes {
    rand::thread_rng()
        .sample_iter(Standard)
        .take(32)
        .collect::<Vec<u8>>()
        .try_into()
        .unwrap()
}
