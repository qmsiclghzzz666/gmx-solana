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
    let usdg = deployment.token("USDG").expect("must exist");

    let long_token_amount = 1_000_005;
    let short_token_amount = 6_000_000_000_003;

    let market_token = deployment
        .prepare_market(
            ("fBTC", "fBTC", "USDG"),
            long_token_amount,
            short_token_amount,
            true,
        )
        .await?;

    let long_collateral_amount = 100_000;
    let short_collateral_amount = 100 * 100_000_000;

    deployment
        .mint_or_transfer_to_user("fBTC", Deployment::DEFAULT_USER, long_collateral_amount * 4)
        .await?;
    deployment
        .mint_or_transfer_to_user(
            "USDG",
            Deployment::DEFAULT_USER,
            short_collateral_amount * 4,
        )
        .await?;

    // Increase position.
    let size = 10_000_000_000_000_000_000_000;

    for side in [true, false] {
        for collateral_side in [true, false] {
            let collateral_amount = if collateral_side {
                long_collateral_amount
            } else {
                short_collateral_amount
            };
            let (rpc, order) = client
                .market_increase(
                    store,
                    market_token,
                    collateral_side,
                    collateral_amount,
                    side,
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
            let (rpc, order) = client
                .market_decrease(store, market_token, collateral_side, 0, side, size)
                .build_with_address()
                .await?;
            let signature = rpc.send().await?;
            tracing::info!(%order, %signature, %size, "created a decrease position order");

            let mut builder = keeper.execute_order(store, oracle, &order, false)?;
            deployment
                .execute_with_pyth(&mut builder, None, true)
                .await?;
        }
    }

    let side = true;
    let collateral_side = true;
    let collateral_amount = short_collateral_amount;

    // Increase position with swap path.
    let size = 10_000_000_000_000_000_000_000;
    let (rpc, order) = client
        .market_increase(
            store,
            market_token,
            collateral_side,
            collateral_amount,
            side,
            size,
        )
        .initial_collateral_token(&usdg.address, None)
        .swap_path(vec![*market_token])
        .build_with_address()
        .await?;
    let signature = rpc.send().await?;
    tracing::info!(%order, %signature, %size, "created an increase position order");

    let mut builder = keeper.execute_order(store, oracle, &order, false)?;
    deployment
        .execute_with_pyth(&mut builder, None, true)
        .await?;

    // Extract collateral.
    let amount = 1_00;
    let (rpc, order) = client
        .market_decrease(store, market_token, true, amount, side, 0)
        .build_with_address()
        .await?;
    let signature = rpc.send().await?;
    tracing::info!(%order, %signature, "created an order to extract collateral");

    let mut builder = keeper.execute_order(store, oracle, &order, false)?;
    deployment
        .execute_with_pyth(&mut builder, None, true)
        .await?;

    // Extract collateral and swap.
    let amount = 1_00;
    let (rpc, order) = client
        .market_decrease(store, market_token, true, amount, side, 0)
        .final_output_token(&usdg.address)
        .swap_path(vec![*market_token])
        .build_with_address()
        .await?;
    let signature = rpc.send().await?;
    tracing::info!(%order, %signature, "created an order to extract collateral and swap");

    let mut builder = keeper.execute_order(store, oracle, &order, false)?;
    deployment
        .execute_with_pyth(&mut builder, None, true)
        .await?;

    // Fully decrease and swap.
    let (rpc, order) = client
        .market_decrease(store, market_token, true, 0, side, size)
        .final_output_token(&usdg.address)
        .swap_path(vec![*market_token])
        .build_with_address()
        .await?;
    let signature = rpc.send().await?;
    tracing::info!(%order, %signature, %size, "created an order to fully decrease the position and swap");

    let mut builder = keeper.execute_order(store, oracle, &order, false)?;
    deployment
        .execute_with_pyth(&mut builder, None, true)
        .await?;
    Ok(())
}
