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

/// Token config management for GMSOL.
pub trait TokenConfigOps<C> {
    /// Initialize [`TokenConfigMap`] account.
    fn initialize_token_config_map(&self, store: &Pubkey) -> (RequestBuilder<C>, Pubkey);

    /// Insert or update config for the given token.
    fn insert_token_config(
        &self,
        store: &Pubkey,
        token: &Pubkey,
        builder: TokenConfigBuilder,
        enable: bool,
    ) -> RequestBuilder<C>;

    /// Insert or update config the given synthetic token.
    fn insert_synthetic_token_config(
        &self,
        store: &Pubkey,
        token: &Pubkey,
        decimals: u8,
        builder: TokenConfigBuilder,
        enable: bool,
    ) -> RequestBuilder<C>;

    /// Get token config of the given token.
    fn get_token_config(&self, store: &Pubkey, token: &Pubkey) -> RequestBuilder<C>;

    /// Toggle token config.
    fn toggle_token_config(
        &self,
        store: &Pubkey,
        token: &Pubkey,
        enable: bool,
    ) -> RequestBuilder<C>;

    /// Set expected provider.
    fn set_expected_provider(
        &self,
        store: &Pubkey,
        token: &Pubkey,
        provider: PriceProviderKind,
    ) -> RequestBuilder<C>;
}

impl<C, S> TokenConfigOps<C> for crate::Client<C>
where
    C: Deref<Target = S> + Clone,
    S: Signer,
{
    fn initialize_token_config_map(&self, store: &Pubkey) -> (RequestBuilder<C>, Pubkey) {
        let authority = self.payer();
        let map = self.find_token_config_map(store);
        let builder = self
            .data_store()
            .request()
            .accounts(accounts::InitializeTokenConfigMap {
                authority,
                store: *store,
                map,
                system_program: system_program::ID,
            })
            .args(instruction::InitializeTokenConfigMap { len: 0 });
        (builder, map)
    }

    fn insert_token_config(
        &self,
        store: &Pubkey,
        token: &Pubkey,
        builder: TokenConfigBuilder,
        enable: bool,
    ) -> RequestBuilder<C> {
        let authority = self.payer();
        let map = self.find_token_config_map(store);
        self.data_store()
            .request()
            .accounts(accounts::InsertTokenConfig {
                authority,
                store: *store,
                map,
                token: *token,
                system_program: system_program::ID,
            })
            .args(instruction::InsertTokenConfig { builder, enable })
    }

    fn insert_synthetic_token_config(
        &self,
        store: &Pubkey,
        token: &Pubkey,
        decimals: u8,
        builder: TokenConfigBuilder,
        enable: bool,
    ) -> RequestBuilder<C> {
        let authority = self.payer();
        let map = self.find_token_config_map(store);
        self.data_store()
            .request()
            .accounts(accounts::InsertSyntheticTokenConfig {
                authority,
                store: *store,
                map,
                system_program: system_program::ID,
            })
            .args(instruction::InsertSyntheticTokenConfig {
                token: *token,
                decimals,
                builder,
                enable,
            })
    }

    fn get_token_config(&self, store: &Pubkey, token: &Pubkey) -> RequestBuilder<C> {
        let map = self.find_token_config_map(store);
        self.data_store()
            .request()
            .accounts(accounts::GetTokenConfig { map })
            .args(instruction::GetTokenConfig {
                store: *store,
                token: *token,
            })
    }

    fn toggle_token_config(
        &self,
        store: &Pubkey,
        token: &Pubkey,
        enable: bool,
    ) -> RequestBuilder<C> {
        let authority = self.payer();
        let map = self.find_token_config_map(store);
        self.data_store()
            .request()
            .accounts(accounts::ToggleTokenConfig {
                authority,
                store: *store,
                map,
            })
            .args(instruction::ToggleTokenConfig {
                token: *token,
                enable,
            })
    }

    fn set_expected_provider(
        &self,
        store: &Pubkey,
        token: &Pubkey,
        provider: PriceProviderKind,
    ) -> RequestBuilder<C> {
        let authority = self.payer();
        let map = self.find_token_config_map(store);
        self.data_store()
            .request()
            .accounts(accounts::SetExpectedProvider {
                authority,
                store: *store,
                map,
            })
            .args(instruction::SetExpectedProvider {
                token: *token,
                provider: provider as u8,
            })
    }
}
