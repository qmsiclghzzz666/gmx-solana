/// Deposit.
pub mod deposit;

/// Withdrawal.
pub mod withdrawal;

use std::ops::Deref;

use anchor_client::{
    solana_sdk::{pubkey::Pubkey, signer::Signer},
    Program,
};
use data_store::states::{DataStore, NonceBytes, Seed};
use gmx_solana_utils::to_seed;
use rand::{distributions::Standard, Rng};

use self::{
    deposit::{CancelDepositBuilder, CreateDepositBuilder, ExecuteDepositBuilder},
    withdrawal::{CancelWithdrawalBuilder, CreateWithdrawalBuilder},
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
}

fn generate_nonce() -> NonceBytes {
    rand::thread_rng()
        .sample_iter(Standard)
        .take(32)
        .collect::<Vec<u8>>()
        .try_into()
        .unwrap()
}
