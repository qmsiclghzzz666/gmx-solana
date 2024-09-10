use std::{
    fmt,
    future::Future,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

use anchor_client::{
    solana_sdk::{
        commitment_config::CommitmentConfig,
        pubkey::Pubkey,
        signature::Keypair,
        signer::{EncodableKey, Signer},
        system_instruction,
    },
    Cluster,
};
use event_listener::Event;
use gmsol::{
    utils::{shared_signer, SignerRef, TransactionBuilder},
    Client, ClientOptions,
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

        let client = Client::new_with_options(
            endpoint.parse().map_err(eyre::Error::msg)?,
            shared_signer(
                Keypair::read_from_file(&*wallet)
                    .map_err(|err| eyre::Error::msg(err.to_string()))?,
            ),
            ClientOptions::builder()
                .commitment(CommitmentConfig::confirmed())
                .build(),
        )?;
        let store = client.find_store_address(&store_key);
        Ok((client, store))
    }

    fn init_tracing() -> eyre::Result<()> {
        tracing_subscriber::fmt()
            .with_env_filter(
                EnvFilter::builder()
                    .with_default_directive(LevelFilter::ERROR.into())
                    .from_env_lossy(),
            )
            .try_init()
            .map_err(eyre::Error::msg)?;
        Ok(())
    }

    async fn setup(&self) -> eyre::Result<()> {
        tracing::info!("[Setting up everything...]");
        let _guard = self.use_accounts().await?;

        Ok(())
    }

    pub(crate) async fn use_accounts(&self) -> eyre::Result<Guard> {
        let guard = self.users.use_accounts();

        if self
            .users
            .funded
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            self.fund_users().await?;
        }

        Ok(guard)
    }

    async fn fund_users(&self) -> eyre::Result<()> {
        const LAMPORTS: u64 = 4_000_000_000;

        let client = self.client.data_store().async_rpc();
        let payer = self.client.payer();
        let lamports = client.get_balance(&payer).await?;
        tracing::info!(%payer, "before funding users: {lamports}");

        let mut builder = TransactionBuilder::new(client);
        builder.try_push_many(
            self.users
                .users()
                .into_iter()
                .inspect(|user| tracing::debug!(%user, "funding user with lamports {LAMPORTS}"))
                .map(|user| system_instruction::transfer(&payer, &user, LAMPORTS))
                .map(|ix| self.client.data_store_rpc().pre_instruction(ix)),
            false,
        )?;

        match builder.send_all().await {
            Ok(signatures) => {
                tracing::debug!("funded users with {signatures:#?}");
            }
            Err((signatures, err)) => {
                tracing::error!(%err, "failed to fund users, successful txns: {signatures:#?}");
            }
        }

        Ok(())
    }

    async fn refund_payer(&self) -> eyre::Result<()> {
        let client = self.client.data_store().async_rpc();
        let payer = self.client.payer();

        let mut builder = TransactionBuilder::new(self.client.data_store().async_rpc());

        for user in self.users.keypairs() {
            let lamports = client.get_balance(&user.pubkey()).await?;
            if lamports == 0 {
                continue;
            }
            tracing::debug!(user = %user.pubkey(), %lamports, "refund from user");
            let ix = system_instruction::transfer(&user.pubkey(), &payer, lamports);
            builder
                .try_push(
                    self.client
                        .data_store_rpc()
                        .signer(user)
                        .pre_instruction(ix),
                )
                .map_err(|(_, err)| err)?;
        }

        match builder.send_all().await {
            Ok(signatures) => {
                tracing::debug!("refunded the payer with {signatures:#?}");
            }
            Err((signatures, err)) => {
                tracing::error!(%err, "failed to refund the payer, successful txns: {signatures:#?}");
            }
        }

        self.users.funded.store(false, Ordering::SeqCst);

        let lamports = client.get_balance(&payer).await?;
        tracing::info!(%payer, "after refunding the payer: {lamports}");
        Ok(())
    }

    pub(crate) async fn refund_payer_when_not_in_use(&self, wait: Duration) -> eyre::Result<()> {
        tokio::time::sleep(wait).await;
        self.users.wait_until_not_in_use().await;
        tracing::info!("[Cleanup...]");
        self.refund_payer().await?;
        Ok(())
    }
}

/// Users.
pub struct Users {
    /// User 0.
    pub user_0: Keypair,
    /// Keeper 0.
    pub keeper_0: Keypair,
    funded: Arc<AtomicBool>,
    used: Arc<AtomicUsize>,
    event: Arc<Event>,
}

impl fmt::Debug for Users {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Users")
            .field("user_0", &self.user_0.pubkey())
            .field("keepr_0", &self.keeper_0.pubkey())
            .finish()
    }
}

impl Default for Users {
    fn default() -> Self {
        Self {
            user_0: Keypair::new(),
            keeper_0: Keypair::new(),
            funded: Arc::new(AtomicBool::new(false)),
            used: Arc::new(AtomicUsize::new(0)),
            event: Arc::new(Event::new()),
        }
    }
}

impl Users {
    fn use_accounts(&self) -> Guard {
        self.used.fetch_add(1, Ordering::SeqCst);
        self.event.notify(usize::MAX);
        Guard {
            used: self.used.clone(),
            event: self.event.clone(),
        }
    }

    fn wait_until_not_in_use(&self) -> impl Future<Output = ()> {
        let used = self.used.clone();
        let event = self.event.clone();

        async move {
            loop {
                if used.load(Ordering::SeqCst) == 0 {
                    break;
                }

                let listener = event.listen();

                if used.load(Ordering::SeqCst) == 0 {
                    break;
                }

                listener.await;
            }
        }
    }

    fn users(&self) -> impl IntoIterator<Item = Pubkey> {
        [self.user_0.pubkey(), self.keeper_0.pubkey()]
    }

    fn keypairs(&self) -> impl IntoIterator<Item = &Keypair> {
        [&self.user_0, &self.keeper_0]
    }
}

#[must_use]
pub(crate) struct Guard {
    used: Arc<AtomicUsize>,
    event: Arc<Event>,
}

impl Drop for Guard {
    fn drop(&mut self) {
        self.used.fetch_sub(1, Ordering::SeqCst);
        self.event.notify(usize::MAX);
    }
}

/// Get current deployment.
pub async fn current_deployment() -> eyre::Result<&'static Deployment> {
    static DEPLOYMENT: OnceCell<Deployment> = OnceCell::const_new();
    DEPLOYMENT.get_or_try_init(Deployment::init).await
}

#[tokio::test]
async fn refund_payer() -> eyre::Result<()> {
    let wait = std::env::var("GMSOL_REFUND_WAIT")
        .ok()
        .and_then(|wait| wait.parse().ok())
        .unwrap_or(1);
    let deployment = current_deployment().await?;

    deployment
        .refund_payer_when_not_in_use(Duration::from_secs(wait))
        .await?;

    Ok(())
}
