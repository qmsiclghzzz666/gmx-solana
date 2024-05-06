use anchor_client::solana_sdk::pubkey::Pubkey;
use data_store::states::{PriceProviderKind, TokenConfigBuilder};
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
        #[command(flatten)]
        feeds: Feeds,
        #[arg(long)]
        expected_provider: PriceProviderKind,
        #[arg(long, default_value_t = 60)]
        heartbeat_duration: u32,
        #[arg(long, default_value_t = 4)]
        precision: u8,
        /// Provide to create a synthetic token with the given decimals.
        #[arg(long)]
        synthetic: Option<u8>,
    },
    /// Toggle token config of token.
    ToggleTokenConfig {
        token: Pubkey,
        #[command(flatten)]
        toggle: Toggle,
    },
}

#[derive(clap::Args)]
#[group(required = true, multiple = true)]
struct Feeds {
    /// Pyth feed id.
    #[arg(long)]
    pyth_feed_id: Option<String>,
    /// Pyth feed account (Devnet)
    #[arg(long)]
    pyth_feed_devnet: Option<Pubkey>,
    /// Chainlink feed.
    #[arg(long)]
    chainlink_feed: Option<Pubkey>,
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
                feeds,
                expected_provider,
                heartbeat_duration,
                precision,
                synthetic: fake_decimals,
            } => {
                let mut builder = TokenConfigBuilder::default()
                    .with_heartbeat_duration(*heartbeat_duration)
                    .with_precision(*precision)
                    .with_expected_provider(*expected_provider);
                if let Some(feed_id) = feeds.pyth_feed_id.as_ref() {
                    let feed_id =
                        pyth_sdk::Identifier::from_hex(feed_id).map_err(gmsol::Error::unknown)?;
                    let feed_id_as_key = Pubkey::new_from_array(feed_id.to_bytes());
                    builder = builder
                        .update_price_feed(&PriceProviderKind::Pyth, feed_id_as_key)
                        .map_err(anchor_client::ClientError::from)?;
                }
                if let Some(feed) = feeds.pyth_feed_devnet {
                    builder = builder
                        .update_price_feed(&PriceProviderKind::PythDev, feed)
                        .map_err(anchor_client::ClientError::from)?;
                }
                if let Some(feed) = feeds.chainlink_feed {
                    builder = builder
                        .update_price_feed(&PriceProviderKind::Chainlink, feed)
                        .map_err(anchor_client::ClientError::from)?;
                }
                let signature = if let Some(decimals) = fake_decimals {
                    program
                        .insert_synthetic_token_config(store, token, *decimals, builder, true)
                        .send()
                        .await?
                } else {
                    program
                        .insert_token_config(store, token, builder, true)
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
