use serde::{Deserialize, Serialize};

/// Network statistics: a mix of indexed counts and the node's live status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    pub chain_id: i64,
    pub symbol: String,
    pub decimals: i16,
    pub height: i64,
    pub view: i64,
    pub total_transactions: i64,
    pub total_accounts: i64,
    pub validator_count: i64,
    pub total_stake: String,
    pub total_supply: String,
    pub program_count: i64,
    pub avg_block_time_ms: f64,
    pub peer_count: i32,
    pub mempool_size: i32,
    pub node_syncing: bool,
    pub faucet: bool,
    pub confidential: bool,
    pub current_leader: Option<String>,
    pub updated_at: String,
}
