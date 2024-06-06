use std::ops::Deref;

use anchor_client::{
    solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey, signer::Signer},
    Cluster, Program,
};

use typed_builder::TypedBuilder;

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
}
