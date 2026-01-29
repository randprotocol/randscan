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
        self.send("getBlockHeight", serde_json::json!({})).await
    }

    /// Get block by height
    pub async fn get_block(&self, height: u64) -> Result<BlockResponse> {
        self.send("getBlock", serde_json::json!({ "height": height }))
            .await
    }

    /// Get block by ID
    pub async fn get_block_by_id(&self, block_id: &str) -> Result<BlockResponse> {
        self.send("getBlock", serde_json::json!({ "block_id": block_id }))
            .await
    }

    /// Get blocks in range
    pub async fn get_blocks(&self, start: u64, end: u64) -> Result<Vec<BlockResponse>> {
        self.send(
            "getBlocks",
            serde_json::json!({
                "start_height": start,
                "end_height": end
            }),
        )
        .await
    }

    /// Get transaction by ID
    pub async fn get_transaction(&self, tx_id: &str) -> Result<TransactionResponse> {
        self.send("getTransaction", serde_json::json!({ "signature": tx_id }))
            .await
    }

    /// Get account info
    pub async fn get_account_info(&self, address: &str) -> Result<AccountInfoResponse> {
        self.send("getAccountInfo", serde_json::json!({ "address": address }))
            .await
    }

    /// Get validators
    pub async fn get_validators(&self) -> Result<Vec<ValidatorResponse>> {
        self.send("getValidators", serde_json::json!({})).await
    }

    /// Get token supply
    pub async fn get_token_supply(&self, token: &str) -> Result<TokenSupplyResponse> {
        self.send("getTokenSupply", serde_json::json!({ "token": token }))
            .await
    }

    /// Get epoch info
    pub async fn get_epoch_info(&self) -> Result<EpochInfoResponse> {
        self.send("getEpochInfo", serde_json::json!({})).await
    }

    /// Get node health
    pub async fn get_health(&self) -> Result<HealthResponse> {
        self.send("getHealth", serde_json::json!({})).await
    }

    /// Get latest blockhash
    pub async fn get_latest_blockhash(&self) -> Result<String> {
        self.send("getLatestBlockhash", serde_json::json!({})).await
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

/// Block response from RPC
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
    // QC fields
    pub qc_vote_type: Option<String>,
    pub qc_view: Option<u64>,
    pub qc_block_id: Option<String>,
    pub qc_signers: Option<Vec<String>>,
}

/// Transaction response from RPC
#[derive(Debug, Clone, Deserialize)]
pub struct TransactionResponse {
    pub signature: String, // tx_id
    pub sender: String,
    pub nonce: u64,
    pub tx_type: String,
    pub fee: u64,
    pub compute_budget: Option<u64>,
    pub timestamp: u64,
    pub block_height: Option<u64>,
    pub block_id: Option<String>,
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
    pub commission_rate: u16,
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
    pub start_height: u64,
    pub slots_in_epoch: u64,
    pub absolute_slot: u64,
    pub block_height: u64,
    pub transaction_count: Option<u64>,
}

/// Health response
#[derive(Debug, Clone, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: Option<String>,
}
