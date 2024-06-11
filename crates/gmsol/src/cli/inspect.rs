use anchor_client::solana_sdk::pubkey::Pubkey;
use data_store::states::{self};
use eyre::ContextCompat;
use gmsol::{
    store::utils::{read_market, read_store, token_map},
    utils::{self, try_deserailize_account},
};
use pyth_sdk::Identifier;

use crate::{utils::Oracle, GMSOLClient};

#[derive(clap::Args)]
pub(super) struct InspectArgs {
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    /// `Store` account.
    Store {
        address: Option<Pubkey>,
        #[arg(long, short)]
        key: Option<String>,
        #[arg(long)]
        current: bool,
        #[arg(long, group = "get")]
        debug: bool,
        #[arg(long, group = "get")]
        get_amount: Option<String>,
        #[arg(long, group = "get")]
        get_factor: Option<String>,
        #[arg(long, group = "get")]
        get_address: Option<String>,
    },
    /// `TokenMap` account.
    TokenMap { address: Option<Pubkey> },
    /// `Market` account.
    Market {
        address: Pubkey,
        /// Consider the address as market address rather than the address of its market token.
        #[arg(long)]
        as_market_address: bool,
        /// Whether to display the market address.
        #[arg(long)]
        show_market_address: bool,
        #[arg(long)]
        debug: bool,
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
            Command::Store {
                address,
                key,
                current,
                debug,
                get_address,
                get_amount,
                get_factor,
            } => {
                let address = if *current {
                    *store.wrap_err("current store address not set")?
                } else {
                    address.unwrap_or_else(|| {
                        client.find_store_address(key.as_deref().unwrap_or_default())
                    })
                };
                let store = read_store(&client.data_store().async_rpc(), &address).await?;
                if let Some(key) = get_amount {
                    println!("{}", store.get_amount(key)?);
                } else if let Some(key) = get_factor {
                    println!("{}", store.get_factor(key)?);
                } else if let Some(key) = get_address {
                    println!("{}", store.get_address(key)?);
                } else if *debug {
                    println!("Store Address: {address}");
                    println!("{store:?}");
                } else {
                    println!("Store Address: {address}");
                    println!("{store}");
                }
            }
            Command::TokenMap { address } => {
                let address = if let Some(address) = address {
                    *address
                } else {
                    token_map(program, store.wrap_err("neither the address of config account nor the address of store provided")?).await?
                };
                println!(
                    "{:#?}",
                    try_deserailize_account::<states::TokenMapHeader>(
                        &program.async_rpc(),
                        &address
                    )
                    .await?,
                );
            }
            Command::Market {
                mut address,
                as_market_address,
                show_market_address,
                debug,
            } => {
                if !as_market_address {
                    address = client
                        .find_market_address(store.wrap_err("`store` not provided")?, &address);
                }
                let market = read_market(&program.async_rpc(), &address).await?;
                if *debug {
                    println!("{:?}", market);
                } else {
                    println!("{:#?}", market);
                }
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
            Command::TokenConfig { token: _ } => {
                unimplemented!();
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
