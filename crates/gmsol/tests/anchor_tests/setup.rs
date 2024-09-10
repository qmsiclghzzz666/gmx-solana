use std::fmt;

use anchor_client::{
    solana_sdk::{
        pubkey::Pubkey,
        signature::Keypair,
        signer::{EncodableKey, Signer},
    },
    Cluster,
};
use gmsol::{
    utils::{shared_signer, SignerRef},
    Client,
};
use tokio::sync::OnceCell;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

/// Deployment.
pub struct Deployment {
    /// Client.
    pub client: Client<SignerRef>,
    /// Users.
    pub users: Users,
    /// Keepers.
    pub keepers: Keepers,
    /// Store.
    pub store: Pubkey,
    /// Token Map.
    pub token_map: Keypair,
    /// Oracle.
    pub oracle: Pubkey,
}

impl fmt::Debug for Deployment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Deployment")
            .field("cluster", self.client.cluster())
            .field("payer", &self.client.payer())
            .field("users", &self.users)
            .field("keepers", &self.keepers)
            .field("store", &self.store)
            .field("token_map", &self.token_map.pubkey())
            .field("oracle", &self.oracle)
            .finish_non_exhaustive()
    }
}

impl Deployment {
    async fn connect() -> eyre::Result<Self> {
        let (client, store) = Self::get_client_and_store().await?;
        let oracle = client.find_oracle_address(&store, 255);
        Ok(Self {
            client,
            users: Default::default(),
            keepers: Default::default(),
            store,
            token_map: Keypair::new(),
            oracle,
        })
    }

    async fn init() -> eyre::Result<Self> {
        Self::init_tracing()?;

        let client = Self::connect().await?;

        client.setup().await?;

        Ok(client)
    }

    async fn get_client_and_store() -> eyre::Result<(Client<SignerRef>, Pubkey)> {
        use rand::{distributions::Alphanumeric, thread_rng, Rng};
        use std::env;

        let endpoint = env::var("ANCHOR_PROVIDER_URL")
            .map_err(|_| eyre::Error::msg("env `ANCHOR_PROVIDER_URL` is not set"))?;
        let wallet = env::var("ANCHOR_WALLET")
            .map_err(|_| eyre::Error::msg("env `ANCHOR_WALLET` is not set"))?;
        let wallet = shellexpand::full(&wallet)?;

        let random_store =
            env::var("GMSOL_RANDOM_STORE").is_ok() || endpoint == Cluster::Devnet.url();
        let store_key = random_store
            .then(|| {
                let mut rng = thread_rng();
                (&mut rng)
                    .sample_iter(Alphanumeric)
                    .take(16)
                    .map(char::from)
                    .collect::<String>()
            })
            .unwrap_or_default();

        let client = Client::new(
            endpoint.parse().map_err(eyre::Error::msg)?,
            shared_signer(
                Keypair::read_from_file(&*wallet)
                    .map_err(|err| eyre::Error::msg(err.to_string()))?,
            ),
        )?;
        let store = client.find_store_address(&store_key);
        Ok((client, store))
    }

    fn init_tracing() -> eyre::Result<()> {
        tracing_subscriber::fmt()
            .with_env_filter(
                EnvFilter::builder()
                    .with_default_directive(LevelFilter::INFO.into())
                    .from_env_lossy(),
            )
            .try_init()
            .map_err(eyre::Error::msg)?;
        Ok(())
    }

    async fn setup(&self) -> eyre::Result<()> {
        Ok(())
    }
}

/// Users.
pub struct Users {
    /// User 0.
    pub user_0: Keypair,
}

impl fmt::Debug for Users {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Users")
            .field("user_0", &self.user_0.pubkey())
            .finish()
    }
}

impl Default for Users {
    fn default() -> Self {
        Self {
            user_0: Keypair::new(),
        }
    }
}

/// Keepers.
pub struct Keepers {
    /// Keeper 0
    pub keeper_0: Keypair,
}

impl fmt::Debug for Keepers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Keepers")
            .field("keeper_0", &self.keeper_0.pubkey())
            .finish()
    }
}

impl Default for Keepers {
    fn default() -> Self {
        Self {
            keeper_0: Keypair::new(),
        }
    }
}

/// Get current deployment.
pub async fn current_deployment() -> eyre::Result<&'static Deployment> {
    static DEPLOYMENT: OnceCell<Deployment> = OnceCell::const_new();

    DEPLOYMENT.get_or_try_init(Deployment::init).await
}
