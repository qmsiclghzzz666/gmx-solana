use futures_util::TryStreamExt;
use gmsol::chainlink::Client;
use time::OffsetDateTime;
use tracing::level_filters::LevelFilter;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    let client = Client::from_testnet_defaults()?;

    // Fetch available feeds.
    let feeds = client.feeds().await?;

    println!("{feeds:?}");

    // Fetch latest report.
    if let Some(feed) = feeds.feeds.first() {
        let report = client.latest_report(&feed.feed_id).await?;
        println!("latest report: {report:?}");
    }

    // Fetch a bulk of reports.
    let reports = client
        .bulk_report(
            feeds.feeds.iter().map(|feed| feed.feed_id.as_str()),
            OffsetDateTime::now_utc(),
        )
        .await?;

    for (idx, report) in reports.iter().enumerate() {
        tracing::info!("[{idx}] {report:?}");
        if let Ok(report) = report.decode() {
            println!("[{idx}] {report:#?}");
        }
    }

    // Subscribe to reports.
    let stream = client
        .subscribe(feeds.feeds.iter().map(|feed| feed.feed_id.as_str()))
        .await?;

    futures_util::pin_mut!(stream);

    while let Some(report) = stream.try_next().await? {
        tracing::info!("{report:?}");
    }
    Ok(())
}
