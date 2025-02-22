use gmsol::{
    cli::wallet::load_keypair,
    utils::{shared_signer, SignerRef},
};
use gmsol_solana_utils::cluster::Cluster;
use serde_with::DisplayFromStr;
use solana_sdk::pubkey::Pubkey;

const ENV_GMSOL_IT: &str = "GMSOL_IT";
const GMSOL_IT_PREFIX: &str = "GMSOL_IT_";

/// Config for [`TestClient`].
#[serde_with::serde_as]
#[derive(serde::Deserialize)]
pub struct Config {
    #[serde(default = "default_cluster")]
    cluster: Cluster,
    wallet: String,
    #[serde(default)]
    keeper: Option<String>,
    #[serde(default)]
    #[serde_as(as = "Option<DisplayFromStr>")]
    store: Option<Pubkey>,
    #[serde(default)]
    #[serde_as(as = "Option<DisplayFromStr>")]
    oracle: Option<Pubkey>,
}

fn default_cluster() -> Cluster {
    Cluster::Devnet
}

type Client = gmsol::Client<SignerRef>;

/// A test client for integration test.
pub struct TestClient {
    client: Client,
    keeper_client: Option<Client>,
    store: Pubkey,
    oracle: Option<Pubkey>,
}

impl TestClient {
    /// Get client.
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Get keeper client.
    pub fn keeper_client(&self) -> Option<&Client> {
        self.keeper_client.as_ref()
    }

    /// Get store.
    pub fn store(&self) -> &Pubkey {
        &self.store
    }

    /// Get oracle.
    pub fn oracle(&self) -> Option<&Pubkey> {
        self.oracle.as_ref()
    }

    /// From envs.
    pub fn from_envs() -> eyre::Result<Option<Self>> {
        use figment::{
            providers::{Env, Format, Toml},
            Figment,
        };
        use std::env;

        let config = match env::var(ENV_GMSOL_IT) {
            Ok(path) => {
                tracing::trace!("Using config: {path}");
                Some(
                    Figment::new()
                        .merge(Toml::file(path))
                        .merge(Env::prefixed(GMSOL_IT_PREFIX))
                        .extract::<Config>()?,
                )
            }
            Err(_) => Figment::new()
                .merge(Env::prefixed(GMSOL_IT_PREFIX))
                .extract()
                .ok(),
        };
        let Some(config) = config else {
            return Ok(None);
        };

        let payer = load_keypair(&config.wallet)?;
        let client = gmsol::Client::new(config.cluster.clone(), shared_signer(payer))?;

        let keeper_client = config
            .keeper
            .as_ref()
            .map(|wallet| load_keypair(wallet))
            .transpose()?
            .map(|payer| gmsol::Client::new(config.cluster, shared_signer(payer)))
            .transpose()?;

        let store = config.store.unwrap_or(client.find_store_address(""));

        if keeper_client.is_some() && config.oracle.is_none() {
            eyre::bail!("oracle is required when the keeper is provided");
        }

        Ok(Some(Self {
            client,
            keeper_client,
            store,
            oracle: config.oracle,
        }))
    }
}
