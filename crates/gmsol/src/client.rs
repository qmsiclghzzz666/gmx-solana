use std::ops::Deref;

use anchor_client::{
    solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey, signer::Signer},
    Cluster, Program,
};

use typed_builder::TypedBuilder;

/// Options for [`Client`].
#[derive(Debug, Clone, TypedBuilder)]
pub struct ClientOptions {
    #[builder(default = data_store::ID)]
    data_store_program_id: Pubkey,
    #[builder(default = exchange::ID)]
    exchange_program_id: Pubkey,
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
    anchor: anchor_client::Client<C>,
    data_store_program_id: Pubkey,
    exchange_program_id: Pubkey,
}

impl<C: Clone + Deref<Target = impl Signer>> Client<C> {
    /// Create a new [`Client`] with the given options.
    pub fn new_with_options(cluster: Cluster, payer: C, options: ClientOptions) -> Self {
        let ClientOptions {
            data_store_program_id,
            exchange_program_id,
            commitment,
        } = options;
        Self {
            anchor: anchor_client::Client::new_with_options(cluster, payer, commitment),
            data_store_program_id,
            exchange_program_id,
        }
    }

    /// Create a new [`Client`] with default options.
    pub fn new(cluster: Cluster, payer: C) -> Self {
        Self::new_with_options(cluster, payer, ClientOptions::default())
    }

    /// Get `DataStore` Program.
    pub fn data_store(&self) -> crate::Result<Program<C>> {
        Ok(self.anchor.program(self.data_store_program_id)?)
    }

    /// Get `Exchange` Program
    pub fn exchange(&self) -> crate::Result<Program<C>> {
        Ok(self.anchor.program(self.exchange_program_id)?)
    }
}
