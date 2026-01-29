//! Statistics types for RandScan

use serde::{Deserialize, Serialize};

/// Network statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    pub block_height: i64,
    pub total_transactions: i64,
    pub total_accounts: i64,
    pub total_validators: i64,
    pub active_validators: i64,
    pub atlas_total_supply: i64,
    pub atlas_staked: i64,
    pub shrug_total_supply: i64,
    pub shrug_burned: i64,
    pub avg_block_time: f64,
    pub tps_current: f64,
    pub tps_peak: f64,
    pub current_epoch: i64,
    pub updated_at: i64,
}

impl NetworkStats {
    pub fn atlas_staked_percentage(&self) -> f64 {
        if self.atlas_total_supply == 0 {
            0.0
        } else {
            self.atlas_staked as f64 / self.atlas_total_supply as f64 * 100.0
        }
    }

    pub fn shrug_burned_percentage(&self) -> f64 {
        if self.shrug_total_supply == 0 {
            0.0
        } else {
            self.shrug_burned as f64 / self.shrug_total_supply as f64 * 100.0
        }
    }
}

/// Hourly statistics for charts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsHourly {
    pub hour: i64, // Unix timestamp of hour start
    pub block_count: i32,
    pub tx_count: i32,
    pub unique_senders: i32,
    pub total_fees: i64,
    pub avg_block_time: f64,
    pub tps_avg: f64,
    pub tps_peak: f64,
    // Transaction type breakdown
    pub tx_public: i32,
    pub tx_private: i32,
    pub tx_stealth: i32,
    pub tx_stake: i32,
    pub tx_unstake: i32,
    pub tx_transfer: i32,
    pub tx_deploy: i32,
    pub tx_invoke: i32,
    pub tx_private_transfer: i32,
}

/// Daily statistics summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsDaily {
    pub date: String, // YYYY-MM-DD
    pub block_count: i32,
    pub tx_count: i32,
    pub unique_senders: i32,
    pub unique_receivers: i32,
    pub new_accounts: i32,
    pub total_fees: i64,
    pub avg_block_time: f64,
    pub avg_tps: f64,
    pub peak_tps: f64,
    pub total_staked: i64,
    pub total_unstaked: i64,
    pub total_transferred: i64,
}

/// Epoch statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpochStats {
    pub epoch: i64,
    pub start_height: i64,
    pub end_height: Option<i64>,
    pub start_timestamp: i64,
    pub end_timestamp: Option<i64>,
    pub block_count: i32,
    pub tx_count: i32,
    pub validator_count: i32,
    pub total_stake: i64,
    pub total_rewards: i64,
    pub is_current: bool,
}

/// Transaction type distribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxTypeDistribution {
    pub payload_type: String,
    pub count: i64,
    pub percentage: f64,
}

/// Privacy statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyStats {
    pub total_nullifiers: i64,
    pub total_commitments: i64,
    pub unspent_commitments: i64,
    pub private_tx_count: i64,
    pub stealth_tx_count: i64,
    pub private_transfer_count: i64,
}

/// Chart data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartPoint {
    pub timestamp: i64,
    pub value: f64,
    pub label: Option<String>,
}

/// Chart series for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartSeries {
    pub name: String,
    pub data: Vec<ChartPoint>,
}
