use std::ops::Deref;

use anchor_client::{
    solana_sdk::{
        compute_budget::ComputeBudgetInstruction, message::Message, pubkey::Pubkey, signer::Signer,
    },
    Program, RequestBuilder,
};
use exchange::events::DepositCreatedEvent;
use gmsol::{
    exchange::ExchangeOps,
    store::{market::VaultOps, oracle::find_oracle_address},
};

use crate::SharedClient;

#[derive(clap::Args, Clone)]
pub(super) struct KeeperArgs {
    /// Set the compute unit limit.
    #[arg(long, short = 'u')]
    compute_unit_limit: Option<u32>,
    /// Set the compute unit price in micro lamports.
    #[arg(long, short = 'p', default_value_t = 1)]
    compute_unit_price: u64,
    /// The oracle to use.
    #[command(flatten)]
    oracle: Oracle,
    /// Set the execution fee to the given instead of estimating one.
    #[arg(long)]
    execution_fee: Option<u64>,
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

#[derive(clap::Args, Clone)]
#[group(required = false, multiple = false)]
struct Oracle {
    #[arg(long, env)]
    oracle: Option<Pubkey>,
    #[arg(long, default_value_t = 0)]
    oracle_index: u8,
}

impl Oracle {
    fn address(&self, store: &Pubkey) -> Pubkey {
        match self.oracle {
            Some(address) => address,
            None => find_oracle_address(store, self.oracle_index).0,
        }
    }
}

impl KeeperArgs {
    fn insert_compute_budget_instructions<'a, S, C>(
        &self,
        builder: RequestBuilder<'a, C>,
    ) -> RequestBuilder<'a, C>
    where
        C: Deref<Target = S> + Clone,
        S: Signer,
    {
        if let Some(units) = self.compute_unit_limit {
            builder
                .instruction(ComputeBudgetInstruction::set_compute_unit_limit(units))
                .instruction(ComputeBudgetInstruction::set_compute_unit_price(
                    self.compute_unit_price,
                ))
        } else {
            builder
        }
    }

    async fn get_or_estimate_execution_fee<S, C>(
        &self,
        program: &Program<C>,
        builder: RequestBuilder<'_, C>,
    ) -> gmsol::Result<u64>
    where
        C: Deref<Target = S> + Clone,
        S: Signer,
    {
        if let Some(fee) = self.execution_fee {
            return Ok(fee);
        }
        let client = program.async_rpc();
        let ixs = self
            .insert_compute_budget_instructions(builder)
            .instructions()?;
        let blockhash = client
            .get_latest_blockhash()
            .await
            .map_err(anchor_client::ClientError::from)?;
        let message = Message::new_with_blockhash(&ixs, None, &blockhash);
        let fee = client
            .get_fee_for_message(&message)
            .await
            .map_err(anchor_client::ClientError::from)?;
        Ok(fee)
    }

    pub(super) async fn run(&self, client: &SharedClient, store: &Pubkey) -> gmsol::Result<()> {
        match &self.command {
            Command::Watch => {
                let task = Box::pin(self.start_watching(client, store));
                task.await?;
            }
            Command::ExecuteDeposit { deposit } => {
                let program = client.program(exchange::id())?;
                let mut builder =
                    program.execute_deposit(store, &self.oracle.address(store), deposit);
                let execution_fee = self
                    .get_or_estimate_execution_fee(&program, builder.build().await?)
                    .await?;
                let signature = self
                    .insert_compute_budget_instructions(
                        builder.execution_fee(execution_fee).build().await?,
                    )
                    .send()
                    .await?;
                tracing::info!(%deposit, "executed deposit at tx {signature}");
                println!("{signature}");
            }
            Command::ExecuteWithdrawal { withdrawal } => {
                let program = client.program(exchange::id())?;
                let mut builder =
                    program.execute_withdrawal(store, &self.oracle.address(store), withdrawal);
                let execution_fee = self
                    .get_or_estimate_execution_fee(&program, builder.build().await?)
                    .await?;
                let signature = self
                    .insert_compute_budget_instructions(
                        builder.execution_fee(execution_fee).build().await?,
                    )
                    .send()
                    .await?;
                tracing::info!(%withdrawal, "executed withdrawal at tx {signature}");
                println!("{signature}");
            }
            Command::ExecuteOrder { order } => {
                let program = client.program(exchange::id())?;
                let mut builder = program.execute_order(store, &self.oracle.address(store), order);
                let execution_fee = self
                    .get_or_estimate_execution_fee(&program, builder.build().await?)
                    .await?;
                let signature = self
                    .insert_compute_budget_instructions(
                        builder.execution_fee(execution_fee).build().await?,
                    )
                    .send()
                    .await?;
                tracing::info!(%order, "executed order at tx {signature}");
                println!("{signature}");
            }
            Command::InitializeVault { token } => {
                let program = client.program(data_store::id())?;
                let (request, vault) = program.initialize_market_vault(store, token);
                let signature = request.send().await?;
                println!("created a new vault {vault} at tx {signature}");
            }
            Command::CreateMarket {
                index_token,
                long_token,
                short_token,
            } => {
                let program = client.program(exchange::id())?;
                let (request, market_token) =
                    program.create_market(store, index_token, long_token, short_token);
                let signature = request.send().await?;
                println!(
                    "created a new market with {market_token} as its token address at tx {signature}"
                );
            }
        }
        Ok(())
    }

    fn with_command(&self, command: Command) -> Self {
        let mut args = self.clone();
        args.command = command;
        args
    }

    async fn start_watching(&self, client: &SharedClient, store: &Pubkey) -> gmsol::Result<()> {
        use tokio::sync::mpsc;

        let program = client.program(exchange::id())?;
        let store = *store;
        let (tx, mut rx) = mpsc::unbounded_channel();
        let unsubscriber = program
            .on::<DepositCreatedEvent>(move |ctx, event| {
                if event.store == store {
                    tracing::info!(slot=%ctx.slot, ?event, "received a new deposit event");
                    tx.send(Command::ExecuteDeposit {
                        deposit: event.deposit,
                    })
                    .unwrap();
                } else {
                    tracing::debug!(slot=%ctx.slot, ?event, "received deposit event from other store");
                }
            })
            .await?;
        let worker = async move {
            while let Some(command) = rx.recv().await {
                match self.with_command(command).run(client, &store).await {
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
        unsubscriber.unsubscribe().await;
        Ok(())
    }
}
