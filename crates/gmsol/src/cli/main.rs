use std::sync::Arc;

use anchor_client::{
    solana_sdk::{
        signature::{read_keypair_file, Keypair},
        signer::Signer,
    },
    Client, Cluster,
};
use clap::Parser;
use eyre::eyre;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

mod store;

#[derive(Parser)]
struct Cli {
    /// Path to the wallet.
    #[arg(long, short, env, default_value = "~/.config/solana/id.json")]
    wallet: String,
    /// Cluster to connect to.
    #[arg(long, short, env, default_value = "devnet")]
    cluster: String,
    /// Commands.
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    /// Commands for `DataStore` program.
    Store(store::StoreArgs),
}

fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();
    Cli::try_parse()?.run()?;
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

    fn client(&self) -> eyre::Result<SharedClient> {
        let cluster = self.cluster()?;
        tracing::info!("using cluster: {cluster}");
        let wallet = self.wallet()?;
        tracing::info!("using wallet: {}", wallet.pubkey());
        let client = Client::new(cluster, wallet);
        Ok(Arc::new(client))
    }

    fn run(&self) -> eyre::Result<()> {
        let client = self.client()?;
        match &self.command {
            Command::Store(args) => args.run(&client)?,
        }
        Ok(())
    }
}
