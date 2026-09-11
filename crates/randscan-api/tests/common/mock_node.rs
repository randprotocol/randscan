//! An in-process stand-in for `shrugg-node`'s JSON-RPC (`fullnode/docs/rpc.md`), scripted from
//! the test: the chain id, the committed blocks and the validator set can be swapped at any time,
//! which is how the tests simulate a hard fork under a running indexer.

#![allow(dead_code)]

use axum::{extract::State, routing::post, Json, Router};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::sync::{Arc, Mutex};

pub const VALIDATOR: &str = "2nRdFChBXRmKoe2sQE3ZYDzvdg53QmBZJJ9iweY7hk1v";
pub const ALICE: &str = "ByDkxsEfDCR5DrmDufKftvcRsgvufypnZ4SgDQzJAQ7Z";
pub const BOB: &str = "F6rYLexPhyMmwPNqbEmyyp5FiTmtQqDgZyqScUqYY4F6";
pub const CAROL: &str = "5tMgLSzXL8keU1vg2wtGEXRJkmfBK6GzhjNjxrCFgCaj";

/// 64 lowercase hex characters derived from `seed`.
pub fn h(seed: &str) -> String {
    hex::encode(Sha256::digest(seed.as_bytes()))
}

pub struct MockChain {
    pub chain_id: u64,
    /// Distinguishes two genesis blocks with the same chain id (a re-created testnet).
    pub salt: String,
    pub blocks: Vec<Value>,
    pub view: u64,
    pub validators: Vec<String>,
    /// Every JSON-RPC method name the indexer called, in order.
    pub calls: Vec<String>,
}

impl MockChain {
    /// A chain with only its genesis block.
    pub fn new(chain_id: u64, salt: &str) -> Self {
        let mut c = MockChain {
            chain_id,
            salt: salt.to_string(),
            blocks: Vec::new(),
            view: 0,
            validators: vec![VALIDATOR.to_string()],
            calls: Vec::new(),
        };
        c.push_block(vec![]);
        c
    }

    pub fn head(&self) -> &Value {
        self.blocks.last().expect("genesis")
    }

    /// Append a block holding `txs` (already built with [`tx`]); returns its hash.
    pub fn push_block(&mut self, txs: Vec<Value>) -> String {
        let height = self.blocks.len() as u64;
        let parent = if height == 0 {
            "0".repeat(64)
        } else {
            self.head()["hash"].as_str().unwrap().to_string()
        };
        let hash = h(&format!("block-{}-{}-{}", self.chain_id, self.salt, height));
        self.view = height * 2;
        self.blocks.push(json!({
            "hash": hash,
            "height": height,
            "view": self.view,
            "parent": parent,
            "proposer": VALIDATOR,
            "timestamp_ms": 1_789_000_000_000u64 + height * 1000,
            "tx_root": h(&format!("txroot-{height}")),
            "state_root": h(&format!("state-{}-{}-{height}", self.chain_id, self.salt)),
            "justify_view": self.view.saturating_sub(1),
            "tx_count": txs.len(),
            "transactions": txs,
        }));
        hash
    }

    fn dispatch(&mut self, method: &str, params: &Value) -> Result<Value, (i64, String)> {
        self.calls.push(method.to_string());
        let p = |i: usize| params.get(i).cloned().unwrap_or(Value::Null);
        Ok(match method {
            "shrugg_chainId" => json!(self.chain_id),
            "shrugg_tokenInfo" => json!({ "symbol": "SHRUGG", "decimals": 9 }),
            "shrugg_getHead" => {
                let head = self.head();
                json!({ "height": head["height"], "hash": head["hash"], "view": self.view })
            }
            "shrugg_status" | "shrugg_syncStatus" => {
                let head = self.head();
                json!({
                    "height": head["height"], "head_hash": head["hash"], "view": self.view,
                    "high_qc_view": self.view.saturating_sub(1), "syncing": false,
                    "sync_target": head["height"], "peer_count": 0, "mempool_size": 0,
                    "is_validator": false, "faucet": true, "confidential": true,
                    "fri_profile": "test", "programs": 0, "address": VALIDATOR,
                    "peer_id": "12D3KooWmockmockmockmockmockmockmockmockmock"
                })
            }
            "shrugg_getBlockByHeight" => {
                let height = p(0).as_u64().ok_or((-32602, "height".to_string()))? as usize;
                self.blocks.get(height).cloned().unwrap_or(Value::Null)
            }
            "shrugg_getBlockByHash" => {
                let hash = p(0).as_str().unwrap_or("").to_ascii_lowercase();
                self.blocks
                    .iter()
                    .find(|b| b["hash"] == hash)
                    .cloned()
                    .unwrap_or(Value::Null)
            }
            "shrugg_getAccount" => {
                let address = p(0).as_str().unwrap_or("").to_string();
                json!({ "address": address, "nonce": 1, "balance": "1000000000" })
            }
            "shrugg_getValidators" => json!(self
                .validators
                .iter()
                .map(|a| json!({ "address": a, "stake": "100000" }))
                .collect::<Vec<_>>()),
            "shrugg_getPeers" => json!([]),
            "shrugg_getReceipt" | "shrugg_getProgram" => Value::Null,
            other => return Err((-32601, format!("unknown method {other}"))),
        })
    }
}

/// A transaction as the node serializes it inside a block.
pub fn tx(chain_id: u64, from: &str, nonce: u64, kind: Value) -> Value {
    let hash = h(&format!("tx-{chain_id}-{from}-{nonce}-{kind}"));
    json!({ "hash": hash, "from": from, "nonce": nonce, "fee": "1000", "chain_id": chain_id, "kind": kind })
}

pub struct MockNode {
    pub chain: Arc<Mutex<MockChain>>,
    pub url: String,
}

impl MockNode {
    pub fn with_chain<R>(&self, f: impl FnOnce(&mut MockChain) -> R) -> R {
        f(&mut self.chain.lock().unwrap())
    }
}

async fn rpc(State(chain): State<Arc<Mutex<MockChain>>>, Json(req): Json<Value>) -> Json<Value> {
    let id = req["id"].clone();
    let method = req["method"].as_str().unwrap_or("");
    let params = req["params"].clone();
    let out = chain.lock().unwrap().dispatch(method, &params);
    Json(match out {
        Ok(result) => json!({ "jsonrpc": "2.0", "id": id, "result": result }),
        Err((code, message)) => {
            json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
        }
    })
}

/// Serve `chain` on a random localhost port for the rest of the test.
pub async fn start_mock_node(chain: MockChain) -> MockNode {
    let chain = Arc::new(Mutex::new(chain));
    let app = Router::new().route("/", post(rpc)).with_state(chain.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    MockNode { chain, url }
}
