use gmsol::store::glv::GlvOps;
use tracing::Instrument;

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
        .market_token("fBTC", "WSOL", "USDG")
        .expect("must exist");
    let market_token_2 = deployment
        .market_token("SOL", "WSOL", "USDG")
        .expect("must exist");

    let index = 255;
    let (rpc, glv_token) = keeper.initialize_glv(store, 255, [*market_token_1, *market_token_2])?;
    let signature = rpc.send_without_preflight().await?;
    tracing::info!(%signature, %index, %glv_token, "initialized a new GLV token");

    Ok(())
}

#[tokio::test]
async fn glv_deposit() -> eyre::Result<()> {
    let deployment = current_deployment().await?;
    let _guard = deployment.use_accounts().await?;
    let span = tracing::info_span!("glv_deposit");
    let _enter = span.enter();

    let user = deployment.user_client(Deployment::DEFAULT_USER)?;
    let keeper = deployment.user_client(Deployment::DEFAULT_KEEPER)?;

    let store = &deployment.store;
    let oracle = &deployment.oracle;
    let glv_token = &deployment.glv_token;
    let market_token = deployment.market_token("fBTC", "fBTC", "USDG").unwrap();

    let long_token_amount = 1_000;

    deployment
        .mint_or_transfer_to_user("fBTC", Deployment::DEFAULT_USER, long_token_amount + 14)
        .await?;

    let (rpc, deposit) = user
        .create_glv_deposit(store, glv_token, market_token)
        .long_token_deposit(long_token_amount, None, None)
        .build_with_address()
        .await?;
    let signature = rpc.send_without_preflight().await?;
    tracing::info!(%signature, %deposit, "created a glv deposit");

    let signature = user
        .close_glv_deposit(&deposit)
        .build()
        .await?
        .send_without_preflight()
        .await?;
    tracing::info!(%signature, %deposit, "cancelled a glv deposit");

    let (rpc, deposit) = user
        .create_glv_deposit(store, glv_token, market_token)
        .long_token_deposit(long_token_amount, None, None)
        .build_with_address()
        .await?;
    let signature = rpc.send_without_preflight().await?;
    tracing::info!(%signature, %deposit, "created a glv deposit again");

    let mut execute = keeper.execute_glv_deposit(oracle, &deposit, false);
    deployment
        .execute_with_pyth(
            execute
                .add_alt(deployment.common_alt().clone())
                .add_alt(deployment.market_alt().clone()),
            None,
            false,
            true,
        )
        .instrument(tracing::info_span!("executing glv deposit", glv_deposit=%deposit))
        .await?;

    Ok(())
}
