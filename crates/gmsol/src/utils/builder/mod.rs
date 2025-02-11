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

pub use gmsol_solana_utils::{bundle_builder::BundleBuilder, bundle_builder::BundleOptions};

/// Builder for [`BundleBuilder`]s.
pub trait MakeBundleBuilder<'a, C> {
    /// Build with options.
    fn build_with_options(
        &mut self,
        options: BundleOptions,
    ) -> impl Future<Output = crate::Result<BundleBuilder<'a, C>>>;

    /// Build.
    fn build(&mut self) -> impl Future<Output = crate::Result<BundleBuilder<'a, C>>> {
        self.build_with_options(Default::default())
    }
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
