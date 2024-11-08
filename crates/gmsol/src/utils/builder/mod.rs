/// Estimate Execution Fee.
pub mod estimate_fee;

/// Set Oracle Price Feed.
pub mod oracle;

use std::future::Future;

use super::TransactionBuilder;

pub use estimate_fee::{EstimateFee, SetExecutionFee};

/// Builder for [`TransactionBuilder`]s.
pub trait Builder<'a, C> {
    /// Build.
    fn build(&mut self) -> impl Future<Output = crate::Result<TransactionBuilder<'a, C>>>;
}
