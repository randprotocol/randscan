//! RPC client for RandProtocol node

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use reqwest::Client;
use std::time::Duration;

/// RPC client for RandProtocol node
#[derive(Clone)]
pub struct RpcClient {
    client: Client,
    url: String,
}

impl RpcClient {
    pub fn new(url: &str) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            url: url.to_string(),
        }
    }

    /// Send a JSON-RPC request
    async fn send<T: for<'de> Deserialize<'de>>(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<T> {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: method.to_string(),
            params,
        };

        let response = self
            .client
            .post(&self.url)
            .json(&request)
            .send()
            .await?;

        let rpc_response: JsonRpcResponse<T> = response.json().await?;

        if let Some(error) = rpc_response.error {
            return Err(anyhow!("RPC error: {} - {}", error.code, error.message));
        }

        rpc_response
            .result
            .ok_or_else(|| anyhow!("Empty RPC response"))
    }

    /// Get current block height
    pub async fn get_block_height(&self) -> Result<u64> {
        // RandProtocol returns {"height": u64}
        let response: BlockHeightResponse = self.send("getBlockHeight", serde_json::json!([])).await?;
        Ok(response.height)
    }

    /// Get block by height
    pub async fn get_block(&self, height: u64) -> Result<BlockResponse> {
        // RandProtocol expects positional params: [height] or ["block_id"]
        let rpc_response: RpcBlockResponse = self.send("getBlock", serde_json::json!([height])).await?;
        Ok(rpc_response.into())
    }

    /// Get block by ID
    pub async fn get_block_by_id(&self, block_id: &str) -> Result<BlockResponse> {
        let rpc_response: RpcBlockResponse = self.send("getBlock", serde_json::json!([block_id])).await?;
        Ok(rpc_response.into())
    }

    /// Get blocks in range
    pub async fn get_blocks(&self, start: u64, end: u64) -> Result<Vec<BlockResponse>> {
        let rpc_response: GetBlocksResponse = self.send(
            "getBlocks",
            serde_json::json!({
                "start_height": start,
                "end_height": end,
                "limit": (end - start + 1) as usize
            }),
        )
        .await?;

        Ok(rpc_response.blocks.into_iter().map(|b| b.into()).collect())
    }

    /// Get transaction by ID
    pub async fn get_transaction(&self, tx_id: &str) -> Result<TransactionResponse> {
        // RandProtocol expects positional params
        self.send("getTransaction", serde_json::json!([tx_id]))
            .await
    }

    /// Get account info
    pub async fn get_account_info(&self, address: &str) -> Result<AccountInfoResponse> {
        // RandProtocol expects positional params
        self.send("getAccountInfo", serde_json::json!([address]))
            .await
    }

    /// Get validators
    pub async fn get_validators(&self) -> Result<Vec<ValidatorResponse>> {
        let response: GetValidatorsResponse = self.send("getValidators", serde_json::json!([])).await?;
        Ok(response.validators)
    }

    /// Get token supply
    pub async fn get_token_supply(&self, token: &str) -> Result<TokenSupplyResponse> {
        // RandProtocol expects positional params
        self.send("getTokenSupply", serde_json::json!([token]))
            .await
    }

    /// Get epoch info
    pub async fn get_epoch_info(&self) -> Result<EpochInfoResponse> {
        self.send("getEpochInfo", serde_json::json!([])).await
    }

    /// Get node health
    pub async fn get_health(&self) -> Result<HealthResponse> {
        self.send("getHealth", serde_json::json!([])).await
    }

    /// Get latest blockhash
    pub async fn get_latest_blockhash(&self) -> Result<String> {
        let response: LatestBlockhashResponse = self.send("getLatestBlockhash", serde_json::json!([])).await?;
        Ok(response.blockhash)
    }

    /// Check if connected
    pub async fn is_connected(&self) -> bool {
        self.get_health().await.is_ok()
    }
}

/// JSON-RPC request
#[derive(Serialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: u64,
    method: String,
    params: serde_json::Value,
}

/// JSON-RPC response
#[derive(Deserialize)]
struct JsonRpcResponse<T> {
    #[allow(dead_code)]
    jsonrpc: String,
    #[allow(dead_code)]
    id: u64,
    result: Option<T>,
    error: Option<RpcError>,
}

/// RPC error
#[derive(Deserialize)]
struct RpcError {
    code: i64,
    message: String,
}

/// Block height response from RandProtocol
#[derive(Debug, Deserialize)]
struct BlockHeightResponse {
    height: u64,
}

/// Latest blockhash response from RandProtocol
#[derive(Debug, Deserialize)]
struct LatestBlockhashResponse {
    blockhash: String,
    #[allow(dead_code)]
    last_valid_block_height: u64,
}

/// Block response from RandProtocol RPC (internal format)
#[derive(Debug, Deserialize)]
struct RpcBlockResponse {
    block_id: String,
    height: u64,
    view: u64,
    epoch: u64,
    parent_id: String,
    proposer: String,
    timestamp: u64,
    state_root: String,
    transactions_root: String,
    transaction_count: usize,
    transactions: Vec<String>, // Transaction IDs as strings
    finalized: bool,
}

/// Block summary response (from getBlocks)
#[derive(Debug, Deserialize)]
struct RpcBlockSummary {
    height: u64,
    block_id: String,
    timestamp: u64,
    transaction_count: usize,
}

impl From<RpcBlockSummary> for BlockResponse {
    fn from(summary: RpcBlockSummary) -> Self {
        BlockResponse {
            block_id: summary.block_id,
            height: summary.height,
            view: 0,
            epoch: 0,
            parent_id: String::new(),
            proposer: String::new(),
            timestamp: summary.timestamp,
            state_root: String::new(),
            transactions_root: String::new(),
            supply_commitment: None,
            transaction_count: summary.transaction_count,
            transactions: Vec::new(),
            finalized: true,
            qc_vote_type: None,
            qc_view: None,
            qc_block_id: None,
            qc_signers: None,
        }
    }
}

/// Get blocks response wrapper
#[derive(Debug, Deserialize)]
struct GetBlocksResponse {
    blocks: Vec<RpcBlockSummary>,
}

/// Get validators response wrapper
#[derive(Debug, Deserialize)]
struct GetValidatorsResponse {
    validators: Vec<ValidatorResponse>,
}

/// Block response (external format used by indexer)
#[derive(Debug, Clone, Deserialize)]
pub struct BlockResponse {
    pub block_id: String,
    pub height: u64,
    pub view: u64,
    pub epoch: u64,
    pub parent_id: String,
    pub proposer: String,
    pub timestamp: u64,
    pub state_root: String,
    pub transactions_root: String,
    pub supply_commitment: Option<String>,
    pub transaction_count: usize,
    pub transactions: Vec<TransactionResponse>,
    pub finalized: bool,
    // QC fields (optional, may not be provided by RandProtocol)
    pub qc_vote_type: Option<String>,
    pub qc_view: Option<u64>,
    pub qc_block_id: Option<String>,
    pub qc_signers: Option<Vec<String>>,
}

impl From<RpcBlockResponse> for BlockResponse {
    fn from(rpc: RpcBlockResponse) -> Self {
        let block_id = rpc.block_id;
        let height = rpc.height;
        let timestamp = rpc.timestamp;

        BlockResponse {
            block_id: block_id.clone(),
            height,
            view: rpc.view,
            epoch: rpc.epoch,
            parent_id: rpc.parent_id,
            proposer: rpc.proposer,
            timestamp,
            state_root: rpc.state_root,
            transactions_root: rpc.transactions_root,
            supply_commitment: None,
            transaction_count: rpc.transaction_count,
            // Convert transaction IDs to empty TransactionResponse placeholders
            // The processor will fetch full transaction details separately if needed
            transactions: rpc.transactions.into_iter().map(|sig| TransactionResponse {
                signature: sig,
                sender: String::new(),
                nonce: 0,
                tx_type: String::new(),
                fee: 0,
                compute_budget: None,
                timestamp,
                block_height: Some(height),
                block_id: Some(block_id.clone()),
                status: "finalized".to_string(),
                logs: None,
                payload: None,
            }).collect(),
            finalized: rpc.finalized,
            qc_vote_type: None,
            qc_view: None,
            qc_block_id: None,
            qc_signers: None,
        }
    }
}

/// Transaction response from RPC
#[derive(Debug, Clone, Deserialize)]
pub struct TransactionResponse {
    pub signature: String, // tx_id
    #[serde(default)]
    pub sender: String,
    #[serde(default)]
    pub nonce: u64,
    #[serde(default, rename = "tx_type")]
    pub tx_type: String,
    #[serde(default)]
    pub fee: u64,
    pub compute_budget: Option<u64>,
    #[serde(default)]
    pub timestamp: u64,
    pub block_height: Option<u64>,
    pub block_id: Option<String>,
    #[serde(default)]
    pub status: String,
    pub logs: Option<Vec<String>>,
    // Type-specific fields
    pub payload: Option<TransactionPayloadResponse>,
}

/// Transaction payload response
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TransactionPayloadResponse {
    Public {
        instructions_size: usize,
    },
    Private {
        encrypted_payload_size: usize,
        proof_size: usize,
        nullifiers: Vec<String>,
        commitments: Vec<String>,
    },
    Stealth {
        ephemeral_pubkey: String,
        stealth_address: String,
        proof_size: usize,
    },
    Stake {
        amount: u64,
    },
    Unstake {
        amount: u64,
    },
    Transfer {
        to: String,
        amount: u64,
    },
    Deploy {
        code_size: usize,
        program_id: Option<String>,
    },
    Invoke {
        program_id: String,
        instruction_size: usize,
    },
    PrivateTransfer {
        proof_size: usize,
        nullifiers: Vec<String>,
        commitments: Vec<String>,
    },
    Mint {
        to: String,
        amount: u64,
    },
}

/// Account info response
#[derive(Debug, Clone, Deserialize)]
pub struct AccountInfoResponse {
    pub address: String,
    pub atlas_balance: u64,
    pub shrug_balance: u64,
    pub nonce: u64,
    pub executable: bool,
    pub owner: Option<String>,
    pub data: Option<String>,
    pub data_len: usize,
}

/// Validator response
#[derive(Debug, Clone, Deserialize)]
pub struct ValidatorResponse {
    pub validator_id: String,
    pub pubkey: String,
    pub stake: u64,
    #[serde(alias = "commission")]
    pub commission_rate: u16,
    #[serde(alias = "active")]
    pub is_active: bool,
    pub blocks_produced: Option<u64>,
    pub last_vote_height: Option<u64>,
}

/// Token supply response
#[derive(Debug, Clone, Deserialize)]
pub struct TokenSupplyResponse {
    pub token: String,
    pub total_supply: u64,
    pub circulating_supply: u64,
    pub burned: u64,
    pub decimals: u8,
}

/// Epoch info response
#[derive(Debug, Clone, Deserialize)]
pub struct EpochInfoResponse {
    pub epoch: u64,
    #[serde(default)]
    pub slots_in_epoch: u64,
    #[serde(default, alias = "slot_index")]
    pub absolute_slot: u64,
    #[serde(default)]
    pub block_height: u64,
    pub transaction_count: Option<u64>,
}

/// Health response
#[derive(Debug, Clone, Deserialize)]
pub struct HealthResponse {
    #[serde(alias = "healthy")]
    pub status: String,
    pub version: Option<String>,
    #[serde(default)]
    pub height: u64,
    #[serde(default)]
    pub peer_count: usize,
    #[serde(default)]
    pub is_syncing: bool,
}
