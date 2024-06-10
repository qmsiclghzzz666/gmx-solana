use anchor_client::solana_sdk::{pubkey::Pubkey, signature::Keypair};
use data_store::states::{PriceProviderKind, TokenConfigBuilder};
use gmsol::store::token_config::TokenConfigOps;

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
        #[arg(long)]
        serialize_only: bool,
        #[arg(long)]
        update: bool,
    },
    /// Toggle token config of token.
    ToggleTokenConfig {
        token: Pubkey,
        #[command(flatten)]
        toggle: Toggle,
        #[arg(long)]
        serialize_only: bool,
    },
    /// Set expected provider of token.
    SetExpectedProvider {
        token: Pubkey,
        provider: PriceProviderKind,
        #[arg(long)]
        serialize_only: bool,
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

impl Args {
    pub(super) async fn run(&self, client: &GMSOLClient, store: &Pubkey) -> gmsol::Result<()> {
        match &self.command {
            Command::InitializeTokenMap => {
                let token_map = Keypair::new();
                let (request, map) = client.initialize_token_map(store, &token_map);
                crate::utils::send_or_serialize(request, false, |signature| {
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
                serialize_only,
                update,
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
                let token_map = self.token_map(client, store).await?;
                let req = if let Some(decimals) = fake_decimals {
                    client.insert_synthetic_token_config(
                        store, &token_map, token, *decimals, builder, true, !*update,
                    )
                } else {
                    client.insert_token_config(store, &token_map, token, builder, true, !*update)
                };
                crate::utils::send_or_serialize(req, *serialize_only, |signature| {
                    println!("{signature}");
                    Ok(())
                })
                .await?;
            }
            Command::ToggleTokenConfig {
                token,
                toggle,
                serialize_only,
            } => {
                let token_map = self.token_map(client, store).await?;
                crate::utils::send_or_serialize(
                    client.toggle_token_config(store, &token_map, token, toggle.is_enable()),
                    *serialize_only,
                    |signature| {
                        println!("{signature}");
                        Ok(())
                    },
                )
                .await?;
            }
            Command::SetExpectedProvider {
                token,
                provider,
                serialize_only,
            } => {
                let token_map = self.token_map(client, store).await?;
                crate::utils::send_or_serialize(
                    client.set_expected_provider(store, &token_map, token, *provider),
                    *serialize_only,
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

    async fn token_map(&self, client: &GMSOLClient, store: &Pubkey) -> gmsol::Result<Pubkey> {
        if let Some(token_map) = self.token_map {
            Ok(token_map)
        } else {
            gmsol::store::utils::token_map(client.data_store(), store).await
        }
    }
}
