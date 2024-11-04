use gmsol::{
    chainlink::{self, pull_oracle::parse_feed_id},
    store::oracle::OracleOps,
    types::PriceProviderKind,
};

use anchor_client::solana_sdk::pubkey::Pubkey;

use crate::anchor_tests::setup::{current_deployment, Deployment};

#[tokio::test]
async fn update_chainlink_price_feed() -> eyre::Result<()> {
    let deployment = current_deployment().await?;
    let _guard = deployment.use_accounts().await?;
    let span = tracing::info_span!("update_chainlink_price_feed");
    let _enter = span.enter();

    let index = 255;
    let store = &deployment.store;
    let verifier_account = &deployment.chainlink_verifier;
    let chainlink_verifier_program = &deployment.chainlink_verifier_program;
    let keeper = deployment.user_client(Deployment::DEFAULT_KEEPER)?;

    let usdg = deployment.token("USDG").unwrap();

    let feed_id_hex = "0x0003dc85e8b01946bf9dfd8b0db860129181eb6105a8c8981d9f28e00b6f60d9";
    let feed_id = Pubkey::new_from_array(parse_feed_id(feed_id_hex)?);

    let (rpc, feed) = keeper.initailize_price_feed(
        store,
        index,
        PriceProviderKind::ChainlinkDataStreams,
        &usdg.address,
        &feed_id,
    );

    let signature = rpc.send_without_preflight().await?;
    tracing::info!(%signature, %feed, "created a chainlink feed for USDG");

    let chainlink = chainlink::Client::from_testnet_defaults()?;

    let report = chainlink.latest_report(feed_id_hex).await?;

    let signature = keeper
        .update_price_feed_with_chainlink(
            store,
            verifier_account,
            &feed,
            chainlink_verifier_program,
            report.report_bytes()?,
        )
        .send_without_preflight()
        .await?;

    tracing::info!(%signature, %feed, "updated price feed with chainlink report");

    Ok(())
}
