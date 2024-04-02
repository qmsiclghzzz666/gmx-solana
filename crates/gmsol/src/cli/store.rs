use anchor_client::solana_sdk::pubkey::Pubkey;
use data_store::states;

use crate::SharedClient;

#[derive(clap::Args)]
pub(super) struct StoreArgs {
    #[command(subcommand)]
    action: Action,
}

#[derive(clap::Subcommand)]
enum Action {
    /// Get.
    Get { kind: Kind, address: Pubkey },
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
}

impl StoreArgs {
    pub(super) fn run(&self, client: &SharedClient) -> eyre::Result<()> {
        let program = client.program(data_store::id())?;

        match &self.action {
            Action::Get { kind, address } => match kind {
                Kind::DataStore => {
                    println!("{:#?}", program.account::<states::DataStore>(*address)?);
                }
                Kind::Roles => {
                    println!("{:#?}", program.account::<states::Roles>(*address)?);
                }
                Kind::TokenConfigMap => {
                    println!(
                        "{:#?}",
                        program.account::<states::TokenConfigMap>(*address)?
                    );
                }
                Kind::Market => {
                    println!("{:#?}", program.account::<states::Market>(*address)?);
                }
                Kind::Deposit => {
                    println!("{:#?}", program.account::<states::Deposit>(*address)?);
                }
                Kind::Withdrawal => {
                    println!("{:#?}", program.account::<states::Withdrawal>(*address)?);
                }
            },
        }
        Ok(())
    }
}
