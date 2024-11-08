use std::{collections::HashSet, future::Future};

use anchor_client::solana_sdk::pubkey::Pubkey;
use gmsol_store::states::common::TokensWithFeed;

use crate::utils::TransactionBuilder;

use super::Builder;

/// A mapping from feed id to the corresponding feed address.
pub type FeedAddressMap = std::collections::HashMap<Pubkey, Pubkey>;

/// Build with pull oracle instructions.
pub struct WithPullOracle<O, T> {
    builder: T,
    pull_oracle: O,
}

impl<'a, C, O, T> Builder<'a, C> for WithPullOracle<O, T>
where
    O: PullOracle<'a, C>,
    T: PullOraclePriceConsumer + Builder<'a, C>,
{
    async fn build(&mut self) -> crate::Result<TransactionBuilder<'a, C>> {
        todo!()
    }
}

/// Price Update Instructions.
pub struct PriceUpdateInstructions<'a, C> {
    post: TransactionBuilder<'a, C>,
    close: Option<TransactionBuilder<'a, C>>,
}

/// Pull Oracle.
pub trait PullOracle<'a, C> {
    /// Price Updates.
    type PriceUpdates;

    /// Fetch Price Update.
    fn fetch_price_updates(
        &self,
        feed_ids: &HashSet<Pubkey>,
    ) -> impl Future<Output = crate::Result<Self::PriceUpdates>>;

    /// Fetch instructions to post the price updates.
    fn fetch_price_update_instructions(
        &self,
        price_updates: &Self::PriceUpdates,
    ) -> impl Future<Output = crate::Result<(PriceUpdateInstructions<'a, C>, FeedAddressMap)>>;
}

/// Pull Oracle Price Consumer.
pub trait PullOraclePriceConsumer {
    /// Returns a reference to tokens and their associated feed IDs that require price updates.
    fn feed_ids(&self) -> &TokensWithFeed;

    /// Processes the feed address map returned from the pull oracle.
    fn process_feeds(&mut self, map: FeedAddressMap) -> crate::Result<()>;
}
