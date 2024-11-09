use std::{future::Future, ops::Deref};

use anchor_client::solana_sdk::{pubkey::Pubkey, signer::Signer};
use gmsol_store::states::common::TokensWithFeed;

use crate::utils::TransactionBuilder;

use super::{Builder, SetExecutionFee};

/// A mapping from feed id to the corresponding feed address.
pub type FeedAddressMap = std::collections::HashMap<Pubkey, Pubkey>;

/// Build with pull oracle instructions.
pub struct WithPullOracle<O, T>
where
    O: PullOracle,
{
    builder: T,
    pull_oracle: O,
    price_updates: O::PriceUpdates,
}

impl<O: PullOracle, T> WithPullOracle<O, T> {
    /// Construct transactions with the given pull oracle and price updates.
    pub fn with_price_updates(pull_oracle: O, builder: T, price_updates: O::PriceUpdates) -> Self {
        Self {
            builder,
            pull_oracle,
            price_updates,
        }
    }

    /// Fetch the required price updates and use them to construct transactions.
    pub async fn new(pull_oracle: O, builder: T) -> crate::Result<Self>
    where
        T: PullOraclePriceConsumer,
    {
        let feed_ids = builder.feed_ids();
        let price_updates = pull_oracle.fetch_price_updates(feed_ids).await?;

        Ok(Self::with_price_updates(
            pull_oracle,
            builder,
            price_updates,
        ))
    }
}

impl<'a, C: Deref<Target = impl Signer> + Clone, O, T> Builder<'a, C> for WithPullOracle<O, T>
where
    O: PullOracleOps<'a, C>,
    T: PullOraclePriceConsumer + Builder<'a, C>,
{
    async fn build(&mut self) -> crate::Result<TransactionBuilder<'a, C>> {
        let (instructions, map) = self
            .pull_oracle
            .fetch_price_update_instructions(&self.price_updates)
            .await?;

        self.builder.process_feeds(map)?;

        let consume = self.builder.build().await?;

        let PriceUpdateInstructions {
            post: mut tx,
            close,
        } = instructions;

        tx.append(consume, false)?;

        if let Some(close) = close {
            tx.append(close, true)?;
        }

        Ok(tx)
    }
}

impl<O: PullOracle, T: SetExecutionFee> SetExecutionFee for WithPullOracle<O, T> {
    fn set_execution_fee(&mut self, lamports: u64) {
        self.builder.set_execution_fee(lamports)
    }
}

/// Pull Oracle.
pub trait PullOracle {
    /// Price Updates.
    type PriceUpdates;

    /// Fetch Price Update.
    fn fetch_price_updates(
        &self,
        feed_ids: &TokensWithFeed,
    ) -> impl Future<Output = crate::Result<Self::PriceUpdates>>;
}

/// Price Update Instructions.
pub struct PriceUpdateInstructions<'a, C> {
    post: TransactionBuilder<'a, C>,
    close: Option<TransactionBuilder<'a, C>>,
}

/// Pull Oracle Operations.
pub trait PullOracleOps<'a, C>: PullOracle {
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
