use crate::anchor_tests::setup;

#[tokio::test]
async fn create_deposit() -> eyre::Result<()> {
    let deployment = setup::current_deployment().await?;

    let _guard = deployment.use_accounts().await?;

    tracing::info!("hello: {deployment:#?}");

    Ok(())
}

#[tokio::test]
async fn create_deposit_2() -> eyre::Result<()> {
    let deployment = setup::current_deployment().await?;

    let _guard = deployment.use_accounts().await?;

    tracing::info!("hello 2: {deployment:#?}");

    Ok(())
}
