use anchor_client::solana_sdk::{native_token::lamports_to_sol, pubkey::Pubkey};
use data_store::states::{
    self, AddressKey, AmountKey, FactorKey, MarketConfigKey, PriceProviderKind,
};
use gmsol::{
    store::{
        token_config::TokenConfigOps,
        utils::{read_market, read_store, token_map, token_map_optional},
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
        /// Ger roles for a member.
        #[arg(long, group = "get", value_name = "USER")]
        get_roles: Option<Pubkey>,
        /// Get members for the role.
        #[arg(long, group = "get", value_name = "ROLE")]
        get_members: Option<String>,
        /// Get all roles.
        #[arg(long, group = "get")]
        get_all_roles: bool,
        /// Get all members.
        #[arg(long, group = "get")]
        get_all_members: bool,
    },
    /// `TokenMap` account.
    TokenMap {
        address: Option<Pubkey>,
        #[arg(long, value_name = "TOKEN", group = "get")]
        get_token: Option<Pubkey>,
        /// Modify the get command to get the feed of the given provider.
        #[arg(long, value_name = "PROVIDER")]
        feed: Option<PriceProviderKind>,
        #[arg(long, group = "get")]
        debug: bool,
        /// List all tokens.
        #[arg(long, group = "get")]
        get_all_tokens: bool,
        #[arg(long)]
        show_address: bool,
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
    /// Get the CONTROLLER address.
    Controller,
    /// Get the event authority address.
    EventAuthority,
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
                get_members,
                get_all_roles,
                get_all_members,
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
                    let role_store = store.role();
                    for role in role_store.roles() {
                        let name = role?;
                        if role_store.has_role(user, name)? {
                            println!("{name}");
                        }
                    }
                } else if let Some(role) = get_members {
                    let role_store = store.role();
                    for member in role_store.members() {
                        if role_store.has_role(&member, role)? {
                            println!("{member}");
                        }
                    }
                } else if *get_all_roles {
                    for role in store.role().roles() {
                        println!("{}", role.unwrap_or("*failed to parse role name*"));
                    }
                } else if *get_all_members {
                    let role_store = store.role();
                    let roles = role_store.roles().collect::<Result<Vec<_>, _>>()?;
                    for member in role_store.members() {
                        let roles = roles
                            .iter()
                            .filter_map(|role| match role_store.has_role(&member, role) {
                                Ok(true) => Some(Ok(*role)),
                                Ok(false) => None,
                                Err(err) => Some(Err(err)),
                            })
                            .collect::<Result<Vec<_>, _>>()?
                            .join("|");
                        println!("{member}, roles={roles}");
                    }
                } else if *debug {
                    println!("{store:?}");
                } else {
                    println!("{store}");
                }
                if *show_address {
                    println!("Address: {address}");
                }
            }
            Command::TokenMap {
                address,
                get_token,
                feed,
                debug,
                get_all_tokens,
                show_address,
            } => {
                let address = if let Some(address) = address {
                    let authorized_token_map = token_map_optional(program, store).await?;
                    if authorized_token_map != Some(*address) {
                        tracing::warn!("this token map is not authorized by the store");
                    }
                    *address
                } else {
                    token_map(program, store).await?
                };
                if let Some(token) = get_token {
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
                    if *debug {
                        println!("{token_map:#?}");
                    } else if *get_all_tokens {
                        let mut is_empty = true;
                        for token in token_map.tokens() {
                            println!("{token}");
                            is_empty = false;
                        }
                        if is_empty {
                            println!("*no tokens*");
                        }
                    } else {
                        println!("{token_map}");
                    }
                }
                if *show_address {
                    println!("Address: {address}");
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
                println!("Exchange: {}", client.exchange_program_id());
                println!("Controller: {controller}");
                match client
                    .data_store()
                    .async_rpc()
                    .get_balance(&controller)
                    .await
                {
                    Ok(lamports) => {
                        println!("Balance: {} SOL", lamports_to_sol(lamports));
                    }
                    Err(err) => {
                        println!("Balance: *failed to get balance*");
                        tracing::info!(%err, "failed to get balance");
                    }
                }
            }
            Command::EventAuthority => {
                println!("DataStore: {}", client.data_store_program_id());
                println!("Event Authority: {}", client.data_store_event_authority());
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
                    match prices.send_all(None, true).await {
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
