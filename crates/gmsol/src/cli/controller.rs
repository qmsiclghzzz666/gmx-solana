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
        /// Provide to create a fake token with the given decimals.
        #[arg(long)]
        fake_decimals: Option<u8>,
    },
    /// Toggle token config of token.
    ToggleTokenConfig {
        token: Pubkey,
        #[command(flatten)]
        toggle: Toggle,
    },
}

#[derive(clap::Args)]
#[group(required = true, multiple = false)]
struct Toggle {
    #[arg(long)]
    enable: bool,
    #[arg(long)]
    disable: bool,
}

impl Toggle {
    fn is_enable(&self) -> bool {
        debug_assert!(self.enable != self.disable);
        self.enable
    }
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
                fake_decimals,
            } => {
                let signature = if let Some(decimals) = fake_decimals {
                    program
                        .insert_fake_token_config(
                            store,
                            token,
                            *decimals,
                            price_feed,
                            *heartbeat_duration,
                            *precision,
                        )
                        .send()
                        .await?
                } else {
                    program
                        .insert_token_config(
                            store,
                            token,
                            price_feed,
                            *heartbeat_duration,
                            *precision,
                        )
                        .send()
                        .await?
                };
                println!("{signature}");
            }
            Command::ToggleTokenConfig { token, toggle } => {
                let signature = program
                    .toggle_token_config(store, token, toggle.is_enable())
                    .send()
                    .await?;
                println!("{signature}");
            }
        }
        Ok(())
    }
}
