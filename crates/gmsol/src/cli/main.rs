use admin::AdminArgs;
use anchor_client::{
    solana_sdk::{
        commitment_config::CommitmentConfig,
        pubkey::Pubkey,
        signature::{read_keypair_file, NullSigner},
        signer::Signer,
    },
    Client, Cluster,
};
use clap::Parser;
use eyre::{eyre, ContextCompat};
use gmsol::utils::SignerRef;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

mod admin;
mod controller;
mod exchange;
mod inspect;
mod keeper;
mod roles;
mod utils;

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
    /// `DataStore` Program ID.
    #[arg(long, env)]
    store_program: Option<Pubkey>,
    /// `Exchange` Program ID.
    #[arg(long, env)]
    exchange_program: Option<Pubkey>,
    /// Print the Based64 encoded serialized instructions,
    /// instead of sending the transaction.
    #[arg(long)]
    serialize_only: bool,
    /// Use this address as payer.
    ///
    /// Only available in `serialize-only` mode.
    #[arg(long, requires = "serialize_only")]
    payer: Option<Pubkey>,
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
    /// Commands for keepers.
    Keeper(keeper::KeeperArgs),
    /// Commands for controllers.
    Controller(controller::ControllerArgs),
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

type SharedClient = Client<SignerRef>;
type GMSOLClient = gmsol::Client<SignerRef>;

impl Cli {
    fn wallet(&self) -> eyre::Result<SignerRef> {
        if let Some(payer) = self.payer {
            if self.serialize_only {
                let payer = NullSigner::new(&payer);
                Ok(gmsol::utils::shared_signer(payer))
            } else {
                eyre::bail!("Setting payer is only allowed in `serialize-only` mode");
            }
        } else {
            let payer = read_keypair_file(&*shellexpand::full(&self.wallet)?)
                .map_err(|err| eyre!("Failed to read keypair: {err}"))?;
            Ok(gmsol::utils::shared_signer(payer))
        }
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
        Ok((client, payer))
    }

    fn store(&self) -> eyre::Result<&Pubkey> {
        self.store.as_ref().wrap_err("missing store address")
    }

    fn options(&self) -> gmsol::ClientOptions {
        gmsol::ClientOptions::builder()
            .commitment(self.commitment)
            .data_store_program_id(self.store_program)
            .exchange_program_id(self.exchange_program)
            .build()
    }

    fn gmsol_client(&self) -> eyre::Result<GMSOLClient> {
        let cluster = self.cluster()?;
        tracing::debug!("using cluster: {cluster}");
        let wallet = self.wallet()?;
        let payer = wallet.pubkey();
        tracing::debug!("using wallet: {}", payer);
        let commitment = self.commitment;
        tracing::debug!("using commitment config: {}", commitment.commitment);
        let client = gmsol::Client::new_with_options(cluster, wallet, self.options())?;
        Ok(client)
    }

    async fn run(&self) -> eyre::Result<()> {
        let (client, payer) = self.client()?;
        let gmsol = self.gmsol_client()?;
        match &self.command {
            Command::Whoami => {
                println!("{payer}");
            }
            Command::Admin(args) => {
                args.run(&gmsol, self.store.as_ref(), self.serialize_only)
                    .await?
            }
            Command::Inspect(args) => args.run(&gmsol, self.store.as_ref()).await?,
            Command::Roles(args) => args.run(&gmsol, self.store()?, self.serialize_only).await?,
            Command::Exchange(args) => args.run(&client, self.store()?).await?,
            Command::Keeper(args) => args.run(&client, self.store()?).await?,
            Command::Controller(args) => args.run(&client, self.store()?).await?,
        }
        Ok(())
    }
}
