use anchor_client::solana_sdk::pubkey::Pubkey;
use gmsol::exchange::ExchangeOps;

use crate::SharedClient;

#[derive(clap::Args)]
pub(super) struct KeeperArgs {
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    /// Execute Deposit.
    ExecuteDeposit {
        deposit: Pubkey,
        #[arg(long)]
        oracle: Pubkey,
    },
}

impl KeeperArgs {
    pub(super) async fn run(&self, client: &SharedClient, store: &Pubkey) -> gmsol::Result<()> {
        match &self.command {
            Command::ExecuteDeposit { deposit, oracle } => {
                let program = client.program(exchange::id())?;
                let builder = program.execute_deposit(store, oracle, deposit);
                let signature = builder.build().await?.send().await?;
                tracing::info!(%deposit, "executed deposit at tx {signature}");
                println!("{deposit}");
            }
        }
        Ok(())
    }
}
