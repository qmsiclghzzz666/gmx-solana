use futures_util::StreamExt;
use gmsol::pyth::Hermes;
use pyth_sdk::Identifier;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }

    tracing_subscriber::fmt::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let hermes = Hermes::default();

    let ids = vec![Identifier::from_hex(
        "e62df6c8b4a85fe1a67db44dc12de5db330f7ac66b72dc658afedf0f4a415b43",
    )?];

    let stream = hermes.price_updates(&ids, None).await?;
    futures_util::pin_mut!(stream);

    while let Some(update) = stream.next().await {
        match update {
            Ok(update) => {
                println!("{update:?}");
            }
            Err(err) => {
                tracing::error!(%err, "Stream error");
            }
        }
    }
    Ok(())
}
