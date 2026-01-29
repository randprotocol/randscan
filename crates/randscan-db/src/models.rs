//! Database models (SQLx row types)

use chrono::{DateTime, Utc};
use sqlx::FromRow;

/// Block row from database
#[derive(Debug, Clone, FromRow)]
pub struct BlockRow {
    pub block_id: String,
    pub height: i64,
    pub view_number: i64,
    pub epoch: i64,
    pub parent_id: String,
    pub proposer_id: String,
    pub transactions_root: String,
    pub state_root: String,
    pub supply_commitment: String,
    pub timestamp: i64,
    pub transaction_count: i32,
    pub finalized: bool,
    pub created_at: DateTime<Utc>,
}

/// Quorum certificate row
#[derive(Debug, Clone, FromRow)]
pub struct QcRow {
    pub block_id: String,
    pub vote_type: String,
    pub view_number: i64,
    pub certified_block_id: String,
    pub certified_block_height: i64,
    pub signer_count: i32,
}

/// QC signer row
#[derive(Debug, Clone, FromRow)]
pub struct QcSignerRow {
    pub id: i32,
    pub block_id: String,
    pub validator_id: String,
    pub signature: String,
}

/// Transaction row from database
#[derive(Debug, Clone, FromRow)]
pub struct TransactionRow {
    pub tx_id: String,
    pub block_id: Option<String>,
    pub block_height: Option<i64>,
    pub sender: String,
    pub nonce: i64,
    pub compute_budget: i64,
    pub fee: i64,
    pub payload_type: String,
    pub status: String,
    pub timestamp: i64,
    pub signature: String,
    pub created_at: DateTime<Utc>,
}

/// Account row
#[derive(Debug, Clone, FromRow)]
pub struct AccountRow {
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
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Account transaction row
#[derive(Debug, Clone, FromRow)]
pub struct AccountTransactionRow {
    pub id: i32,
    pub account: String,
    pub tx_id: String,
    pub role: String,
    pub block_height: i64,
    pub timestamp: i64,
}

/// Validator row
#[derive(Debug, Clone, FromRow)]
pub struct ValidatorRow {
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
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Validator stake history row
#[derive(Debug, Clone, FromRow)]
pub struct StakeHistoryRow {
    pub id: i32,
    pub validator_id: String,
    pub epoch: i64,
    pub stake: i64,
    pub delegators: i32,
    pub rewards: i64,
    pub timestamp: i64,
}

/// Epoch row
#[derive(Debug, Clone, FromRow)]
pub struct EpochRow {
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
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Token mint row
#[derive(Debug, Clone, FromRow)]
pub struct TokenMintRow {
    pub mint_address: String,
    pub symbol: String,
    pub name: String,
    pub decimals: i16,
    pub total_supply: i64,
    pub circulating_supply: i64,
    pub burned: i64,
    pub holder_count: i32,
    pub tx_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Token account row
#[derive(Debug, Clone, FromRow)]
pub struct TokenAccountRow {
    pub id: i32,
    pub account_address: String,
    pub owner: String,
    pub mint: String,
    pub balance: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Nullifier row
#[derive(Debug, Clone, FromRow)]
pub struct NullifierRow {
    pub nullifier: String,
    pub tx_id: String,
    pub created_at: DateTime<Utc>,
}

/// Commitment row
#[derive(Debug, Clone, FromRow)]
pub struct CommitmentRow {
    pub commitment: String,
    pub tx_id: String,
    pub spent: bool,
    pub spent_tx_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Program row
#[derive(Debug, Clone, FromRow)]
pub struct ProgramRow {
    pub program_id: String,
    pub deployer: String,
    pub deploy_tx_id: Option<String>,
    pub code_size: i32,
    pub invoke_count: i64,
    pub last_invoked: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Network stats row
#[derive(Debug, Clone, FromRow)]
pub struct NetworkStatsRow {
    pub id: i32,
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
    pub updated_at: DateTime<Utc>,
}

/// Stats hourly row
#[derive(Debug, Clone, FromRow)]
pub struct StatsHourlyRow {
    pub hour: i64,
    pub block_count: i32,
    pub tx_count: i32,
    pub unique_senders: i32,
    pub total_fees: i64,
    pub avg_block_time: f64,
    pub tps_avg: f64,
    pub tps_peak: f64,
    pub tx_public: i32,
    pub tx_private: i32,
    pub tx_stealth: i32,
    pub tx_stake: i32,
    pub tx_unstake: i32,
    pub tx_transfer: i32,
    pub tx_deploy: i32,
    pub tx_invoke: i32,
    pub tx_private_transfer: i32,
    pub created_at: DateTime<Utc>,
}

/// Indexer state row
#[derive(Debug, Clone, FromRow)]
pub struct IndexerStateRow {
    pub id: i32,
    pub last_indexed_height: i64,
    pub last_indexed_block_id: Option<String>,
    pub last_finalized_height: i64,
    pub is_syncing: bool,
    pub sync_started_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

/// Count result helper
#[derive(Debug, Clone, FromRow)]
pub struct CountRow {
    pub count: Option<i64>,
}

impl CountRow {
    pub fn value(&self) -> i64 {
        self.count.unwrap_or(0)
    }
}
