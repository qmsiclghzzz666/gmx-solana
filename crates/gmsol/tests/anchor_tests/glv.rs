use gmsol::store::glv::GlvOps;

use crate::anchor_tests::setup::{current_deployment, Deployment};

#[tokio::test]
async fn initialize_glv() -> eyre::Result<()> {
    let deployment = current_deployment().await?;
    let _guard = deployment.use_accounts().await?;
    let span = tracing::info_span!("initialize_glv");
    let _enter = span.enter();

    let store = &deployment.store;
    let keeper = deployment.user_client(Deployment::DEFAULT_KEEPER)?;

    let market_token_1 = deployment
        .market_token("fBTC", "fBTC", "USDG")
        .expect("must exist");
    let market_token_2 = deployment
        .market_token("SOL", "fBTC", "USDG")
        .expect("must exist");

    let index = 255;
    let (rpc, glv_token) = keeper.initialize_glv(store, 255, [*market_token_1, *market_token_2]);
    let signature = rpc.send_without_preflight().await?;
    tracing::info!(%signature, %index, %glv_token, "initalized a new GLV token");

    Ok(())
}
