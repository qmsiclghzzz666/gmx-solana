/// Deposit.
pub mod deposit;

/// Withdrawal.
pub mod withdrawal;

/// Order.
pub mod order;

use std::ops::Deref;

use anchor_client::{
    anchor_lang::system_program,
    solana_sdk::{pubkey::Pubkey, signer::Signer},
    RequestBuilder,
};
use data_store::states::{
    order::{OrderKind, OrderParams},
    NonceBytes,
};
use exchange::{accounts, instruction};
use rand::{distributions::Standard, Rng};

use self::{
    deposit::{CancelDepositBuilder, CreateDepositBuilder, ExecuteDepositBuilder},
    order::{CreateOrderBuilder, ExecuteOrderBuilder},
    withdrawal::{CancelWithdrawalBuilder, CreateWithdrawalBuilder, ExecuteWithdrawalBuilder},
};

/// Exchange instructions for GMSOL.
pub trait ExchangeOps<C> {
    /// Create a new market and return its token mint address.
    fn create_market(
        &self,
        store: &Pubkey,
        index_token: &Pubkey,
        long_token: &Pubkey,
        short_token: &Pubkey,
    ) -> (RequestBuilder<C>, Pubkey);

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

    /// Create an order.
    fn create_order(
        &self,
        store: &Pubkey,
        market_token: &Pubkey,
        is_output_token_long: bool,
        params: OrderParams,
    ) -> CreateOrderBuilder<C>;

    /// Execute an order.
    fn execute_order(
        &self,
        store: &Pubkey,
        oracle: &Pubkey,
        order: &Pubkey,
    ) -> crate::Result<ExecuteOrderBuilder<C>>;

    /// Create a market increase position order.
    fn market_increase(
        &self,
        store: &Pubkey,
        market_token: &Pubkey,
        is_collateral_token_long: bool,
        initial_collateral_amount: u64,
        is_long: bool,
        increment_size_in_usd: u128,
    ) -> CreateOrderBuilder<C> {
        let params = OrderParams {
            kind: OrderKind::MarketIncrease,
            min_output_amount: 0,
            size_delta_usd: increment_size_in_usd,
            initial_collateral_delta_amount: initial_collateral_amount,
            acceptable_price: None,
            is_long,
        };
        self.create_order(store, market_token, is_collateral_token_long, params)
    }

    /// Create a market decrease position order.
    fn market_decrease(
        &self,
        store: &Pubkey,
        market_token: &Pubkey,
        is_collateral_token_long: bool,
        collateral_withdrawal_amount: u64,
        is_long: bool,
        decrement_size_in_usd: u128,
    ) -> CreateOrderBuilder<C> {
        let params = OrderParams {
            kind: OrderKind::MarketDecrease,
            min_output_amount: 0,
            size_delta_usd: decrement_size_in_usd,
            initial_collateral_delta_amount: collateral_withdrawal_amount,
            acceptable_price: None,
            is_long,
        };
        self.create_order(store, market_token, is_collateral_token_long, params)
    }

    /// Create a liquidation order.
    fn liquidate(
        &self,
        store: &Pubkey,
        market_token: &Pubkey,
        is_collateral_token_long: bool,
        is_long: bool,
        size_in_usd: Option<u128>,
    ) -> CreateOrderBuilder<C> {
        let params = OrderParams {
            kind: OrderKind::Liquidation,
            min_output_amount: 0,
            size_delta_usd: size_in_usd.unwrap_or(u128::MAX),
            initial_collateral_delta_amount: 0,
            acceptable_price: None,
            is_long,
        };
        self.create_order(store, market_token, is_collateral_token_long, params)
    }
}

impl<S, C> ExchangeOps<C> for crate::Client<C>
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
        let market_token =
            self.find_market_token_address(store, index_token, long_token, short_token);
        let builder = self
            .exchange()
            .request()
            .accounts(accounts::CreateMarket {
                authority,
                data_store: *store,
                market: self.find_market_address(store, &market_token),
                market_token_mint: market_token,
                long_token_mint: *long_token,
                short_token_mint: *short_token,
                market_token_vault: self.find_market_vault_address(store, &market_token),
                data_store_program: self.data_store_program_id(),
                system_program: system_program::ID,
                token_program: anchor_spl::token::ID,
            })
            .args(instruction::CreateMarket {
                index_token_mint: *index_token,
            });
        (builder, market_token)
    }

    fn create_order(
        &self,
        store: &Pubkey,
        market_token: &Pubkey,
        is_output_token_long: bool,
        params: OrderParams,
    ) -> CreateOrderBuilder<C> {
        CreateOrderBuilder::new(self, store, market_token, params, is_output_token_long)
    }

    fn execute_order(
        &self,
        store: &Pubkey,
        oracle: &Pubkey,
        order: &Pubkey,
    ) -> crate::Result<ExecuteOrderBuilder<C>> {
        ExecuteOrderBuilder::try_new(self, store, oracle, order)
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
