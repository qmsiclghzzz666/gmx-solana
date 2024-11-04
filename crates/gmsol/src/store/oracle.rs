use std::{future::Future, ops::Deref};

use anchor_client::{
    anchor_lang::system_program,
    solana_sdk::{pubkey::Pubkey, signer::Signer},
};
use gmsol_store::{
    accounts, instruction,
    states::{Oracle, PriceProviderKind},
};
use gmsol_utils::InitSpace;

use crate::utils::RpcBuilder;

/// Oracle management for GMSOL.
pub trait OracleOps<C> {
    /// Initialize [`Oracle`] account.
    fn initialize_oracle<'a>(
        &'a self,
        store: &Pubkey,
        oracle: &'a dyn Signer,
    ) -> impl Future<Output = crate::Result<(RpcBuilder<'a, C>, Pubkey)>>;

    /// Initialize Price Feed.
    fn initailize_price_feed(
        &self,
        store: &Pubkey,
        index: u8,
        provider: PriceProviderKind,
        token: &Pubkey,
        feed_id: &Pubkey,
    ) -> (RpcBuilder<C>, Pubkey);

    /// Update price feed with chainlink.
    fn update_price_feed_with_chainlink(
        &self,
        store: &Pubkey,
        verifier_account: &Pubkey,
        price_feed: &Pubkey,
        chainlink: &Pubkey,
        signed_report: Vec<u8>,
    ) -> RpcBuilder<C>;
}

impl<C, S> OracleOps<C> for crate::Client<C>
where
    C: Deref<Target = S> + Clone,
    S: Signer,
{
    async fn initialize_oracle<'a>(
        &'a self,
        store: &Pubkey,
        oracle: &'a dyn Signer,
    ) -> crate::Result<(RpcBuilder<'a, C>, Pubkey)> {
        use anchor_client::solana_sdk::system_instruction::create_account;

        let payer = self.payer();
        let oracle_address = oracle.pubkey();

        let size = 8 + Oracle::INIT_SPACE;
        let lamports = self
            .data_store()
            .solana_rpc()
            .get_minimum_balance_for_rent_exemption(size)
            .await
            .map_err(anchor_client::ClientError::from)?;
        let create = create_account(
            &payer,
            &oracle_address,
            lamports,
            size as u64,
            self.store_program_id(),
        );

        let builder = self
            .store_rpc()
            .pre_instruction(create)
            .accounts(accounts::InitializeOracle {
                payer,
                store: *store,
                oracle: oracle_address,
                system_program: system_program::ID,
            })
            .args(instruction::InitializeOracle {})
            .signer(oracle);
        Ok((builder, oracle_address))
    }

    fn initailize_price_feed(
        &self,
        store: &Pubkey,
        index: u8,
        provider: PriceProviderKind,
        token: &Pubkey,
        feed_id: &Pubkey,
    ) -> (RpcBuilder<C>, Pubkey) {
        let authority = self.payer();
        let price_feed = self.find_price_feed_address(store, &authority, index, provider, token);
        let rpc = self
            .store_rpc()
            .accounts(accounts::InitializePriceFeed {
                authority,
                store: *store,
                price_feed,
                system_program: system_program::ID,
            })
            .args(instruction::InitializePriceFeed {
                index,
                provider: provider.into(),
                token: *token,
                feed_id: *feed_id,
            });
        (rpc, price_feed)
    }

    fn update_price_feed_with_chainlink(
        &self,
        store: &Pubkey,
        verifier_account: &Pubkey,
        price_feed: &Pubkey,
        chainlink: &Pubkey,
        signed_report: Vec<u8>,
    ) -> RpcBuilder<C> {
        let authority = self.payer();
        self.store_rpc()
            .accounts(accounts::UpdatePriceFeedWithChainlink {
                authority,
                store: *store,
                verifier_account: *verifier_account,
                price_feed: *price_feed,
                chainlink: *chainlink,
            })
            .args(instruction::UpdatePriceFeedWithChainlink { signed_report })
    }
}
