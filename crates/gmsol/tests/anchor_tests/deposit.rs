use crate::anchor_tests::setup;

#[tokio::test]
async fn create_deposit() -> eyre::Result<()> {
    let deployment = setup::current_deployment().await?;
    let _guard = deployment.use_accounts().await?;

    deployment
        .mint_or_transfer_to("WSOL", "user_0", 1_000_000_000)
        .await?;

    tracing::info!("token map: {}", deployment.token_map());

    Ok(())
}

#[tokio::test]
async fn create_deposit_2() -> eyre::Result<()> {
    let deployment = setup::current_deployment().await?;
    let _guard = deployment.use_accounts().await?;

    deployment
        .mint_or_transfer_to("fBTC", "user_0", 1_000_000_000)
        .await?;

    Ok(())
}
