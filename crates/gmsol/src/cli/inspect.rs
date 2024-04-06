use anchor_client::solana_sdk::pubkey::Pubkey;
use data_store::states;
use exchange::utils::ControllerSeeds;
use eyre::{eyre, ContextCompat};

use crate::SharedClient;

#[derive(clap::Args)]
pub(super) struct InspectArgs {
    kind: Kind,
    address: Option<Pubkey>,
}

#[derive(clap::ValueEnum, Clone, Copy)]
enum Kind {
    /// `DataStore` account.
    DataStore,
    /// `Roles` account.
    Roles,
    /// `TokenConfigMap` account.
    TokenConfigMap,
    /// `Market` account.
    Market,
    /// `Deposit` account.
    Deposit,
    /// `Withdrawal` account.
    Withdrawal,
    /// Get the CONTROLLER address.
    Controller,
}

#[derive(clap::Subcommand)]
enum RolesAction {
    /// Get.
    Get,
    /// Init.
    Init {
        /// Authority.
        #[arg(long)]
        authority: Option<Pubkey>,
    },
    /// Grant,
    Grant {
        /// User.
        #[arg(long)]
        user: Pubkey,
        /// Role.
        #[arg(long)]
        role: String,
    },
}

impl InspectArgs {
    pub(super) async fn run(
        &self,
        client: &SharedClient,
        store: Option<&Pubkey>,
    ) -> eyre::Result<()> {
        let program = client.program(data_store::id())?;
        let address = self.address;
        match self.kind {
            Kind::DataStore => {
                let address = address
                    .or(store.copied())
                    .ok_or(eyre!("missing address for DataStore account"))?;
                println!(
                    "{:#?}",
                    program.account::<states::DataStore>(address).await?
                );
            }
            Kind::Roles => {
                println!(
                    "{:#?}",
                    program
                        .account::<states::Roles>(address.wrap_err("address not provided")?)
                        .await?
                );
            }
            Kind::TokenConfigMap => {
                println!(
                    "{:#?}",
                    program
                        .account::<states::TokenConfigMap>(
                            address.wrap_err("address not provided")?
                        )
                        .await?
                );
            }
            Kind::Market => {
                println!(
                    "{:#?}",
                    program
                        .account::<states::Market>(address.wrap_err("address not provided")?)
                        .await?
                );
            }
            Kind::Deposit => {
                println!(
                    "{:#?}",
                    program
                        .account::<states::Deposit>(address.wrap_err("address not provided")?)
                        .await?
                );
            }
            Kind::Withdrawal => {
                println!(
                    "{:#?}",
                    program
                        .account::<states::Withdrawal>(address.wrap_err("address not provided")?)
                        .await?
                );
            }
            Kind::Controller => {
                let controller =
                    ControllerSeeds::find_with_address(store.wrap_err("missing `store` address")?)
                        .1;
                println!("{controller}");
            }
        }
        Ok(())
    }
}
