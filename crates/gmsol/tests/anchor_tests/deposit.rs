use gmsol::exchange::ExchangeOps;

use crate::anchor_tests::setup::{self, Deployment};

#[tokio::test]
async fn single_token_pool_deposit() -> eyre::Result<()> {
    let deployment = setup::current_deployment().await?;
    let _guard = deployment.use_accounts().await?;

    deployment
        .mint_or_transfer_to("WSOL", None, 21_000_000)
        .await?;

    {
        let span = tracing::info_span!("single_token_pool_deposit");
        let _enter = span.enter();

        let client = deployment.locked_user_client().await?;

        let keeper = deployment.user_client(Deployment::DEFAULT_KEEPER)?.unwrap();
        let store = &deployment.store;
        let oracle = &deployment.oracle;
        let market_token = deployment.market_token("SOL", "WSOL", "WSOL").unwrap();
        let wsol = deployment.token("WSOL").expect("must exist");

        let wsol_before = deployment
            .get_user_ata_amount(&wsol.address, None)
            .await?
            .expect("must exist");
        let market_token_before = deployment
            .get_user_ata_amount(market_token, None)
            .await?
            .unwrap_or(0);

        let amount = 10_000_000;

        // Create a both sides deposit to single token pool.
        let (rpc, deposit) = client
            .create_deposit(store, market_token)
            .long_token(amount, None, None)
            .short_token(amount, None, None)
            .execution_fee(200_000)
            .build_with_address()
            .await?;
        let signature = rpc.send().await?;
        tracing::info!(%deposit, %signature, "create deposit");

        let wsol_after_creation = deployment
            .get_user_ata_amount(&wsol.address, None)
            .await?
            .expect("must exist");
        let market_token_after_creation = deployment
            .get_user_ata_amount(market_token, None)
            .await?
            .expect("market token ata must exist");
        let token_escrow = deployment
            .get_ata_amount(&wsol.address, &deposit)
            .await?
            .expect("token escrow must exist");
        let market_token_escrow = deployment
            .get_ata_amount(market_token, &deposit)
            .await?
            .expect("market token escrow must exist");

        assert_eq!(wsol_after_creation + amount * 2, wsol_before);
        assert_eq!(market_token_after_creation, market_token_before);
        assert_eq!(token_escrow, amount * 2);
        assert_eq!(market_token_escrow, 0);

        // Execute.
        let mut builder = keeper.execute_deposit(store, oracle, &deposit, true);
        deployment
            .execute_with_pyth(&mut builder, None, true)
            .await?;

        let wsol_after_execution = deployment
            .get_user_ata_amount(&wsol.address, None)
            .await?
            .expect("must exist");
        let market_token_after_execution = deployment
            .get_user_ata_amount(market_token, None)
            .await?
            .expect("market token ata must exist");
        assert!(deployment
            .get_ata_amount(&wsol.address, &deposit)
            .await?
            .is_none());
        assert!(deployment
            .get_ata_amount(market_token, &deposit)
            .await?
            .is_none());

        assert_eq!(wsol_after_creation, wsol_after_execution);
        assert!(market_token_after_execution >= market_token_after_creation);
        let minted = market_token_after_execution - market_token_after_creation;
        tracing::info!("minted {minted}");
    }

    Ok(())
}

#[tokio::test]
async fn create_deposit_2() -> eyre::Result<()> {
    let deployment = setup::current_deployment().await?;
    let _guard = deployment.use_accounts().await?;

    deployment
        .mint_or_transfer_to("fBTC", Some("user_0"), 1_000_000_000)
        .await?;

    Ok(())
}
