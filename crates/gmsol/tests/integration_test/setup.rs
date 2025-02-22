use tokio::sync::OnceCell;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

use super::test_client::TestClient;

/// Initialize tracing
fn init_tracing() {
    if let Err(err) = tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::ERROR.into())
                .from_env_lossy(),
        )
        .try_init()
    {
        tracing::trace!(%err, "failed to initialize tracing");
    }
}

/// Initialize.
pub async fn init() -> eyre::Result<Option<&'static TestClient>> {
    static CLIENT: OnceCell<Option<TestClient>> = OnceCell::const_new();

    Ok(CLIENT
        .get_or_try_init::<eyre::Error, _, _>(|| async {
            init_tracing();
            match TestClient::from_envs()? {
                Some(client) => eyre::Result::Ok(Some(client)),
                None => {
                    tracing::debug!("Integration test is not enabled");
                    eyre::Result::Ok(None)
                }
            }
        })
        .await?
        .as_ref())
}
