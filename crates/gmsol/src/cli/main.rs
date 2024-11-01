use admin::AdminArgs;
use anchor_client::{
    solana_sdk::{
        commitment_config::CommitmentConfig,
        pubkey::Pubkey,
        signature::{read_keypair_file, NullSigner},
        signer::Signer,
    },
    Cluster,
};
use clap::Parser;
use eyre::eyre;
use gmsol::{store::utils::read_store, utils::SignerRef};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

mod admin;
mod controller;
mod exchange;
mod feature_keeper;
mod gt;
mod inspect;
mod market_keeper;
mod order_keeper;
mod ser;
mod treasury;
mod utils;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Path to the wallet.
    #[arg(long, short, env, default_value = "~/.config/solana/id.json")]
    wallet: String,
    /// Cluster to connect to.
    #[arg(long = "url", short = 'u', env, default_value = "devnet")]
    cluster: String,
    /// Commitment level.
    #[arg(long, env, default_value = "confirmed")]
    commitment: CommitmentConfig,
    /// The address of the `Store` account.
    #[arg(long, env, group = "store-address")]
    store_address: Option<Pubkey>,
    /// The key fo the `Store` account to use.
    #[arg(
        long = "store",
        short = 's',
        group = "store-address",
        default_value = ""
    )]
    store: String,
    /// `DataStore` Program ID.
    #[arg(long, env)]
    store_program: Option<Pubkey>,
    /// `Exchange` Program ID.
    #[arg(long, env)]
    exchange_program: Option<Pubkey>,
    /// Print the Based64 encoded serialized instructions,
    /// instead of sending the transaction.
    #[arg(long, group = "tx-opts")]
    serialize_only: bool,
    /// Whether to skip preflight.
    #[arg(long, group = "ts-opts")]
    skip_preflight: bool,
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
    /// Commands for treasury.
    Treasury(treasury::Args),
    /// Inspect the accounts defined by `DataStore` program.
    Inspect(inspect::InspectArgs),
    /// Commands for `Exchange` program.
    Exchange(exchange::ExchangeArgs),
    /// Commands for ORDER_KEEPER.
    Order(order_keeper::KeeperArgs),
    /// Commands for MARKET_KEEPER.
    Market(market_keeper::Args),
    /// Commands for GT.
    Gt(gt::Args),
    /// Commands for CONTROLLER.
    Controller(controller::ControllerArgs),
    /// Commands for FEATURE_KEEPER.
    Feature(feature_keeper::Args),
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .with_writer(std::io::stderr)
        .init();
    Cli::parse().run().await?;
    Ok(())
}

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

    async fn store(&self, client: &GMSOLClient) -> eyre::Result<(Pubkey, String)> {
        if let Some(address) = self.store_address {
            let store = read_store(&client.data_store().solana_rpc(), &address).await?;
            Ok((address, store.key()?.to_owned()))
        } else {
            let store = client.find_store_address(&self.store);
            Ok((store, self.store.clone()))
        }
    }

    fn options(&self) -> gmsol::ClientOptions {
        gmsol::ClientOptions::builder()
            .commitment(self.commitment)
            .data_store_program_id(self.store_program)
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
        let client = self.gmsol_client()?;
        let (store, store_key) = self.store(&client).await?;
        match &self.command {
            Command::Whoami => {
                println!("{}", client.payer());
            }
            Command::Admin(args) => args.run(&client, &store_key, self.serialize_only).await?,
            Command::Treasury(args) => args.run(&client, &store, self.serialize_only).await?,
            Command::Inspect(args) => args.run(&client, &store).await?,
            Command::Exchange(args) => {
                if self.serialize_only {
                    eyre::bail!("serialize-only mode not supported");
                }
                args.run(&client, &store).await?
            }
            Command::Order(args) => args.run(&client, &store, self.serialize_only).await?,
            Command::Market(args) => args.run(&client, &store, self.serialize_only).await?,
            Command::Gt(args) => {
                args.run(&client, &store, self.serialize_only, self.skip_preflight)
                    .await?
            }
            Command::Controller(args) => args.run(&client, &store, self.serialize_only).await?,
            Command::Feature(args) => args.run(&client, &store, self.serialize_only).await?,
        }
        client.shutdown().await?;
        Ok(())
    }
}
