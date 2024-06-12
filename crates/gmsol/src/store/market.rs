use std::ops::Deref;

use anchor_client::{
    anchor_lang::system_program,
    solana_sdk::{pubkey::Pubkey, signer::Signer},
    RequestBuilder,
};
use data_store::{
    accounts, instruction,
    states::{Factor, MarketConfigKey},
};

use crate::utils::RpcBuilder;

/// Vault Operations.
pub trait VaultOps<C> {
    /// Initialize a market vault for the given token.
    fn initialize_market_vault(
        &self,
        store: &Pubkey,
        token: &Pubkey,
    ) -> (RequestBuilder<C>, Pubkey);

    /// Transfer tokens out from the given market vault.
    fn market_vault_transfer_out(
        &self,
        store: &Pubkey,
        token: &Pubkey,
        to: &Pubkey,
        amount: u64,
    ) -> RequestBuilder<C>;
}

impl<C, S> VaultOps<C> for crate::Client<C>
where
    C: Deref<Target = S> + Clone,
    S: Signer,
{
    fn initialize_market_vault(
        &self,
        store: &Pubkey,
        token: &Pubkey,
    ) -> (RequestBuilder<C>, Pubkey) {
        let authority = self.payer();
        let vault = self.find_market_vault_address(store, token);
        let builder = self
            .data_store()
            .request()
            .accounts(accounts::InitializeMarketVault {
                authority,
                store: *store,
                mint: *token,
                vault,
                system_program: system_program::ID,
                token_program: anchor_spl::token::ID,
            })
            .args(instruction::InitializeMarketVault {
                market_token_mint: None,
            });
        (builder, vault)
    }

    fn market_vault_transfer_out(
        &self,
        store: &Pubkey,
        token: &Pubkey,
        to: &Pubkey,
        amount: u64,
    ) -> RequestBuilder<C> {
        let authority = self.payer();
        self.data_store()
            .request()
            .accounts(accounts::MarketVaultTransferOut {
                authority,
                store: *store,
                market_vault: self.find_market_vault_address(store, token),
                to: *to,
                token_program: anchor_spl::token::ID,
            })
            .args(instruction::MarketVaultTransferOut { amount })
    }
}

/// Market Ops.
pub trait MarketOps<C> {
    /// Update market config.
    fn update_market_config(
        &self,
        store: &Pubkey,
        market_token: &Pubkey,
        key: &str,
        value: &Factor,
    ) -> crate::Result<RpcBuilder<C>>;

    /// Update market config by key.
    fn update_market_config_by_key(
        &self,
        store: &Pubkey,
        market_token: &Pubkey,
        key: MarketConfigKey,
        value: &Factor,
    ) -> crate::Result<RpcBuilder<C>> {
        let key = key.to_string();
        self.update_market_config(store, market_token, &key, value)
    }
}

impl<C, S> MarketOps<C> for crate::Client<C>
where
    C: Deref<Target = S> + Clone,
    S: Signer,
{
    fn update_market_config(
        &self,
        store: &Pubkey,
        market_token: &Pubkey,
        key: &str,
        value: &Factor,
    ) -> crate::Result<RpcBuilder<C>> {
        let req = self
            .data_store_request()
            .args(instruction::UpdateMarketConfig {
                key: key.to_string(),
                value: *value,
            })
            .accounts(accounts::UpdateMarketConfig {
                authority: self.payer(),
                store: *store,
                market: self.find_market_address(store, market_token),
            });
        Ok(req)
    }
}
