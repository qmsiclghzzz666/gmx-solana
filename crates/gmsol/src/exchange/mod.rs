/// Deposit.
pub mod deposit;

/// Withdrawal.
pub mod withdrawal;

/// Order.
pub mod order;

use std::{future::Future, ops::Deref};

use anchor_client::{
    anchor_lang::system_program,
    solana_sdk::{pubkey::Pubkey, signer::Signer},
};
use gmsol_exchange::{accounts, instruction};
use gmsol_store::states::{
    order::{OrderKind, OrderParams},
    NonceBytes,
};
use order::CancelOrderBuilder;
use rand::{distributions::Standard, Rng};

use crate::utils::RpcBuilder;

use self::{
    deposit::{CancelDepositBuilder, CreateDepositBuilder, ExecuteDepositBuilder},
    order::{CreateOrderBuilder, ExecuteOrderBuilder},
    withdrawal::{CancelWithdrawalBuilder, CreateWithdrawalBuilder, ExecuteWithdrawalBuilder},
};

/// Exchange instructions for GMSOL.
pub trait ExchangeOps<C> {
    /// Initialize Controller Account.
    fn initialize_controller(&self, store: &Pubkey) -> RpcBuilder<C>;

    /// Create a new market and return its token mint address.
    #[allow(clippy::too_many_arguments)]
    fn create_market(
        &self,
        store: &Pubkey,
        name: &str,
        index_token: &Pubkey,
        long_token: &Pubkey,
        short_token: &Pubkey,
        enable: bool,
        token_map: Option<&Pubkey>,
    ) -> impl Future<Output = crate::Result<(RpcBuilder<C>, Pubkey)>>;

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
        cancel_on_execution_error: bool,
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
        cancel_on_execution_error: bool,
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
        cancel_on_execution_error: bool,
    ) -> crate::Result<ExecuteOrderBuilder<C>>;

    /// Cancel an order.
    fn cancel_order(&self, order: &Pubkey) -> crate::Result<CancelOrderBuilder<C>>;

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

    /// Create a market swap order.
    fn market_swap<'a, S>(
        &self,
        store: &Pubkey,
        market_token: &Pubkey,
        is_output_token_long: bool,
        initial_swap_in_token: &Pubkey,
        initial_swap_in_token_amount: u64,
        swap_path: impl IntoIterator<Item = &'a Pubkey>,
    ) -> CreateOrderBuilder<C>
    where
        C: Deref<Target = S> + Clone,
        S: Signer,
    {
        let params = OrderParams {
            kind: OrderKind::MarketSwap,
            min_output_amount: 0,
            size_delta_usd: 0,
            initial_collateral_delta_amount: initial_swap_in_token_amount,
            acceptable_price: None,
            is_long: true,
        };
        let mut builder = self.create_order(store, market_token, is_output_token_long, params);
        builder
            .initial_collateral_token(initial_swap_in_token, None)
            .swap_path(swap_path.into_iter().copied().collect());
        builder
    }
}

impl<S, C> ExchangeOps<C> for crate::Client<C>
where
    C: Deref<Target = S> + Clone,
    S: Signer,
{
    fn initialize_controller(&self, store: &Pubkey) -> RpcBuilder<C> {
        self.exchange_rpc()
            .args(instruction::InitializeController {})
            .accounts(accounts::InitializeController {
                payer: self.payer(),
                store: *store,
                controller: self.controller_address(store),
                system_program: system_program::ID,
            })
    }

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
        cancel_on_execution_error: bool,
    ) -> ExecuteDepositBuilder<C> {
        ExecuteDepositBuilder::new(self, store, oracle, deposit, cancel_on_execution_error)
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
        cancel_on_execution_error: bool,
    ) -> ExecuteWithdrawalBuilder<C> {
        ExecuteWithdrawalBuilder::new(self, store, oracle, withdrawal, cancel_on_execution_error)
    }

    async fn create_market(
        &self,
        store: &Pubkey,
        name: &str,
        index_token: &Pubkey,
        long_token: &Pubkey,
        short_token: &Pubkey,
        enable: bool,
        token_map: Option<&Pubkey>,
    ) -> crate::Result<(RpcBuilder<C>, Pubkey)> {
        let token_map = match token_map {
            Some(token_map) => *token_map,
            None => crate::store::utils::token_map(self.data_store(), store).await?,
        };
        let authority = self.payer();
        let market_token =
            self.find_market_token_address(store, index_token, long_token, short_token);
        let builder = self
            .exchange_rpc()
            .accounts(accounts::CreateMarket {
                authority,
                data_store: *store,
                token_map,
                market: self.find_market_address(store, &market_token),
                market_token_mint: market_token,
                long_token_mint: *long_token,
                short_token_mint: *short_token,
                market_token_vault: self.find_market_vault_address(store, &market_token),
                long_token_vault: self.find_market_vault_address(store, long_token),
                short_token_vault: self.find_market_vault_address(store, short_token),
                data_store_program: self.data_store_program_id(),
                system_program: system_program::ID,
                token_program: anchor_spl::token::ID,
            })
            .args(instruction::CreateMarket {
                name: name.to_string(),
                index_token_mint: *index_token,
                enable,
            });
        Ok((builder, market_token))
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
        cancel_on_execution_error: bool,
    ) -> crate::Result<ExecuteOrderBuilder<C>> {
        ExecuteOrderBuilder::try_new(self, store, oracle, order, cancel_on_execution_error)
    }

    fn cancel_order(&self, order: &Pubkey) -> crate::Result<CancelOrderBuilder<C>> {
        Ok(CancelOrderBuilder::new(self, order))
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
