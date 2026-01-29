//! RandScan Indexer - Blockchain syncer for RandProtocol
//!
//! This crate implements the blockchain indexer that connects to a RandProtocol
//! node, fetches blocks and transactions, and stores them in the database.

pub mod rpc;
pub mod processor;
pub mod service;
pub mod broadcast;

pub use rpc::*;
pub use processor::*;
pub use service::*;
pub use broadcast::*;

use std::time::Duration;

/// Indexer configuration
#[derive(Debug, Clone)]
pub struct IndexerConfig {
    /// RPC endpoint URL
    pub rpc_url: String,
    /// Polling interval for new blocks
    pub poll_interval: Duration,
    /// Batch size for initial sync
    pub batch_size: u64,
    /// Enable real-time broadcasting
    pub enable_broadcast: bool,
    /// Finality depth (blocks behind tip considered final)
    pub finality_depth: u64,
}

impl Default for IndexerConfig {
    fn default() -> Self {
        Self {
            rpc_url: "http://localhost:8899".to_string(),
            poll_interval: Duration::from_millis(500),
            batch_size: 100,
            enable_broadcast: true,
            finality_depth: 32,
        }
    }
}

impl IndexerConfig {
    pub fn from_env() -> Self {
        Self {
            rpc_url: std::env::var("RPC_URL")
                .unwrap_or_else(|_| "http://localhost:8899".to_string()),
            poll_interval: Duration::from_millis(
                std::env::var("POLL_INTERVAL_MS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(500),
            ),
            batch_size: std::env::var("BATCH_SIZE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(100),
            enable_broadcast: std::env::var("ENABLE_BROADCAST")
                .map(|s| s == "true" || s == "1")
                .unwrap_or(true),
            finality_depth: std::env::var("FINALITY_DEPTH")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(32),
        }
    }
}
