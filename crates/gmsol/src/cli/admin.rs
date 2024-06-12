use anchor_client::solana_sdk::pubkey::Pubkey;
use gmsol::{
    store::{data_store::StoreOps, roles::RolesOps},
    utils::TransactionBuilder,
};

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
    /// Initialize all.
    InitializeAll {
        #[arg(long)]
        skip_init_store: bool,
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
            Command::InitializeAll {
                skip_init_store: skip_store_initialization,
            } => {
                self.initialize_all(
                    client,
                    store_key,
                    serialize_only,
                    *skip_store_initialization,
                )
                .await?
            }
            Command::InitializeStore => {
                println!("Initialize store with key={store_key}, address={store}",);
                crate::utils::send_or_serialize(
                    client
                        .initialize_store(store_key)
                        .build_without_compute_budget(),
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

    async fn initialize_all(
        &self,
        client: &GMSOLClient,
        store_key: &str,
        serialize_only: bool,
        skip_store_initialization: bool,
    ) -> gmsol::Result<()> {
        // let store = client.find_store_address(store_key);

        let mut builder = TransactionBuilder::new(client.data_store().async_rpc());

        if !skip_store_initialization {
            // Insert initialize store instruction.
            builder.try_push(client.initialize_store(store_key))?;
        }

        crate::utils::send_or_serialize_transactions(
            builder,
            serialize_only,
            |signatures, error| {
                println!("{signatures:#?}");
                match error {
                    None => Ok(()),
                    Some(err) => Err(err),
                }
            },
        )
        .await?;
        Ok(())
    }
}
