//! An in-process stand-in for `shrugg-node`'s JSON-RPC on the shielded chain
//! (`fullnode/docs/rpc.md`), scripted from the test: the chain id, the committed blocks, the
//! tree leaves and the validator register can be swapped at any time, which is how the tests
//! simulate a hard fork under a running indexer.

#![allow(dead_code)]

use axum::{extract::State, routing::post, Json, Router};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::sync::{Arc, Mutex};

pub const VALIDATOR: &str = "2nRdFChBXRmKoe2sQE3ZYDzvdg53QmBZJJ9iweY7hk1v";
pub const VALIDATOR_B: &str = "ByDkxsEfDCR5DrmDufKftvcRsgvufypnZ4SgDQzJAQ7Z";
pub const SHIELDED_ADDR: &str = "shrugg1q9fexampleexampleexampleexampleexampleexampleexample";

/// 64 lowercase hex characters derived from `seed`.
pub fn h(seed: &str) -> String {
    hex::encode(Sha256::digest(seed.as_bytes()))
}

/// A register entry as the S2 node serves it.
#[derive(Clone)]
pub struct MockValidator {
    pub address: String,
    pub stake: String,
    pub rewards: String,
    pub pending: Vec<(u64, String)>,
    pub active: bool,
}

impl MockValidator {
    pub fn new(address: &str, stake: &str) -> Self {
        MockValidator {
            address: address.into(),
            stake: stake.into(),
            rewards: "0".into(),
            pending: vec![],
            active: true,
        }
    }
}

pub struct MockChain {
    pub chain_id: u64,
    /// Distinguishes two genesis blocks with the same chain id (a re-created testnet).
    pub salt: String,
    pub blocks: Vec<Value>,
    pub view: u64,
    pub validators: Vec<MockValidator>,
    /// Serve the register in the S3-branch shape (`{address, stake, rewards: number}`) and
    /// answer "unknown method" to `shrugg_getEpoch` / `shrugg_getSupply`.
    pub pre_s2: bool,
    /// The commitment tree: (cm, height), leaf index = position.
    pub leaves: Vec<(String, u64)>,
    /// Every JSON-RPC method name the indexer called, in order.
    pub calls: Vec<String>,
}

impl MockChain {
    /// A chain with only its genesis block and one genesis deposit note.
    pub fn new(chain_id: u64, salt: &str) -> Self {
        let mut c = MockChain {
            chain_id,
            salt: salt.to_string(),
            blocks: Vec::new(),
            view: 0,
            validators: vec![MockValidator::new(VALIDATOR, "100000000000000")],
            pre_s2: false,
            leaves: vec![(h(&format!("genesis-note-{chain_id}-{salt}")), 0)],
            calls: Vec::new(),
        };
        c.push_block(vec![]);
        c
    }

    pub fn head(&self) -> &Value {
        self.blocks.last().expect("genesis")
    }

    /// Append a block holding `txs` (already built with [`tx`]); returns its hash. Every
    /// commitment the block's bundles and mints carry becomes a tree leaf.
    pub fn push_block(&mut self, txs: Vec<Value>) -> String {
        let height = self.blocks.len() as u64;
        let parent = if height == 0 {
            "0".repeat(64)
        } else {
            self.head()["hash"].as_str().unwrap().to_string()
        };
        let hash = h(&format!("block-{}-{}-{}", self.chain_id, self.salt, height));
        self.view = height * 2;
        for t in &txs {
            for b in [&t["bundle"], &t["action"]["asset_bundle"]] {
                if let Some(cms) = b["commitments"].as_array() {
                    for cm in cms {
                        self.leaves.push((cm.as_str().unwrap().to_string(), height));
                    }
                }
            }
            if let Some(cm) = t["action"]["cm"].as_str() {
                self.leaves.push((cm.to_string(), height));
            }
        }
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

    fn nullifier_count(&self) -> usize {
        self.blocks
            .iter()
            .flat_map(|b| b["transactions"].as_array().cloned().unwrap_or_default())
            .map(|t| {
                let mut n = 0;
                if t["bundle"].is_object() {
                    n += 2;
                }
                if t["action"]["asset_bundle"].is_object() {
                    n += 2;
                }
                n
            })
            .sum()
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
                    "fri_profile": "test", "programs": 0,
                    "notes": self.leaves.len(), "nullifiers": self.nullifier_count(),
                    "tree_root": h(&format!("root-{}", self.leaves.len())),
                    "hc_bundle": h("hc_bundle"),
                    "address": null,
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
            "shrugg_getCommitments" => {
                let from = p(0).as_u64().ok_or((-32602, "from_index".to_string()))? as usize;
                let limit = p(1).as_u64().unwrap_or(1000).min(1000) as usize;
                json!(self
                    .leaves
                    .iter()
                    .enumerate()
                    .skip(from)
                    .take(limit)
                    .map(|(i, (cm, height))| json!({
                        "index": i, "cm": cm, "height": height,
                        "envelope": { "kem_ct": "00", "to_receiver": "00", "to_sender": "00", "body": "00" }
                    }))
                    .collect::<Vec<_>>())
            }
            "shrugg_getTreeInfo" => json!({
                "next_index": self.leaves.len(), "root": h(&format!("root-{}", self.leaves.len())),
                "nullifiers": self.nullifier_count()
            }),
            "shrugg_getValidators" if self.pre_s2 => json!(self
                .validators
                .iter()
                .map(|v| json!({ "address": v.address, "stake": v.stake, "rewards": v.rewards.parse::<u64>().unwrap_or(0) }))
                .collect::<Vec<_>>()),
            "shrugg_getValidators" => json!(self
                .validators
                .iter()
                .map(|v| json!({
                    "address": v.address, "stake": v.stake, "rewards": v.rewards,
                    "pending": v.pending.iter().map(|(e, a)| json!({ "release_epoch": e, "amount": a })).collect::<Vec<_>>(),
                    "payout": SHIELDED_ADDR, "nonce": 0, "active": v.active
                }))
                .collect::<Vec<_>>()),
            "shrugg_getEpoch" if !self.pre_s2 => {
                let height = self.head()["height"].as_u64().unwrap();
                json!({ "epoch": height / 1000, "epoch_blocks": 1000,
                        "next_set": self.validators.iter().filter(|v| v.active).map(|v| v.address.clone()).collect::<Vec<_>>() })
            }
            "shrugg_getSupply" if !self.pre_s2 => json!({
                "height": self.head()["height"], "genesis_deposited": "1000000000000", "genesis_staked": "100000000000000",
                "faucet_minted": "100000000000", "withdraw_deposited": "0", "fees_paid": "3000000", "burned": "0",
                "pool_value": "1099997000000", "register_total": "100000003000000", "total_supply": "101100000000000",
                "invariant_holds": true
            }),
            "shrugg_getBridgeState" => json!({
                "enabled": true, "emitter": "01".repeat(32), "emitters": { "2": "02".repeat(32) },
                "guardian_set_index": 0, "guardians": ["aa".repeat(20)], "burn_sequence": 1, "next_index": 3,
                "assets": [
                    { "index": 1, "chain": 2, "token": "cc".repeat(32), "asset_id": h("asset-1") },
                    { "index": 2, "chain": 2, "token": format!("{}dac17f958d2ee523a2206206994597c13d831ec7", "0".repeat(24)), "asset_id": h("asset-2") }
                ]
            }),
            "shrugg_getPeers" => json!([]),
            "shrugg_getCallEnvelope" => {
                // A transcript for every call the mock knows about, sealed to nobody (opaque bytes).
                let hash = p(0).as_str().unwrap_or("").to_string();
                let is_call = self.blocks.iter().flat_map(|b| b["transactions"].as_array().cloned().unwrap_or_default())
                    .any(|t| t["hash"] == hash && t["action"]["kind"] == "call");
                if is_call { json!({ "tx": hash, "h_in": h("h_in"), "kem_ct": "", "to_sender": "0a".repeat(60), "to_auditor": "", "body": "0b".repeat(60) }) } else { Value::Null }
            }
            "shrugg_getReceipt" | "shrugg_getProgram" => Value::Null,
            other => return Err((-32601, format!("unknown method {other}"))),
        })
    }
}

/// The public fields of a bundle, unique per `seed`.
pub fn bundle(seed: &str) -> Value {
    json!({
        "anchor": h(&format!("anchor-{seed}")),
        "nullifiers": [h(&format!("nf1-{seed}")), h(&format!("nf2-{seed}"))],
        "commitments": [h(&format!("cm1-{seed}")), h(&format!("cm2-{seed}"))],
        "fee": 1000000, "burn": 0, "asset": 0, "time": 1,
        "proof_len": 302857, "envelope_len": [1380, 1380]
    })
}

/// A transaction as the node serialises it inside a block: a bundle (or `null`) and an action.
pub fn tx(chain_id: u64, seed: &str, bundle: Option<Value>, action: Value) -> Value {
    let hash = h(&format!("tx-{chain_id}-{seed}-{action}"));
    json!({ "hash": hash, "chain_id": chain_id, "bundle": bundle, "action": action })
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
