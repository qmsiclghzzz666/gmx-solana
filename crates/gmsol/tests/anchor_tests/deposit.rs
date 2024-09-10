use crate::anchor_tests::setup;

#[tokio::test]
async fn create_deposit() -> eyre::Result<()> {
    let deployment = setup::current_deployment().await?;

    tracing::info!("hello: {deployment:#?}");
    Ok(())
}
