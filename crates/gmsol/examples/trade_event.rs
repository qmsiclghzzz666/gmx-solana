use std::{sync::Arc, time::Duration};

use anchor_client::{solana_sdk::signature::Keypair, Cluster};
use futures_util::StreamExt;
use gmsol::{pda::find_default_store, Client};

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

    let client = Arc::new(Client::new(Cluster::Devnet, Arc::new(Keypair::new()))?);

    let mut idx = 0;
    let mut interval = tokio::time::interval(Duration::from_secs(5));
    let handle = tokio::spawn({
        let client = client.clone();
        async move {
            loop {
                interval.tick().await;
                let Ok(stream) = client
                    .store_cpi_events(None)
                    .await
                    .inspect_err(|err| tracing::error!(%err, "[{idx}] subscription error"))
                else {
                    continue;
                };
                futures_util::pin_mut!(stream);
                while let Some(res) = stream.next().await {
                    let Ok(event) =
                        res.inspect_err(|err| tracing::error!(%err, "[{idx}] stream error"))
                    else {
                        continue;
                    };
                    tracing::info!(slot=%event.slot(), "[{idx}] {:?}", event.value());
                }
                tracing::info!("[{idx}] stream end");
                idx += 1;
            }
        }
    });

    tokio::signal::ctrl_c().await?;
    tracing::info!("received `ctrl + c`, shutting down gracefully...");
    handle.abort();
    _ = handle.await;
    client.shutdown().await?;
    Ok(())
}
