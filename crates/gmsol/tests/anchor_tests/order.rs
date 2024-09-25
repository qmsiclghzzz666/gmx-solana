use anchor_client::solana_client::rpc_config::RpcSendTransactionConfig;
use gmsol::exchange::ExchangeOps;

use crate::anchor_tests::setup::{current_deployment, Deployment};

#[tokio::test]
async fn balanced_market_order() -> eyre::Result<()> {
    let deployment = current_deployment().await?;
    let _guard = deployment.use_accounts().await?;
    let span = tracing::info_span!("balanced_market_order");
    let _enter = span.enter();

    let keeper = deployment.user_client(Deployment::DEFAULT_KEEPER)?;
    let client = deployment.user_client(Deployment::DEFAULT_USER)?;
    let store = &deployment.store;
    let oracle = &deployment.oracle;
    let market_token = deployment
        .market_token("fBTC", "fBTC", "USDG")
        .expect("must exist");

    let long_token_amount = 1_000_005;
    let short_token_amount = 6_000_000_000_003;
    let long_collateral_amount = 100_000;

    deployment
        .mint_or_transfer_to_user(
            "fBTC",
            Deployment::DEFAULT_USER,
            long_token_amount + long_collateral_amount,
        )
        .await?;
    deployment
        .mint_or_transfer_to_user("USDG", Deployment::DEFAULT_USER, short_token_amount)
        .await?;

    // Deposit.
    let (rpc, deposit) = client
        .create_deposit(store, market_token)
        .long_token(long_token_amount, None, None)
        .short_token(short_token_amount, None, None)
        .build_with_address()
        .await?;
    let signature = rpc
        .build()
        .send_with_spinner_and_config(RpcSendTransactionConfig {
            skip_preflight: true,
            ..Default::default()
        })
        .await?;
    tracing::info!(%deposit, %signature, "created a deposit");
    let mut builder = keeper.execute_deposit(store, oracle, &deposit, false);
    deployment
        .execute_with_pyth(&mut builder, None, true)
        .await?;

    // Increase position.
    let size = 10_000_000_000_000_000_000_000;
    let (rpc, order) = client
        .market_increase(
            store,
            market_token,
            true,
            long_collateral_amount,
            true,
            size,
        )
        .build_with_address()
        .await?;
    let signature = rpc.send().await?;
    tracing::info!(%order, %signature, %size, "created an increase position order");

    let mut builder = keeper.execute_order(store, oracle, &order, false)?;
    deployment
        .execute_with_pyth(&mut builder, None, true)
        .await?;

    // Decrease position.
    let size = 10_000_000_000_000_000_000_000;
    let (rpc, order) = client
        .market_decrease(
            store,
            market_token,
            true,
            long_collateral_amount,
            true,
            size,
        )
        .build_with_address()
        .await?;
    let signature = rpc.send().await?;
    tracing::info!(%order, %signature, %size, "created a decrease position order");

    let mut builder = keeper.execute_order(store, oracle, &order, false)?;
    deployment
        .execute_with_pyth(&mut builder, None, true)
        .await?;

    Ok(())
}
