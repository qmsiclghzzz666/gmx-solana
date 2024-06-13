use std::collections::HashSet;

use anchor_client::solana_sdk::{pubkey::Pubkey, signature::Keypair};
use data_store::states::RoleKey;
use gmsol::{
    store::{roles::RolesOps, store_ops::StoreOps, token_config::TokenConfigOps},
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
    InitializeStore {
        #[arg(long)]
        admin: Option<Pubkey>,
    },
    /// Transfer store authority.
    TransferStoreAuthority {
        #[arg(long)]
        new_authority: Pubkey,
        #[arg(long)]
        send: bool,
    },
    /// Enable a role.
    EnableRole { role: String },
    /// Disable a role.
    DisableRole { role: String },
    /// Grant a role to a user.
    GrantRole {
        /// User.
        authority: Pubkey,
        /// Role.
        role: String,
    },
    /// Revoke a role from the user.
    RevokeRole {
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
            Command::InitializeStore { admin } => {
                println!(
                    "Initialize store with key={store_key}, address={store}, admin={}",
                    admin.unwrap_or(client.payer())
                );
                crate::utils::send_or_serialize(
                    client
                        .initialize_store(store_key, admin.as_ref())
                        .build_without_compute_budget(),
                    serialize_only,
                    |signature| {
                        tracing::info!("initialized a new data store at tx {signature}");
                        Ok(())
                    },
                )
                .await?;
            }
            Command::TransferStoreAuthority {
                new_authority,
                send,
            } => {
                let rpc = client.transfer_store_authority(&store, new_authority);
                if *send || serialize_only {
                    crate::utils::send_or_serialize(
                        rpc.build_without_compute_budget(),
                        serialize_only,
                        |signature| {
                            tracing::info!(
                                "transferred store authority to `{new_authority}` at tx {signature}"
                            );
                            Ok(())
                        },
                    )
                    .await?;
                } else {
                    let transaction = rpc.build().signed_transaction().await?;
                    let response = client
                        .data_store()
                        .async_rpc()
                        .simulate_transaction(&transaction)
                        .await
                        .map_err(anchor_client::ClientError::from)?;
                    println!("Simulation result: {:#?}", response.value);
                    if response.value.err.is_none() {
                        println!("The simulation was successful, but this operation is very dangerous. If you are sure you want to proceed, please reauthorize the command with `--send` flag");
                    }
                }
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
            Command::DisableRole { role } => {
                crate::utils::send_or_serialize(
                    client
                        .disable_role(&store, role)
                        .build_without_compute_budget(),
                    serialize_only,
                    |signature| {
                        tracing::info!("disabled role `{role}` at tx {signature}");
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
                        tracing::info!("granted a role for user {authority} at tx {signature}");
                        Ok(())
                    },
                )
                .await?;
            }
            Command::RevokeRole { role, authority } => {
                crate::utils::send_or_serialize(
                    client
                        .revoke_role(&store, authority, role)
                        .build_without_compute_budget(),
                    serialize_only,
                    |signature| {
                        tracing::info!("revoked a role for user {authority} at tx {signature}");
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
    allow_multiple_transactions: bool,
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
            !self.allow_multiple_transactions,
        );

        if !self.skip_init_store {
            // Insert initialize store instruction.
            builder.try_push(client.initialize_store(store_key, None))?;
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
