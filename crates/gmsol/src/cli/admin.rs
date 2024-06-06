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
                crate::utils::send_or_serialize(
                    client.initialize_store(key),
                    serialize_only,
                    |signature| {
                        tracing::info!("initialized a new data store at tx {signature}");
                        Ok(())
                    },
                )
                .await?;
            }
            (Command::EnableRole { role }, Some(store)) => {
                crate::utils::send_or_serialize(
                    client.enable_role(store, role),
                    serialize_only,
                    |signature| {
                        tracing::info!("enabled role `{role}` at tx {signature}");
                        Ok(())
                    },
                )
                .await?;
            }
            (Command::GrantRole { role, authority }, Some(store)) => {
                crate::utils::send_or_serialize(
                    client.grant_role(store, authority, role),
                    serialize_only,
                    |signature| {
                        tracing::info!("grant a role for user {authority} at tx {signature}");
                        Ok(())
                    },
                )
                .await?;
            }
            (_, None) => return Err(gmsol::Error::unknown("missing `store` address")),
        }
        Ok(())
    }
}
