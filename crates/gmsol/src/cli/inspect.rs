use anchor_client::solana_sdk::{native_token::lamports_to_sol, pubkey::Pubkey};
use bytemuck::offset_of;
use gmsol::{
    client::StoreFilter,
    store::utils::{read_market, read_store, token_map, token_map_optional},
    types::{Market, TokenConfig, TokenMap, TokenMapAccess},
    utils::{self, zero_copy::ZeroCopy},
};
use gmsol_model::{Balance, BalanceExt, ClockKind, PoolKind};
use gmsol_store::states::{
    self, AddressKey, AmountKey, FactorKey, MarketConfigKey, PriceProviderKind,
};
use indexmap::IndexMap;
use num_format::{Locale, ToFormattedString};
use prettytable::{row, Table};
use pyth_sdk::Identifier;
use rust_decimal_macros::dec;
use strum::IntoEnumIterator;

use crate::{
    ser::SerializeMarket,
    utils::{signed_value_to_decimal, table_format, unsigned_value_to_decimal, Oracle, Output},
    GMSOLClient,
};

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
        address: Option<Pubkey>,
        /// Consider the address as market address rather than the address of its market token.
        #[arg(long)]
        as_market_address: bool,
        #[arg(long)]
        get_config: Option<MarketConfigKey>,
        /// Output format.
        #[arg(long, short)]
        output: Option<Output>,
    },
    /// `MarketConfigBuffer` account.
    MarketConfigBuffer {
        address: Pubkey,
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
                let token_map = program.account::<TokenMap>(address).await?;
                if let Some(token) = get_token {
                    let config = token_map
                        .get(token)
                        .ok_or(gmsol::Error::invalid_argument("token not found"))?;
                    if let Some(kind) = feed {
                        println!("{}", config.get_feed(kind)?);
                    } else {
                        println!("{}", format_token_config(config)?);
                    }
                } else if *debug {
                    println!("{token_map:#?}");
                } else if *get_all_tokens {
                    let mut is_empty = true;
                    for token in token_map.header().tokens() {
                        println!("{token}");
                        is_empty = false;
                    }
                    if is_empty {
                        println!("*no tokens*");
                    }
                } else {
                    println!("{}", token_map.header());
                }

                if *show_address {
                    println!("Address: {address}");
                }
            }
            Command::Market {
                address,
                as_market_address,
                get_config,
                output,
            } => {
                let output = output.unwrap_or_default();
                if let Some(mut address) = address {
                    if !as_market_address {
                        address = client.find_market_address(store, &address);
                    }
                    let market = read_market(&program.async_rpc(), &address).await?;
                    let serialized = SerializeMarket::from_market(&address, &market)?;
                    if let Some(key) = get_config {
                        let value = market.get_config_by_key(*key);
                        output.print(value, |value| Ok(value.to_string()))?;
                    } else {
                        output.print(&serialized, format_market)?;
                    }
                } else {
                    let markets = client
                        .store_accounts::<ZeroCopy<Market>>(
                            Some(&StoreFilter::new(store, offset_of!(Market, store))),
                            false,
                        )
                        .await?
                        .into_iter()
                        .filter_map(|(pubkey, market)| {
                            SerializeMarket::from_market(&pubkey, &market.0)
                                .inspect_err(
                                    |err| tracing::error!(%pubkey, %err, "parse market error"),
                                )
                                .ok()
                        })
                        .collect::<Vec<_>>();
                    output.print(&markets, |markets| {
                        use std::cmp::Reverse;

                        let mut table = Table::new();
                        let mut markets = markets
                            .iter()
                            .map(|market| {
                                let pools = &market.pools.0;
                                let oi =
                                    pools.get(&PoolKind::OpenInterestForLong).and_then(|long| {
                                        pools
                                            .get(&PoolKind::OpenInterestForShort)
                                            .map(|short| long.merge(short))
                                    });
                                (market, oi)
                            })
                            .collect::<Vec<_>>();
                        markets.sort_by_key(|(m, oi)| {
                            let oi = oi
                                .as_ref()
                                .map(|oi| {
                                    oi.long_amount().unwrap_or(0) + oi.short_amount().unwrap_or(0)
                                })
                                .unwrap_or_default();
                            (Reverse(m.enabled), Reverse(oi), &m.name)
                        });
                        for (market, oi) in markets {
                            let (oi_long, oi_short) = if let Some(oi) = oi {
                                (
                                    format!(
                                        "{}",
                                        unsigned_value_to_decimal(oi.long_amount()?).normalize()
                                    ),
                                    format!(
                                        "{}",
                                        unsigned_value_to_decimal(oi.short_amount()?).normalize()
                                    ),
                                )
                            } else {
                                ("*missing*".to_string(), "*missing*".to_string())
                            };
                            let name = &market.name;
                            let token = market.meta.market_token;
                            let enabled = market.enabled;
                            let funding_per_hour =
                                signed_value_to_decimal(market.state.funding_factor_per_second)
                                    * dec!(3600)
                                    * dec!(100);
                            table.add_row(row![
                                name,
                                token,
                                enabled,
                                oi_long,
                                oi_short,
                                funding_per_hour.normalize(),
                            ]);
                        }

                        table.set_titles(row![
                            "Name",
                            "Token",
                            "Enabled",
                            "OI Long ($)",
                            "OI Short ($)",
                            "Funding Rate (% / hour)",
                        ]);
                        table.set_format(table_format());
                        Ok(table.to_string())
                    })?;
                }
            }
            Command::MarketConfigBuffer { address, debug } => {
                let buffer = program
                    .account::<states::MarketConfigBuffer>(*address)
                    .await?;
                if *debug {
                    println!("{buffer:#?}");
                } else {
                    println!("Authority: {}", buffer.authority);
                    println!("Store: {}", buffer.store);

                    // Format expiry.
                    let expiry = time::OffsetDateTime::from_unix_timestamp(buffer.expiry)
                        .map_err(gmsol::Error::unknown)?;
                    let now = time::OffsetDateTime::now_utc();
                    let msg = if expiry > now {
                        let dur = expiry - now;
                        format!(
                            "will expire in {}",
                            humantime::format_duration(
                                dur.try_into().map_err(gmsol::Error::unknown)?
                            )
                        )
                    } else {
                        let dur = now - expiry;
                        format!(
                            "expired {} ago",
                            humantime::format_duration(
                                dur.try_into().map_err(gmsol::Error::unknown)?
                            )
                        )
                    };
                    println!(
                        "Expiry: {} ({msg})",
                        humantime::format_rfc3339(expiry.into())
                    );

                    // Print configs.
                    if buffer.is_empty() {
                        println!("*buffer is empty*");
                    } else {
                        println!("Parameter count: {}", buffer.len());
                    }
                    let map = buffer
                        .iter()
                        .map(|entry| Ok((entry.key()?, entry.value())))
                        .collect::<Result<IndexMap<_, _>, gmsol::Error>>()?;
                    for (key, value) in map.iter() {
                        println!("{key} = {}", value.to_formatted_string(&Locale::en));
                    }
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
                    utils::try_deserailize_zero_copy_account::<states::Position>(
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

fn format_market(market: &SerializeMarket) -> gmsol::Result<String> {
    use std::fmt::Write;

    let mut buf = String::new();

    let f = &mut buf;

    writeln!(f, "Name: {}", market.name)?;

    writeln!(f, "\nEnabled: {}", market.enabled)?;

    writeln!(f, "\nAddress: {}", market.address)?;

    writeln!(f, "\nStore: {}", market.store)?;

    writeln!(f, "\nMeta:")?;
    let meta = &market.meta;
    writeln!(f, "market_token: {}", meta.market_token)?;
    writeln!(f, "index_token: {}", meta.index_token)?;
    if meta.long_token == meta.short_token {
        writeln!(f, "single_token: {}", meta.long_token)?;
    } else {
        writeln!(f, "long_token: {}", meta.long_token)?;
        writeln!(f, "short_token: {}", meta.short_token)?;
    }

    writeln!(f, "\nState:")?;
    let state = &market.state;
    if market.is_pure {
        writeln!(
            f,
            "token_balance: {}",
            state.long_token_balance.to_formatted_string(&Locale::en)
        )?;
    } else {
        writeln!(
            f,
            "long_token_balance: {}",
            state.long_token_balance.to_formatted_string(&Locale::en)
        )?;
        writeln!(
            f,
            "short_token_balance: {}",
            state.short_token_balance.to_formatted_string(&Locale::en)
        )?;
    }
    writeln!(
        f,
        "funding_factor_per_second: {}",
        state
            .funding_factor_per_second
            .to_formatted_string(&Locale::en)
    )?;
    writeln!(
        f,
        "deposit_count: {}",
        state.deposit_count.to_formatted_string(&Locale::en)
    )?;
    writeln!(
        f,
        "withdrawal_count: {}",
        state.withdrawal_count.to_formatted_string(&Locale::en)
    )?;
    writeln!(
        f,
        "order_count: {}",
        state.order_count.to_formatted_string(&Locale::en)
    )?;

    writeln!(f, "\nClocks:")?;
    let now = time::OffsetDateTime::now_utc();
    for kind in ClockKind::iter() {
        if let Some(clock) = market.clocks.0.get(&kind) {
            let ts =
                time::OffsetDateTime::from_unix_timestamp(*clock).map_err(gmsol::Error::unknown)?;
            let msg = if now >= ts {
                let dur = now - ts;
                format!(
                    " ({} ago)",
                    humantime::format_duration(dur.try_into().map_err(gmsol::Error::unknown)?)
                )
            } else {
                String::new()
            };
            writeln!(f, "{kind}: {}{msg}", humantime::format_rfc3339(ts.into()))?;
        } else {
            writeln!(f, "{kind}: not enabled")?;
        }
    }

    writeln!(f, "\nPools (LONG - SHORT):")?;
    for kind in PoolKind::iter() {
        if let Some(pool) = market.pools.0.get(&kind) {
            writeln!(
                f,
                "{kind}: {} - {}",
                pool.long_amount()?.to_formatted_string(&Locale::en),
                pool.short_amount()?.to_formatted_string(&Locale::en),
            )?;
        } else {
            writeln!(f, "{kind}: not enabled")?;
        }
    }

    writeln!(f, "\nParameters:")?;
    for (key, value) in market.params.0.iter() {
        writeln!(f, "{key} = {}", value.0.to_formatted_string(&Locale::en))?;
    }

    Ok(buf)
}

fn format_token_config(token_config: &TokenConfig) -> gmsol::Result<String> {
    use std::fmt::Write;

    let mut buf = String::new();

    let f = &mut buf;

    writeln!(f, "{token_config}")?;

    let expected_provider = token_config.expected_provider()?;
    writeln!(f, "Feeds:")?;
    for kind in PriceProviderKind::iter() {
        let Ok(feed) = token_config.get_feed_config(&kind) else {
            continue;
        };
        let expected = if expected_provider == kind { "*" } else { "" };
        writeln!(f, "{kind}{expected} = {{ {feed} }}")?;
    }

    Ok(buf)
}
