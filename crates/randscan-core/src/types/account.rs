//! Account types for RandScan

use serde::{Deserialize, Serialize};

/// Account address type alias
pub type Address = super::Id32;

/// Account information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub address: String,
    pub atlas_balance: i64,
    pub shrug_balance: i64,
    pub nonce: i64,
    pub is_executable: bool,
    pub owner: Option<String>,
    pub data_len: i32,
    pub tx_count: i32,
    pub first_seen: i64,
    pub last_seen: i64,
}

impl Account {
    pub fn atlas_display(&self) -> f64 {
        self.atlas_balance as f64 / 1_000_000_000.0
    }

    pub fn shrug_display(&self) -> f64 {
        self.shrug_balance as f64 / 1_000_000_000.0
    }
}

/// Account detail with additional information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountDetail {
    pub address: String,
    pub atlas_balance: i64,
    pub atlas_balance_display: f64,
    pub shrug_balance: i64,
    pub shrug_balance_display: f64,
    pub nonce: i64,
    pub is_executable: bool,
    pub owner: Option<String>,
    pub data_len: i32,
    pub tx_count: i32,
    pub first_seen: i64,
    pub last_seen: i64,
    // Derived fields
    pub is_validator: bool,
    pub stake_amount: Option<i64>,
    pub token_accounts: Vec<TokenAccountInfo>,
}

/// Token account information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenAccountInfo {
    pub token_mint: String,
    pub token_symbol: String,
    pub balance: i64,
    pub balance_display: f64,
    pub decimals: i32,
}

/// Account transaction record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountTransaction {
    pub account: String,
    pub tx_id: String,
    pub role: String, // sender, receiver, signer
    pub block_height: i64,
    pub timestamp: i64,
}
