use std::{collections::BTreeMap, ops::Deref, sync::Arc};

use anchor_client::{
    anchor_lang::{AccountDeserialize, AnchorSerialize, Discriminator},
    solana_client::{
        rpc_config::RpcAccountInfoConfig,
        rpc_filter::{Memcmp, RpcFilterType},
    },
    solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey, signer::Signer},
    Cluster, Program,
};

use gmsol_model::{action::Prices, PnlFactorKind};
use gmsol_store::states::{position::PositionKind, status::MarketStatus, NonceBytes};
use solana_account_decoder::UiAccountEncoding;
use tokio::sync::OnceCell;
use typed_builder::TypedBuilder;

use crate::{
    store::market::MarketOps,
    types,
    utils::{
        account_with_context, accounts_lazy_with_context, ProgramAccountsConfig, PubsubClient,
        RpcBuilder, SubscriptionConfig, TransactionBuilder, WithContext, ZeroCopy,
    },
};

const DISC_OFFSET: usize = 8;

/// Options for [`Client`].
#[derive(Debug, Clone, TypedBuilder)]
pub struct ClientOptions {
    #[builder(default)]
    data_store_program_id: Option<Pubkey>,
    #[builder(default)]
    exchange_program_id: Option<Pubkey>,
    #[builder(default)]
    commitment: CommitmentConfig,
    #[builder(default)]
    subscription: SubscriptionConfig,
}

impl Default for ClientOptions {
    fn default() -> Self {
        Self::builder().build()
    }
}

/// GMSOL Client.
pub struct Client<C> {
    cluster: Cluster,
    wallet: C,
    anchor: Arc<anchor_client::Client<C>>,
    data_store: Program<C>,
    exchange: Program<C>,
    pub_sub: OnceCell<PubsubClient>,
    subscription_config: SubscriptionConfig,
    commitment: CommitmentConfig,
}

impl<C: Clone + Deref<Target = impl Signer>> Client<C> {
    /// Create a new [`Client`] with the given options.
    pub fn new_with_options(
        cluster: Cluster,
        payer: C,
        options: ClientOptions,
    ) -> crate::Result<Self> {
        let ClientOptions {
            data_store_program_id,
            exchange_program_id,
            commitment,
            subscription,
        } = options;
        let anchor =
            anchor_client::Client::new_with_options(cluster.clone(), payer.clone(), commitment);
        Ok(Self {
            cluster,
            wallet: payer,
            data_store: anchor.program(data_store_program_id.unwrap_or(gmsol_store::id()))?,
            exchange: anchor.program(exchange_program_id.unwrap_or(gmsol_exchange::id()))?,
            anchor: Arc::new(anchor),
            pub_sub: OnceCell::default(),
            subscription_config: subscription,
            commitment,
        })
    }

    /// Create a new [`Client`] with default options.
    pub fn new(cluster: Cluster, payer: C) -> crate::Result<Self> {
        Self::new_with_options(cluster, payer, ClientOptions::default())
    }

    /// Try to clone a new client with a new payer.
    pub fn try_clone_with_payer<C2: Clone + Deref<Target = impl Signer>>(
        &self,
        payer: C2,
    ) -> crate::Result<Client<C2>> {
        Client::new_with_options(
            self.cluster.clone(),
            payer,
            ClientOptions {
                data_store_program_id: Some(self.store_program_id()),
                exchange_program_id: Some(self.exchange_program_id()),
                commitment: self.commitment,
                subscription: self.subscription_config.clone(),
            },
        )
    }

    /// Try to clone the client.
    pub fn try_clone(&self) -> crate::Result<Self> {
        Ok(Self {
            cluster: self.cluster.clone(),
            wallet: self.wallet.clone(),
            anchor: self.anchor.clone(),
            data_store: self.anchor.program(self.store_program_id())?,
            exchange: self.anchor.program(self.exchange_program_id())?,
            pub_sub: OnceCell::default(),
            subscription_config: self.subscription_config.clone(),
            commitment: self.commitment,
        })
    }

    /// Replace the subscription config.
    pub fn set_subscription_config(&mut self, config: SubscriptionConfig) -> &mut Self {
        self.subscription_config = config;
        self
    }

    /// Get anchor client.
    pub fn anchor(&self) -> &anchor_client::Client<C> {
        &self.anchor
    }

    /// Get the cluster.
    pub fn cluster(&self) -> &Cluster {
        &self.cluster
    }

    /// Get the commitment config.
    pub fn commitment(&self) -> CommitmentConfig {
        self.commitment
    }

    /// Get the payer.
    pub fn payer(&self) -> Pubkey {
        self.wallet.pubkey()
    }

    /// Create other program using client's wallet.
    pub fn program(&self, program_id: Pubkey) -> crate::Result<Program<C>> {
        Ok(self.anchor.program(program_id)?)
    }

    /// Get `DataStore` Program.
    pub fn data_store(&self) -> &Program<C> {
        &self.data_store
    }

    /// Get `Exchange` Program
    pub fn exchange(&self) -> &Program<C> {
        &self.exchange
    }

    /// Create a new `DataStore` Program.
    pub fn new_data_store(&self) -> crate::Result<Program<C>> {
        self.program(self.store_program_id())
    }

    /// Create a new `Exchange` Program.
    pub fn new_exchange(&self) -> crate::Result<Program<C>> {
        self.program(self.exchange_program_id())
    }

    /// Get the program id of `Store` program.
    pub fn store_program_id(&self) -> Pubkey {
        self.data_store().id()
    }

    /// Get the program id of `Exchange` program.
    pub fn exchange_program_id(&self) -> Pubkey {
        self.exchange().id()
    }

    /// Create a rpc request for `DataStore` Program.
    pub fn data_store_rpc(&self) -> RpcBuilder<'_, C> {
        RpcBuilder::new(&self.data_store)
    }

    /// Create a transaction builder with the given options.
    pub fn transaction_with_options(
        &self,
        force_one_transaction: bool,
        max_packet_size: Option<usize>,
    ) -> TransactionBuilder<'_, C> {
        TransactionBuilder::new_with_options(
            self.data_store.async_rpc(),
            force_one_transaction,
            max_packet_size,
        )
    }

    /// Create a transaction builder with default options.
    pub fn transaction(&self) -> TransactionBuilder<'_, C> {
        self.transaction_with_options(false, None)
    }

    /// Create a rpc request for `Exchange` Program.
    pub fn exchange_rpc(&self) -> RpcBuilder<'_, C> {
        RpcBuilder::new(&self.exchange)
    }

    /// Find Event Authority Address.
    pub fn find_event_authority_address(&self) -> Pubkey {
        crate::pda::find_event_authority_address(&self.exchange_program_id()).0
    }

    /// Find PDA for [`Store`](gmsol_store::states::Store) account.
    pub fn find_store_address(&self, key: &str) -> Pubkey {
        crate::pda::find_store_address(key, &self.store_program_id()).0
    }

    /// Get the controller address for the exchange program.
    pub fn controller_address(&self, store: &Pubkey) -> Pubkey {
        crate::pda::find_controller_address(store, &self.exchange_program_id()).0
    }

    /// Get the event authority address for the data store program.
    pub fn data_store_event_authority(&self) -> Pubkey {
        crate::pda::find_event_authority_address(&self.store_program_id()).0
    }

    /// Find PDA for [`Oracle`](gmsol_store::states::Oracle) account.
    pub fn find_oracle_address(&self, store: &Pubkey, index: u8) -> Pubkey {
        crate::pda::find_oracle_address(store, index, &self.store_program_id()).0
    }

    /// Find PDA for market vault account.
    pub fn find_market_vault_address(&self, store: &Pubkey, token: &Pubkey) -> Pubkey {
        crate::pda::find_market_vault_address(store, token, &self.store_program_id()).0
    }

    /// Find PDA for market token mint account.
    pub fn find_market_token_address(
        &self,
        store: &Pubkey,
        index_token: &Pubkey,
        long_token: &Pubkey,
        short_token: &Pubkey,
    ) -> Pubkey {
        crate::pda::find_market_token_address(
            store,
            index_token,
            long_token,
            short_token,
            &self.store_program_id(),
        )
        .0
    }

    /// Find PDA for market account.
    pub fn find_market_address(&self, store: &Pubkey, token: &Pubkey) -> Pubkey {
        crate::pda::find_market_address(store, token, &self.store_program_id()).0
    }

    /// Find PDA for deposit account.
    pub fn find_deposit_address(
        &self,
        store: &Pubkey,
        user: &Pubkey,
        nonce: &NonceBytes,
    ) -> Pubkey {
        crate::pda::find_deposit_address(store, user, nonce, &self.store_program_id()).0
    }

    /// Find DPA for withdrawal account.
    pub fn find_withdrawal_address(
        &self,
        store: &Pubkey,
        user: &Pubkey,
        nonce: &NonceBytes,
    ) -> Pubkey {
        crate::pda::find_withdrawal_address(store, user, nonce, &self.store_program_id()).0
    }

    /// Find PDA for order.
    pub fn find_order_address(&self, store: &Pubkey, user: &Pubkey, nonce: &NonceBytes) -> Pubkey {
        crate::pda::find_order_address(store, user, nonce, &self.store_program_id()).0
    }

    /// Find PDA for position.
    pub fn find_position_address(
        &self,
        store: &Pubkey,
        user: &Pubkey,
        market_token: &Pubkey,
        collateral_token: &Pubkey,
        kind: PositionKind,
    ) -> crate::Result<Pubkey> {
        Ok(crate::pda::find_position_address(
            store,
            user,
            market_token,
            collateral_token,
            kind,
            &self.store_program_id(),
        )?
        .0)
    }

    /// Find claimable account address.
    pub fn find_claimable_account_address(
        &self,
        store: &Pubkey,
        mint: &Pubkey,
        user: &Pubkey,
        time_key: &[u8],
    ) -> Pubkey {
        crate::pda::find_claimable_account_pda(
            store,
            mint,
            user,
            time_key,
            &self.store_program_id(),
        )
        .0
    }

    /// Get slot.
    pub async fn get_slot(&self, commitment: Option<CommitmentConfig>) -> crate::Result<u64> {
        let slot = self
            .data_store()
            .async_rpc()
            .get_slot_with_commitment(commitment.unwrap_or(self.commitment))
            .await
            .map_err(anchor_client::ClientError::from)?;
        Ok(slot)
    }

    /// Fetch accounts owned by the Store Program.
    pub async fn store_accounts_with_config<T>(
        &self,
        filter_by_store: Option<StoreFilter>,
        other_filters: impl IntoIterator<Item = RpcFilterType>,
        config: ProgramAccountsConfig,
    ) -> crate::Result<WithContext<Vec<(Pubkey, T)>>>
    where
        T: AccountDeserialize + Discriminator,
    {
        let filters = std::iter::empty()
            .chain(
                filter_by_store
                    .inspect(|filter| {
                        let store = &filter.store;
                        tracing::debug!(%store, offset=%filter.store_offset(), "store bytes to filter: {}", hex::encode(store));
                    })
                    .map(RpcFilterType::from),
            )
            .chain(other_filters);
        accounts_lazy_with_context(self.data_store(), filters, config)
            .await?
            .map(|iter| iter.collect())
            .transpose()
    }

    /// Fetch account at the given address with config.
    ///
    /// The value inside the returned context will be `None` if the account does not exist.
    pub async fn account_with_config<T>(
        &self,
        address: &Pubkey,
        mut config: RpcAccountInfoConfig,
    ) -> crate::Result<WithContext<Option<T>>>
    where
        T: AccountDeserialize,
    {
        config.encoding = Some(config.encoding.unwrap_or(UiAccountEncoding::Base64));
        let client = self.data_store().async_rpc();
        account_with_context(&client, address, config).await
    }

    /// Fetch accounts owned by the Store Program.
    pub async fn store_accounts<T>(
        &self,
        filter_by_store: Option<StoreFilter>,
        other_filters: impl IntoIterator<Item = RpcFilterType>,
    ) -> crate::Result<Vec<(Pubkey, T)>>
    where
        T: AccountDeserialize + Discriminator,
    {
        let res = self
            .store_accounts_with_config(
                filter_by_store,
                other_filters,
                ProgramAccountsConfig::default(),
            )
            .await?;
        tracing::debug!(slot=%res.slot(), "accounts fetched");
        Ok(res.into_value())
    }

    /// Fetch [`Store`](types::Store) account with its address.
    pub async fn store(&self, address: &Pubkey) -> crate::Result<types::Store> {
        Ok(self
            .data_store()
            .account::<ZeroCopy<types::Store>>(*address)
            .await?
            .0)
    }

    /// Fetch the [`TokenMap`](types::TokenMap) address of the given store.
    pub async fn authorized_token_map_address(
        &self,
        store: &Pubkey,
    ) -> crate::Result<Option<Pubkey>> {
        let store = self.store(store).await?;
        let token_map = store.token_map;
        if token_map == Pubkey::default() {
            Ok(None)
        } else {
            Ok(Some(token_map))
        }
    }

    /// Fetch [`TokenMap`](types::TokenMap) account with its address.
    pub async fn token_map(&self, address: &Pubkey) -> crate::Result<types::TokenMap> {
        Ok(self.data_store().account(*address).await?)
    }

    /// Fetch the authorized token map of the given store.
    pub async fn authorized_token_map(&self, store: &Pubkey) -> crate::Result<types::TokenMap> {
        let address = self
            .authorized_token_map_address(store)
            .await?
            .ok_or(crate::Error::invalid_argument("token map is not set"))?;
        self.token_map(&address).await
    }

    /// Fetch all [`Market`](types::Market) accounts of the given store.
    pub async fn markets_with_config(
        &self,
        store: &Pubkey,
        config: ProgramAccountsConfig,
    ) -> crate::Result<WithContext<BTreeMap<Pubkey, types::Market>>> {
        let markets = self
            .store_accounts_with_config::<ZeroCopy<types::Market>>(
                Some(StoreFilter::new(
                    store,
                    bytemuck::offset_of!(types::Market, store),
                )),
                None,
                config,
            )
            .await?
            .map(|accounts| {
                accounts
                    .into_iter()
                    .map(|(pubkey, m)| (pubkey, m.0))
                    .collect::<BTreeMap<_, _>>()
            });
        Ok(markets)
    }

    /// Fetch all [`Market`](types::Market) accounts of the given store.
    pub async fn markets(&self, store: &Pubkey) -> crate::Result<BTreeMap<Pubkey, types::Market>> {
        let markets = self
            .markets_with_config(store, ProgramAccountsConfig::default())
            .await?
            .into_value();
        Ok(markets)
    }

    /// Fetch [`Market`](types::Market) at the given address with config.
    ///
    /// The value inside the returned context will be `None` if the account does not exist.
    pub async fn market_with_config<T>(
        &self,
        address: &Pubkey,
        config: RpcAccountInfoConfig,
    ) -> crate::Result<WithContext<Option<types::Market>>> {
        let market = self
            .account_with_config::<ZeroCopy<types::Market>>(address, config)
            .await?;
        Ok(market.map(|m| m.map(|m| m.0)))
    }

    /// Fetch [`Market`](types::Market) account with its address.
    pub async fn market(&self, address: &Pubkey) -> crate::Result<types::Market> {
        Ok(self
            .data_store()
            .account::<ZeroCopy<types::Market>>(*address)
            .await?
            .0)
    }

    /// Fetch [`MarketStatus`] with the market token address.
    pub async fn market_status(
        &self,
        store: &Pubkey,
        market_token: &Pubkey,
        prices: Prices<u128>,
        maximize_pnl: bool,
        maximize_pool_value: bool,
    ) -> crate::Result<MarketStatus> {
        let req = self.get_market_status(
            store,
            market_token,
            prices,
            maximize_pnl,
            maximize_pool_value,
        );
        let status = crate::utils::view::<MarketStatus>(
            &self.data_store().async_rpc(),
            &req.signed_transaction().await?,
        )
        .await?;
        Ok(status)
    }

    /// Fetch current market token price with the market token address.
    pub async fn market_token_price(
        &self,
        store: &Pubkey,
        market_token: &Pubkey,
        prices: Prices<u128>,
        pnl_factor: PnlFactorKind,
        maximize: bool,
    ) -> crate::Result<u128> {
        let req = self.get_market_token_price(store, market_token, prices, pnl_factor, maximize);
        let price = crate::utils::view::<u128>(
            &self.data_store().async_rpc(),
            &req.signed_transaction().await?,
        )
        .await?;
        Ok(price)
    }

    /// Fetch all [`Position`](types::Position) accounts of the given owner of the given store.
    pub async fn positions(
        &self,
        store: &Pubkey,
        owner: Option<&Pubkey>,
        market_token: Option<&Pubkey>,
    ) -> crate::Result<BTreeMap<Pubkey, types::Position>> {
        let filter = match owner {
            Some(owner) => {
                let mut bytes = owner.as_ref().to_owned();
                if let Some(market_token) = market_token {
                    bytes.extend_from_slice(market_token.as_ref());
                }
                let filter = RpcFilterType::Memcmp(Memcmp::new_raw_bytes(
                    bytemuck::offset_of!(types::Position, owner) + DISC_OFFSET,
                    bytes,
                ));
                Some(filter)
            }
            None => market_token.and_then(|token| {
                Some(RpcFilterType::Memcmp(Memcmp::new_raw_bytes(
                    bytemuck::offset_of!(types::Position, market_token) + DISC_OFFSET,
                    token.try_to_vec().ok()?,
                )))
            }),
        };

        let store_filter = StoreFilter::new(store, bytemuck::offset_of!(types::Position, store));

        let positions = self
            .store_accounts::<ZeroCopy<types::Position>>(Some(store_filter), filter)
            .await?
            .into_iter()
            .map(|(pubkey, p)| (pubkey, p.0))
            .collect();

        Ok(positions)
    }

    /// Fetch [`Position`](types::Position) account with its address.
    pub async fn position(&self, address: &Pubkey) -> crate::Result<types::Position> {
        let position = self
            .data_store()
            .account::<ZeroCopy<types::Position>>(*address)
            .await?;
        Ok(position.0)
    }

    /// Fetch [`Order`](types::Order) account with its address.
    pub async fn order(&self, address: &Pubkey) -> crate::Result<types::Order> {
        Ok(self.data_store().account(*address).await?)
    }

    /// Fetch [`Order`](types::Order) account at the the given address with config.
    ///
    /// The value inside the returned context will be `None` if the account does not exist.
    pub async fn order_with_config(
        &self,
        address: &Pubkey,
        config: RpcAccountInfoConfig,
    ) -> crate::Result<WithContext<Option<types::Order>>> {
        self.account_with_config(address, config).await
    }

    /// Fetch all [`Order`](types::Order) accounts of the given owner of the given store.
    pub async fn orders(
        &self,
        store: &Pubkey,
        owner: Option<&Pubkey>,
        market_token: Option<&Pubkey>,
    ) -> crate::Result<BTreeMap<Pubkey, types::Order>> {
        let mut filters = Vec::default();
        if let Some(owner) = owner {
            filters.push(RpcFilterType::Memcmp(Memcmp::new_raw_bytes(
                8 + 1 + 32 + 32 + 8 + 8 + 8 + 1 + 32,
                owner.as_ref().to_owned(),
            )));
        }
        if let Some(market_token) = market_token {
            let market = self.find_market_address(store, market_token);
            filters.push(RpcFilterType::Memcmp(Memcmp::new_raw_bytes(
                8 + 1 + 32,
                market.as_ref().to_owned(),
            )));
        }
        let store_filter = StoreFilter::new(store, 1);

        let orders = self
            .store_accounts::<types::Order>(Some(store_filter), filters)
            .await?
            .into_iter()
            .collect();

        Ok(orders)
    }

    /// Fetch [`Depsoit`](types::Deposit) account with its address.
    pub async fn deposit(&self, address: &Pubkey) -> crate::Result<types::Order> {
        Ok(self.data_store().account(*address).await?)
    }

    /// Fetch [`Withdrawal`](types::Withdrawal) account with its address.
    pub async fn withdrawal(&self, address: &Pubkey) -> crate::Result<types::Order> {
        Ok(self.data_store().account(*address).await?)
    }

    /// Get the [`PubsubClient`].
    pub async fn pub_sub(&self) -> crate::Result<&PubsubClient> {
        let client = self
            .pub_sub
            .get_or_try_init(|| {
                PubsubClient::new(self.cluster.clone(), self.subscription_config.clone())
            })
            .await?;
        Ok(client)
    }

    /// Subscribe to [`StoreCPIEvent`](crate::store::events::StoreCPIEvent)s from the store program.
    #[cfg(feature = "decode")]
    pub async fn subscribe_store_cpi_events(
        &self,
        commitment: Option<CommitmentConfig>,
    ) -> crate::Result<
        impl futures_util::Stream<
            Item = crate::Result<crate::utils::WithSlot<Vec<crate::store::events::StoreCPIEvent>>>,
        >,
    > {
        use futures_util::TryStreamExt;

        use crate::{
            store::events::StoreCPIEvent,
            utils::{extract_cpi_events, WithSlot},
        };

        let program_id = self.store_program_id();
        let event_authority = self.data_store_event_authority();
        let query = Arc::new(self.data_store().async_rpc());
        let commitment = commitment.unwrap_or(self.subscription_config.commitment);
        let signatures = self
            .pub_sub()
            .await?
            .logs_subscribe(&event_authority, Some(commitment))
            .await?
            .and_then(|txn| {
                let signature = WithSlot::from(txn)
                    .map(|txn| {
                        txn.signature
                            .parse()
                            .map_err(crate::Error::invalid_argument)
                    })
                    .transpose();
                async move { signature }
            });
        let events =
            extract_cpi_events(signatures, query, &program_id, &event_authority, commitment)
                .try_filter_map(|event| {
                    let decoded = event
                        .map(|event| {
                            event
                                .decode::<StoreCPIEvent>()
                                .collect::<crate::Result<Vec<_>>>()
                        })
                        .transpose()
                        .inspect_err(|err| tracing::error!(%err, "decode error"))
                        .ok();
                    async move { Ok(decoded) }
                });
        Ok(events)
    }

    /// Fetch historical [`StoreCPIEvent`](crate::store::events::StoreCPIEvent)s for the given account.
    #[cfg(feature = "decode")]
    pub async fn historical_store_cpi_events(
        &self,
        address: &Pubkey,
        commitment: Option<CommitmentConfig>,
    ) -> crate::Result<
        impl futures_util::Stream<
            Item = crate::Result<crate::utils::WithSlot<Vec<crate::store::events::StoreCPIEvent>>>,
        >,
    > {
        use futures_util::TryStreamExt;

        use crate::{
            store::events::StoreCPIEvent,
            utils::{extract_cpi_events, fetch_transaction_history_with_config},
        };

        let commitment = commitment.unwrap_or(self.commitment);
        let client = Arc::new(self.data_store().async_rpc());
        let signatures = fetch_transaction_history_with_config(
            client.clone(),
            address,
            commitment,
            None,
            None,
            None,
        )
        .await?;
        let events = extract_cpi_events(
            signatures,
            client,
            &self.store_program_id(),
            &self.data_store_event_authority(),
            commitment,
        )
        .and_then(|encoded| {
            let decoded = encoded
                .map(|event| {
                    event
                        .decode::<StoreCPIEvent>()
                        .collect::<crate::Result<Vec<_>>>()
                })
                .transpose();
            async move { decoded }
        });
        Ok(events)
    }

    /// Wait for an order to be completed using current slot as min context slot.
    #[cfg(feature = "decode")]
    pub async fn complete_order(
        &self,
        address: &Pubkey,
        commitment: Option<CommitmentConfig>,
    ) -> crate::Result<Option<crate::types::TradeEvent>> {
        let slot = self.get_slot(None).await?;
        self.complete_order_with_config(
            address,
            slot,
            std::time::Duration::from_secs(5),
            commitment,
        )
        .await
    }

    #[cfg(feature = "decode")]
    async fn last_order_events(
        &self,
        order: &Pubkey,
        before_slot: u64,
        commitment: CommitmentConfig,
    ) -> crate::Result<Vec<crate::store::events::StoreCPIEvent>> {
        use futures_util::{StreamExt, TryStreamExt};

        let events = self
            .historical_store_cpi_events(order, Some(commitment))
            .await?
            .try_filter(|events| {
                let pass = events.slot() <= before_slot;
                async move { pass }
            })
            .take(1);
        futures_util::pin_mut!(events);
        match events.next().await.transpose()? {
            Some(events) => Ok(events.into_value()),
            None => Err(crate::Error::unknown("events not found")),
        }
    }

    /// Wait for an order to be completed with the given config.
    #[cfg(feature = "decode")]
    pub async fn complete_order_with_config(
        &self,
        address: &Pubkey,
        mut slot: u64,
        polling: std::time::Duration,
        commitment: Option<CommitmentConfig>,
    ) -> crate::Result<Option<crate::types::TradeEvent>> {
        use crate::store::events::StoreCPIEvent;
        use futures_util::{StreamExt, TryStreamExt};
        use solana_account_decoder::UiAccountEncoding;

        let mut trade = None;
        let commitment = commitment.unwrap_or(self.subscription_config.commitment);

        let events = self.subscribe_store_cpi_events(Some(commitment)).await?;

        let config = RpcAccountInfoConfig {
            encoding: Some(UiAccountEncoding::Base64),
            commitment: Some(commitment),
            min_context_slot: Some(slot),
            ..Default::default()
        };
        let current = self.get_slot(Some(commitment)).await?;
        let mut slot_reached = current >= slot;
        if slot_reached {
            let order = self.order_with_config(address, config.clone()).await?;
            slot = order.slot();
            let order = order.into_value();
            if order.is_none() {
                let events = self.last_order_events(address, current, commitment).await?;
                return Ok(events
                    .into_iter()
                    .filter_map(|event| {
                        if let StoreCPIEvent::TradeEvent(event) = event {
                            Some(event)
                        } else {
                            None
                        }
                    })
                    .next());
            }
        }
        let address = *address;
        let stream = events
            .try_filter_map(|events| async {
                if events.slot() < slot {
                    return Ok(None);
                }
                let events = events
                    .into_value()
                    .into_iter()
                    .filter(|event| {
                        matches!(
                            event,
                            StoreCPIEvent::TradeEvent(_) | StoreCPIEvent::RemoveOrderEvent(_)
                        )
                    })
                    .map(Ok);
                Ok(Some(futures_util::stream::iter(events)))
            })
            .try_flatten();
        let stream =
            tokio_stream::StreamExt::timeout_repeating(stream, tokio::time::interval(polling));
        futures_util::pin_mut!(stream);
        while let Some(res) = stream.next().await {
            match res {
                Ok(Ok(event)) => match event {
                    StoreCPIEvent::TradeEvent(event) => {
                        trade = Some(event);
                    }
                    StoreCPIEvent::RemoveOrderEvent(_remove) => {
                        return Ok(trade);
                    }
                    _ => unreachable!(),
                },
                Ok(Err(err)) => {
                    return Err(err);
                }
                Err(_elapsed) => {
                    if slot_reached {
                        if self
                            .order_with_config(&address, config.clone())
                            .await?
                            .value()
                            .is_none()
                        {
                            let events = self
                                .last_order_events(&address, current, commitment)
                                .await?;
                            return Ok(events
                                .into_iter()
                                .filter_map(|event| {
                                    if let StoreCPIEvent::TradeEvent(event) = event {
                                        Some(event)
                                    } else {
                                        None
                                    }
                                })
                                .next());
                        }
                    } else {
                        let current = self.get_slot(Some(commitment)).await?;
                        slot_reached = current >= slot;
                    }
                }
            }
        }
        Err(crate::Error::unknown("the watch stream end"))
    }

    /// Shutdown the client gracefully.
    pub async fn shutdown(&self) -> crate::Result<()> {
        self.pub_sub().await?.shutdown().await
    }
}

/// System Program Ops.
pub trait SystemProgramOps<C> {
    /// Transfer to.
    fn transfer(&self, to: &Pubkey, lamports: u64) -> crate::Result<RpcBuilder<C>>;
}

impl<C: Clone + Deref<Target = impl Signer>> SystemProgramOps<C> for Client<C> {
    fn transfer(&self, to: &Pubkey, lamports: u64) -> crate::Result<RpcBuilder<C>> {
        use anchor_client::solana_sdk::system_instruction::transfer;

        if lamports == 0 {
            return Err(crate::Error::invalid_argument(
                "transferring amount is zero",
            ));
        }
        Ok(self
            .data_store_rpc()
            .pre_instruction(transfer(&self.payer(), to, lamports)))
    }
}

/// Store Filter.
#[derive(Debug)]
pub struct StoreFilter {
    /// Store.
    store: Pubkey,
    /// Store offset.
    store_offset: usize,
    /// Ignore disc bytes.
    ignore_disc_offset: bool,
}

impl StoreFilter {
    /// Create a new store filter.
    pub fn new(store: &Pubkey, store_offset: usize) -> Self {
        Self {
            store: *store,
            store_offset,
            ignore_disc_offset: false,
        }
    }

    /// Ignore disc offset.
    pub fn ignore_disc_offset(mut self, ignore: bool) -> Self {
        self.ignore_disc_offset = ignore;
        self
    }

    /// Store offset.
    pub fn store_offset(&self) -> usize {
        if self.ignore_disc_offset {
            self.store_offset
        } else {
            self.store_offset + DISC_OFFSET
        }
    }
}

impl From<StoreFilter> for RpcFilterType {
    fn from(filter: StoreFilter) -> Self {
        let store = filter.store;
        let store_offset = filter.store_offset();
        RpcFilterType::Memcmp(Memcmp::new_raw_bytes(
            store_offset,
            store.as_ref().to_owned(),
        ))
    }
}
