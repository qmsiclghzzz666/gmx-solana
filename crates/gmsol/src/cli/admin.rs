use anchor_client::solana_sdk::pubkey::Pubkey;
use gmsol::{
    client::SystemProgramOps,
    exchange::ExchangeOps,
    store::{roles::RolesOps, store_ops::StoreOps},
    utils::TransactionBuilder,
};
use gmsol_store::states::RoleKey;
use indexmap::IndexSet;

use crate::GMSOLClient;

#[derive(clap::Args)]
pub(super) struct AdminArgs {
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    /// Create a new data store.
    CreateStore {
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
    /// Initialize roles and controller account.
    InitRolesAndController(InitializeRolesAndController),
    /// Initialize Controller.
    InitController,
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
            Command::InitRolesAndController(args) => {
                args.run(client, store_key, serialize_only).await?
            }
            Command::CreateStore { admin } => {
                tracing::info!(
                    "Initialize store with key={store_key}, address={store}, admin={}",
                    admin.unwrap_or(client.payer())
                );
                crate::utils::send_or_serialize(
                    client
                        .initialize_store(store_key, admin.as_ref())
                        .into_anchor_request_without_compute_budget(),
                    serialize_only,
                    |signature| {
                        tracing::info!("initialized a new data store at tx {signature}");
                        println!("{store}");
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
                        rpc.into_anchor_request_without_compute_budget(),
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
                    let transaction = rpc.into_anchor_request().signed_transaction().await?;
                    let response = client
                        .data_store()
                        .solana_rpc()
                        .simulate_transaction(&transaction)
                        .await
                        .map_err(anchor_client::ClientError::from)?;
                    tracing::info!("Simulation result: {:#?}", response.value);
                    if response.value.err.is_none() {
                        tracing::info!("The simulation was successful, but this operation is very dangerous. If you are sure you want to proceed, please reauthorize the command with `--send` flag");
                    }
                }
            }
            Command::EnableRole { role } => {
                crate::utils::send_or_serialize(
                    client
                        .enable_role(&store, role)
                        .into_anchor_request_without_compute_budget(),
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
                        .into_anchor_request_without_compute_budget(),
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
                        .into_anchor_request_without_compute_budget(),
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
                        .into_anchor_request_without_compute_budget(),
                    serialize_only,
                    |signature| {
                        tracing::info!("revoked a role for user {authority} at tx {signature}");
                        Ok(())
                    },
                )
                .await?;
            }
            Command::InitController => {
                crate::utils::send_or_serialize(
                    client.initialize_controller(&store).into_anchor_request(),
                    serialize_only,
                    |signature| {
                        tracing::info!("initialized the controller account at tx {signature}");
                        Ok(())
                    },
                )
                .await?
            }
        }
        Ok(())
    }
}

#[derive(clap::Args)]
struct InitializeRolesAndController {
    #[arg(long)]
    init_store: bool,
    #[arg(long)]
    not_init_controller: bool,
    #[arg(long)]
    market_keeper: Pubkey,
    #[arg(long)]
    order_keeper: Vec<Pubkey>,
    #[arg(long)]
    allow_multiple_transactions: bool,
    #[arg(long)]
    skip_preflight: bool,
    #[arg(long, value_name = "LAMPORTS")]
    fund_the_controller: Option<u64>,
    #[arg(long)]
    max_transaction_size: Option<usize>,
}

impl InitializeRolesAndController {
    async fn run(
        &self,
        client: &GMSOLClient,
        store_key: &str,
        serialize_only: bool,
    ) -> gmsol::Result<()> {
        let store = client.find_store_address(store_key);
        let controller = client.controller_address(&store);

        let mut builder = TransactionBuilder::new_with_options(
            client.data_store().solana_rpc(),
            !self.allow_multiple_transactions,
            self.max_transaction_size,
        );

        if self.init_store {
            // Insert initialize store instruction.
            builder.try_push(client.initialize_store(store_key, None))?;
        }

        if !self.not_init_controller {
            // Insert initialize controller instuction.
            builder.try_push(client.initialize_controller(&store))?;
        }

        builder
            .try_push(client.enable_role(&store, RoleKey::CONTROLLER))?
            .try_push(client.enable_role(&store, RoleKey::MARKET_KEEPER))?
            .try_push(client.enable_role(&store, RoleKey::ORDER_KEEPER))?
            .try_push(client.grant_role(&store, &controller, RoleKey::CONTROLLER))?
            .try_push(client.grant_role(&store, &self.market_keeper, RoleKey::MARKET_KEEPER))?;

        for keeper in self.unique_order_keepers() {
            builder.try_push(client.grant_role(&store, keeper, RoleKey::ORDER_KEEPER))?;
        }

        if let Some(lamports) = self.fund_the_controller {
            builder.try_push(client.transfer(&controller, lamports)?)?;
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
        self.order_keeper.iter().collect::<IndexSet<_>>()
    }
}
