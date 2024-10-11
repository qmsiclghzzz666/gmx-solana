use gmsol::exchange::ExchangeOps;

use crate::anchor_tests::setup::current_deployment;

#[tokio::test]
async fn claim_fees() -> eyre::Result<()> {
    let deployment = current_deployment().await?;
    let _guard = deployment.use_accounts().await?;
    let span = tracing::info_span!("claim_fees");
    let _enter = span.enter();

    let store = &deployment.store;
    let market_token = deployment
        .market_token("fBTC", "fBTC", "USDG")
        .expect("must exist");

    deployment.wait_until_claim_fees_enabled().await;

    let signature = deployment
        .client
        .claim_fees(store, market_token, false)
        .build()
        .await?
        .send_without_preflight()
        .await?;

    tracing::info!(%signature, "claimed fees");

    Ok(())
}
