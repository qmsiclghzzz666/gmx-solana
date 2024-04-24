use std::ops::Deref;

use anchor_client::{
    anchor_lang::system_program,
    solana_sdk::{pubkey::Pubkey, signer::Signer},
    Program, RequestBuilder,
};
use data_store::{
    accounts, instruction,
    states::{Seed, TokenConfig, TokenConfigMap},
};

use crate::utils::view;

use super::roles::find_roles_address;

/// Find PDA for [`TokenConfigMap`] account.
pub fn find_token_config_map(store: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[TokenConfigMap::SEED, store.as_ref()], &data_store::id())
}

/// Get token config for the given token.
pub async fn get_token_config<C, S>(
    program: &Program<C>,
    store: &Pubkey,
    token: &Pubkey,
) -> crate::Result<Option<TokenConfig>>
where
    C: Deref<Target = S> + Clone + Send + Sync,
    S: Signer,
{
    let client = program.async_rpc();
    let output = view(
        &client,
        &program
            .get_token_config(store, token)
            .signed_transaction()
            .await?,
    )
    .await?;
    Ok(output)
}

/// Token config management for GMSOL.
pub trait TokenConfigOps<C> {
    /// Initialize [`TokenConfigMap`] account.
    fn initialize_token_config_map(&self, store: &Pubkey) -> (RequestBuilder<C>, Pubkey);

    /// Insert or update config for the given token.
    fn insert_token_config(
        &self,
        store: &Pubkey,
        token: &Pubkey,
        price_feed: &Pubkey,
        heartbeat_duration: u32,
        precision: u8,
    ) -> RequestBuilder<C>;

    /// Insert or update config the given fake token.
    fn insert_fake_token_config(
        &self,
        store: &Pubkey,
        token: &Pubkey,
        decimals: u8,
        price_feed: &Pubkey,
        heartbeat_duration: u32,
        precision: u8,
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
}

impl<C, S> TokenConfigOps<C> for Program<C>
where
    C: Deref<Target = S> + Clone + Send + Sync,
    S: Signer,
{
    fn initialize_token_config_map(&self, store: &Pubkey) -> (RequestBuilder<C>, Pubkey) {
        let authority = self.payer();
        let map = find_token_config_map(store).0;
        let builder = self
            .request()
            .accounts(accounts::InitializeTokenConfigMap {
                authority,
                only_controller: find_roles_address(store, &authority).0,
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
        price_feed: &Pubkey,
        heartbeat_duration: u32,
        precision: u8,
    ) -> RequestBuilder<C> {
        let authority = self.payer();
        let only_controller = find_roles_address(store, &authority).0;
        let map = find_token_config_map(store).0;
        self.request()
            .accounts(accounts::InsertTokenConfig {
                authority,
                only_controller,
                store: *store,
                map,
                token: *token,
                system_program: system_program::ID,
            })
            .args(instruction::InsertTokenConfig {
                price_feed: *price_feed,
                heartbeat_duration,
                precision,
            })
    }

    fn insert_fake_token_config(
        &self,
        store: &Pubkey,
        token: &Pubkey,
        decimals: u8,
        price_feed: &Pubkey,
        heartbeat_duration: u32,
        precision: u8,
    ) -> RequestBuilder<C> {
        let authority = self.payer();
        let only_controller = find_roles_address(store, &authority).0;
        let map = find_token_config_map(store).0;
        self.request()
            .accounts(accounts::InsertFakeTokenConfig {
                authority,
                only_controller,
                store: *store,
                map,
                system_program: system_program::ID,
            })
            .args(instruction::InsertFakeTokenConfig {
                token: *token,
                decimals,
                price_feed: *price_feed,
                heartbeat_duration,
                precision,
            })
    }

    fn get_token_config(&self, store: &Pubkey, token: &Pubkey) -> RequestBuilder<C> {
        let map = find_token_config_map(store).0;
        self.request()
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
        let only_controller = find_roles_address(store, &authority).0;
        let map = find_token_config_map(store).0;
        self.request()
            .accounts(accounts::ToggleTokenConfig {
                authority,
                store: *store,
                only_controller,
                map,
            })
            .args(instruction::ToggleTokenConfig {
                token: *token,
                enable,
            })
    }
}
