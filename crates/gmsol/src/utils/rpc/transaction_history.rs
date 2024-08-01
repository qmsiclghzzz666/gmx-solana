use anchor_client::{
    solana_client::{
        nonblocking::rpc_client::RpcClient, rpc_client::GetConfirmedSignaturesForAddress2Config,
    },
    solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey, signature::Signature},
    ClientError,
};
use async_stream::try_stream;
use futures_util::Stream;

use crate::utils::WithSlot;

/// Fetch transaction history for an address.
pub async fn fetch_transaction_history_with_config(
    client: RpcClient,
    address: &Pubkey,
    commitment: CommitmentConfig,
    until: Option<Signature>,
    mut before: Option<Signature>,
    batch: Option<usize>,
) -> crate::Result<impl Stream<Item = crate::Result<WithSlot<Signature>>>> {
    let limit = batch;
    let commitment = Some(commitment);
    let address = *address;

    let stream = try_stream! {
        loop {
            let txns = client.get_signatures_for_address_with_config(&address, GetConfirmedSignaturesForAddress2Config {
                before,
                until,
                limit,
                commitment,
            }).await.map_err(ClientError::from)?;
            match txns.last() {
                Some(next) => {
                    let next = next.signature.parse().map_err(crate::Error::unknown)?;
                    for txn in txns {
                        let slot = txn.slot;
                        let signature = txn.signature.parse().map_err(crate::Error::unknown)?;
                        yield WithSlot::new(slot, signature);
                    }
                    before = Some(next);
                },
                None => {
                    break;
                }
            }
        }
    };
    Ok(stream)
}

#[cfg(test)]
mod tests {
    use futures_util::StreamExt;

    use super::*;
    use crate::test::{default_cluster, setup_fmt_tracing};

    #[tokio::test]
    async fn test_transaction_hisotry_fetching() -> eyre::Result<()> {
        let _guard = setup_fmt_tracing("info");
        let cluster = default_cluster();
        let client = RpcClient::new(cluster.url().to_string());
        let stream = fetch_transaction_history_with_config(
            client,
            &crate::program_ids::DEFAULT_GMSOL_STORE_ID,
            CommitmentConfig::confirmed(),
            None,
            None,
            Some(5),
        )
        .await?
        .take(5);
        futures_util::pin_mut!(stream);
        while let Some(Ok(signature)) = stream.next().await {
            tracing::info!("{signature:?}");
        }
        Ok(())
    }
}
