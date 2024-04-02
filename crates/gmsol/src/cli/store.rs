use anchor_client::solana_sdk::pubkey::Pubkey;
use data_store::states;
use eyre::ContextCompat;
use gmsol::store::roles::RolesOps;

use crate::SharedClient;

#[derive(clap::Args)]
pub(super) struct StoreArgs {
    /// The address of the `DataStore` account.
    #[arg(long, env = "STORE")]
    address: Option<Pubkey>,
    #[command(subcommand)]
    action: Action,
}

#[derive(clap::Subcommand)]
enum Action {
    /// Inspect.
    Inspect { kind: Kind, address: Option<Pubkey> },
    /// Roles account actions.
    Roles {
        #[command(subcommand)]
        action: Option<RolesAction>,
    },
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

#[derive(clap::Subcommand)]
enum RolesAction {
    /// Get.
    Get,
    /// Init.
    Init,
    /// Grant,
    Grant {
        /// User.
        #[arg(long)]
        user: Pubkey,
        /// Role.
        #[arg(long)]
        role: String,
    },
}

impl StoreArgs {
    pub(super) async fn run(&self, client: &SharedClient) -> eyre::Result<()> {
        let program = client.program(data_store::id())?;

        match &self.action {
            Action::Inspect { kind, address } => match kind {
                Kind::DataStore => {
                    let address = address.unwrap_or(*self.store()?);
                    println!(
                        "{:#?}",
                        program.account::<states::DataStore>(address).await?
                    );
                }
                Kind::Roles => {
                    println!(
                        "{:#?}",
                        program
                            .account::<states::Roles>(address.wrap_err("address not provided")?)
                            .await?
                    );
                }
                Kind::TokenConfigMap => {
                    println!(
                        "{:#?}",
                        program
                            .account::<states::TokenConfigMap>(
                                address.wrap_err("address not provided")?
                            )
                            .await?
                    );
                }
                Kind::Market => {
                    println!(
                        "{:#?}",
                        program
                            .account::<states::Market>(address.wrap_err("address not provided")?)
                            .await?
                    );
                }
                Kind::Deposit => {
                    println!(
                        "{:#?}",
                        program
                            .account::<states::Deposit>(address.wrap_err("address not provided")?)
                            .await?
                    );
                }
                Kind::Withdrawal => {
                    println!(
                        "{:#?}",
                        program
                            .account::<states::Withdrawal>(
                                address.wrap_err("address not provided")?
                            )
                            .await?
                    );
                }
            },
            Action::Roles { action } => match action {
                Some(RolesAction::Get) | None => {
                    let address = program
                        .find_roles_address(self.store()?, &program.payer())
                        .0;
                    println!("{address}");
                }
                Some(RolesAction::Init) => {
                    let signature = program.initialize_roles(self.store()?, None).send().await?;
                    tracing::info!("initialized a new roles account at {signature}");
                }
                Some(RolesAction::Grant { role, user }) => {
                    let signature = program.grant_role(self.store()?, user, role).send().await?;
                    tracing::info!("grant a role for user {user} at {signature}");
                }
            },
        }
        Ok(())
    }

    fn store(&self) -> eyre::Result<&Pubkey> {
        self.address
            .as_ref()
            .wrap_err("`DataStore` account not provided")
    }
}
