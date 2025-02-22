use crate::integration_test::setup::init;

#[tokio::test]
async fn use_switchboard() -> eyre::Result<()> {
    let Some(client) = init().await? else {
        return Ok(());
    };

    let user = client.client();
    let keeper = client
        .keeper_client()
        .expect("keeper client is not defined");
    let store = client.store();
    let oracle = client.oracle().expect("oracle account is not defined");
    tracing::info!(user=%user.payer(), keeper=%keeper.payer(), %store, %oracle, "testing switchboard integration");

    Ok(())
}
