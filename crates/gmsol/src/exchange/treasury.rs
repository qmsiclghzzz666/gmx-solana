use std::ops::Deref;

use anchor_client::{
    anchor_lang::system_program,
    solana_sdk::{pubkey::Pubkey, signer::Signer},
};
use anchor_spl::associated_token::get_associated_token_address;
use gmsol_exchange::{accounts, instruction};
use gmsol_store::states::Store;

use crate::utils::RpcBuilder;

/// Claim fees builder.
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
        let store = if let Some(hint) = self.store_hint {
            hint
        } else {
            self.client.store(&self.store).await?
        };
        let market = self
            .client
            .find_market_address(&self.store, &self.market_token);
        let token = if let Some(token) = self.token {
            token
        } else {
            self.client
                .market(&market)
                .await?
                .meta()
                .pnl_token(self.is_long_token)
        };

        let receiver = store.receiver();
        let treasury = store.treasury();

        Ok(self
            .client
            .exchange_rpc()
            .args(instruction::ClaimFees {})
            .accounts(accounts::ClaimFees {
                authority: self.client.payer(),
                store: self.store,
                controller: self.client.controller_address(&self.store),
                token,
                market,
                vault: self.client.find_market_vault_address(&self.store, &token),
                receiver,
                treasury,
                receiver_token_account: get_associated_token_address(&receiver, &token),
                treasury_token_account: get_associated_token_address(&treasury, &token),
                store_program: self.client.store_program_id(),
                system_program: system_program::ID,
                token_program: anchor_spl::token::ID,
                associated_token_program: anchor_spl::associated_token::ID,
            }))
    }
}
