use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
    task::Poll,
    time::Duration,
};

use anchor_client::{
    solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey, signature::Keypair},
    Cluster,
};
use futures_util::Stream;
use tokio::sync::watch;
use tokio_stream::wrappers::WatchStream;
use tower::discover::Change;
use tracing::Instrument;

use crate::{
    pda::find_default_store,
    types,
    utils::{accounts::WithContext, shared_signer, ProgramAccountsConfig, SignerRef},
    Client, ClientOptions,
};

type SharedMarket = Arc<types::Market>;
type Cache = HashMap<Pubkey, SharedMarket>;

pin_project_lite::pin_project! {
    /// Market Discovery.
    pub struct MarketDiscovery {
        #[pin]
        stream: WatchStream<Cache>,
        cache: Cache,
        changes: std::vec::IntoIter<Change<Pubkey, types::MarketMeta>>,
    }
}

impl MarketDiscovery {
    /// Create a new marekt discovery service for the default store.
    pub fn new(cluster: Cluster) -> crate::Result<Self> {
        Self::new_with_store(cluster, find_default_store().0)
    }

    /// Create a new market discovery service for the given store with default options.
    pub fn new_with_store(cluster: Cluster, store: Pubkey) -> crate::Result<Self> {
        Self::new_with_options(
            cluster,
            store,
            Duration::from_secs(30),
            ClientOptions::default(),
        )
    }

    /// Create a new market discovery service with [`ClientOptions`].
    pub fn new_with_options(
        cluster: Cluster,
        store: Pubkey,
        interval: Duration,
        client_options: ClientOptions,
    ) -> crate::Result<Self> {
        let client =
            Client::new_with_options(cluster, shared_signer(Keypair::new()), client_options)?;
        let (sender, receiver) = watch::channel(Cache::default());
        let watcher = Watcher {
            store,
            interval,
            client,
            sender,
        };

        let worker = watcher
            .run()
            .instrument(tracing::debug_span!("market_watcher", %store));

        tokio::spawn(
            async move {
                match worker.await {
                    Ok(()) => {
                        tracing::warn!("stopped");
                    }
                    Err(err) => {
                        tracing::warn!(%err, "stopped with error");
                    }
                }
            }
            .in_current_span(),
        );

        Ok(Self {
            stream: WatchStream::new(receiver),
            cache: Cache::default(),
            changes: Vec::new().into_iter(),
        })
    }
}

impl Stream for MarketDiscovery {
    type Item = crate::Result<Change<Pubkey, types::MarketMeta>>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        loop {
            if let Some(change) = this.changes.next() {
                return Poll::Ready(Some(Ok(change)));
            }
            let Some(markets) = futures_util::ready!(this.stream.as_mut().poll_next(cx)) else {
                return Poll::Ready(Some(Err(crate::Error::unknown("disconnected"))));
            };
            let mut changes = Vec::default();
            for (pubkey, market) in markets.iter() {
                if !this.cache.contains_key(pubkey) {
                    changes.push(Change::Insert(*pubkey, *market.meta()));
                }
            }
            for pubkey in this.cache.keys() {
                if !markets.contains_key(pubkey) {
                    changes.push(Change::Remove(*pubkey));
                }
            }
            tracing::info!(len = %changes.len(), "updated");
            *this.cache = markets;
            *this.changes = changes.into_iter();
        }
    }
}

struct Watcher {
    store: Pubkey,
    interval: Duration,
    client: Client<SignerRef>,
    sender: watch::Sender<Cache>,
}

impl Watcher {
    async fn run(self) -> crate::Result<()> {
        let mut interval = tokio::time::interval(self.interval);
        loop {
            interval.tick().await;
            let Ok(markets) = self.fetch_once().await else {
                continue;
            };
            let slot = markets.slot();
            let cache: Cache = markets
                .into_value()
                .into_iter()
                .map(|(pubkey, m)| (pubkey, Arc::new(m)))
                .collect();
            tracing::debug!(%slot, len=%cache.len(), "fetched new market list");
            self.sender.send(cache).map_err(crate::Error::unknown)?;
        }
    }

    async fn fetch_once(
        &self,
    ) -> crate::Result<WithContext<BTreeMap<Pubkey, gmsol_store::states::Market>>> {
        self.client
            .markets_with_config(
                &self.store,
                ProgramAccountsConfig {
                    commitment: Some(CommitmentConfig::finalized()),
                    ..Default::default()
                },
            )
            .await
    }
}

#[cfg(test)]
mod tests {
    use futures_util::{future::poll_fn, pin_mut};
    use tower::discover::Discover;
    use tracing::level_filters::LevelFilter;
    use tracing_subscriber::EnvFilter;

    use super::*;

    #[tokio::test]
    async fn test_market_discover() -> eyre::Result<()> {
        tracing_subscriber::fmt()
            .with_env_filter(
                EnvFilter::builder()
                    .with_default_directive(LevelFilter::DEBUG.into())
                    .from_env_lossy(),
            )
            .init();
        let markets = MarketDiscovery::new(crate::test::default_cluster())?;

        pin_mut!(markets);
        if let Some(Ok(change)) = poll_fn(|cx| markets.as_mut().poll_discover(cx)).await {
            tracing::info!("{change:?}");
        }
        Ok(())
    }
}
