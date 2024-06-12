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
    InitializeStore,
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
        store_key: &str,
        serialize_only: bool,
    ) -> gmsol::Result<()> {
        let store = client.find_store_address(store_key);
        match &self.command {
            Command::InitializeStore => {
                println!("Initialize store with key={store_key}, address={store}",);
                crate::utils::send_or_serialize(
                    client.initialize_store(store_key),
                    serialize_only,
                    |signature| {
                        tracing::info!("initialized a new data store at tx {signature}");
                        Ok(())
                    },
                )
                .await?;
            }
            Command::EnableRole { role } => {
                crate::utils::send_or_serialize(
                    client.enable_role(&store, role),
                    serialize_only,
                    |signature| {
                        tracing::info!("enabled role `{role}` at tx {signature}");
                        Ok(())
                    },
                )
                .await?;
            }
            Command::GrantRole { role, authority } => {
                crate::utils::send_or_serialize(
                    client.grant_role(&store, authority, role),
                    serialize_only,
                    |signature| {
                        tracing::info!("grant a role for user {authority} at tx {signature}");
                        Ok(())
                    },
                )
                .await?;
            }
        }
        Ok(())
    }
}
