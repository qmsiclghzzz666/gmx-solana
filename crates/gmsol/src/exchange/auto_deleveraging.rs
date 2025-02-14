use std::ops::Deref;

use anchor_client::{
    anchor_lang::Id,
    solana_sdk::{pubkey::Pubkey, signer::Signer},
};
use gmsol_solana_utils::{
    bundle_builder::{BundleBuilder, BundleOptions},
    transaction_builder::TransactionBuilder,
};
use gmsol_store::states::{common::TokensWithFeed, Market, PriceProviderKind, Pyth};

use crate::{
    store::utils::FeedsParser,
    utils::{
        builder::{
            FeedAddressMap, FeedIds, MakeBundleBuilder, PullOraclePriceConsumer, SetExecutionFee,
        },
        fix_optional_account_metas,
    },
};

/// The compute budget for `auto_deleverage`.
pub const ADL_COMPUTE_BUDGET: u32 = 800_000;

/// Update ADL state Instruction Builder.
pub struct UpdateAdlBuilder<'a, C> {
    client: &'a crate::Client<C>,
    store: Pubkey,
    market_token: Pubkey,
    oracle: Pubkey,
    is_long: bool,
    price_provider: Pubkey,
    hint: Option<UpdateAdlHint>,
    feeds_parser: FeedsParser,
}

impl<'a, C: Deref<Target = impl Signer> + Clone> UpdateAdlBuilder<'a, C> {
    pub(super) fn try_new(
        client: &'a crate::Client<C>,
        store: &Pubkey,
        oracle: &Pubkey,
        market_token: &Pubkey,
        is_long: bool,
    ) -> crate::Result<Self> {
        Ok(Self {
            client,
            store: *store,
            market_token: *market_token,
            oracle: *oracle,
            is_long,
            price_provider: Pyth::id(),
            hint: None,
            feeds_parser: FeedsParser::default(),
        })
    }

    /// Prepare hint for auto-deleveraging.
    pub async fn prepare_hint(&mut self) -> crate::Result<UpdateAdlHint> {
        match &self.hint {
            Some(hint) => Ok(hint.clone()),
            None => {
                let market_address = self
                    .client
                    .find_market_address(&self.store, &self.market_token);
                let market = self.client.market(&market_address).await?;
                let hint = UpdateAdlHint::from_market(self.client, &market).await?;
                self.hint = Some(hint.clone());
                Ok(hint)
            }
        }
    }

    /// Set price provider to the given.
    pub fn price_provider(&mut self, program: &Pubkey) -> &mut Self {
        self.price_provider = *program;
        self
    }

    /// Build [`TransactionBuilder`] for auto-delevearaging the position.
    pub async fn build_txn(&mut self) -> crate::Result<TransactionBuilder<'a, C>> {
        let hint = self.prepare_hint().await?;
        let feeds = self
            .feeds_parser
            .parse(hint.feeds())
            .collect::<Result<Vec<_>, _>>()?;
        let rpc = self
            .client
            .store_transaction()
            .accounts(fix_optional_account_metas(
                gmsol_store::accounts::UpdateAdlState {
                    authority: self.client.payer(),
                    store: self.store,
                    token_map: hint.token_map,
                    oracle: self.oracle,
                    market: self
                        .client
                        .find_market_address(&self.store, &self.market_token),
                    chainlink_program: None,
                },
                &crate::program_ids::DEFAULT_GMSOL_STORE_ID,
                self.client.store_program_id(),
            ))
            .anchor_args(gmsol_store::instruction::UpdateAdlState {
                is_long: self.is_long,
            })
            .accounts(feeds);
        Ok(rpc)
    }
}

/// Hint for `update_adl_state`.
#[derive(Clone)]
pub struct UpdateAdlHint {
    token_map: Pubkey,
    tokens_with_feed: TokensWithFeed,
}

impl UpdateAdlHint {
    async fn from_market<C: Deref<Target = impl Signer> + Clone>(
        client: &crate::Client<C>,
        market: &Market,
    ) -> crate::Result<Self> {
        use gmsol_store::states::common::token_with_feeds::token_records;

        let store_address = market.store;
        let token_map_address = client
            .authorized_token_map_address(&store_address)
            .await?
            .ok_or(crate::Error::invalid_argument(
                "token map is not configurated for the store",
            ))?;
        let token_map = client.token_map(&token_map_address).await?;
        let meta = market.meta();

        let records = token_records(
            &token_map,
            &[
                meta.index_token_mint,
                meta.long_token_mint,
                meta.short_token_mint,
            ]
            .into(),
        )?;
        let tokens_with_feed = TokensWithFeed::try_from_records(records)?;

        Ok(Self {
            token_map: token_map_address,
            tokens_with_feed,
        })
    }

    /// Get feeds.
    pub fn feeds(&self) -> &TokensWithFeed {
        &self.tokens_with_feed
    }
}

impl<'a, C: Deref<Target = impl Signer> + Clone> MakeBundleBuilder<'a, C>
    for UpdateAdlBuilder<'a, C>
{
    async fn build_with_options(
        &mut self,
        options: BundleOptions,
    ) -> crate::Result<BundleBuilder<'a, C>> {
        let mut tx = self.client.bundle_with_options(options);

        tx.try_push(self.build_txn().await?)?;

        Ok(tx)
    }
}

impl<C: Deref<Target = impl Signer> + Clone> PullOraclePriceConsumer for UpdateAdlBuilder<'_, C> {
    async fn feed_ids(&mut self) -> crate::Result<FeedIds> {
        let hint = self.prepare_hint().await?;
        Ok(FeedIds::new(self.store, hint.tokens_with_feed))
    }

    fn process_feeds(
        &mut self,
        provider: PriceProviderKind,
        map: FeedAddressMap,
    ) -> crate::Result<()> {
        self.feeds_parser
            .insert_pull_oracle_feed_parser(provider, map);
        Ok(())
    }
}

impl<C> SetExecutionFee for UpdateAdlBuilder<'_, C> {
    fn is_execution_fee_estimation_required(&self) -> bool {
        false
    }

    fn set_execution_fee(&mut self, _lamports: u64) -> &mut Self {
        self
    }
}
