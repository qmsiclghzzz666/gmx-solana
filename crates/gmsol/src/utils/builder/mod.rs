/// Estimate Execution Fee.
pub mod estimate_fee;

/// Set Oracle Price Feed.
pub mod oracle;

/// Surround transaction.
pub mod surround;

use std::future::Future;

pub use self::{
    estimate_fee::{EstimateFee, SetExecutionFee},
    oracle::{
        FeedAddressMap, FeedIds, PostPullOraclePrices, PriceUpdateInstructions, PullOracle,
        PullOraclePriceConsumer, WithPullOracle,
    },
    surround::Surround,
};
use gmsol_solana_utils::bundle_builder::BundleBuilder;

/// Builder for [`BundleBuilder`]s.
pub trait MakeBundleBuilder<'a, C> {
    /// Build.
    fn build(&mut self) -> impl Future<Output = crate::Result<BundleBuilder<'a, C>>>;
}

/// Extension trait for [`MakeBundleBuilder`].
pub trait MakeBundleBuilderExt<'a, C>: MakeBundleBuilder<'a, C> {
    /// Surround the current builder.
    fn surround(self) -> Surround<'a, C, Self>
    where
        Self: Sized,
    {
        self.into()
    }
}

impl<'a, C, T: MakeBundleBuilder<'a, C> + ?Sized> MakeBundleBuilderExt<'a, C> for T {}
