use std::{future::Future, time::Duration};

use solana_client::{
    client_error::ClientError as SolanaClientError,
    nonblocking::rpc_client::RpcClient,
    rpc_client::SerializableTransaction,
    rpc_config::RpcSendTransactionConfig,
    rpc_request::{RpcError, RpcRequest},
    rpc_response::Response,
};
use solana_sdk::{commitment_config::CommitmentConfig, signature::Signature};
use solana_transaction_status::TransactionStatus;
use tokio::time::sleep;

use crate::utils::WithSlot;

/// Add `send_and_confirm_transaction_with_config` method.
pub trait SendAndConfirm {
    /// Send and confirm a transaction.
    fn send_and_confirm_transaction_with_config(
        &self,
        transaction: &impl SerializableTransaction,
        config: RpcSendTransactionConfig,
    ) -> impl Future<Output = std::result::Result<WithSlot<Signature>, SolanaClientError>>;
}

impl SendAndConfirm for RpcClient {
    async fn send_and_confirm_transaction_with_config(
        &self,
        transaction: &impl SerializableTransaction,
        config: RpcSendTransactionConfig,
    ) -> std::result::Result<WithSlot<Signature>, SolanaClientError> {
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
                let result: Response<Vec<Option<TransactionStatus>>> = self
                    .send(
                        RpcRequest::GetSignatureStatuses,
                        serde_json::json!([[signature.to_string()]]),
                    )
                    .await?;
                let status = result.value[0]
                    .clone()
                    .filter(|result| result.satisfies_commitment(self.commitment()));

                match status {
                    Some(status) => match status.status {
                        Ok(()) => return Ok(WithSlot::new(status.slot, signature)),
                        Err(err) => return Err(err.into()),
                    },
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
