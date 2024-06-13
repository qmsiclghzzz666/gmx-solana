use std::ops::Deref;

use anchor_client::{
    anchor_lang::system_program,
    solana_sdk::{pubkey::Pubkey, signer::Signer},
    RequestBuilder,
};
use data_store::{
    accounts, instruction,
    states::{PriceProviderKind, TokenConfigBuilder},
};

use crate::utils::RpcBuilder;

/// Token config management for GMSOL.
pub trait TokenConfigOps<C> {
    /// Initialize a  `TokenMap` account.
    fn initialize_token_map<'a>(
        &'a self,
        store: &Pubkey,
        token_map: &'a dyn Signer,
    ) -> (RpcBuilder<'a, C>, Pubkey);

    /// Insert or update config for the given token.
    #[allow(clippy::too_many_arguments)]
    fn insert_token_config(
        &self,
        store: &Pubkey,
        token_map: &Pubkey,
        name: &str,
        token: &Pubkey,
        builder: TokenConfigBuilder,
        enable: bool,
        new: bool,
    ) -> RpcBuilder<C>;

    /// Insert or update config the given synthetic token.
    // FIXME: reduce the number of args.
    #[allow(clippy::too_many_arguments)]
    fn insert_synthetic_token_config(
        &self,
        store: &Pubkey,
        token_map: &Pubkey,
        name: &str,
        token: &Pubkey,
        decimals: u8,
        builder: TokenConfigBuilder,
        enable: bool,
        new: bool,
    ) -> RpcBuilder<C>;

    /// Toggle token config.
    fn toggle_token_config(
        &self,
        store: &Pubkey,
        token_map: &Pubkey,
        token: &Pubkey,
        enable: bool,
    ) -> RequestBuilder<C>;

    /// Set expected provider.
    fn set_expected_provider(
        &self,
        store: &Pubkey,
        token_map: &Pubkey,
        token: &Pubkey,
        provider: PriceProviderKind,
    ) -> RequestBuilder<C>;
}

impl<C, S> TokenConfigOps<C> for crate::Client<C>
where
    C: Deref<Target = S> + Clone,
    S: Signer,
{
    fn initialize_token_map<'a>(
        &'a self,
        store: &Pubkey,
        token_map: &'a dyn Signer,
    ) -> (RpcBuilder<'a, C>, Pubkey) {
        let builder = self
            .data_store_rpc()
            .accounts(accounts::InitializeTokenMap {
                payer: self.payer(),
                store: *store,
                token_map: token_map.pubkey(),
                system_program: system_program::ID,
            })
            .args(instruction::InitializeTokenMap {})
            .signer(token_map);
        (builder, token_map.pubkey())
    }

    fn insert_token_config(
        &self,
        store: &Pubkey,
        token_map: &Pubkey,
        name: &str,
        token: &Pubkey,
        builder: TokenConfigBuilder,
        enable: bool,
        new: bool,
    ) -> RpcBuilder<C> {
        let authority = self.payer();
        self.data_store_rpc()
            .accounts(accounts::PushToTokenMap {
                authority,
                store: *store,
                token_map: *token_map,
                token: *token,
                system_program: system_program::ID,
            })
            .args(instruction::PushToTokenMap {
                name: name.to_owned(),
                builder,
                enable,
                new,
            })
    }

    fn insert_synthetic_token_config(
        &self,
        store: &Pubkey,
        token_map: &Pubkey,
        name: &str,
        token: &Pubkey,
        decimals: u8,
        builder: TokenConfigBuilder,
        enable: bool,
        new: bool,
    ) -> RpcBuilder<C> {
        let authority = self.payer();
        self.data_store_rpc()
            .accounts(accounts::PushToTokenMapSynthetic {
                authority,
                store: *store,
                token_map: *token_map,
                system_program: system_program::ID,
            })
            .args(instruction::PushToTokenMapSynthetic {
                name: name.to_owned(),
                token: *token,
                token_decimals: decimals,
                builder,
                enable,
                new,
            })
    }

    fn toggle_token_config(
        &self,
        store: &Pubkey,
        token_map: &Pubkey,
        token: &Pubkey,
        enable: bool,
    ) -> RequestBuilder<C> {
        let authority = self.payer();
        self.data_store()
            .request()
            .accounts(accounts::ToggleTokenConfig {
                authority,
                store: *store,
                token_map: *token_map,
            })
            .args(instruction::ToggleTokenConfig {
                token: *token,
                enable,
            })
    }

    fn set_expected_provider(
        &self,
        store: &Pubkey,
        token_map: &Pubkey,
        token: &Pubkey,
        provider: PriceProviderKind,
    ) -> RequestBuilder<C> {
        let authority = self.payer();
        self.data_store()
            .request()
            .accounts(accounts::SetExpectedProvider {
                authority,
                store: *store,
                token_map: *token_map,
            })
            .args(instruction::SetExpectedProvider {
                token: *token,
                provider: provider as u8,
            })
    }
}
