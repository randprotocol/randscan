//! Event broadcasting for real-time updates

use randscan_core::{
    Account, BlockSummary, BroadcastEvent, NetworkStats, TransactionSummary, Validator,
};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::debug;

/// Channel capacity for broadcast events
const BROADCAST_CAPACITY: usize = 1000;

/// Event broadcaster for real-time updates
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

    /// Get a receiver for broadcast events
    pub fn subscribe(&self) -> broadcast::Receiver<BroadcastEvent> {
        self.sender.subscribe()
    }

    /// Broadcast a new block
    pub fn broadcast_block(&self, block: BlockSummary) {
        let event = BroadcastEvent::NewBlock(block);
        if let Err(e) = self.sender.send(event) {
            debug!("No subscribers for block broadcast: {}", e);
        }
    }

    /// Broadcast a new transaction
    pub fn broadcast_transaction(&self, tx: TransactionSummary) {
        let event = BroadcastEvent::NewTransaction(tx);
        if let Err(e) = self.sender.send(event) {
            debug!("No subscribers for transaction broadcast: {}", e);
        }
    }

    /// Broadcast an account update
    pub fn broadcast_account_update(&self, account: Account) {
        let event = BroadcastEvent::AccountUpdate(account);
        if let Err(e) = self.sender.send(event) {
            debug!("No subscribers for account broadcast: {}", e);
        }
    }

    /// Broadcast a validator update
    pub fn broadcast_validator_update(&self, validator: Validator) {
        let event = BroadcastEvent::ValidatorUpdate(validator);
        if let Err(e) = self.sender.send(event) {
            debug!("No subscribers for validator broadcast: {}", e);
        }
    }

    /// Broadcast network stats update
    pub fn broadcast_stats_update(&self, stats: NetworkStats) {
        let event = BroadcastEvent::StatsUpdate(stats);
        if let Err(e) = self.sender.send(event) {
            debug!("No subscribers for stats broadcast: {}", e);
        }
    }

    /// Get number of current subscribers
    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

impl Default for Broadcaster {
    fn default() -> Self {
        Self::new()
    }
}
