//! API client for fetching data from the backend

use gloo_net::http::Request;
use serde::{Deserialize, Serialize};

const API_BASE: &str = "/api/v1";

/// Generic paginated response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub pagination: PaginationInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationInfo {
    pub page: u32,
    pub limit: u32,
    pub total: i64,
    pub total_pages: u32,
    pub has_next: bool,
    pub has_prev: bool,
}

/// Network stats
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
}

/// Block summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockSummary {
    pub block_id: String,
    pub height: u64,
    pub view: u64,
    pub epoch: u64,
    pub parent_id: String,
    pub proposer: String,
    pub timestamp: u64,
    pub transaction_count: i32,
    pub finalized: bool,
}

/// Block detail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockDetail {
    pub block_id: String,
    pub height: u64,
    pub view: u64,
    pub epoch: u64,
    pub parent_id: String,
    pub proposer: String,
    pub timestamp: u64,
    pub transactions_root: String,
    pub state_root: String,
    pub supply_commitment: String,
    pub transaction_count: i32,
    pub transactions: Vec<String>,
    pub finalized: bool,
    pub qc_vote_type: String,
    pub qc_view: u64,
    pub qc_signers: Vec<String>,
}

/// Transaction summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionSummary {
    pub tx_id: String,
    pub block_id: Option<String>,
    pub block_height: Option<i64>,
    pub sender: String,
    pub nonce: i64,
    pub fee: i64,
    pub payload_type: String,
    pub status: String,
    pub timestamp: i64,
    pub privacy_level: String,
}

/// Transaction detail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionDetail {
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
    pub privacy_level: String,
    pub nullifier_count: i32,
    pub commitment_count: i32,
}

/// Account detail
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
    pub is_validator: bool,
    pub stake_amount: Option<i64>,
    pub token_accounts: Vec<TokenAccountInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenAccountInfo {
    pub token_mint: String,
    pub token_symbol: String,
    pub balance: i64,
    pub balance_display: f64,
    pub decimals: i32,
}

/// Validator
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

/// Validator detail
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
    pub recent_blocks: Vec<ValidatorBlock>,
    pub stake_history: Vec<StakeHistoryEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorBlock {
    pub block_id: String,
    pub height: i64,
    pub timestamp: i64,
    pub transaction_count: i32,
}

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

/// Token mint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenMint {
    pub mint_address: String,
    pub symbol: String,
    pub name: String,
    pub decimals: i16,
    pub total_supply: i64,
    pub circulating_supply: i64,
    pub burned: i64,
    pub holder_count: i32,
    pub tx_count: i32,
}

/// Token supply
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenSupply {
    pub token: String,
    pub total_supply: i64,
    pub total_supply_display: f64,
    pub circulating_supply: i64,
    pub circulating_supply_display: f64,
    pub burned: i64,
    pub burned_display: f64,
    pub decimals: i16,
    pub holder_count: i32,
}

/// Search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    #[serde(rename = "type")]
    pub result_type: SearchResultType,
    pub id: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SearchResultType {
    Block,
    Transaction,
    Account,
    Validator,
    Token,
}

/// Fetch network stats
pub async fn get_stats() -> Result<NetworkStats, String> {
    Request::get(&format!("{}/stats", API_BASE))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}

/// Fetch latest blocks
pub async fn get_latest_blocks(limit: u32) -> Result<PaginatedResponse<BlockSummary>, String> {
    Request::get(&format!("{}/blocks?limit={}", API_BASE, limit))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}

/// Fetch blocks with pagination
pub async fn get_blocks(page: u32, limit: u32) -> Result<PaginatedResponse<BlockSummary>, String> {
    Request::get(&format!("{}/blocks?page={}&limit={}", API_BASE, page, limit))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}

/// Fetch block by ID or height
pub async fn get_block(id: &str) -> Result<BlockDetail, String> {
    Request::get(&format!("{}/blocks/{}", API_BASE, id))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}

/// Fetch latest transactions
pub async fn get_latest_transactions(limit: u32) -> Result<PaginatedResponse<TransactionSummary>, String> {
    Request::get(&format!("{}/transactions?limit={}", API_BASE, limit))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}

/// Fetch transactions with pagination
pub async fn get_transactions(page: u32, limit: u32) -> Result<PaginatedResponse<TransactionSummary>, String> {
    Request::get(&format!("{}/transactions?page={}&limit={}", API_BASE, page, limit))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}

/// Fetch transaction by ID
pub async fn get_transaction(id: &str) -> Result<TransactionDetail, String> {
    Request::get(&format!("{}/transactions/{}", API_BASE, id))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}

/// Fetch account by address
pub async fn get_account(address: &str) -> Result<AccountDetail, String> {
    Request::get(&format!("{}/accounts/{}", API_BASE, address))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}

/// Fetch validators with pagination
pub async fn get_validators(page: u32, limit: u32) -> Result<PaginatedResponse<Validator>, String> {
    Request::get(&format!("{}/validators?page={}&limit={}", API_BASE, page, limit))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}

/// Fetch validator by ID
pub async fn get_validator(id: &str) -> Result<ValidatorDetail, String> {
    Request::get(&format!("{}/validators/{}", API_BASE, id))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}

/// Fetch tokens with pagination
pub async fn get_tokens(page: u32, limit: u32) -> Result<PaginatedResponse<TokenMint>, String> {
    Request::get(&format!("{}/tokens?page={}&limit={}", API_BASE, page, limit))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}

/// Fetch token by mint address
pub async fn get_token(mint: &str) -> Result<TokenSupply, String> {
    Request::get(&format!("{}/tokens/{}", API_BASE, mint))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}

/// Search
pub async fn search(query: &str) -> Result<Vec<SearchResult>, String> {
    Request::get(&format!("{}/search?q={}", API_BASE, query))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}
