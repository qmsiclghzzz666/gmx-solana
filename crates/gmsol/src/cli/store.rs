use anchor_client::solana_sdk::pubkey::Pubkey;
use data_store::states;

use crate::SharedClient;

#[derive(clap::Args)]
pub(super) struct StoreArgs {
    action: Action,
    kind: Kind,
    address: Pubkey,
}

#[derive(clap::ValueEnum, Clone, Copy)]
enum Action {
    /// Get.
    Get,
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

        match (&self.kind, &self.action) {
            (Kind::DataStore, Action::Get) => {
                println!("{:#?}", program.account::<states::DataStore>(self.address)?);
            }
            (Kind::Roles, Action::Get) => {
                println!("{:#?}", program.account::<states::Roles>(self.address)?);
            }
            (Kind::TokenConfigMap, Action::Get) => {
                println!(
                    "{:#?}",
                    program.account::<states::TokenConfigMap>(self.address)?
                );
            }
            (Kind::Market, Action::Get) => {
                println!("{:#?}", program.account::<states::Market>(self.address)?);
            }
            (Kind::Deposit, Action::Get) => {
                println!("{:#?}", program.account::<states::Deposit>(self.address)?);
            }
            (Kind::Withdrawal, Action::Get) => {
                println!(
                    "{:#?}",
                    program.account::<states::Withdrawal>(self.address)?
                );
            }
        }
        Ok(())
    }
}
