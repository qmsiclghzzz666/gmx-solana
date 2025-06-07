use std::env;

use anchor_spl::token::Mint;
use gmsol_sdk::{
    client::pyth::Hermes,
    market::MarketCalculations,
    model::{LiquidityMarketExt, MarketModel, PnlFactorKind},
    solana_utils::solana_sdk::signature::Keypair,
    utils::Value,
    Client,
};

#[tokio::main]
async fn main() -> gmsol_sdk::Result<()> {
    use tracing_subscriber::{fmt::format::FmtSpan, EnvFilter};

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive("market=info".parse().map_err(gmsol_sdk::Error::custom)?),
        )
        .with_span_events(FmtSpan::FULL)
        .init();

    let cluster = env::var("CLUSTER")
        .unwrap_or_else(|_| "devnet".to_string())
        .parse()?;
    let payer = Keypair::new();

    let client = Client::new(cluster, &payer)?;

    // Passing an empty string returns the default store address.
    let store = client.find_store_address("");

    // Retrieve all available markets.
    let markets = client.markets(&store).await?;
    tracing::info!("Loaded {} markets.", markets.len());

    // Use the last available market.
    let Some(market) = markets.last_key_value().map(|(_, v)| v) else {
        return Err(gmsol_sdk::Error::custom("No available markets"));
    };

    // Load the token mint associated with the selected market.
    let mint_address = &market.meta.market_token_mint;
    let Some(mint) = client.account::<Mint>(mint_address).await? else {
        return Err(gmsol_sdk::Error::custom(format!(
            "The token mint `{mint_address}` does not exist"
        )));
    };

    // Load the store's authorized token map.
    let token_map = client.authorized_token_map(&store).await?;

    // Construct a `MarketModel` which implements traits from `gmsol_sdk::model`.
    let model = MarketModel::from_parts(market.clone(), mint.supply);

    // Fetch token prices using Pyth.
    let hermes = Hermes::default();
    let prices = hermes.unit_prices_for_market(&token_map, &**market).await?;

    // Display the market status.
    println!("Market: {}", market.name()?);
    let market_token_price = Value::from_u128(model.market_token_price(
        &prices,
        PnlFactorKind::MaxAfterDeposit,
        true,
    )?);
    println!("GM Price: {market_token_price} USD/GM");
    println!("Status: {:#?}", model.status(&prices)?);

    Ok(())
}
