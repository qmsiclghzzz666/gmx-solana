use anchor_client::solana_sdk::pubkey::Pubkey;
use data_store::states;
use exchange::utils::ControllerSeeds;
use eyre::ContextCompat;
use gmsol::store::data_store::find_market_address;

use crate::SharedClient;

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
    TokenConfigMap { address: Pubkey },
    /// `Market` account.
    Market {
        address: Pubkey,
        /// Consider the address as market address rather than the address of its market token.
        #[arg(long)]
        as_market_address: bool,
    },
    /// `Deposit` account.
    Deposit { address: Pubkey },
    /// `Withdrawal` account.
    Withdrawal { address: Pubkey },
    /// `Oracle` account.
    Oracle { address: Pubkey },
    /// Get the CONTROLLER address.
    Controller,
}

impl InspectArgs {
    pub(super) async fn run(
        &self,
        client: &SharedClient,
        store: Option<&Pubkey>,
    ) -> gmsol::Result<()> {
        let program = client.program(data_store::id())?;
        match self.command {
            Command::DataStore { address } => {
                let address = address.or(store.copied()).ok_or(gmsol::Error::unknown(
                    "missing address for DataStore account",
                ))?;
                println!(
                    "{:#?}",
                    program.account::<states::DataStore>(address).await?
                );
            }
            Command::Roles { address } => {
                println!("{:#?}", program.account::<states::Roles>(address).await?);
            }
            Command::TokenConfigMap { address } => {
                println!(
                    "{:#?}",
                    program.account::<states::TokenConfigMap>(address).await?
                );
            }
            Command::Market {
                mut address,
                as_market_address,
            } => {
                if !as_market_address {
                    address =
                        find_market_address(store.wrap_err("`store` not provided")?, &address).0;
                }
                println!("{:#?}", program.account::<states::Market>(address).await?);
            }
            Command::Deposit { address } => {
                println!("{:#?}", program.account::<states::Deposit>(address).await?);
            }
            Command::Withdrawal { address } => {
                println!(
                    "{:#?}",
                    program.account::<states::Withdrawal>(address).await?
                );
            }
            Command::Controller => {
                let controller =
                    ControllerSeeds::find_with_address(store.wrap_err("missing `store` address")?)
                        .1;
                println!("{controller}");
            }
            Command::Oracle { address } => {
                println!("{:#?}", program.account::<states::Oracle>(address).await?);
            }
        }
        Ok(())
    }
}
