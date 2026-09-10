//! JSON-RPC client for `shrugg-node` (see fullnode/docs/rpc.md).

use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::time::Duration;

#[derive(Clone)]
pub struct RpcClient {
    client: Client,
    url: String,
}

#[derive(Serialize)]
struct JsonRpcRequest<'a> {
    jsonrpc: &'static str,
    id: u64,
    method: &'a str,
    params: serde_json::Value,
}

#[derive(Deserialize)]
struct JsonRpcResponse {
    result: Option<serde_json::Value>,
    error: Option<RpcError>,
}

#[derive(Deserialize, Debug)]
pub struct RpcError {
    pub code: i64,
    pub message: String,
}

impl RpcClient {
    pub fn new(url: &str) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("reqwest client");
        Self {
            client,
            url: url.to_string(),
        }
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    /// Send a request; a JSON `null` result is returned as `None`.
    async fn call<T: DeserializeOwned>(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<Option<T>> {
        let req = JsonRpcRequest {
            jsonrpc: "2.0",
            id: 1,
            method,
            params,
        };
        let resp: JsonRpcResponse = self
            .client
            .post(&self.url)
            .json(&req)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        if let Some(e) = resp.error {
            return Err(anyhow!("rpc {} failed: {} ({})", method, e.message, e.code));
        }
        match resp.result {
            None | Some(serde_json::Value::Null) => Ok(None),
            Some(v) => Ok(Some(serde_json::from_value(v)?)),
        }
    }

    async fn call_required<T: DeserializeOwned>(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<T> {
        self.call(method, params)
            .await?
            .ok_or_else(|| anyhow!("rpc {} returned null", method))
    }

    pub async fn chain_id(&self) -> Result<u64> {
        self.call_required("shrugg_chainId", serde_json::json!([]))
            .await
    }

    pub async fn token_info(&self) -> Result<TokenInfo> {
        self.call_required("shrugg_tokenInfo", serde_json::json!([]))
            .await
    }

    pub async fn head(&self) -> Result<Head> {
        self.call_required("shrugg_getHead", serde_json::json!([]))
            .await
    }

    pub async fn status(&self) -> Result<NodeStatus> {
        self.call_required("shrugg_status", serde_json::json!([]))
            .await
    }

    pub async fn block_by_height(&self, height: u64) -> Result<Option<RpcBlock>> {
        self.call("shrugg_getBlockByHeight", serde_json::json!([height]))
            .await
    }

    pub async fn block_by_hash(&self, hash: &str) -> Result<Option<RpcBlock>> {
        self.call("shrugg_getBlockByHash", serde_json::json!([hash]))
            .await
    }

    pub async fn account(&self, address: &str) -> Result<RpcAccount> {
        self.call_required("shrugg_getAccount", serde_json::json!([address]))
            .await
    }

    pub async fn validators(&self) -> Result<Vec<RpcValidator>> {
        self.call_required("shrugg_getValidators", serde_json::json!([]))
            .await
    }

    pub async fn peers(&self) -> Result<Vec<RpcPeer>> {
        self.call_required("shrugg_getPeers", serde_json::json!([]))
            .await
    }

    pub async fn receipt(&self, tx_hash: &str) -> Result<Option<RpcReceipt>> {
        self.call("shrugg_getReceipt", serde_json::json!([tx_hash]))
            .await
    }

    pub async fn program(&self, id: &str) -> Result<Option<RpcProgram>> {
        self.call("shrugg_getProgram", serde_json::json!([id]))
            .await
    }

    pub async fn is_connected(&self) -> bool {
        self.head().await.is_ok()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct TokenInfo {
    pub symbol: String,
    pub decimals: u8,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Head {
    pub height: u64,
    pub hash: String,
    pub view: u64,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct NodeStatus {
    #[serde(default)]
    pub height: u64,
    #[serde(default)]
    pub head_hash: String,
    #[serde(default)]
    pub view: u64,
    #[serde(default)]
    pub high_qc_view: u64,
    #[serde(default)]
    pub syncing: bool,
    #[serde(default)]
    pub sync_target: u64,
    #[serde(default)]
    pub peer_count: u32,
    #[serde(default)]
    pub mempool_size: u32,
    #[serde(default)]
    pub is_validator: bool,
    #[serde(default)]
    pub faucet: bool,
    #[serde(default)]
    pub confidential: bool,
    #[serde(default)]
    pub fri_profile: String,
    #[serde(default)]
    pub programs: u64,
    #[serde(default)]
    pub address: String,
    #[serde(default)]
    pub peer_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RpcBlock {
    pub hash: String,
    pub height: u64,
    pub view: u64,
    pub parent: String,
    pub proposer: String,
    pub timestamp_ms: u64,
    pub tx_root: String,
    pub state_root: String,
    pub justify_view: u64,
    #[serde(default)]
    pub tx_count: u32,
    #[serde(default)]
    pub transactions: Vec<RpcTx>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RpcTx {
    pub hash: String,
    pub from: String,
    pub nonce: u64,
    pub fee: String,
    pub chain_id: u64,
    pub kind: RpcTxKind,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum RpcTxKind {
    Transfer {
        to: String,
        amount: String,
    },
    Mint {
        to: String,
        amount: String,
    },
    Deploy {
        base_pc: u32,
        words_len: u32,
        program: String,
    },
    Call {
        program: String,
        proof_len: u64,
        #[serde(default)]
        recipients: Vec<String>,
    },
}

#[derive(Debug, Clone, Deserialize)]
pub struct RpcAccount {
    pub address: String,
    pub nonce: u64,
    pub balance: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RpcValidator {
    pub address: String,
    pub stake: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RpcPeer {
    pub peer_id: String,
    #[serde(default)]
    pub addrs: Vec<String>,
    #[serde(default)]
    pub connected_secs: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RpcReceipt {
    pub tx: String,
    pub program: String,
    pub tier: u32,
    #[serde(default)]
    pub outputs: Vec<i64>,
    pub effect: Option<RpcEffect>,
    pub height: u64,
    pub index: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RpcEffect {
    pub to: String,
    pub amount: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RpcProgram {
    pub id: String,
    pub base_pc: u32,
    pub words_len: u32,
    pub code_hash: String,
    pub deployer: String,
    pub deployed_at: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_block_with_all_kinds() {
        let json = r#"{"hash":"a9c8","height":10,"justify_view":31,"parent":"a070","proposer":"2nRd","state_root":"b364","timestamp_ms":1788977138048,
          "transactions":[
            {"chain_id":4,"fee":"4200000","from":"2nRd","hash":"dc97","kind":{"base_pc":0,"program":"675a","type":"deploy","words_len":42},"nonce":0},
            {"chain_id":4,"fee":"1000000","from":"2nRd","hash":"d4c7","kind":{"program":"675a","proof_len":880866,"recipients":["ByDk"],"type":"call"},"nonce":1},
            {"chain_id":4,"fee":"1000","from":"7th5","hash":"e7a7","kind":{"type":"transfer","to":"9W7d","amount":"3500000000"},"nonce":0},
            {"chain_id":4,"fee":"0","from":"CxeG","hash":"ffff","kind":{"type":"mint","to":"ByDk","amount":"100000000000"},"nonce":3}
          ],"tx_count":4,"tx_root":"dc97","view":32}"#;
        let b: RpcBlock = serde_json::from_str(json).unwrap();
        assert_eq!(b.height, 10);
        assert_eq!(b.transactions.len(), 4);
        assert!(matches!(
            &b.transactions[0].kind,
            RpcTxKind::Deploy { words_len: 42, .. }
        ));
        assert!(matches!(
            &b.transactions[1].kind,
            RpcTxKind::Call {
                proof_len: 880866,
                ..
            }
        ));
        assert!(matches!(
            &b.transactions[2].kind,
            RpcTxKind::Transfer { .. }
        ));
        assert!(matches!(&b.transactions[3].kind, RpcTxKind::Mint { .. }));
    }

    #[test]
    fn parses_receipt_and_status() {
        let r: RpcReceipt = serde_json::from_str(r#"{"effect":{"amount":"25","to":"ByDk"},"height":19,"index":0,"outputs":[1,0,25,0,0,0,0,0],"program":"675a","tier":10,"tx":"d4c7"}"#).unwrap();
        assert_eq!(r.effect.unwrap().amount, "25");
        let r: RpcReceipt = serde_json::from_str(r#"{"effect":null,"height":78,"index":0,"outputs":[0,0,0,0,0,0,0,0],"program":"675a","tier":10,"tx":"dd1a"}"#).unwrap();
        assert!(r.effect.is_none());
        let s: NodeStatus = serde_json::from_str(r#"{"address":"CxeG","confidential":true,"faucet":true,"fri_profile":"production","head_hash":"02f0","height":324,"high_qc_view":664,"is_validator":false,"mempool_size":0,"peer_count":4,"peer_id":"12D3","programs":1,"sync_target":324,"syncing":false,"view":667}"#).unwrap();
        assert_eq!(s.peer_count, 4);
        assert!(s.confidential);
    }
}
