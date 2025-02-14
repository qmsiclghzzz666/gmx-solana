use std::{ops::Deref, sync::Arc};

use gmsol::{
    chainlink::{self, pull_oracle::ChainlinkPullOracleFactory},
    pyth::{pull_oracle::PythPullOracleWithHermes, Hermes, PythPullOracle},
    utils::{
        builder::{
            EstimateFee, MakeBundleBuilder, PullOraclePriceConsumer, SetExecutionFee,
            WithPullOracle,
        },
        instruction::InstructionSerialization,
    },
};
use gmsol_solana_utils::bundle_builder::BundleOptions;
use solana_sdk::{pubkey::Pubkey, signer::Signer};

use crate::InstructionBufferCtx;

pub(crate) struct Executor<'a, C> {
    store: Pubkey,
    client: &'a gmsol::Client<C>,
    chainlink: Option<(chainlink::Client, Arc<ChainlinkPullOracleFactory>)>,
    pyth: PythPullOracle<C>,
    hermes: Hermes,
}

impl<'a, C: Deref<Target = impl Signer> + Clone> Executor<'a, C> {
    pub(crate) fn new_with_envs(
        store: &Pubkey,
        client: &'a gmsol::Client<C>,
        testnet: bool,
        feed_index: u8,
    ) -> gmsol::Result<Self> {
        let pyth = PythPullOracle::try_new(client)?;
        let chainlink = if testnet {
            chainlink::Client::from_testnet_defaults()
        } else {
            chainlink::Client::from_defaults()
        }
        .inspect_err(|_err| {
            tracing::warn!("Chainlink envs is not provided, continue without chainlink support")
        })
        .ok()
        .map(|client| {
            (
                client,
                ChainlinkPullOracleFactory::new(store, feed_index).arced(),
            )
        });

        Ok(Self {
            store: *store,
            client,
            chainlink,
            pyth,
            hermes: Default::default(),
        })
    }

    pub(crate) async fn execute<'b>(
        &'b self,
        consumer: impl PullOraclePriceConsumer + MakeBundleBuilder<'b, C> + SetExecutionFee,
        ctx: Option<InstructionBufferCtx<'_>>,
        serialize_only: Option<InstructionSerialization>,
        skip_preflight: bool,
        max_transaction_size: Option<usize>,
        compute_unit_price: Option<u64>,
    ) -> gmsol::Result<()> {
        let options = BundleOptions {
            max_packet_size: max_transaction_size,
            ..Default::default()
        };
        let pyth = PythPullOracleWithHermes::from_parts(self.client, &self.hermes, &self.pyth);
        let with_pyth = WithPullOracle::new(pyth, consumer, None).await?;

        let chainlink = self
            .chainlink
            .as_ref()
            .map(|(client, factory)| factory.clone().make_oracle(client, self.client, true));

        let bundle = if let Some(chainlink) = chainlink.as_ref() {
            let (with_chainlink, feed_ids) =
                WithPullOracle::from_consumer(chainlink.clone(), with_pyth, None).await?;

            let mut bundle = chainlink
                .prepare_feeds_bundle(&feed_ids, options.clone())
                .await?;

            let mut estiamted_fee = EstimateFee::new(with_chainlink, compute_unit_price);

            bundle.append(estiamted_fee.build_with_options(options).await?, false)?;

            bundle
        } else {
            let mut estiamted_fee = EstimateFee::new(with_pyth, compute_unit_price);
            estiamted_fee.build_with_options(options).await?
        };

        super::send_or_serialize_bundle_with_default_callback(
            &self.store,
            bundle,
            ctx,
            serialize_only,
            skip_preflight,
        )
        .await?;

        Ok(())
    }
}
