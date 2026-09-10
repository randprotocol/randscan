use randscan_core::{BroadcastEvent, WsChannel, WsServerMessage};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};
use tracing::{debug, info};
use uuid::Uuid;

pub type WsSender = mpsc::UnboundedSender<WsServerMessage>;

struct Connection {
    sender: WsSender,
    channels: HashSet<WsChannel>,
}

pub struct WsManager {
    connections: Arc<RwLock<HashMap<String, Connection>>>,
}

impl WsManager {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register(&self, sender: WsSender) -> String {
        let id = Uuid::new_v4().to_string();
        self.connections.write().await.insert(
            id.clone(),
            Connection {
                sender,
                channels: HashSet::new(),
            },
        );
        debug!("ws connection {} registered", id);
        id
    }

    pub async fn unregister(&self, id: &str) {
        self.connections.write().await.remove(id);
        debug!("ws connection {} unregistered", id);
    }

    pub async fn subscribe(&self, connection_id: &str, channel: WsChannel) {
        if let Some(conn) = self.connections.write().await.get_mut(connection_id) {
            conn.channels.insert(channel);
            let _ = conn.sender.send(WsServerMessage::Subscribed {
                channel,
                subscription_id: Uuid::new_v4().to_string(),
            });
        }
    }

    pub async fn unsubscribe(&self, connection_id: &str, channel: WsChannel) {
        if let Some(conn) = self.connections.write().await.get_mut(connection_id) {
            conn.channels.remove(&channel);
            let _ = conn.sender.send(WsServerMessage::Unsubscribed { channel });
        }
    }

    pub async fn broadcast(&self, event: BroadcastEvent) {
        let channel = event.channel();
        let message = event.to_message();
        for conn in self.connections.read().await.values() {
            if conn.channels.contains(&channel) {
                let _ = conn.sender.send(message.clone());
            }
        }
    }

    pub async fn connection_count(&self) -> usize {
        self.connections.read().await.len()
    }

    pub async fn start_broadcast_listener(
        self: Arc<Self>,
        mut receiver: broadcast::Receiver<BroadcastEvent>,
    ) {
        tokio::spawn(async move {
            loop {
                match receiver.recv().await {
                    Ok(event) => self.broadcast(event).await,
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        debug!("ws broadcast lagged by {}", n)
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        info!("broadcast channel closed");
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
