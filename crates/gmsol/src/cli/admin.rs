use anchor_client::solana_sdk::pubkey::Pubkey;
use gmsol::store::{
    data_store::{find_store_address, StoreOps},
    roles::RolesOps,
};

use crate::SharedClient;

#[derive(clap::Args)]
pub(super) struct AdminArgs {
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    /// Initialize a new data store.
    InitializeStore { key: Option<String> },
    /// Enable a role.
    EnableRole { role: String },
    /// Grant a role to a user.
    GrantRole {
        /// User.
        authority: Pubkey,
        /// Role.
        role: String,
    },
}

impl AdminArgs {
    pub(super) async fn run(
        &self,
        client: &SharedClient,
        store: Option<&Pubkey>,
    ) -> gmsol::Result<()> {
        let program = client.program(data_store::id())?;
        match (&self.command, store) {
            (Command::InitializeStore { key }, _) => {
                let key = key.clone().unwrap_or_else(|| program.payer().to_string());
                let signature = program.initialize_store(&key).send().await?;
                tracing::info!("initialized a new data store at tx {signature}");
                println!("{}", find_store_address(&key).0);
            }
            (Command::EnableRole { role }, Some(store)) => {
                let signature = program.enable_role(store, role).send().await?;
                tracing::info!("enabled role `{role}` at tx {signature}");
            }
            (Command::GrantRole { role, authority }, Some(store)) => {
                let signature = program.grant_role(store, authority, role).send().await?;
                tracing::info!("grant a role for user {authority} at tx {signature}");
            }
            (_, None) => return Err(gmsol::Error::unknown("missing `store` address")),
        }
        Ok(())
    }
}
