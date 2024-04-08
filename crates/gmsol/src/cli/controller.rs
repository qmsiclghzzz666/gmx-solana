use anchor_client::solana_sdk::pubkey::Pubkey;
use gmsol::store::{oracle::OracleOps, token_config::TokenConfigOps};

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
    /// Initialize the [`TokenConfigMap`](data_store::states::TokenConfigMap) account.
    InitializeTokenConfigMap,
    /// Insert or update the config of token.
    InsertTokenConfig {
        token: Pubkey,
        #[arg(long)]
        price_feed: Pubkey,
        #[arg(long, default_value_t = 60)]
        heartbeat_duration: u32,
        #[arg(long, default_value_t = 4)]
        precision: u8,
    },
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
            Command::InitializeTokenConfigMap => {
                let (request, map) = program.initialize_token_config_map(store);
                let signature = request.send().await?;
                println!("created token config map {map} at tx {signature}");
            }
            Command::InsertTokenConfig {
                token,
                price_feed,
                heartbeat_duration,
                precision,
            } => {
                let signature = program
                    .insert_token_config(store, token, price_feed, *heartbeat_duration, *precision)
                    .send()
                    .await?;
                println!("{signature}");
            }
        }
        Ok(())
    }
}
