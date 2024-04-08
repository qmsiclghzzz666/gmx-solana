use anchor_client::solana_sdk::pubkey::Pubkey;
use gmsol::store::oracle::OracleOps;

use crate::SharedClient;

#[derive(clap::Args)]
pub(super) struct ControllerArgs {
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    /// Initialize a [`Oracle`](data_store::states::Oracle) account.
    InitializeOracle { index: u8 },
}

impl ControllerArgs {
    pub(super) async fn run(&self, client: &SharedClient, store: &Pubkey) -> gmsol::Result<()> {
        let program = client.program(data_store::id())?;
        match &self.command {
            Command::InitializeOracle { index } => {
                let (request, oracle) = program.initialize_oracle(store, *index);
                let signature = request.send().await?;
                println!("created oracle {oracle} at tx {signature}");
            }
        }
        Ok(())
    }
}
