use std::{ops::Deref, time::Duration};

use anchor_client::{
    solana_client::{
        client_error::ClientError as SolanaClientError, nonblocking::rpc_client::RpcClient,
        rpc_client::SerializableTransaction, rpc_config::RpcSendTransactionConfig,
        rpc_request::RpcError,
    },
    solana_sdk::{
        commitment_config::CommitmentConfig, packet::PACKET_DATA_SIZE, signature::Signature,
        signer::Signer, transaction::VersionedTransaction,
    },
    ClientError,
};
use futures_util::{stream::FuturesOrdered, TryStreamExt};
use tokio::time::sleep;

use super::RpcBuilder;

use self::transaction_size::transaction_size;

/// Compute Budget.
pub mod compute_budget;

/// RPC Builder.
pub mod rpc_builder;

/// Transaction size.
pub mod transaction_size;

/// Build transactions from [`RpcBuilder`].
pub struct TransactionBuilder<'a, C> {
    client: RpcClient,
    builders: Vec<RpcBuilder<'a, C>>,
    force_one_transaction: bool,
    max_packet_size: Option<usize>,
}

impl<'a, C> TransactionBuilder<'a, C> {
    /// Create a new [`TransactionBuilder`].
    pub fn new(client: RpcClient) -> Self {
        Self::new_with_options(client, false, None)
    }

    /// Create a new [`TransactionBuilder`] with the given options.
    pub fn new_with_options(
        client: RpcClient,
        force_one_transaction: bool,
        max_packet_size: Option<usize>,
    ) -> Self {
        Self {
            client,
            builders: Default::default(),
            force_one_transaction,
            max_packet_size,
        }
    }

    /// Get packet size.
    pub fn packet_size(&self) -> usize {
        match self.max_packet_size {
            Some(size) => size.min(PACKET_DATA_SIZE),
            None => PACKET_DATA_SIZE,
        }
    }
}

impl<'a, C: Deref<Target = impl Signer> + Clone> TransactionBuilder<'a, C> {
    /// Push a [`RpcBuilder`] with options.
    pub fn try_push_with_opts(
        &mut self,
        mut rpc: RpcBuilder<'a, C>,
        new_transaction: bool,
    ) -> Result<&mut Self, (RpcBuilder<'a, C>, crate::Error)> {
        let packet_size = self.packet_size();
        let mut ix = rpc.instructions_with_options(true, None);
        if transaction_size(&ix, true) > packet_size {
            return Err((
                rpc,
                crate::Error::invalid_argument("the size of this instruction is too big"),
            ));
        }
        if self.builders.is_empty() || new_transaction {
            tracing::debug!("adding to a new tx");
            if !self.builders.is_empty() && self.force_one_transaction {
                return Err((rpc, crate::Error::invalid_argument("cannot create more than one transaction because `force_one_transaction` is set")));
            }
            self.builders.push(rpc);
        } else {
            let last = self.builders.last_mut().unwrap();
            let mut ixs_after_merge = last.instructions_with_options(false, None);
            ixs_after_merge.append(&mut ix);
            let size_after_merge = transaction_size(&ixs_after_merge, true);
            if size_after_merge <= packet_size {
                tracing::debug!(size_after_merge, "adding to the last tx");
                last.try_merge(&mut rpc).map_err(|err| (rpc, err))?;
            } else {
                tracing::debug!(
                    size_after_merge,
                    "exceed packet data size limit, adding to a new tx"
                );
                if self.force_one_transaction {
                    return Err((rpc, crate::Error::invalid_argument("cannot create more than one transaction because `force_one_transaction` is set")));
                }
                self.builders.push(rpc);
            }
        }
        Ok(self)
    }

    /// Try to push a [`RpcBuilder`] to the builder.
    #[inline]
    pub fn try_push(
        &mut self,
        rpc: RpcBuilder<'a, C>,
    ) -> Result<&mut Self, (RpcBuilder<'a, C>, crate::Error)> {
        self.try_push_with_opts(rpc, false)
    }

    /// Push a [`RpcBuilder`].
    pub fn push(&mut self, rpc: RpcBuilder<'a, C>) -> crate::Result<&mut Self> {
        self.try_push(rpc).map_err(|(_, err)| err)
    }

    /// Push [`RpcBuilder`]s.
    pub fn push_many(
        &mut self,
        rpcs: impl IntoIterator<Item = RpcBuilder<'a, C>>,
        new_transaction: bool,
    ) -> crate::Result<&mut Self> {
        for (idx, rpc) in rpcs.into_iter().enumerate() {
            self.try_push_with_opts(rpc, (idx == 0) && new_transaction)?;
        }
        Ok(self)
    }

    /// Get back all collected [`RpcBuilder`]s.
    pub fn into_builders(self) -> Vec<RpcBuilder<'a, C>> {
        self.builders
    }

    /// Send all in order and returns the signatures of the success transactions.
    pub async fn send_all(self) -> Result<Vec<Signature>, (Vec<Signature>, crate::Error)> {
        self.send_all_with_opts(None, RpcSendTransactionConfig::default(), false)
            .await
    }

    /// Send all in order with the given options and returns the signatures of the success transactions.
    pub async fn send_all_with_opts(
        self,
        compute_unit_price_micro_lamports: Option<u64>,
        mut config: RpcSendTransactionConfig,
        without_compute_budget: bool,
    ) -> Result<Vec<Signature>, (Vec<Signature>, crate::Error)> {
        config.preflight_commitment = config
            .preflight_commitment
            .or(Some(self.client.commitment().commitment));
        let txs = FuturesOrdered::from_iter(self.builders.into_iter().enumerate().map(
            |(idx, builder)| async move {
                tracing::debug!(
                    size = builder.transaction_size(false),
                    "signing transaction {idx}"
                );
                builder
                    .signed_transaction_with_options(
                        without_compute_budget,
                        compute_unit_price_micro_lamports,
                    )
                    .await
            },
        ))
        .try_collect::<Vec<_>>()
        .await
        .map_err(|err| (vec![], err))?;
        send_all_txs(&self.client, txs, config).await
    }

    /// Estimated execution fee.
    pub async fn estimated_execution_fee(
        &self,
        compute_unit_price_micro_lamports: Option<u64>,
    ) -> crate::Result<u64> {
        self.builders
            .iter()
            .map(|rpc| rpc.estimate_execution_fee(&self.client, compute_unit_price_micro_lamports))
            .collect::<futures_util::stream::FuturesUnordered<_>>()
            .try_fold(0, |acc, fee| futures_util::future::ready(Ok(acc + fee)))
            .await
    }
}

impl<'a, C> IntoIterator for TransactionBuilder<'a, C> {
    type Item = RpcBuilder<'a, C>;

    type IntoIter = <Vec<RpcBuilder<'a, C>> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.builders.into_iter()
    }
}

impl<T> From<(T, crate::Error)> for crate::Error {
    fn from(value: (T, crate::Error)) -> Self {
        value.1
    }
}

async fn send_all_txs(
    client: &RpcClient,
    txs: impl IntoIterator<Item = VersionedTransaction>,
    config: RpcSendTransactionConfig,
) -> Result<Vec<Signature>, (Vec<Signature>, crate::Error)> {
    let txs = txs.into_iter();
    let (min, max) = txs.size_hint();
    let mut signatures = Vec::with_capacity(max.unwrap_or(min));
    let mut error = None;
    for (idx, tx) in txs.into_iter().enumerate() {
        tracing::debug!("sending transaction {idx}");
        match client
            .send_and_confirm_transaction_with_config(&tx, config)
            .await
        {
            Ok(signature) => {
                signatures.push(signature);
            }
            Err(err) => {
                error = Some(ClientError::from(err).into());
                break;
            }
        }
    }
    match error {
        None => Ok(signatures),
        Some(err) => Err((signatures, err)),
    }
}

trait SendAndConfirm {
    async fn send_and_confirm_transaction_with_config(
        &self,
        transaction: &impl SerializableTransaction,
        config: RpcSendTransactionConfig,
    ) -> Result<Signature, SolanaClientError>;
}

impl SendAndConfirm for RpcClient {
    async fn send_and_confirm_transaction_with_config(
        &self,
        transaction: &impl SerializableTransaction,
        config: RpcSendTransactionConfig,
    ) -> Result<Signature, SolanaClientError> {
        const SEND_RETRIES: usize = 1;
        const GET_STATUS_RETRIES: usize = usize::MAX;

        'sending: for _ in 0..SEND_RETRIES {
            let signature = self
                .send_transaction_with_config(transaction, config)
                .await?;

            let recent_blockhash = if transaction.uses_durable_nonce() {
                let (recent_blockhash, ..) = self
                    .get_latest_blockhash_with_commitment(CommitmentConfig::processed())
                    .await?;
                recent_blockhash
            } else {
                *transaction.get_recent_blockhash()
            };

            for status_retry in 0..GET_STATUS_RETRIES {
                match self.get_signature_status(&signature).await? {
                    Some(Ok(_)) => return Ok(signature),
                    Some(Err(e)) => return Err(e.into()),
                    None => {
                        if !self
                            .is_blockhash_valid(&recent_blockhash, CommitmentConfig::processed())
                            .await?
                        {
                            // Block hash is not found by some reason
                            break 'sending;
                        } else if cfg!(not(test))
                            // Ignore sleep at last step.
                            && status_retry < GET_STATUS_RETRIES
                        {
                            // Retry twice a second
                            sleep(Duration::from_millis(500)).await;
                            continue;
                        }
                    }
                }
            }
        }

        Err(RpcError::ForUser(
            "unable to confirm transaction. \
             This can happen in situations such as transaction expiration \
             and insufficient fee-payer funds"
                .to_string(),
        )
        .into())
    }
}
