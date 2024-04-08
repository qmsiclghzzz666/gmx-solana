use std::ops::Deref;

use anchor_client::{
    solana_sdk::{
        compute_budget::ComputeBudgetInstruction, message::Message, pubkey::Pubkey, signer::Signer,
    },
    Program, RequestBuilder,
};
use gmsol::{
    exchange::ExchangeOps,
    store::{oracle::find_oracle_address, vault::VaultOps},
};

use crate::SharedClient;

#[derive(clap::Args)]
pub(super) struct KeeperArgs {
    /// Set the compute unit limit.
    #[arg(long, short = 'u')]
    compute_unit_limit: Option<u32>,
    /// Set the compute unit price in micro lamports.
    #[arg(long, short = 'p', default_value_t = 1)]
    compute_unit_price: u64,
    /// Set the execution fee to the given instead of estimating one.
    #[arg(long)]
    execution_fee: Option<u64>,
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    /// Execute Deposit.
    ExecuteDeposit {
        deposit: Pubkey,
        #[command(flatten)]
        oracle: Oracle,
    },
    /// Execute Withdrawal.
    ExecuteWithdrawal {
        withdrawal: Pubkey,
        #[command(flatten)]
        oracle: Oracle,
    },
    /// Initialize Market Vault.
    InitializeVault { token: Pubkey },
}

#[derive(clap::Args)]
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
            Command::ExecuteDeposit { deposit, oracle } => {
                let program = client.program(exchange::id())?;
                let mut builder = program.execute_deposit(store, &oracle.address(store), deposit);
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
            Command::ExecuteWithdrawal { withdrawal, oracle } => {
                let program = client.program(exchange::id())?;
                let mut builder =
                    program.execute_withdrawal(store, &oracle.address(store), withdrawal);
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
            Command::InitializeVault { token } => {
                let program = client.program(data_store::id())?;
                let (request, vault) = program.initialize_market_vault(store, token);
                let signature = request.send().await?;
                println!("created a new vault {vault} at tx {signature}");
            }
        }
        Ok(())
    }
}
