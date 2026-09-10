use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountDetail {
    pub address: String,
    pub balance: String,
    pub nonce: i64,
    pub tx_count: i64,
    pub first_seen_height: i64,
    pub last_seen_height: i64,
    pub is_validator: bool,
    pub stake: Option<String>,
    pub programs_deployed: i64,
}
