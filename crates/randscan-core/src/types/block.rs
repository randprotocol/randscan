use serde::{Deserialize, Serialize};

use super::TransactionSummary;

/// Block as shown in lists and pushed over WebSocket.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockSummary {
    pub hash: String,
    pub height: i64,
    pub view: i64,
    pub parent: String,
    pub proposer: String,
    pub timestamp_ms: i64,
    pub tx_count: i32,
    pub justify_view: i64,
}

/// Block detail with roots and its transactions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockDetail {
    #[serde(flatten)]
    pub summary: BlockSummary,
    pub tx_root: String,
    pub state_root: String,
    pub transactions: Vec<TransactionSummary>,
}
