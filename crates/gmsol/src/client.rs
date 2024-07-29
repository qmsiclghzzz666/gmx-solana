use std::{collections::BTreeMap, ops::Deref, sync::Arc};

use anchor_client::{
    anchor_lang::{AccountDeserialize, Discriminator},
    solana_client::rpc_filter::{Memcmp, RpcFilterType},
    solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey, signer::Signer},
    Cluster, Program,
};

use gmsol_store::states::{position::PositionKind, NonceBytes};
use typed_builder::TypedBuilder;

use crate::{
    types,
    utils::{
        accounts::{accounts_lazy_with_context, ProgramAccountsConfig, WithContext},
        zero_copy::ZeroCopy,
        RpcBuilder,
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
        } = options;
        let anchor =
            anchor_client::Client::new_with_options(cluster.clone(), payer.clone(), commitment);
        Ok(Self {
            cluster,
            wallet: payer,
            data_store: anchor.program(data_store_program_id.unwrap_or(gmsol_store::id()))?,
            exchange: anchor.program(exchange_program_id.unwrap_or(gmsol_exchange::id()))?,
            anchor: Arc::new(anchor),
        })
    }

    /// Create a new [`Client`] with default options.
    pub fn new(cluster: Cluster, payer: C) -> crate::Result<Self> {
        Self::new_with_options(cluster, payer, ClientOptions::default())
    }

    /// Try to clone the client.
    pub fn try_clone(&self) -> crate::Result<Self> {
        Ok(Self {
            cluster: self.cluster.clone(),
            wallet: self.wallet.clone(),
            anchor: self.anchor.clone(),
            data_store: self.anchor.program(self.data_store_program_id())?,
            exchange: self.anchor.program(self.exchange_program_id())?,
        })
    }

    /// Get anchor client.
    pub fn anchor(&self) -> &anchor_client::Client<C> {
        &self.anchor
    }

    /// Get the cluster.
    pub fn cluster(&self) -> &Cluster {
        &self.cluster
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
        self.program(self.data_store_program_id())
    }

    /// Create a new `Exchange` Program.
    pub fn new_exchange(&self) -> crate::Result<Program<C>> {
        self.program(self.exchange_program_id())
    }

    /// Get the program id of `DataStore` program.
    pub fn data_store_program_id(&self) -> Pubkey {
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
        crate::pda::find_store_address(key, &self.data_store_program_id()).0
    }

    /// Get the controller address for the exchange program.
    pub fn controller_address(&self, store: &Pubkey) -> Pubkey {
        crate::pda::find_controller_address(store, &self.exchange_program_id()).0
    }

    /// Get the event authority address for the data store program.
    pub fn data_store_event_authority(&self) -> Pubkey {
        crate::pda::find_event_authority_address(&self.data_store_program_id()).0
    }

    /// Find PDA for [`Oracle`](gmsol_store::states::Oracle) account.
    pub fn find_oracle_address(&self, store: &Pubkey, index: u8) -> Pubkey {
        crate::pda::find_oracle_address(store, index, &self.data_store_program_id()).0
    }

    /// Find PDA for market vault account.
    pub fn find_market_vault_address(&self, store: &Pubkey, token: &Pubkey) -> Pubkey {
        crate::pda::find_market_vault_address(store, token, &self.data_store_program_id()).0
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
            &self.data_store_program_id(),
        )
        .0
    }

    /// Find PDA for market account.
    pub fn find_market_address(&self, store: &Pubkey, token: &Pubkey) -> Pubkey {
        crate::pda::find_market_address(store, token, &self.data_store_program_id()).0
    }

    /// Find PDA for deposit account.
    pub fn find_deposit_address(
        &self,
        store: &Pubkey,
        user: &Pubkey,
        nonce: &NonceBytes,
    ) -> Pubkey {
        crate::pda::find_deposit_address(store, user, nonce, &self.data_store_program_id()).0
    }

    /// Find DPA for withdrawal account.
    pub fn find_withdrawal_address(
        &self,
        store: &Pubkey,
        user: &Pubkey,
        nonce: &NonceBytes,
    ) -> Pubkey {
        crate::pda::find_withdrawal_address(store, user, nonce, &self.data_store_program_id()).0
    }

    /// Find PDA for order.
    pub fn find_order_address(&self, store: &Pubkey, user: &Pubkey, nonce: &NonceBytes) -> Pubkey {
        crate::pda::find_order_address(store, user, nonce, &self.data_store_program_id()).0
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
            &self.data_store_program_id(),
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
            &self.data_store_program_id(),
        )
        .0
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
    pub async fn authorized_token_map(&self, store: &Pubkey) -> crate::Result<Option<Pubkey>> {
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

    /// Fetch [`Market`](types::Market) account with its address.
    pub async fn market(&self, address: &Pubkey) -> crate::Result<types::Market> {
        Ok(self
            .data_store()
            .account::<ZeroCopy<types::Market>>(*address)
            .await?
            .0)
    }

    /// Fetch all [`Position`](types::Position) accounts of the given owner of the given store.
    pub async fn positions(
        &self,
        store: &Pubkey,
        owner: &Pubkey,
        market_token: Option<&Pubkey>,
    ) -> crate::Result<BTreeMap<Pubkey, types::Position>> {
        let mut bytes = owner.as_ref().to_owned();
        if let Some(market_token) = market_token {
            bytes.extend_from_slice(market_token.as_ref());
        }
        let filter = RpcFilterType::Memcmp(Memcmp::new_raw_bytes(
            bytemuck::offset_of!(types::Position, owner) + DISC_OFFSET,
            bytes,
        ));
        let store_filter = StoreFilter::new(store, bytemuck::offset_of!(types::Position, store));

        let positions = self
            .store_accounts::<ZeroCopy<types::Position>>(Some(store_filter), Some(filter))
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

    /// Fetch [`Depsoit`](types::Deposit) account with its address.
    pub async fn deposit(&self, address: &Pubkey) -> crate::Result<types::Order> {
        Ok(self.data_store().account(*address).await?)
    }

    /// Fetch [`Withdrawal`](types::Withdrawal) account with its address.
    pub async fn withdrawal(&self, address: &Pubkey) -> crate::Result<types::Order> {
        Ok(self.data_store().account(*address).await?)
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
