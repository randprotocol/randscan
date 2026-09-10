use serde::{Deserialize, Serialize};

use super::BlockSummary;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Validator {
    pub address: String,
    pub stake: String,
    pub share_percent: f64,
    pub blocks_proposed: i64,
    pub last_proposed_height: Option<i64>,
    pub last_proposed_timestamp_ms: Option<i64>,
    pub sort_index: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorDetail {
    #[serde(flatten)]
    pub validator: Validator,
    pub recent_blocks: Vec<BlockSummary>,
}
