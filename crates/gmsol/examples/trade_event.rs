use std::sync::Arc;

use anchor_client::{
    solana_client::{
        nonblocking::pubsub_client::PubsubClient,
        rpc_config::{RpcTransactionLogsConfig, RpcTransactionLogsFilter},
    },
    solana_sdk::signature::Keypair,
    Cluster,
};
use futures_util::StreamExt;
use gmsol::{pda::find_default_store, Client};
use solana_transaction_status::UiTransactionEncoding;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }

    tracing_subscriber::fmt::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let _store = std::env::var("STORE")
        .ok()
        .map(|store| store.parse())
        .transpose()?
        .unwrap_or(find_default_store().0);

    let client = Client::new(Cluster::Devnet, Arc::new(Keypair::new()))?;
    let pub_sub = PubsubClient::new(client.cluster().ws_url()).await?;
    let query = client.data_store().async_rpc();
    let (mut stream, _unsubscribe) = pub_sub
        .logs_subscribe(
            RpcTransactionLogsFilter::Mentions(vec![client
                .data_store_event_authority()
                .to_string()]),
            RpcTransactionLogsConfig { commitment: None },
        )
        .await?;
    while let Some(res) = stream.next().await {
        let update = res.value;
        let signature = update.signature.parse()?;
        tracing::info!(%signature, "received");
        let res = query
            .get_transaction(&signature, UiTransactionEncoding::Base58)
            .await?;
        let slot = res.slot;
        let Some(iix) = res
            .transaction
            .meta
            .and_then(|meta| Option::<Vec<_>>::from(meta.inner_instructions))
        else {
            eyre::bail!("invalid encoding");
        };
        tracing::info!(%slot, "{iix:#?}");
    }
    Ok(())
}
