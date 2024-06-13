use anchor_client::solana_sdk::pubkey::Pubkey;
use data_store::states::{
    self, AddressKey, AmountKey, FactorKey, MarketConfigKey, PriceProviderKind,
};
use gmsol::{
    store::{
        token_config::TokenConfigOps,
        utils::{read_market, read_store, token_map},
    },
    utils::{self, try_deserailize_account, view},
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
        #[arg(long, short, group = "other-store")]
        address: Option<Pubkey>,
        #[arg(long, short, group = "other-store")]
        key: Option<String>,
        #[arg(long)]
        show_address: bool,
        #[arg(long, group = "get")]
        debug: bool,
        #[arg(long, group = "get")]
        get_amount: Option<AmountKey>,
        #[arg(long, group = "get")]
        get_factor: Option<FactorKey>,
        #[arg(long, group = "get")]
        get_address: Option<AddressKey>,
        #[arg(long, group = "get", value_name = "USER")]
        get_roles: Option<Pubkey>,
    },
    /// `TokenMap` account.
    TokenMap {
        address: Option<Pubkey>,
        #[arg(long, value_name = "TOKEN")]
        get: Option<Pubkey>,
        /// Modify the get command to get the feed of the given provider.
        #[arg(long, value_name = "PROVIDER")]
        feed: Option<PriceProviderKind>,
    },
    /// `Market` account.
    Market {
        address: Pubkey,
        /// Consider the address as market address rather than the address of its market token.
        #[arg(long)]
        as_market_address: bool,
        /// Whether to display the market address.
        #[arg(long)]
        show_market_address: bool,
        #[arg(long, group = "get")]
        debug: bool,
        #[arg(long, group = "get")]
        get_config: Option<MarketConfigKey>,
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
    pub(super) async fn run(&self, client: &GMSOLClient, store: &Pubkey) -> gmsol::Result<()> {
        let program = client.data_store();
        match &self.command {
            Command::Discriminator { name } => {
                println!("{:?}", crate::utils::generate_discriminator(name));
            }
            Command::Store {
                address,
                key,
                show_address,
                debug,
                get_address,
                get_amount,
                get_factor,
                get_roles,
            } => {
                let address = if let Some(address) = address {
                    *address
                } else if let Some(key) = key {
                    client.find_store_address(key)
                } else {
                    *store
                };
                let store = read_store(&client.data_store().async_rpc(), &address).await?;
                if let Some(key) = get_amount {
                    println!("{}", store.get_amount_by_key(*key));
                } else if let Some(key) = get_factor {
                    println!("{}", store.get_factor_by_key(*key));
                } else if let Some(key) = get_address {
                    println!("{}", store.get_address_by_key(*key));
                } else if let Some(user) = get_roles {
                    match store.role().role_value(user) {
                        Some(value) => println!("{value:#b}"),
                        None => return Err(gmsol::Error::invalid_argument("Not a member")),
                    }
                } else if *debug {
                    println!("{store:?}");
                } else {
                    println!("{store}");
                }
                if *show_address {
                    println!("Store Address: {address}");
                }
            }
            Command::TokenMap { address, get, feed } => {
                let address = if let Some(address) = address {
                    *address
                } else {
                    token_map(program, store).await?
                };
                if let Some(token) = get {
                    if let Some(provider) = feed {
                        let transaction = client
                            .token_feed(&address, token, *provider)
                            .signed_transaction()
                            .await?;
                        let feed: Pubkey =
                            view(&client.data_store().async_rpc(), &transaction).await?;
                        match provider {
                            PriceProviderKind::Pyth => {
                                println!("0x{}", hex::encode(feed));
                            }
                            _ => {
                                println!("{feed}");
                            }
                        }
                    } else {
                        let config = client.token_config(&address, token).await?;
                        println!("{config:#?}");
                    }
                } else {
                    let token_map = try_deserailize_account::<states::TokenMapHeader>(
                        &program.async_rpc(),
                        &address,
                    )
                    .await?;
                    println!("{token_map:#?}");
                }
            }
            Command::Market {
                mut address,
                as_market_address,
                show_market_address: show_address,
                debug,
                get_config,
            } => {
                if !as_market_address {
                    address = client.find_market_address(store, &address);
                }
                let market = read_market(&program.async_rpc(), &address).await?;
                if let Some(key) = get_config {
                    println!("{}", market.get_config_by_key(*key));
                } else if *debug {
                    println!("{:?}", market);
                } else {
                    println!("{:#?}", market);
                }
                if *show_address {
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
                let controller = client.controller_address(store);
                println!("{controller}");
            }
            Command::Oracle { oracle } => {
                let address = oracle.address(Some(store), &client.data_store_program_id())?;
                println!("{address}");
                println!("{:#?}", program.account::<states::Oracle>(address).await?);
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
