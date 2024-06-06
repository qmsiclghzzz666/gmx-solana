use std::ops::Deref;

use anchor_client::{
    solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey, signer::Signer},
    Cluster, Program,
};

use data_store::states::TokenConfig;
use typed_builder::TypedBuilder;

use crate::utils::RpcBuilder;

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
    wallet: C,
    anchor: anchor_client::Client<C>,
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
        let anchor = anchor_client::Client::new_with_options(cluster, payer.clone(), commitment);
        Ok(Self {
            wallet: payer,
            data_store: anchor.program(data_store_program_id.unwrap_or(data_store::id()))?,
            exchange: anchor.program(exchange_program_id.unwrap_or(exchange::id()))?,
            anchor,
        })
    }

    /// Create a new [`Client`] with default options.
    pub fn new(cluster: Cluster, payer: C) -> crate::Result<Self> {
        Self::new_with_options(cluster, payer, ClientOptions::default())
    }

    /// Get anchor client.
    pub fn anchor(&self) -> &anchor_client::Client<C> {
        &self.anchor
    }

    /// Get payer.
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

    /// Get the program id of `DataStore` program.
    pub fn data_store_program_id(&self) -> Pubkey {
        self.data_store().id()
    }

    /// Get the program id of `Exchange` program.
    pub fn exchange_program_id(&self) -> Pubkey {
        self.exchange().id()
    }

    /// Create a request for `DataStore` Program.
    pub fn data_store_request(&self) -> RpcBuilder<'_, C> {
        RpcBuilder::new(&self.data_store)
    }

    /// Create a request for `Exchange` Program.
    pub fn exchange_request(&self) -> RpcBuilder<'_, C> {
        RpcBuilder::new(&self.exchange)
    }

    /// Get token config for the given token.
    pub async fn token_config(
        &self,
        store: &Pubkey,
        token: &Pubkey,
    ) -> crate::Result<Option<TokenConfig>> {
        use crate::{store::token_config::TokenConfigOps, utils::view};

        let client = self.data_store().async_rpc();
        let output = view(
            &client,
            &self
                .get_token_config(store, token)
                .signed_transaction()
                .await?,
        )
        .await?;
        Ok(output)
    }

    /// Find PDA for [`DataStore`](data_store::states::DataStore) account.
    pub fn find_store_address(&self, key: &str) -> Pubkey {
        crate::pda::find_store_address(key, &self.data_store_program_id()).0
    }

    /// Find PDA for [`Roles`](data_store::states::Roles) account.
    pub fn find_roles_address(&self, store: &Pubkey, authority: &Pubkey) -> Pubkey {
        crate::pda::find_roles_address(store, authority, &self.data_store_program_id()).0
    }

    /// Get the roles address for payer.
    pub fn payer_roles_address(&self, store: &Pubkey) -> Pubkey {
        self.find_roles_address(store, &self.payer())
    }

    /// Get the controller address for the exchange program.
    pub fn controller_address(&self, store: &Pubkey) -> Pubkey {
        crate::pda::find_controller_address(store, &self.exchange_program_id()).0
    }

    /// Find PDA for [`Oracle`](data_store::states::Oracle) account.
    pub fn find_oracle_address(&self, store: &Pubkey, index: u8) -> Pubkey {
        crate::pda::find_oracle_address(store, index, &self.data_store_program_id()).0
    }

    /// Find PDA for [`TokenConfigMap`](data_store::states::TokenConfigMap) account.
    pub fn find_token_config_map(&self, store: &Pubkey) -> Pubkey {
        crate::pda::find_token_config_map(store, &self.data_store_program_id()).0
    }

    /// Find PDA for [`Config`](data_store::states::Config) account.
    pub fn find_config_address(&self, store: &Pubkey) -> Pubkey {
        crate::pda::find_config_pda(store, &self.data_store_program_id()).0
    }
}
