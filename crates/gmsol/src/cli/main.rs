use std::sync::Arc;

use admin::AdminArgs;
use anchor_client::{
    solana_sdk::{
        commitment_config::CommitmentConfig,
        pubkey::Pubkey,
        signature::{read_keypair_file, Keypair},
        signer::Signer,
    },
    Client, Cluster,
};
use clap::Parser;
use eyre::eyre;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

mod admin;
mod exchange;
mod inspect;
mod keeper;
mod roles;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Path to the wallet.
    #[arg(long, short, env, default_value = "~/.config/solana/id.json")]
    wallet: String,
    /// Cluster to connect to.
    #[arg(long, short, env, default_value = "devnet")]
    cluster: String,
    /// Commitment level.
    #[arg(long, env, default_value = "confirmed")]
    commitment: CommitmentConfig,
    /// The address of the `DataStore` account.
    #[arg(long, env)]
    store: Option<Pubkey>,
    /// Commands.
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand)]
#[allow(clippy::large_enum_variant)]
enum Command {
    /// Show current wallet pubkey.
    Whoami,
    /// Commands for admin.
    Admin(AdminArgs),
    /// Inspect the accounts defined by `DataStore` program.
    Inspect(inspect::InspectArgs),
    /// Commands for roles management.
    Roles(roles::RolesArgs),
    /// Commands for `Exchange` program.
    Exchange(exchange::ExchangeArgs),
    /// Coomands for keepers.
    Keeper(keeper::KeeperArgs),
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();
    Cli::parse().run().await?;
    Ok(())
}

type SharedClient = Arc<Client<Arc<Keypair>>>;

impl Cli {
    fn wallet(&self) -> eyre::Result<Arc<Keypair>> {
        let payer = read_keypair_file(&*shellexpand::full(&self.wallet)?)
            .map_err(|err| eyre!("Failed to read keypair: {err}"))?;
        Ok(Arc::new(payer))
    }

    fn cluster(&self) -> eyre::Result<Cluster> {
        self.cluster
            .parse()
            .map_err(|err| eyre!("Invalid cluster: {err}"))
    }

    fn client(&self) -> eyre::Result<(SharedClient, Pubkey)> {
        let cluster = self.cluster()?;
        tracing::debug!("using cluster: {cluster}");
        let wallet = self.wallet()?;
        let payer = wallet.pubkey();
        tracing::debug!("using wallet: {}", payer);
        let commitment = self.commitment;
        tracing::debug!("using commitment config: {}", commitment.commitment);
        let client = Client::new_with_options(cluster, wallet, self.commitment);
        Ok((Arc::new(client), payer))
    }

    async fn run(&self) -> eyre::Result<()> {
        let (client, payer) = self.client()?;
        match &self.command {
            Command::Whoami => {
                println!("{payer}");
            }
            Command::Admin(args) => args.run(&client, self.store.as_ref()).await?,
            Command::Inspect(args) => args.run(&client, self.store.as_ref()).await?,
            Command::Roles(args) => {
                let store = self.store.ok_or(eyre::eyre!("missing store address"))?;
                args.run(&client, &store).await?
            }
            Command::Exchange(args) => {
                let store = self.store.ok_or(eyre::eyre!("missing store address"))?;
                args.run(&client, &store).await?
            }
            Command::Keeper(args) => {
                let store = self.store.ok_or(eyre::eyre!("missing store address"))?;
                args.run(&client, &store).await?
            }
        }
        Ok(())
    }
}
