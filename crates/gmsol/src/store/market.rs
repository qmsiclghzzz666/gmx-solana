use std::ops::Deref;

use anchor_client::{
    anchor_lang::system_program,
    solana_sdk::{pubkey::Pubkey, signer::Signer},
};
use gmsol_model::{action::Prices, PnlFactorKind, PoolKind};
use gmsol_store::{
    accounts, instruction,
    states::{config::EntryArgs, Factor, MarketConfigKey},
};

use crate::utils::RpcBuilder;

/// Vault Operations.
pub trait VaultOps<C> {
    /// Initialize a market vault for the given token.
    fn initialize_market_vault(&self, store: &Pubkey, token: &Pubkey) -> (RpcBuilder<C>, Pubkey);

    /// Transfer tokens out from the given market vault.
    fn market_vault_transfer_out(
        &self,
        store: &Pubkey,
        token: &Pubkey,
        to: &Pubkey,
        amount: u64,
    ) -> RpcBuilder<C>;
}

impl<C, S> VaultOps<C> for crate::Client<C>
where
    C: Deref<Target = S> + Clone,
    S: Signer,
{
    fn initialize_market_vault(&self, store: &Pubkey, token: &Pubkey) -> (RpcBuilder<C>, Pubkey) {
        let authority = self.payer();
        let vault = self.find_market_vault_address(store, token);
        let builder = self
            .data_store_rpc()
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
    ) -> RpcBuilder<C> {
        let authority = self.payer();
        self.data_store_rpc()
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
    /// Get market status.
    fn get_market_status(
        &self,
        store: &Pubkey,
        market_token: &Pubkey,
        prices: Prices<u128>,
        maximize_pnl: bool,
        maximize_pool_value: bool,
    ) -> RpcBuilder<C>;

    /// Get market token price.
    fn get_market_token_price(
        &self,
        store: &Pubkey,
        market_token: &Pubkey,
        prices: Prices<u128>,
        pnl_factor: PnlFactorKind,
        maximize: bool,
    ) -> RpcBuilder<C>;

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

    /// Toggle market.
    fn toggle_market(&self, store: &Pubkey, market_token: &Pubkey, enable: bool) -> RpcBuilder<C>;

    /// Toggle GT minting.
    fn toggle_gt_minting(
        &self,
        store: &Pubkey,
        market_token: &Pubkey,
        enable: bool,
    ) -> RpcBuilder<C>;

    /// Initialize Market Config Buffer.
    fn initialize_market_config_buffer<'a>(
        &'a self,
        store: &Pubkey,
        buffer: &'a dyn Signer,
        expire_after_secs: u32,
    ) -> RpcBuilder<'a, C>;

    /// Close Market Config Buffer.
    fn close_marekt_config_buffer(
        &self,
        buffer: &Pubkey,
        receiver: Option<&Pubkey>,
    ) -> RpcBuilder<C>;

    /// Push to Market Config Buffer.
    fn push_to_market_config_buffer<S: ToString>(
        &self,
        buffer: &Pubkey,
        new_configs: impl IntoIterator<Item = (S, Factor)>,
    ) -> RpcBuilder<C>;

    /// Set the authority of the Market Config Buffer.
    fn set_market_config_buffer_authority(
        &self,
        buffer: &Pubkey,
        new_authority: &Pubkey,
    ) -> RpcBuilder<C>;

    /// Update Market Config with the buffer.
    fn update_market_config_with_buffer(
        &self,
        store: &Pubkey,
        market_token: &Pubkey,
        buffer: &Pubkey,
    ) -> RpcBuilder<C>;

    /// Turn an impure pool into a pure pool.
    fn turn_into_pure_pool(
        &self,
        store: &Pubkey,
        market_token: &Pubkey,
        kind: PoolKind,
    ) -> RpcBuilder<C>;

    /// Turn an pure pool into a impure pool.
    fn turn_into_impure_pool(
        &self,
        store: &Pubkey,
        market_token: &Pubkey,
        kind: PoolKind,
    ) -> RpcBuilder<C>;
}

impl<C, S> MarketOps<C> for crate::Client<C>
where
    C: Deref<Target = S> + Clone,
    S: Signer,
{
    fn get_market_status(
        &self,
        store: &Pubkey,
        market_token: &Pubkey,
        prices: Prices<u128>,
        maximize_pnl: bool,
        maximize_pool_value: bool,
    ) -> RpcBuilder<C> {
        self.data_store_rpc()
            .args(instruction::GetMarketStatus {
                prices,
                maximize_pnl,
                maximize_pool_value,
            })
            .accounts(accounts::ReadMarket {
                market: self.find_market_address(store, market_token),
            })
    }

    fn get_market_token_price(
        &self,
        store: &Pubkey,
        market_token: &Pubkey,
        prices: Prices<u128>,
        pnl_factor: PnlFactorKind,
        maximize: bool,
    ) -> RpcBuilder<C> {
        self.data_store_rpc()
            .args(instruction::GetMarketTokenPrice {
                prices,
                pnl_factor: pnl_factor.to_string(),
                maximize,
            })
            .accounts(accounts::ReadMarketWithToken {
                market: self.find_market_address(store, market_token),
                market_token: *market_token,
            })
    }

    fn update_market_config(
        &self,
        store: &Pubkey,
        market_token: &Pubkey,
        key: &str,
        value: &Factor,
    ) -> crate::Result<RpcBuilder<C>> {
        let req = self
            .data_store_rpc()
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

    fn toggle_market(&self, store: &Pubkey, market_token: &Pubkey, enable: bool) -> RpcBuilder<C> {
        self.data_store_rpc()
            .args(instruction::ToggleMarket { enable })
            .accounts(accounts::ToggleMarket {
                authority: self.payer(),
                store: *store,
                market: self.find_market_address(store, market_token),
            })
    }

    fn toggle_gt_minting(
        &self,
        store: &Pubkey,
        market_token: &Pubkey,
        enable: bool,
    ) -> RpcBuilder<C> {
        self.data_store_rpc()
            .args(instruction::ToggleGtMinting { enable })
            .accounts(accounts::ToggleGTMinting {
                authority: self.payer(),
                store: *store,
                market: self.find_market_address(store, market_token),
            })
    }

    fn initialize_market_config_buffer<'a>(
        &'a self,
        store: &Pubkey,
        buffer: &'a dyn Signer,
        expire_after_secs: u32,
    ) -> RpcBuilder<'a, C> {
        self.data_store_rpc()
            .args(instruction::InitializeMarketConfigBuffer { expire_after_secs })
            .accounts(accounts::InitializeMarketConfigBuffer {
                authority: self.payer(),
                store: *store,
                buffer: buffer.pubkey(),
                system_program: system_program::ID,
            })
            .signer(buffer)
    }

    fn close_marekt_config_buffer(
        &self,
        buffer: &Pubkey,
        receiver: Option<&Pubkey>,
    ) -> RpcBuilder<C> {
        self.data_store_rpc()
            .args(instruction::CloseMarketConfigBuffer {})
            .accounts(accounts::CloseMarketConfigBuffer {
                authority: self.payer(),
                buffer: *buffer,
                receiver: receiver.copied().unwrap_or(self.payer()),
            })
    }

    fn push_to_market_config_buffer<K: ToString>(
        &self,
        buffer: &Pubkey,
        new_configs: impl IntoIterator<Item = (K, Factor)>,
    ) -> RpcBuilder<C> {
        self.data_store_rpc()
            .args(instruction::PushToMarketConfigBuffer {
                new_configs: new_configs
                    .into_iter()
                    .map(|(key, value)| EntryArgs {
                        key: key.to_string(),
                        value,
                    })
                    .collect(),
            })
            .accounts(accounts::PushToMarketConfigBuffer {
                authority: self.payer(),
                buffer: *buffer,
                system_program: system_program::ID,
            })
    }

    fn set_market_config_buffer_authority(
        &self,
        buffer: &Pubkey,
        new_authority: &Pubkey,
    ) -> RpcBuilder<C> {
        self.data_store_rpc()
            .args(instruction::SetMarketConfigBufferAuthority {
                new_authority: *new_authority,
            })
            .accounts(accounts::SetMarketConfigBufferAuthority {
                authority: self.payer(),
                buffer: *buffer,
            })
    }

    fn update_market_config_with_buffer(
        &self,
        store: &Pubkey,
        market_token: &Pubkey,
        buffer: &Pubkey,
    ) -> RpcBuilder<C> {
        self.data_store_rpc()
            .args(instruction::UpdateMarketConfigWithBuffer {})
            .accounts(accounts::UpdateMarketConfigWithBuffer {
                authority: self.payer(),
                store: *store,
                market: self.find_market_address(store, market_token),
                buffer: *buffer,
            })
    }

    fn turn_into_pure_pool(
        &self,
        store: &Pubkey,
        market_token: &Pubkey,
        kind: PoolKind,
    ) -> RpcBuilder<C> {
        self.data_store_rpc()
            .args(instruction::TurnIntoPurePool { kind: kind.into() })
            .accounts(accounts::TurnPureFlag {
                authority: self.payer(),
                store: *store,
                market: self.find_market_address(store, market_token),
            })
    }

    fn turn_into_impure_pool(
        &self,
        store: &Pubkey,
        market_token: &Pubkey,
        kind: PoolKind,
    ) -> RpcBuilder<C> {
        self.data_store_rpc()
            .args(instruction::TurnIntoImpurePool { kind: kind.into() })
            .accounts(accounts::TurnPureFlag {
                authority: self.payer(),
                store: *store,
                market: self.find_market_address(store, market_token),
            })
    }
}
