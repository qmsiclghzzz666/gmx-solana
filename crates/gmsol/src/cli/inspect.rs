use anchor_client::solana_sdk::pubkey::Pubkey;
use data_store::states::{self};
use eyre::ContextCompat;
use gmsol::utils;
use pyth_sdk::Identifier;

use crate::{utils::Oracle, GMSOLClient};

#[derive(clap::Args)]
pub(super) struct InspectArgs {
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    /// `DataStore` account.
    DataStore {
        address: Option<Pubkey>,
        #[arg(long, short)]
        key: Option<String>,
        #[arg(long)]
        current: bool,
    },
    /// `Config` account.
    Config { address: Option<Pubkey> },
    /// `TokenConfigMap` account.
    TokenConfigMap { address: Option<Pubkey> },
    /// `Market` account.
    Market {
        address: Pubkey,
        /// Consider the address as market address rather than the address of its market token.
        #[arg(long)]
        as_market_address: bool,
        /// Whether to display the market address.
        #[arg(long)]
        show_market_address: bool,
    },
    /// `Deposit` account.
    Deposit { address: Pubkey },
    /// `Withdrawal` account.
    Withdrawal { address: Pubkey },
    /// `Oracle` account.
    Oracle {
        #[command(flatten)]
        oracle: Oracle,
    },
    /// Get the CONTROLLER address.
    Controller,
    /// Get token config.
    TokenConfig { token: Pubkey },
    /// `Order` account.
    Order { address: Pubkey },
    /// `Position` account.
    Position { address: Pubkey },
    /// Watch Pyth Price Updates.
    WatchPyth {
        #[arg(required = true)]
        feed_ids: Vec<String>,
    },
    /// Get Pyth Price Updates.
    GetPyth {
        #[arg(required = true)]
        feed_ids: Vec<String>,
        #[arg(long)]
        post: bool,
    },
    /// Generate Anchor Discriminator with the given name.
    Discriminator { name: String },
}

impl InspectArgs {
    pub(super) async fn run(
        &self,
        client: &GMSOLClient,
        store: Option<&Pubkey>,
    ) -> gmsol::Result<()> {
        let program = client.data_store();
        match &self.command {
            Command::Discriminator { name } => {
                println!("{:?}", crate::utils::generate_discriminator(name));
            }
            Command::DataStore {
                address,
                key,
                current,
            } => {
                let address = if *current {
                    *store.wrap_err("current store address not set")?
                } else {
                    address.unwrap_or_else(|| {
                        client.find_store_address(key.as_deref().unwrap_or_default())
                    })
                };
                println!("Store: {address}");
                println!("{:#?}", program.account::<states::Store>(address).await?);
            }
            Command::Config { address } => {
                let address = address
                    .or_else(|| store.map(|store| client.find_config_address(store)))
                    .wrap_err(
                        "neither the address of config account nor the address of store provided",
                    )?;
                println!("{:#?}", program.account::<states::Config>(address).await?);
            }
            Command::TokenConfigMap { address } => {
                let address = address
                    .or_else(|| {
                        store
                            .as_ref()
                            .map(|store| client.find_token_config_map(store))
                    })
                    .wrap_err("missing store address")?;
                println!(
                    "{:#?}",
                    program.account::<states::TokenConfigMap>(address).await?
                );
            }
            Command::Market {
                mut address,
                as_market_address,
                show_market_address,
            } => {
                if !as_market_address {
                    address = client
                        .find_market_address(store.wrap_err("`store` not provided")?, &address);
                }
                println!("{:#?}", program.account::<states::Market>(address).await?);
                if *show_market_address {
                    println!("Market address: {address}");
                }
            }
            Command::Deposit { address } => {
                println!("{:#?}", program.account::<states::Deposit>(*address).await?);
            }
            Command::Withdrawal { address } => {
                println!(
                    "{:#?}",
                    program.account::<states::Withdrawal>(*address).await?
                );
            }
            Command::Controller => {
                let controller = client.controller_address(store.wrap_err("`store` not provided")?);
                println!("{controller}");
            }
            Command::Oracle { oracle } => {
                let address = oracle.address(store, &client.data_store_program_id())?;
                println!("{address}");
                println!("{:#?}", program.account::<states::Oracle>(address).await?);
            }
            Command::TokenConfig { token } => {
                let store = store.wrap_err("missing store address")?;
                let config = client
                    .token_config(store, token)
                    .await?
                    .ok_or(gmsol::Error::NotFound)?;
                println!("{config:#?}");
            }
            Command::Order { address } => {
                println!("{:#?}", program.account::<states::Order>(*address).await?);
            }
            Command::Position { address } => {
                println!(
                    "{:#?}",
                    utils::try_deserailize_account::<states::Position>(
                        &program.async_rpc(),
                        address
                    )
                    .await?
                );
            }
            Command::WatchPyth { feed_ids } => {
                use futures_util::TryStreamExt;
                use gmsol::pyth::{EncodingType, Hermes};

                let hermes = Hermes::default();
                let feed_ids = parse_feed_ids(feed_ids)?;
                let stream = hermes
                    .price_updates(&feed_ids, Some(EncodingType::Base64))
                    .await?;
                futures_util::pin_mut!(stream);
                while let Some(update) = stream.try_next().await? {
                    tracing::info!("{:#?}", update.parsed());
                }
            }
            Command::GetPyth { feed_ids, post } => {
                use gmsol::pyth::{
                    EncodingType, Hermes, PythPullOracle, PythPullOracleContext, PythPullOracleOps,
                };

                let hermes = Hermes::default();
                let feed_ids = parse_feed_ids(feed_ids)?;
                let update = hermes
                    .latest_price_updates(&feed_ids, Some(EncodingType::Base64))
                    .await?;
                tracing::info!("{:#?}", update.parsed());

                if *post {
                    let oracle = PythPullOracle::try_new(client.anchor())?;
                    let ctx = PythPullOracleContext::new(feed_ids);
                    let prices = oracle
                        .with_pyth_prices(&ctx, &update, |prices| {
                            for (feed_id, price_update) in prices {
                                tracing::info!(%feed_id, %price_update, "posting price update");
                            }
                            async { Ok(None) }
                        })
                        .await?;
                    match prices.send_all(None).await {
                        Ok(signatures) => {
                            tracing::info!("successfully sent all txs: {signatures:#?}");
                        }
                        Err((signatures, err)) => {
                            tracing::error!(%err, "sent txs error, successful list: {signatures:#?}");
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

fn parse_feed_ids(feed_ids: &[String]) -> gmsol::Result<Vec<Identifier>> {
    let feed_ids = feed_ids
        .iter()
        .map(|id| {
            let hex = id.strip_prefix("0x").unwrap_or(id);
            Identifier::from_hex(hex).map_err(gmsol::Error::unknown)
        })
        .collect::<gmsol::Result<Vec<_>>>()?;
    Ok(feed_ids)
}
