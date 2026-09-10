//! Fan-out of indexer events to WebSocket subscribers.

use randscan_core::{BlockSummary, BroadcastEvent, NetworkStats, TransactionSummary};
use std::sync::Arc;
use tokio::sync::broadcast;

const BROADCAST_CAPACITY: usize = 1000;

#[derive(Clone)]
pub struct Broadcaster {
    sender: Arc<broadcast::Sender<BroadcastEvent>>,
}

impl Broadcaster {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(BROADCAST_CAPACITY);
        Self {
            sender: Arc::new(sender),
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<BroadcastEvent> {
        self.sender.subscribe()
    }

    pub fn block(&self, block: BlockSummary) {
        let _ = self.sender.send(BroadcastEvent::NewBlock(block));
    }

    pub fn transaction(&self, tx: TransactionSummary) {
        let _ = self.sender.send(BroadcastEvent::NewTransaction(tx));
    }

    pub fn stats(&self, stats: NetworkStats) {
        let _ = self.sender.send(BroadcastEvent::StatsUpdate(stats));
    }
}

impl Default for Broadcaster {
    fn default() -> Self {
        Self::new()
    }
}
