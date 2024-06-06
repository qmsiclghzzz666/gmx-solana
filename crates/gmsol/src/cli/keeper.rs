use std::ops::Deref;

use anchor_client::{
    solana_sdk::{pubkey::Pubkey, signer::Signer},
    Program,
};
use data_store::states::PriceProviderKind;
use exchange::events::{DepositCreatedEvent, OrderCreatedEvent, WithdrawalCreatedEvent};
use gmsol::{
    exchange::ExchangeOps,
    pyth::{EncodingType, Hermes, PythPullOracle, PythPullOracleContext, PythPullOracleOps},
    store::market::VaultOps,
    utils::{ComputeBudget, RpcBuilder},
};

use crate::{utils::Oracle, GMSOLClient};

#[derive(clap::Args, Clone)]
pub(super) struct KeeperArgs {
    /// Set the compute unit limit.
    #[arg(long, short = 'u')]
    compute_unit_limit: Option<u32>,
    /// Set the compute unit price in micro lamports.
    #[arg(long, short = 'p', default_value_t = 50_000)]
    compute_unit_price: u64,
    /// The oracle to use.
    #[command(flatten)]
    oracle: Oracle,
    /// Set the execution fee to the given instead of estimating one.
    #[arg(long)]
    execution_fee: Option<u64>,
    /// Set the price provider to use.
    #[arg(long, default_value = "pyth")]
    provider: PriceProviderKind,
    /// Whether to use pull oracle when available.
    #[arg(long)]
    pull_oracle: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand, Clone, Debug)]
enum Command {
    /// Watch for items creation and execute them.
    Watch,
    /// Execute Deposit.
    ExecuteDeposit { deposit: Pubkey },
    /// Execute Withdrawal.
    ExecuteWithdrawal { withdrawal: Pubkey },
    /// Execute Order.
    ExecuteOrder { order: Pubkey },
    /// Initialize Market Vault.
    InitializeVault { token: Pubkey },
    /// Create Market.
    CreateMarket {
        #[arg(long)]
        index_token: Pubkey,
        #[arg(long)]
        long_token: Pubkey,
        #[arg(long)]
        short_token: Pubkey,
    },
}

impl KeeperArgs {
    fn get_compute_budget(&self) -> Option<ComputeBudget> {
        let units = self.compute_unit_limit?;
        Some(
            ComputeBudget::default()
                .with_limit(units)
                .with_price(self.compute_unit_price),
        )
    }

    async fn get_or_estimate_execution_fee<S, C>(
        &self,
        program: &Program<C>,
        mut rpc: RpcBuilder<'_, C>,
    ) -> gmsol::Result<u64>
    where
        C: Deref<Target = S> + Clone,
        S: Signer,
    {
        if let Some(fee) = self.execution_fee {
            return Ok(fee);
        }
        if let Some(budget) = self.get_compute_budget() {
            rpc = rpc.compute_budget(budget);
        } else {
            rpc.compute_budget_mut().set_price(self.compute_unit_price);
        }
        rpc.estimate_execution_fee(&program.async_rpc()).await
    }

    fn use_pyth_pull_oracle(&self) -> bool {
        self.pull_oracle && matches!(self.provider, PriceProviderKind::Pyth)
    }

    pub(super) async fn run(
        &self,
        client: &GMSOLClient,
        store: &Pubkey,
        serialize_only: bool,
    ) -> gmsol::Result<()> {
        match &self.command {
            Command::Watch => {
                let task = Box::pin(self.start_watching(client, store));
                task.await?;
            }
            Command::ExecuteDeposit { deposit } => {
                let mut builder = client.execute_deposit(
                    store,
                    &self
                        .oracle
                        .address(Some(store), &client.data_store_program_id())?,
                    deposit,
                );
                let execution_fee = self
                    .get_or_estimate_execution_fee(client.data_store(), builder.build().await?)
                    .await?;
                builder
                    .execution_fee(execution_fee)
                    .price_provider(self.provider.program());
                if self.use_pyth_pull_oracle() {
                    let hint = builder.prepare_hint().await?;
                    let ctx = PythPullOracleContext::try_from_feeds(&hint.feeds)?;
                    let feed_ids = ctx.feed_ids();
                    if feed_ids.is_empty() {
                        tracing::error!(%deposit, "empty feed ids");
                    }
                    let oracle = PythPullOracle::try_new(client.anchor())?;
                    let hermes = Hermes::default();
                    let update = hermes
                        .latest_price_updates(feed_ids, Some(EncodingType::Base64))
                        .await?;
                    let with_prices = oracle
                        .with_pyth_prices(&ctx, &update, |prices| async {
                            let rpc = builder
                                .parse_with_pyth_price_updates(prices)
                                .build()
                                .await?;
                            Ok(Some(rpc))
                        })
                        .await?;
                    match with_prices.send_all(Some(self.compute_unit_price)).await {
                        Ok(signatures) => {
                            tracing::info!(%deposit, "executed deposit with txs {signatures:#?}");
                        }
                        Err((signatures, err)) => {
                            tracing::error!(%err, %deposit, "failed to execute deposit, successful txs: {signatures:#?}");
                        }
                    }
                } else {
                    let mut rpc = builder.build().await?;
                    if let Some(budget) = self.get_compute_budget() {
                        rpc = rpc.compute_budget(budget)
                    }
                    let signature = rpc.build().send().await?;
                    tracing::info!(%deposit, "executed deposit at tx {signature}");
                    println!("{signature}");
                }
            }
            Command::ExecuteWithdrawal { withdrawal } => {
                let mut builder = client.execute_withdrawal(
                    store,
                    &self
                        .oracle
                        .address(Some(store), &client.data_store_program_id())?,
                    withdrawal,
                );
                let execution_fee = self
                    .get_or_estimate_execution_fee(client.data_store(), builder.build().await?)
                    .await?;
                builder
                    .execution_fee(execution_fee)
                    .price_provider(self.provider.program());
                if self.use_pyth_pull_oracle() {
                    let hint = builder.prepare_hint().await?;
                    let ctx = PythPullOracleContext::try_from_feeds(&hint.feeds)?;
                    let feed_ids = ctx.feed_ids();
                    if feed_ids.is_empty() {
                        tracing::error!(%withdrawal, "empty feed ids");
                    }
                    let oracle = PythPullOracle::try_new(client.anchor())?;
                    let hermes = Hermes::default();
                    let update = hermes
                        .latest_price_updates(feed_ids, Some(EncodingType::Base64))
                        .await?;
                    let with_prices = oracle
                        .with_pyth_prices(&ctx, &update, |prices| async {
                            let rpc = builder
                                .parse_with_pyth_price_updates(prices)
                                .build()
                                .await?;
                            Ok(Some(rpc))
                        })
                        .await?;
                    match with_prices.send_all(Some(self.compute_unit_price)).await {
                        Ok(signatures) => {
                            tracing::info!(%withdrawal, "executed withdrawal with txs {signatures:#?}");
                        }
                        Err((signatures, err)) => {
                            tracing::error!(%err, %withdrawal, "failed to execute withdrawal, successful txs: {signatures:#?}");
                        }
                    }
                } else {
                    let mut rpc = builder.build().await?;
                    if let Some(budget) = self.get_compute_budget() {
                        rpc = rpc.compute_budget(budget)
                    }
                    let signature = rpc.build().send().await?;
                    tracing::info!(%withdrawal, "executed withdrawal at tx {signature}");
                    println!("{signature}");
                }
            }
            Command::ExecuteOrder { order } => {
                let mut builder = client.execute_order(
                    store,
                    &self
                        .oracle
                        .address(Some(store), &client.data_store_program_id())?,
                    order,
                )?;
                let execution_fee = self
                    .get_or_estimate_execution_fee(client.data_store(), builder.build().await?)
                    .await?;
                builder
                    .execution_fee(execution_fee)
                    .price_provider(self.provider.program());
                if self.use_pyth_pull_oracle() {
                    let hint = builder.prepare_hint().await?;
                    let ctx = PythPullOracleContext::try_from_feeds(&hint.feeds)?;
                    let feed_ids = ctx.feed_ids();
                    if feed_ids.is_empty() {
                        tracing::error!(%order, "empty feed ids");
                    }
                    let oracle = PythPullOracle::try_new(client.anchor())?;
                    let hermes = Hermes::default();
                    let update = hermes
                        .latest_price_updates(feed_ids, Some(EncodingType::Base64))
                        .await?;
                    let with_prices = oracle
                        .with_pyth_prices(&ctx, &update, |prices| async {
                            let rpc = builder
                                .parse_with_pyth_price_updates(prices)
                                .build()
                                .await?;
                            Ok(Some(rpc))
                        })
                        .await?;
                    match with_prices.send_all(Some(self.compute_unit_price)).await {
                        Ok(signatures) => {
                            tracing::info!(%order, "executed order with txs {signatures:#?}");
                        }
                        Err((signatures, err)) => {
                            tracing::error!(%err, %order, "failed to execute order, successful txs: {signatures:#?}");
                        }
                    }
                } else {
                    let mut rpc = builder.build().await?;
                    if let Some(budget) = self.get_compute_budget() {
                        rpc = rpc.compute_budget(budget)
                    }
                    let signature = rpc.build().send().await?;
                    tracing::info!(%order, "executed order at tx {signature}");
                    println!("{signature}");
                }
            }
            Command::InitializeVault { token } => {
                let (request, vault) = client.initialize_market_vault(store, token);
                crate::utils::send_or_serialize(request, serialize_only, |signature| {
                    println!("created a new vault {vault} at tx {signature}");
                    Ok(())
                })
                .await?;
            }
            Command::CreateMarket {
                index_token,
                long_token,
                short_token,
            } => {
                let (request, market_token) =
                    client.create_market(store, index_token, long_token, short_token);
                crate::utils::send_or_serialize(request, serialize_only, |signature| {
                    println!(
                        "created a new market with {market_token} as its token address at tx {signature}"
                    );
                    Ok(())
                }).await?;
            }
        }
        Ok(())
    }

    fn with_command(&self, command: Command) -> Self {
        let mut args = self.clone();
        args.command = command;
        args
    }

    async fn start_watching(&self, client: &GMSOLClient, store: &Pubkey) -> gmsol::Result<()> {
        use tokio::sync::mpsc;

        let store = *store;
        let (tx, mut rx) = mpsc::unbounded_channel();
        let mut unsubscribers = vec![];

        // Subscribe deposit creation event.
        let deposit_program = client.new_exchange()?;
        let unsubscriber =
            deposit_program
            .on::<DepositCreatedEvent>({
                let tx = tx.clone();
                move |ctx, event| {
                if event.store == store {
                    tracing::info!(slot=%ctx.slot, ?event, "received a new deposit creation event");
                    tx.send(Command::ExecuteDeposit {
                        deposit: event.deposit,
                    })
                    .unwrap();
                } else {
                    tracing::debug!(slot=%ctx.slot, ?event, "received deposit creation event from other store");
                }
            }})
            .await?;
        unsubscribers.push(unsubscriber);

        tracing::info!("deposit creation subscribed");

        // Subscribe withdrawal creation event.
        let withdrawal_program = client.new_exchange()?;
        let unsubscriber = withdrawal_program
            .on::<WithdrawalCreatedEvent>({
                let tx = tx.clone();
                move |ctx, event| {
                if event.store == store {
                    tracing::info!(slot=%ctx.slot, ?event, "received a new withdrawal creation event");
                    tx.send(Command::ExecuteWithdrawal {
                        withdrawal: event.withdrawal,
                    })
                    .unwrap();
                } else {
                    tracing::debug!(slot=%ctx.slot, ?event, "received withdrawal creation event from other store");
                }
            }})
            .await?;
        unsubscribers.push(unsubscriber);

        tracing::info!("withdrawal creation subscribed");

        // Subscribe order creation event.
        let order_program = client.new_exchange()?;
        let unsubscriber = order_program
            .on::<OrderCreatedEvent>({
                let tx = tx.clone();
                move |ctx, event| {
                if event.store == store {
                    tracing::info!(slot=%ctx.slot, ?event, "received a new order creation event");
                    tx.send(Command::ExecuteOrder {
                        order: event.order,
                    })
                    .unwrap();
                } else {
                    tracing::debug!(slot=%ctx.slot, ?event, "received order creation event from other store");
                }
            }})
            .await?;
        unsubscribers.push(unsubscriber);

        tracing::info!("order creation subscribed");

        let worker = async move {
            while let Some(command) = rx.recv().await {
                tracing::info!(?command, "received new command");
                match self.with_command(command).run(client, &store, false).await {
                    Ok(()) => {
                        tracing::info!("command executed");
                    }
                    Err(err) => {
                        tracing::error!(%err, "failed to execute, ignore");
                    }
                }
            }
            gmsol::Result::Ok(())
        };
        tokio::select! {
            res = tokio::signal::ctrl_c() => {
                match res {
                    Ok(()) => {
                        tracing::info!("Received `ctrl + c`, stopping...");
                    },
                    Err(err) => {
                        tracing::error!(%err, "Failed to setup signal handler");
                    }
                }

            },
            res = worker => {
                res?;
            }
        }
        for unsubscriber in unsubscribers {
            unsubscriber.unsubscribe().await;
        }
        Ok(())
    }
}
