use std::sync::Arc;

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

    let client = Client::new(Cluster::Devnet, Arc::new(Keypair::new()))?;

    let stream = client.store_cpi_events(None).await?;
    futures_util::pin_mut!(stream);
    while let Some(res) = stream.next().await {
        let Ok(event) = res.inspect_err(|err| tracing::error!(%err, "stream error")) else {
            continue;
        };
        tracing::info!(slot=%event.slot(), "{:?}", event.value());
    }
    tracing::info!("finished");
    Ok(())
}
