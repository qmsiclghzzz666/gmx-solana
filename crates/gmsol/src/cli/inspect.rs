use anchor_client::solana_sdk::pubkey::Pubkey;
use data_store::states::{self};
use exchange::utils::ControllerSeeds;
use eyre::ContextCompat;
use gmsol::{
    store::{
        market::find_market_address,
        token_config::{find_token_config_map, get_token_config},
    },
    utils,
};

use crate::{utils::Oracle, SharedClient};

#[derive(clap::Args)]
pub(super) struct InspectArgs {
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    /// `DataStore` account.
    DataStore { address: Option<Pubkey> },
    /// `Roles` account.
    Roles { address: Pubkey },
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
}

impl InspectArgs {
    pub(super) async fn run(
        &self,
        client: &SharedClient,
        store: Option<&Pubkey>,
    ) -> gmsol::Result<()> {
        let program = client.program(data_store::id())?;
        match &self.command {
            Command::DataStore { address } => {
                let address = address
                    .or(store.copied())
                    .wrap_err("missing store address")?;
                println!(
                    "{:#?}",
                    program.account::<states::DataStore>(address).await?
                );
            }
            Command::Roles { address } => {
                println!("{:#?}", program.account::<states::Roles>(*address).await?);
            }
            Command::TokenConfigMap { address } => {
                let address = address
                    .or_else(|| store.as_ref().map(|store| find_token_config_map(store).0))
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
                    address =
                        find_market_address(store.wrap_err("`store` not provided")?, &address).0;
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
                let controller =
                    ControllerSeeds::find_with_address(store.wrap_err("missing `store` address")?)
                        .1;
                println!("{controller}");
            }
            Command::Oracle { oracle } => {
                let address = oracle.address(store)?;
                println!("{address}");
                println!("{:#?}", program.account::<states::Oracle>(address).await?);
            }
            Command::TokenConfig { token } => {
                let store = store.wrap_err("missing store address")?;
                let config = get_token_config(&program, store, token)
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
                use gmsol::pyth::Hermes;
                use pyth_sdk::Identifier;

                let hermes = Hermes::default();

                let feed_ids = feed_ids
                    .iter()
                    .map(|id| {
                        let hex = id.strip_prefix("0x").unwrap_or(id);
                        Identifier::from_hex(hex).map_err(gmsol::Error::unknown)
                    })
                    .collect::<gmsol::Result<Vec<_>>>()?;

                let stream = hermes.price_updates(feed_ids).await?;
                futures_util::pin_mut!(stream);
                while let Some(update) = stream.try_next().await? {
                    tracing::info!("{update:#?}");
                }
            }
        }
        Ok(())
    }
}
