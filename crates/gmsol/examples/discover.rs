use anchor_client::Cluster;
use futures_util::future::poll_fn;
use gmsol::{
    discover::{market::MarketDiscovery, token::TokenDiscovery},
    pda::find_default_store,
};
use tower::discover::Discover;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }

    tracing_subscriber::fmt::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let store = std::env::var("STORE")
        .ok()
        .map(|store| store.parse())
        .transpose()?
        .unwrap_or(find_default_store().0);

    let markets = MarketDiscovery::new_with_store(Cluster::Devnet, store)?;
    let tokens = TokenDiscovery::new(markets);

    futures_util::pin_mut!(tokens);
    while let Some(Ok(change)) = poll_fn(|cx| tokens.as_mut().poll_discover(cx)).await {
        tracing::info!("{change:?}");
    }

    Ok(())
}
