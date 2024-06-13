use std::{collections::HashMap, io::Read, path::PathBuf};

use anchor_client::solana_sdk::{pubkey::Pubkey, signature::Keypair};
use data_store::states::{
    Factor, MarketConfigKey, PriceProviderKind, TokenConfigBuilder, DEFAULT_HEARTBEAT_DURATION,
    DEFAULT_PRECISION,
};
use gmsol::{
    exchange::ExchangeOps,
    store::{
        market::{MarketOps, VaultOps},
        token_config::TokenConfigOps,
        utils::token_map,
    },
    utils::TransactionBuilder,
};

use crate::GMSOLClient;

#[derive(clap::Args)]
pub(super) struct Args {
    #[arg(long)]
    token_map: Option<Pubkey>,
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    /// Initialize a `TokenMap` account.
    InitializeTokenMap,
    /// Read and insert token configs from file.
    InsertTokenConfigs {
        path: PathBuf,
        #[arg(long)]
        skip_preflight: bool,
    },
    /// Insert or update the config of token.
    InsertTokenConfig {
        token: Pubkey,
        #[arg(long)]
        name: String,
        #[command(flatten)]
        feeds: Feeds,
        #[arg(long)]
        expected_provider: PriceProviderKind,
        #[arg(long, default_value_t = DEFAULT_HEARTBEAT_DURATION)]
        heartbeat_duration: u32,
        #[arg(long, default_value_t = DEFAULT_PRECISION)]
        precision: u8,
        /// Provide to create a synthetic token with the given decimals.
        #[arg(long, value_name = "DECIMALS")]
        synthetic: Option<u8>,
        #[arg(long)]
        update: bool,
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
    /// Update Market Config.
    UpdateConfig {
        market_token: Pubkey,
        #[arg(long, short)]
        key: MarketConfigKey,
        #[arg(long, short)]
        value: Factor,
    },
}

#[serde_with::serde_as]
#[derive(Debug, clap::Args, serde::Serialize, serde::Deserialize)]
#[group(required = true, multiple = true)]
struct Feeds {
    /// Pyth feed id.
    #[arg(long)]
    pyth_feed_id: Option<String>,
    /// Pyth feed account (Legacy)
    #[arg(long)]
    #[serde_as(as = "Option<serde_with::DisplayFromStr>")]
    pyth_feed_legacy: Option<Pubkey>,
    /// Chainlink feed.
    #[arg(long)]
    #[serde_as(as = "Option<serde_with::DisplayFromStr>")]
    chainlink_feed: Option<Pubkey>,
}

impl Feeds {
    fn pyth_feed_id(&self) -> gmsol::Result<Option<Pubkey>> {
        let Some(pyth_feed_id) = self.pyth_feed_id.as_ref() else {
            return Ok(None);
        };
        let feed_id = pyth_feed_id.strip_prefix("0x").unwrap_or(pyth_feed_id);
        let feed_id = pyth_sdk::Identifier::from_hex(feed_id).map_err(gmsol::Error::unknown)?;
        let feed_id_as_key = Pubkey::new_from_array(feed_id.to_bytes());
        Ok(Some(feed_id_as_key))
    }
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

impl Args {
    pub(super) async fn run(
        &self,
        client: &GMSOLClient,
        store: &Pubkey,
        serialize_only: bool,
    ) -> gmsol::Result<()> {
        match &self.command {
            Command::InitializeTokenMap => {
                if serialize_only {
                    return Err(gmsol::Error::invalid_argument(
                        "serialize-only mode is not supported for this command",
                    ));
                }
                let token_map = Keypair::new();
                let (rpc, map) = client.initialize_token_map(store, &token_map);
                crate::utils::send_or_serialize(
                    rpc.build_without_compute_budget(),
                    false,
                    |signature| {
                        println!("created token config map {map} at tx {signature}");
                        Ok(())
                    },
                )
                .await?;
            }
            Command::InsertTokenConfigs {
                path,
                skip_preflight,
            } => {
                let mut buffer = String::new();
                std::fs::File::open(path)?.read_to_string(&mut buffer)?;
                let configs: HashMap<String, TokenConfig> =
                    toml::from_str(&buffer).map_err(gmsol::Error::invalid_argument)?;
                insert_token_configs(client, store, serialize_only, *skip_preflight, &configs)
                    .await?;
            }
            Command::InsertTokenConfig {
                name,
                token,
                feeds,
                expected_provider,
                heartbeat_duration,
                precision,
                synthetic: fake_decimals,
                update,
            } => {
                let mut builder = TokenConfigBuilder::default()
                    .with_heartbeat_duration(*heartbeat_duration)
                    .with_precision(*precision)
                    .with_expected_provider(*expected_provider);
                if let Some(feed_id) = feeds.pyth_feed_id()? {
                    builder = builder
                        .update_price_feed(&PriceProviderKind::Pyth, feed_id)
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
                let token_map = self.token_map(client, store).await?;
                let req = if let Some(decimals) = fake_decimals {
                    client.insert_synthetic_token_config(
                        store, &token_map, name, token, *decimals, builder, true, !*update,
                    )
                } else {
                    client.insert_token_config(
                        store, &token_map, name, token, builder, true, !*update,
                    )
                };
                crate::utils::send_or_serialize(
                    req.build_without_compute_budget(),
                    serialize_only,
                    |signature| {
                        println!("{signature}");
                        Ok(())
                    },
                )
                .await?;
            }
            Command::ToggleTokenConfig { token, toggle } => {
                let token_map = self.token_map(client, store).await?;
                crate::utils::send_or_serialize(
                    client.toggle_token_config(store, &token_map, token, toggle.is_enable()),
                    serialize_only,
                    |signature| {
                        println!("{signature}");
                        Ok(())
                    },
                )
                .await?;
            }
            Command::SetExpectedProvider { token, provider } => {
                let token_map = self.token_map(client, store).await?;
                crate::utils::send_or_serialize(
                    client.set_expected_provider(store, &token_map, token, *provider),
                    serialize_only,
                    |signature| {
                        println!("{signature}");
                        Ok(())
                    },
                )
                .await?;
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
            Command::UpdateConfig {
                market_token,
                key,
                value,
            } => {
                crate::utils::send_or_serialize(
                    client
                        .update_market_config_by_key(store, market_token, *key, value)?
                        .build_without_compute_budget(),
                    serialize_only,
                    |signature| {
                        println!("market config updated at tx {signature}");
                        Ok(())
                    },
                )
                .await?;
            }
        }
        Ok(())
    }

    async fn token_map(&self, client: &GMSOLClient, store: &Pubkey) -> gmsol::Result<Pubkey> {
        if let Some(token_map) = self.token_map {
            Ok(token_map)
        } else {
            gmsol::store::utils::token_map(client.data_store(), store).await
        }
    }
}

/// Token Config.
#[serde_with::serde_as]
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct TokenConfig {
    #[serde_as(as = "serde_with::DisplayFromStr")]
    address: Pubkey,
    #[serde(default)]
    synthetic: Option<u8>,
    enable: bool,
    expected_provider: PriceProviderKind,
    feeds: Feeds,
    #[serde(default = "default_precision")]
    precision: u8,
    #[serde(default = "default_heartbeat_duration")]
    heartbeat_duration: u32,
    #[serde(default)]
    update: bool,
}

fn default_heartbeat_duration() -> u32 {
    DEFAULT_HEARTBEAT_DURATION
}

fn default_precision() -> u8 {
    DEFAULT_PRECISION
}

impl<'a> TryFrom<&'a TokenConfig> for TokenConfigBuilder {
    type Error = gmsol::Error;

    fn try_from(config: &'a TokenConfig) -> Result<Self, Self::Error> {
        let mut builder = Self::default()
            .with_expected_provider(config.expected_provider)
            .with_heartbeat_duration(config.heartbeat_duration)
            .with_precision(config.precision);
        if let Some(pyth_feed_id) = config.feeds.pyth_feed_id()? {
            builder = builder.update_price_feed(&PriceProviderKind::Pyth, pyth_feed_id)?;
        }
        if let Some(chainlink_feed) = config.feeds.chainlink_feed {
            builder = builder.update_price_feed(&PriceProviderKind::Chainlink, chainlink_feed)?;
        }
        if let Some(pyth_legacy_feed) = config.feeds.pyth_feed_legacy {
            builder =
                builder.update_price_feed(&PriceProviderKind::PythLegacy, pyth_legacy_feed)?;
        }
        Ok(builder)
    }
}

async fn insert_token_configs(
    client: &GMSOLClient,
    store: &Pubkey,
    serialize_only: bool,
    skip_preflight: bool,
    configs: &HashMap<String, TokenConfig>,
) -> gmsol::Result<()> {
    let mut builder = TransactionBuilder::new(client.data_store().async_rpc());
    let token_map = token_map(client.data_store(), store).await?;

    for (name, config) in configs {
        let token = &config.address;
        if let Some(decimals) = config.synthetic {
            builder.try_push(client.insert_synthetic_token_config(
                store,
                &token_map,
                name,
                token,
                decimals,
                config.try_into()?,
                config.enable,
                !config.update,
            ))?;
        } else {
            builder.try_push(client.insert_token_config(
                store,
                &token_map,
                name,
                token,
                config.try_into()?,
                config.enable,
                !config.update,
            ))?;
        }
    }

    crate::utils::send_or_serialize_transactions(
        builder,
        serialize_only,
        skip_preflight,
        |signatures, error| {
            println!("{signatures:#?}");
            match error {
                None => Ok(()),
                Some(err) => Err(err),
            }
        },
    )
    .await?;

    Ok(())
}
