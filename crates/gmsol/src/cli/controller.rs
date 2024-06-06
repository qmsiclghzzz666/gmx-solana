use anchor_client::solana_sdk::pubkey::Pubkey;
use data_store::states::{Amount, Factor, PriceProviderKind, TokenConfigBuilder};
use gmsol::store::{config::ConfigOps, oracle::OracleOps, token_config::TokenConfigOps};

use crate::GMSOLClient;

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
        #[arg(long, value_name = "DECIMALS")]
        synthetic: Option<u8>,
    },
    /// Toggle token config of token.
    ToggleTokenConfig {
        token: Pubkey,
        #[command(flatten)]
        toggle: Toggle,
    },
    /// Set expected provider of token.
    SetExpectedProvider {
        token: Pubkey,
        provider: PriceProviderKind,
    },
    /// Initialize Config Account.
    InitializeConfig,
    /// Insert an amount to the config.
    InsertAmount {
        amount: Amount,
        #[arg(long, short)]
        key: String,
        /// Force new.
        #[arg(long)]
        new: bool,
    },
    /// Insert a factor to the config.
    InsertFactor {
        factor: Factor,
        #[arg(long, short)]
        key: String,
        /// Force new.
        #[arg(long)]
        new: bool,
    },
    /// Insert an address to the config.
    InsertAddress {
        address: Pubkey,
        #[arg(long, short)]
        key: String,
        /// Force new.
        #[arg(long)]
        new: bool,
    },
}

#[derive(clap::Args)]
#[group(required = true, multiple = true)]
struct Feeds {
    /// Pyth feed id.
    #[arg(long)]
    pyth_feed_id: Option<String>,
    /// Pyth feed account (Legacy)
    #[arg(long)]
    pyth_feed_legacy: Option<Pubkey>,
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
    pub(super) async fn run(
        &self,
        client: &GMSOLClient,
        store: &Pubkey,
        serialize_only: bool,
    ) -> gmsol::Result<()> {
        match &self.command {
            Command::InitializeOracle { index } => {
                let (request, oracle) = client.initialize_oracle(store, *index);
                crate::utils::send_or_serialize(request, serialize_only, |signature| {
                    println!("created oracle {oracle} at tx {signature}");
                    Ok(())
                })
                .await?;
            }
            Command::InitializeTokenConfigMap => {
                let (request, map) = client.initialize_token_config_map(store);
                crate::utils::send_or_serialize(request, serialize_only, |signature| {
                    println!("created token config map {map} at tx {signature}");
                    Ok(())
                })
                .await?;
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
                    let feed_id = feed_id.strip_prefix("0x").unwrap_or(feed_id);
                    let feed_id =
                        pyth_sdk::Identifier::from_hex(feed_id).map_err(gmsol::Error::unknown)?;
                    let feed_id_as_key = Pubkey::new_from_array(feed_id.to_bytes());
                    builder = builder
                        .update_price_feed(&PriceProviderKind::Pyth, feed_id_as_key)
                        .map_err(anchor_client::ClientError::from)?;
                }
                if let Some(feed) = feeds.pyth_feed_legacy {
                    builder = builder
                        .update_price_feed(&PriceProviderKind::PythLegacy, feed)
                        .map_err(anchor_client::ClientError::from)?;
                }
                if let Some(feed) = feeds.chainlink_feed {
                    builder = builder
                        .update_price_feed(&PriceProviderKind::Chainlink, feed)
                        .map_err(anchor_client::ClientError::from)?;
                }
                let req = if let Some(decimals) = fake_decimals {
                    client.insert_synthetic_token_config(store, token, *decimals, builder, true)
                } else {
                    client.insert_token_config(store, token, builder, true)
                };
                crate::utils::send_or_serialize(req, serialize_only, |signature| {
                    println!("{signature}");
                    Ok(())
                })
                .await?;
            }
            Command::ToggleTokenConfig { token, toggle } => {
                crate::utils::send_or_serialize(
                    client.toggle_token_config(store, token, toggle.is_enable()),
                    serialize_only,
                    |signature| {
                        println!("{signature}");
                        Ok(())
                    },
                )
                .await?;
            }
            Command::SetExpectedProvider { token, provider } => {
                crate::utils::send_or_serialize(
                    client.set_expected_provider(store, token, *provider),
                    serialize_only,
                    |signature| {
                        println!("{signature}");
                        Ok(())
                    },
                )
                .await?;
            }
            Command::InitializeConfig => {
                crate::utils::send_or_serialize(
                    client.initialize_config(store).build(),
                    serialize_only,
                    |signature| {
                        println!("{signature}");
                        Ok(())
                    },
                )
                .await?;
            }
            Command::InsertAmount { amount, key, new } => {
                crate::utils::send_or_serialize(
                    client
                        .insert_global_amount(store, key, *amount, *new)
                        .build(),
                    serialize_only,
                    |signature| {
                        println!("{signature}");
                        Ok(())
                    },
                )
                .await?;
            }
            Command::InsertFactor { factor, key, new } => {
                crate::utils::send_or_serialize(
                    client
                        .insert_global_factor(store, key, *factor, *new)
                        .build(),
                    serialize_only,
                    |signature| {
                        println!("{signature}");
                        Ok(())
                    },
                )
                .await?;
            }
            Command::InsertAddress { address, key, new } => {
                crate::utils::send_or_serialize(
                    client
                        .insert_global_address(store, key, address, *new)
                        .build(),
                    serialize_only,
                    |signature| {
                        println!("{signature}");
                        Ok(())
                    },
                )
                .await?;
            }
        }
        Ok(())
    }
}
