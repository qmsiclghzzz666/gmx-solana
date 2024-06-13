use std::collections::HashSet;

use anchor_client::solana_sdk::{pubkey::Pubkey, signature::Keypair};
use data_store::states::RoleKey;
use gmsol::{
    store::{data_store::StoreOps, roles::RolesOps, token_config::TokenConfigOps},
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
    InitializeAll(InitializeAll),
    /// Set token map.
    SetTokenMap { token_map: Pubkey },
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
            Command::InitializeAll(args) => args.run(client, store_key, serialize_only).await?,
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
                    client
                        .enable_role(&store, role)
                        .build_without_compute_budget(),
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
                    client
                        .grant_role(&store, authority, role)
                        .build_without_compute_budget(),
                    serialize_only,
                    |signature| {
                        tracing::info!("grant a role for user {authority} at tx {signature}");
                        Ok(())
                    },
                )
                .await?;
            }
            Command::SetTokenMap { token_map } => {
                crate::utils::send_or_serialize(
                    client
                        .set_token_map(&store, token_map)
                        .build_without_compute_budget(),
                    serialize_only,
                    |signature| {
                        tracing::info!("set new token map at {signature}");
                        Ok(())
                    },
                )
                .await?;
            }
        }
        Ok(())
    }
}

#[derive(clap::Args)]
struct InitializeAll {
    #[arg(long)]
    skip_init_store: bool,
    #[arg(long)]
    skip_init_token_map: bool,
    #[arg(long)]
    market_keeper: Option<Pubkey>,
    #[arg(long)]
    order_keeper: Vec<Pubkey>,
    #[arg(long)]
    force_one_transaction: bool,
    #[arg(long)]
    skip_preflight: bool,
}

impl InitializeAll {
    async fn run(
        &self,
        client: &GMSOLClient,
        store_key: &str,
        serialize_only: bool,
    ) -> gmsol::Result<()> {
        let store = client.find_store_address(store_key);
        let controller = client.controller_address(&store);
        let token_map = (!self.skip_init_token_map).then(Keypair::new);
        let admin = client.payer();

        let mut builder = TransactionBuilder::new_with_force_one_transaction(
            client.data_store().async_rpc(),
            self.force_one_transaction,
        );

        if !self.skip_init_store {
            // Insert initialize store instruction.
            builder.try_push(client.initialize_store(store_key))?;
        }

        builder
            .try_push(client.enable_role(&store, RoleKey::CONTROLLER))?
            .try_push(client.enable_role(&store, RoleKey::MARKET_KEEPER))?
            .try_push(client.enable_role(&store, RoleKey::ORDER_KEEPER))?
            .try_push(client.grant_role(&store, &controller, RoleKey::CONTROLLER))?;

        if let Some(market_keeper) = self.market_keeper {
            builder.try_push(client.grant_role(&store, &market_keeper, RoleKey::MARKET_KEEPER))?;
        }

        for keeper in self.unique_order_keepers() {
            builder.try_push(client.grant_role(&store, keeper, RoleKey::ORDER_KEEPER))?;
        }

        if let Some(token_map) = token_map.as_ref() {
            let (rpc, token_map) = client.initialize_token_map(&store, token_map);
            builder
                .try_push(client.grant_role(&store, &admin, RoleKey::MARKET_KEEPER))?
                .try_push(rpc)?
                .try_push(client.set_token_map(&store, &token_map))?
                .try_push(client.revoke_role(&store, &admin, RoleKey::MARKET_KEEPER))?;
        }

        crate::utils::send_or_serialize_transactions(
            builder,
            serialize_only,
            self.skip_preflight,
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

    fn unique_order_keepers(&self) -> impl IntoIterator<Item = &Pubkey> {
        self.order_keeper.iter().collect::<HashSet<_>>()
    }
}
