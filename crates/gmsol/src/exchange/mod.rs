/// Deposit.
pub mod deposit;

use std::ops::Deref;

use anchor_client::{
    solana_sdk::{pubkey::Pubkey, signer::Signer},
    Program,
};
use data_store::states::{DataStore, Seed};
use gmx_solana_utils::to_seed;

use self::deposit::{CancelDepositBuilder, CreateDepositBuilder};

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
}
