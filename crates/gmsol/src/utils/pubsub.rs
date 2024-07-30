use std::{
    collections::{hash_map::Entry, HashMap},
    num::NonZeroUsize,
    sync::{Arc, Mutex, RwLock},
    time::Duration,
};

use anchor_client::{
    solana_client::{
        nonblocking::pubsub_client::PubsubClient as SolanaPubsubClient,
        rpc_config::{RpcTransactionLogsConfig, RpcTransactionLogsFilter},
        rpc_response::RpcLogsResponse,
    },
    solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey},
    Cluster,
};
use futures_util::{Stream, StreamExt, TryStreamExt};
use tokio::{
    sync::broadcast,
    task::{AbortHandle, JoinSet},
};
use tokio_stream::wrappers::BroadcastStream;
use tracing::Instrument;

use super::accounts::WithContext;

/// A wrapper of [the solana version of pubsub client](SolanaPubsubClient)
/// with shared subscription support.
#[derive(Debug, Clone)]
pub struct PubsubClient(Arc<Inner>);

impl PubsubClient {
    /// Create a new [`PubsubClient`] with the given config.
    pub async fn new(cluster: Cluster, config: SubscriptionConfig) -> crate::Result<Self> {
        let client = SolanaPubsubClient::new(cluster.ws_url())
            .await
            .map_err(anchor_client::ClientError::from)?;
        Ok(Self(Arc::new(Inner {
            tasks: Default::default(),
            config,
            client: Arc::new(client),
            logs: Default::default(),
        })))
    }

    /// Subscribe to transaction logs.
    pub fn logs_subscribe(
        &self,
        mention: &Pubkey,
        commitment: Option<CommitmentConfig>,
    ) -> crate::Result<impl Stream<Item = crate::Result<WithContext<RpcLogsResponse>>>> {
        self.0.logs_subscribe(mention, commitment)
    }
}

#[derive(Debug)]
struct Inner {
    tasks: Mutex<JoinSet<()>>,
    config: SubscriptionConfig,
    client: Arc<SolanaPubsubClient>,
    logs: LogsSubscriptions,
}

impl Inner {
    fn logs_subscribe(
        &self,
        mention: &Pubkey,
        commitment: Option<CommitmentConfig>,
    ) -> crate::Result<impl Stream<Item = crate::Result<WithContext<RpcLogsResponse>>>> {
        let config = SubscriptionConfig {
            commitment: commitment.or(self.config.commitment),
            ..self.config
        };
        let receiver = self.logs.subscribe(
            &mut self.tasks.lock().unwrap(),
            &self.client,
            mention,
            config,
        )?;
        Ok(BroadcastStream::new(receiver).map_err(crate::Error::from))
    }
}

/// Config for subscription manager.
#[derive(Debug, Clone)]
pub struct SubscriptionConfig {
    /// Commitment.
    pub commitment: Option<CommitmentConfig>,
    /// Cleanup interval.
    pub cleanup_interval: Duration,
    /// Capacity for the broadcast channel.
    pub capacity: NonZeroUsize,
}

impl Default for SubscriptionConfig {
    fn default() -> Self {
        Self {
            commitment: Default::default(),
            cleanup_interval: Duration::from_secs(10),
            capacity: NonZeroUsize::new(256).unwrap(),
        }
    }
}

#[derive(Debug)]
struct LogsSubscription {
    commitment: CommitmentConfig,
    sender: broadcast::Sender<WithContext<RpcLogsResponse>>,
    abort: AbortHandle,
}

impl Drop for LogsSubscription {
    fn drop(&mut self) {
        self.abort.abort();
    }
}

impl LogsSubscription {
    fn init(
        join_set: &mut JoinSet<()>,
        sender: broadcast::Sender<WithContext<RpcLogsResponse>>,
        client: &Arc<SolanaPubsubClient>,
        mention: &Pubkey,
        commitment: CommitmentConfig,
        cleanup_interval: Duration,
    ) -> Self {
        let abort = join_set.spawn({
            let client = client.clone();
            let mention = *mention;
            let sender = sender.clone();
            async move {
                let res = client
                    .logs_subscribe(
                        RpcTransactionLogsFilter::Mentions(vec![mention.to_string()]),
                        RpcTransactionLogsConfig { commitment: Some(commitment) },
                    )
                    .await
                    .inspect_err(
                        |err| tracing::error!(%err, %mention, "failed to subscribe transaction logs"),
                    );
                let Ok((mut stream, unsubscribe)) = res else {
                    return;
                };
                let mut interval = tokio::time::interval(cleanup_interval);
                loop {
                    tokio::select! {
                        _ = interval.tick() => {
                            if sender.receiver_count() == 0 {
                                break;
                            }
                        }
                        res = stream.next() => {
                            match res {
                                Some(res) => {
                                    if sender.send(res.into()).unwrap_or(0) == 0 {
                                        break;
                                    }
                                }
                                None => break,
                            }
                        }
                    }
                }
                (unsubscribe)().await;
                tracing::info!(%mention, "logs subscription end");
            }
            .in_current_span()
        });

        Self {
            commitment,
            abort,
            sender,
        }
    }
}

#[derive(Debug, Default)]
struct LogsSubscriptions(RwLock<HashMap<Pubkey, LogsSubscription>>);

impl LogsSubscriptions {
    fn subscribe(
        &self,
        join_set: &mut JoinSet<()>,
        client: &Arc<SolanaPubsubClient>,
        mention: &Pubkey,
        config: SubscriptionConfig,
    ) -> crate::Result<broadcast::Receiver<WithContext<RpcLogsResponse>>> {
        let mut map = self.0.write().unwrap();
        loop {
            match map.entry(*mention) {
                Entry::Occupied(entry) => {
                    let subscription = entry.get();
                    if subscription.abort.is_finished() {
                        entry.remove();
                    } else {
                        if let Some(commitment) = config.commitment {
                            if commitment != subscription.commitment {
                                return Err(crate::Error::invalid_argument(format!(
                                    "commitment mismatched, current: {}",
                                    subscription.commitment.commitment
                                )));
                            }
                        }
                        return Ok(subscription.sender.subscribe());
                    }
                }
                Entry::Vacant(entry) => {
                    let (sender, receiver) = broadcast::channel(config.capacity.get());
                    let subscription = LogsSubscription::init(
                        join_set,
                        sender,
                        client,
                        mention,
                        config.commitment.unwrap_or(CommitmentConfig::finalized()),
                        config.cleanup_interval,
                    );
                    entry.insert(subscription);
                    return Ok(receiver);
                }
            }
        }
    }
}
