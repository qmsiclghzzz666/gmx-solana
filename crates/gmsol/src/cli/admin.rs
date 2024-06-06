use anchor_client::solana_sdk::pubkey::Pubkey;
use gmsol::store::{data_store::StoreOps, roles::RolesOps};

use crate::GMSOLClient;

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
        client: &GMSOLClient,
        store: Option<&Pubkey>,
        serialize_only: bool,
    ) -> gmsol::Result<()> {
        match (&self.command, store) {
            (Command::InitializeStore { key }, _) => {
                let key = key.as_deref().unwrap_or_default();
                println!("Initialize store: {}", client.find_store_address(key));
                let req = client.initialize_store(key);
                if serialize_only {
                    for ix in req.instructions()? {
                        println!("{}", gmsol::utils::serialize_instruction(&ix)?);
                    }
                } else {
                    let signature = client.initialize_store(key).send().await?;
                    tracing::info!("initialized a new data store at tx {signature}");
                }
            }
            (Command::EnableRole { role }, Some(store)) => {
                let signature = client.enable_role(store, role).send().await?;
                tracing::info!("enabled role `{role}` at tx {signature}");
            }
            (Command::GrantRole { role, authority }, Some(store)) => {
                let signature = client.grant_role(store, authority, role).send().await?;
                tracing::info!("grant a role for user {authority} at tx {signature}");
            }
            (_, None) => return Err(gmsol::Error::unknown("missing `store` address")),
        }
        Ok(())
    }
}
