use std::{
    collections::{hash_map::Entry, HashMap},
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
use eyre::OptionExt;
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
    /// Tokens.
    tokens: HashMap<String, Token>,
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
            .field("tokens", &self.tokens)
            .finish_non_exhaustive()
    }
}

impl Deployment {
    /// Default user.
    pub const DEFAULT_USER: &'static str = "user_0";
    /// Default keeper.
    pub const DEFAULT_KEEPER: &'static str = "keeper_0";

    async fn connect() -> eyre::Result<Self> {
        let (client, store) = Self::get_client_and_store().await?;
        let oracle = client.find_oracle_address(&store, 255);
        Ok(Self {
            client,
            users: Default::default(),
            store,
            token_map: Keypair::new(),
            oracle,
            tokens: Default::default(),
        })
    }

    async fn init() -> eyre::Result<Self> {
        Self::init_tracing()?;

        let mut deployment = Self::connect().await?;

        deployment.setup().await?;

        Ok(deployment)
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

    async fn setup(&mut self) -> eyre::Result<()> {
        tracing::info!("[Setting up everything...]");
        self.add_users();

        let _guard = self.use_accounts().await?;

        self.create_tokens([("fBTC", 9), ("USDG", 8)]).await?;
        self.create_token_accounts().await?;

        Ok(())
    }

    fn add_users(&mut self) {
        self.users.add_user(Self::DEFAULT_USER);
        self.users.add_user(Self::DEFAULT_KEEPER);
    }

    async fn create_tokens<T: ToString>(
        &mut self,
        decimals: impl IntoIterator<Item = (T, u8)>,
    ) -> eyre::Result<()> {
        use spl_token::native_mint;

        self.tokens = self.do_create_tokens(decimals).await?;
        if let Entry::Vacant(entry) = self.tokens.entry("WSOL".to_string()) {
            entry.insert(Token {
                address: native_mint::ID,
                decimals: native_mint::DECIMALS,
            });
        }
        Ok(())
    }

    async fn do_create_tokens<T>(
        &self,
        decimals: impl IntoIterator<Item = (T, u8)>,
    ) -> eyre::Result<HashMap<String, Token>>
    where
        T: ToString,
    {
        use anchor_spl::token::{Mint, ID};
        use spl_token::instruction;

        let client = self.client.data_store().async_rpc();
        let rent = client
            .get_minimum_balance_for_rent_exemption(Mint::LEN)
            .await?;
        let mut builder = TransactionBuilder::new(client);

        let tokens = decimals
            .into_iter()
            .map(|(name, decimals)| (name.to_string(), (Keypair::new(), decimals)))
            .collect::<HashMap<_, _>>();

        let payer = self.client.payer();

        for (name, (token, decimals)) in tokens.iter() {
            let pubkey = token.pubkey();
            tracing::info!(%name, "creating mint account {pubkey}");
            let rpc = self
                .client
                .data_store_rpc()
                .signer(token)
                .pre_instruction(system_instruction::create_account(
                    &payer,
                    &pubkey,
                    rent,
                    Mint::LEN as u64,
                    &ID,
                ))
                .pre_instruction(instruction::initialize_mint2(
                    &ID,
                    &token.pubkey(),
                    &payer,
                    None,
                    *decimals,
                )?);
            builder.try_push(rpc).map_err(|(_, err)| err)?;
        }

        match builder.send_all().await {
            Ok(signatures) => {
                tracing::debug!("created tokens with {signatures:#?}");
            }
            Err((signatures, err)) => {
                tracing::error!(%err, "failed to create tokens, successful txns: {signatures:#?}");
            }
        }

        Ok(tokens
            .into_iter()
            .map(|(name, (keypair, decimals))| {
                (
                    name,
                    Token {
                        address: keypair.pubkey(),
                        decimals,
                    },
                )
            })
            .collect())
    }

    async fn create_token_accounts(&self) -> eyre::Result<()> {
        use anchor_spl::token::ID;
        use spl_associated_token_account::instruction;

        let client = self.client.data_store().async_rpc();
        let mut builder = TransactionBuilder::new(client);

        let payer = self.client.payer();

        for (name, token) in self.tokens.iter() {
            for user in self.users.keypairs() {
                let pubkey = user.pubkey();
                tracing::info!(token=%name, mint=%token.address, "creating token account for {pubkey}");
                let rpc = self.client.data_store_rpc().pre_instruction(
                    instruction::create_associated_token_account(
                        &payer,
                        &pubkey,
                        &token.address,
                        &ID,
                    ),
                );
                builder.try_push(rpc).map_err(|(_, err)| err)?;
            }
        }

        match builder.send_all().await {
            Ok(signatures) => {
                tracing::debug!("created token accounts with {signatures:#?}");
            }
            Err((signatures, err)) => {
                tracing::error!(%err, "failed to create token accounts, successful txns: {signatures:#?}");
            }
        }

        Ok(())
    }

    async fn fund_users(&self) -> eyre::Result<()> {
        const LAMPORTS: u64 = 50_000_000;

        let client = self.client.data_store().async_rpc();
        let payer = self.client.payer();
        let lamports = client.get_balance(&payer).await?;
        tracing::info!(%payer, "before funding users: {lamports}");

        let mut builder = TransactionBuilder::new(client);
        builder.try_push_many(
            self.users
                .pubkeys()
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

    async fn close_native_token_accounts(&self) -> eyre::Result<()> {
        use anchor_spl::token::{TokenAccount, ID};
        use spl_associated_token_account::get_associated_token_address;
        use spl_token::{instruction, native_mint};

        let payer = self.client.payer();
        let client = self.client.data_store().async_rpc();
        let mut builder = TransactionBuilder::new(client);

        for user in self.users.keypairs() {
            let pubkey = user.pubkey();
            let address = get_associated_token_address(&pubkey, &native_mint::ID);
            let Some(_account) = self
                .client
                .account_with_config::<TokenAccount>(&address, Default::default())
                .await?
                .into_value()
            else {
                continue;
            };
            builder
                .try_push(self.client.data_store_rpc().signer(user).pre_instruction(
                    instruction::close_account(&ID, &address, &payer, &pubkey, &[&pubkey])?,
                ))
                .map_err(|(_, err)| err)?;
        }

        match builder.send_all().await {
            Ok(signatures) => {
                tracing::debug!("closed native token accounts with {signatures:#?}");
            }
            Err((signatures, err)) => {
                tracing::error!(%err, "failed to close native token accounts, successful txns: {signatures:#?}");
            }
        }
        Ok(())
    }

    async fn refund_payer(&self) -> eyre::Result<()> {
        let client = self.client.data_store().async_rpc();
        let payer = self.client.payer();

        let mut builder = TransactionBuilder::new(self.client.data_store().async_rpc());

        for user in self.users.keypairs() {
            let pubkey = user.pubkey();
            let lamports = client.get_balance(&user.pubkey()).await?;
            if lamports == 0 {
                continue;
            }
            tracing::debug!(user = %pubkey, %lamports, "refund from user");
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

    pub(crate) async fn refund_payer_when_not_in_use(&self, wait: Duration) -> eyre::Result<()> {
        tokio::time::sleep(wait).await;
        self.users.wait_until_not_in_use().await;
        tracing::info!("[Cleanup...]");
        _ = self
            .close_native_token_accounts()
            .await
            .inspect_err(|err| tracing::error!(%err, "close native token accounts error"));
        self.refund_payer().await?;
        Ok(())
    }

    pub(crate) fn token(&self, token: &str) -> Option<&Token> {
        self.tokens.get(token)
    }

    pub(crate) fn user(&self, name: &str) -> Option<Pubkey> {
        self.users.user(name)
    }

    pub(crate) async fn mint_or_transfer_to(
        &self,
        token_name: &str,
        user: &str,
        amount: u64,
    ) -> eyre::Result<()> {
        use anchor_spl::token::ID;
        use spl_associated_token_account::get_associated_token_address;
        use spl_token::{instruction, native_mint};

        let token = self.token(token_name).ok_or_eyre("no such token")?;
        let user = self.user(user).ok_or_eyre("no such user")?;
        let account = get_associated_token_address(&user, &token.address);
        let payer = self.client.payer();

        let signature = if token.address == native_mint::ID {
            self.client
                .data_store_rpc()
                .pre_instruction(system_instruction::transfer(&payer, &account, amount))
                .pre_instruction(instruction::sync_native(&ID, &account)?)
                .build()
                .send()
                .await?
        } else {
            self.client
                .data_store_rpc()
                .pre_instruction(instruction::mint_to_checked(
                    &ID,
                    &token.address,
                    &account,
                    &payer,
                    &[],
                    amount,
                    token.decimals,
                )?)
                .build()
                .send()
                .await?
        };

        tracing::info!(%signature, token=%token_name, "minted or tranferred {amount} to {user}");
        Ok(())
    }
}

/// Users.
pub struct Users {
    users: HashMap<String, Keypair>,
    funded: Arc<AtomicBool>,
    used: Arc<AtomicUsize>,
    event: Arc<Event>,
}

impl fmt::Debug for Users {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let pubkeys = self
            .users
            .iter()
            .map(|(name, k)| (name, k.pubkey()))
            .collect::<HashMap<_, _>>();
        f.debug_struct("Users")
            .field("users", &pubkeys)
            .finish_non_exhaustive()
    }
}

impl Default for Users {
    fn default() -> Self {
        Self {
            users: Default::default(),
            funded: Arc::new(AtomicBool::new(false)),
            used: Arc::new(AtomicUsize::new(0)),
            event: Arc::new(Event::new()),
        }
    }
}

impl Users {
    fn add_user(&mut self, name: &str) -> bool {
        let Entry::Vacant(entry) = self.users.entry(name.to_string()) else {
            return false;
        };
        let keypair = Keypair::new();
        tracing::info!(%name, pubkey=%keypair.pubkey(), "added a new user");
        entry.insert(keypair);
        true
    }

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

    fn user(&self, name: &str) -> Option<Pubkey> {
        self.users.get(name).map(|k| k.pubkey())
    }

    fn pubkeys(&self) -> impl IntoIterator<Item = Pubkey> + '_ {
        self.users.values().map(|k| k.pubkey())
    }

    fn keypairs(&self) -> impl IntoIterator<Item = &Keypair> {
        self.users.values()
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

#[derive(Debug)]
pub(crate) struct Token {
    pub(crate) address: Pubkey,
    pub(crate) decimals: u8,
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
