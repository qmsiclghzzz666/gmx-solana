/// Wormhole Ops.
pub mod wormhole;

/// Pyth Reciever Ops.
pub mod receiver;

/// Hermes.
pub mod hermes;

/// Utils.
pub mod utils;

use std::{collections::HashMap, future::Future, ops::Deref};

use anchor_client::{
    solana_client::rpc_config::RpcSendTransactionConfig,
    solana_sdk::{
        pubkey::Pubkey,
        signature::{Keypair, Signature},
        signer::Signer,
    },
    Client, Program,
};
use gmsol_store::states::common::TokensWithFeed;
use pyth_sdk::Identifier;

use crate::utils::{RpcBuilder, TransactionBuilder};

use self::wormhole::WORMHOLE_PROGRAM_ID;

pub use self::{receiver::PythReceiverOps, wormhole::WormholeOps};

use self::hermes::PriceUpdate;

const VAA_SPLIT_INDEX: usize = 755;

/// With Pyth Prices.
pub struct WithPythPrices<'a, C> {
    post: TransactionBuilder<'a, C>,
    close: TransactionBuilder<'a, C>,
}

impl<'a, S, C> WithPythPrices<'a, C>
where
    C: Deref<Target = S> + Clone,
    S: Signer,
{
    /// Estimate execution fee.
    pub async fn estimated_execution_fee(
        &self,
        compute_unit_price_micro_lamports: Option<u64>,
    ) -> crate::Result<u64> {
        let mut execution_fee = self
            .post
            .estimated_execution_fee(compute_unit_price_micro_lamports)
            .await?;
        execution_fee = execution_fee.saturating_add(
            self.close
                .estimated_execution_fee(compute_unit_price_micro_lamports)
                .await?,
        );
        Ok(execution_fee)
    }

    /// Send all transactions.
    pub async fn send_all(
        self,
        compute_unit_price_micro_lamports: Option<u64>,
        skip_preflight: bool,
    ) -> Result<Vec<Signature>, (Vec<Signature>, crate::Error)> {
        let mut error = None;

        let mut signatures = match self
            .post
            .send_all_with_opts(
                compute_unit_price_micro_lamports,
                RpcSendTransactionConfig {
                    skip_preflight,
                    ..Default::default()
                },
                false,
            )
            .await
        {
            Ok(signatures) => signatures,
            Err((signatures, err)) => {
                error = Some(err);
                signatures
            }
        };

        let mut close_signatures = match self
            .close
            .send_all_with_opts(
                compute_unit_price_micro_lamports,
                RpcSendTransactionConfig {
                    skip_preflight,
                    ..Default::default()
                },
                false,
            )
            .await
        {
            Ok(signatures) => signatures,
            Err((signatures, err)) => {
                match &error {
                    None => error = Some(err),
                    Some(post_err) => {
                        error = Some(crate::Error::unknown(format!(
                            "post error: {post_err}, close error: {err}"
                        )));
                    }
                }
                signatures
            }
        };

        signatures.append(&mut close_signatures);

        match error {
            None => Ok(signatures),
            Some(err) => Err((signatures, err)),
        }
    }
}

/// Prices.
pub type Prices = HashMap<Identifier, Pubkey>;

/// Pyth Pull Oracle Context.
pub struct PythPullOracleContext {
    encoded_vaa: Keypair,
    feeds: HashMap<Identifier, Keypair>,
    feed_ids: Vec<Identifier>,
}

impl PythPullOracleContext {
    /// Create a new [`PythPullOracleContext`].
    pub fn new(feed_ids: Vec<Identifier>) -> Self {
        let feeds = feed_ids.iter().map(|id| (*id, Keypair::new())).collect();
        Self {
            encoded_vaa: Keypair::new(),
            feeds,
            feed_ids,
        }
    }

    /// Create a new [`PythPullOracleContext`] from [`TokensWithFeed`].
    pub fn try_from_feeds(feeds: &TokensWithFeed) -> crate::Result<Self> {
        let feed_ids = utils::extract_pyth_feed_ids(feeds)?;
        Ok(Self::new(feed_ids))
    }

    /// Get feed ids.
    pub fn feed_ids(&self) -> &[Identifier] {
        &self.feed_ids
    }
}

/// Pyth Pull Oracle Ops.
pub trait PythPullOracleOps<C> {
    /// Get Pyth Program.
    fn pyth(&self) -> &Program<C>;

    /// Get Wormhole Program.
    fn wormhole(&self) -> &Program<C>;

    /// Create transactions to post price updates and consume the prices.
    fn with_pyth_prices<'a, S, It, Fut>(
        &'a self,
        ctx: &'a PythPullOracleContext,
        update: &PriceUpdate,
        consume: impl FnOnce(Prices) -> Fut,
    ) -> impl Future<Output = crate::Result<WithPythPrices<'a, C>>>
    where
        C: Deref<Target = S> + Clone + 'a,
        S: Signer,
        It: IntoIterator<Item = RpcBuilder<'a, C>>,
        Fut: Future<Output = crate::Result<It>>,
    {
        async {
            let wormhole = self.wormhole();
            let pyth = self.pyth();
            let mut prices = HashMap::with_capacity(ctx.feeds.len());
            let mut post = TransactionBuilder::new(pyth.async_rpc());
            let mut close = TransactionBuilder::new(pyth.async_rpc());

            for data in utils::parse_accumulator_update_datas(update)? {
                let proof = &data.proof;
                let guardian_set_index = utils::get_guardian_set_index(proof)?;
                let draft_vaa = ctx.encoded_vaa.pubkey();
                let vaa = utils::get_vaa_buffer(proof);

                let create = wormhole
                    .create_encoded_vaa(&ctx.encoded_vaa, vaa.len() as u64)
                    .await?;
                let write_1 = wormhole.write_encoded_vaa(&draft_vaa, 0, &vaa[0..VAA_SPLIT_INDEX]);
                let write_2 = wormhole.write_encoded_vaa(
                    &draft_vaa,
                    VAA_SPLIT_INDEX as u32,
                    &vaa[VAA_SPLIT_INDEX..],
                );
                let verify = wormhole.verify_encoded_vaa_v1(&draft_vaa, guardian_set_index);
                let close_encoded_vaa = wormhole.close_encoded_vaa(&draft_vaa);

                post.try_push(create.clear_output())?
                    .try_push(write_1)?
                    .try_push(write_2)?
                    .try_push(verify)?;
                close.try_push(close_encoded_vaa)?;

                let mut price_updates = HashMap::<Pubkey, _>::default();
                for update in utils::get_merkle_price_updates(proof) {
                    let feed_id = utils::parse_feed_id(update)?;
                    let Some(price_update) = ctx.feeds.get(&feed_id) else {
                        continue;
                    };
                    let (post_price_update, price_update) = pyth
                        .post_price_update(price_update, update, &draft_vaa)?
                        .swap_output(());
                    prices.insert(feed_id, price_update);
                    price_updates.insert(price_update, post_price_update);
                }
                for (price_update, post_rpc) in price_updates {
                    post.try_push(post_rpc)?;
                    close.try_push(pyth.reclaim_rent(&price_update))?;
                }
            }
            let consume = (consume)(prices).await?;
            post.try_push_many(consume, true)?;
            Ok(WithPythPrices { post, close })
        }
    }
}

/// Pyth Pull Oracle.
pub struct PythPullOracle<C> {
    wormhole: Program<C>,
    pyth: Program<C>,
}

impl<S, C> PythPullOracle<C>
where
    C: Deref<Target = S> + Clone,
    S: Signer,
{
    /// Create a new [`PythPullOracle`] client from [`Client`].
    pub fn try_new(client: &Client<C>) -> crate::Result<Self> {
        Ok(Self {
            wormhole: client.program(WORMHOLE_PROGRAM_ID)?,
            pyth: client.program(pyth_solana_receiver_sdk::ID)?,
        })
    }
}

impl<S, C> PythPullOracleOps<C> for PythPullOracle<C>
where
    C: Deref<Target = S> + Clone,
    S: Signer,
{
    fn pyth(&self) -> &Program<C> {
        &self.pyth
    }

    fn wormhole(&self) -> &Program<C> {
        &self.wormhole
    }
}
