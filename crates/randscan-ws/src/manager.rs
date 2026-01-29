//! WebSocket connection manager

use randscan_core::{
    BroadcastEvent, Subscription, WsChannel, WsServerMessage, WsSubscribeParams,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};
use tracing::{debug, info};
use uuid::Uuid;

/// Message sender type for a WebSocket connection
pub type WsSender = mpsc::UnboundedSender<WsServerMessage>;

/// Connection info
struct Connection {
    sender: WsSender,
    subscriptions: Vec<Subscription>,
}

/// WebSocket connection manager
pub struct WsManager {
    connections: Arc<RwLock<HashMap<String, Connection>>>,
}

impl WsManager {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new connection
    pub async fn register(&self, sender: WsSender) -> String {
        let id = Uuid::new_v4().to_string();
        let connection = Connection {
            sender,
            subscriptions: Vec::new(),
        };

        self.connections.write().await.insert(id.clone(), connection);
        info!("WebSocket connection registered: {}", id);
        id
    }

    /// Unregister a connection
    pub async fn unregister(&self, id: &str) {
        self.connections.write().await.remove(id);
        info!("WebSocket connection unregistered: {}", id);
    }

    /// Subscribe to a channel
    pub async fn subscribe(
        &self,
        connection_id: &str,
        channel: WsChannel,
        params: Option<WsSubscribeParams>,
    ) -> Option<String> {
        let mut connections = self.connections.write().await;

        if let Some(conn) = connections.get_mut(connection_id) {
            let subscription = Subscription::new(channel, params);
            let sub_id = subscription.id.clone();
            conn.subscriptions.push(subscription);

            // Send confirmation
            let _ = conn.sender.send(WsServerMessage::Subscribed {
                channel,
                subscription_id: sub_id.clone(),
            });

            debug!(
                "Connection {} subscribed to {:?}",
                connection_id,
                channel
            );
            Some(sub_id)
        } else {
            None
        }
    }

    /// Unsubscribe from a channel
    pub async fn unsubscribe(&self, connection_id: &str, channel: WsChannel) {
        let mut connections = self.connections.write().await;

        if let Some(conn) = connections.get_mut(connection_id) {
            conn.subscriptions.retain(|s| s.channel != channel);

            let _ = conn.sender.send(WsServerMessage::Unsubscribed { channel });

            debug!(
                "Connection {} unsubscribed from {:?}",
                connection_id,
                channel
            );
        }
    }

    /// Broadcast an event to matching subscribers
    pub async fn broadcast(&self, event: BroadcastEvent) {
        let connections = self.connections.read().await;

        for conn in connections.values() {
            let message = match &event {
                BroadcastEvent::NewBlock(block) => {
                    let matches = conn
                        .subscriptions
                        .iter()
                        .any(|s| s.matches_block(block));
                    if matches {
                        Some(WsServerMessage::NewBlock {
                            block: block.clone(),
                        })
                    } else {
                        None
                    }
                }
                BroadcastEvent::NewTransaction(tx) => {
                    let matches = conn
                        .subscriptions
                        .iter()
                        .any(|s| s.matches_transaction(tx));
                    if matches {
                        Some(WsServerMessage::NewTransaction {
                            transaction: tx.clone(),
                        })
                    } else {
                        None
                    }
                }
                BroadcastEvent::AccountUpdate(account) => {
                    let matches = conn
                        .subscriptions
                        .iter()
                        .any(|s| s.matches_account(account));
                    if matches {
                        Some(WsServerMessage::AccountUpdate {
                            account: account.clone(),
                        })
                    } else {
                        None
                    }
                }
                BroadcastEvent::ValidatorUpdate(validator) => {
                    let matches = conn
                        .subscriptions
                        .iter()
                        .any(|s| s.matches_validator(validator));
                    if matches {
                        Some(WsServerMessage::ValidatorUpdate {
                            validator: validator.clone(),
                        })
                    } else {
                        None
                    }
                }
                BroadcastEvent::StatsUpdate(stats) => {
                    let matches = conn
                        .subscriptions
                        .iter()
                        .any(|s| s.channel == WsChannel::Stats);
                    if matches {
                        Some(WsServerMessage::StatsUpdate {
                            stats: stats.clone(),
                        })
                    } else {
                        None
                    }
                }
            };

            if let Some(msg) = message {
                let _ = conn.sender.send(msg);
            }
        }
    }

    /// Get connection count
    pub async fn connection_count(&self) -> usize {
        self.connections.read().await.len()
    }

    /// Start listening for broadcast events
    pub async fn start_broadcast_listener(
        self: Arc<Self>,
        mut receiver: broadcast::Receiver<BroadcastEvent>,
    ) {
        tokio::spawn(async move {
            loop {
                match receiver.recv().await {
                    Ok(event) => {
                        self.broadcast(event).await;
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        debug!("WebSocket broadcast lagged by {} messages", n);
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        info!("Broadcast channel closed");
                        break;
                    }
                }
            }
        });
    }
}

impl Default for WsManager {
    fn default() -> Self {
        Self::new()
    }
}
