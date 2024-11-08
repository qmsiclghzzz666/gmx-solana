use std::ops::Deref;

use anchor_client::solana_sdk::signer::Signer;

use crate::utils::TransactionBuilder;

use super::Builder;

/// Estimate Execution Fee.
pub struct EstimateFee<T> {
    builder: T,
    compute_unit_price_micro_lamports: Option<u64>,
}

/// Set Execution Fee.
pub trait SetExecutionFee {
    /// Whether the execution fee needed to be estiamted.
    fn is_execution_fee_estimation_required(&self) -> bool {
        true
    }

    /// Set execution fee.
    fn set_execution_fee(&mut self, lamports: u64);
}

impl<'a, C: Deref<Target = impl Signer> + Clone, T> Builder<'a, C> for EstimateFee<T>
where
    T: SetExecutionFee,
    T: Builder<'a, C>,
{
    async fn build(&mut self) -> crate::Result<TransactionBuilder<'a, C>> {
        let mut tx = self.builder.build().await?;

        if self.builder.is_execution_fee_estimation_required() {
            let lamports = tx
                .estimate_execution_fee(self.compute_unit_price_micro_lamports)
                .await?;
            self.builder.set_execution_fee(lamports);
            tracing::info!(%lamports, "execution fee estimated");
            tx = self.builder.build().await?;
        }

        Ok(tx)
    }
}
