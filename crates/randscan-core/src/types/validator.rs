//! Validator types for RandScan

use serde::{Deserialize, Serialize};

/// Validator information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Validator {
    pub validator_id: String,
    pub pubkey: String,
    pub stake: i64,
    pub commission_rate: i16,
    pub is_active: bool,
    pub blocks_produced: i64,
    pub blocks_skipped: i64,
    pub last_vote_height: Option<i64>,
    pub uptime_percentage: f64,
    pub first_seen: i64,
    pub last_seen: i64,
}

impl Validator {
    pub fn stake_display(&self) -> f64 {
        self.stake as f64 / 1_000_000_000.0
    }

    pub fn skip_rate(&self) -> f64 {
        let total = self.blocks_produced + self.blocks_skipped;
        if total == 0 {
            0.0
        } else {
            self.blocks_skipped as f64 / total as f64 * 100.0
        }
    }
}

/// Validator detail with extended information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorDetail {
    pub validator_id: String,
    pub pubkey: String,
    pub stake: i64,
    pub stake_display: f64,
    pub commission_rate: i16,
    pub is_active: bool,
    pub blocks_produced: i64,
    pub blocks_skipped: i64,
    pub skip_rate: f64,
    pub last_vote_height: Option<i64>,
    pub uptime_percentage: f64,
    pub first_seen: i64,
    pub last_seen: i64,
    // Recent activity
    pub recent_blocks: Vec<ValidatorBlock>,
    pub stake_history: Vec<StakeHistoryEntry>,
    // Performance metrics
    pub avg_block_time: Option<f64>,
    pub total_rewards: i64,
}

/// Block produced by validator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorBlock {
    pub block_id: String,
    pub height: i64,
    pub timestamp: i64,
    pub transaction_count: i32,
}

/// Stake history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakeHistoryEntry {
    pub validator_id: String,
    pub epoch: i64,
    pub stake: i64,
    pub stake_display: f64,
    pub delegators: i32,
    pub rewards: i64,
    pub timestamp: i64,
}

/// Validator stake change event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorStakeChange {
    pub validator_id: String,
    pub tx_id: String,
    pub change_type: String, // stake, unstake
    pub amount: i64,
    pub new_stake: i64,
    pub timestamp: i64,
}
