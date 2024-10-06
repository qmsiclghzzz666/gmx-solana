use std::ops::Deref;

use anchor_client::solana_sdk::{pubkey::Pubkey, signer::Signer};
use gmsol_store::states::Store;

use crate::utils::RpcBuilder;

/// Claim fees builder.
// TODO: implement this.
#[allow(dead_code)]
pub struct ClaimFeesBuilder<'a, C> {
    client: &'a crate::Client<C>,
    store: Pubkey,
    market_token: Pubkey,
    is_long_token: bool,
    store_hint: Option<Store>,
    token: Option<Pubkey>,
}

impl<'a, C: Deref<Target = impl Signer> + Clone> ClaimFeesBuilder<'a, C> {
    /// Create a new builder.
    pub fn new(
        client: &'a crate::Client<C>,
        store: &Pubkey,
        market_token: &Pubkey,
        is_long_token: bool,
    ) -> Self {
        Self {
            client,
            store: *store,
            market_token: *market_token,
            is_long_token,
            store_hint: None,
            token: None,
        }
    }

    /// Set hint.
    pub fn set_hint(&mut self, store: Store, token: Pubkey) -> &mut Self {
        self.store_hint = Some(store);
        self.token = Some(token);
        self
    }

    /// Build.
    pub async fn build(&self) -> crate::Result<RpcBuilder<'a, C>> {
        todo!()
    }
}
