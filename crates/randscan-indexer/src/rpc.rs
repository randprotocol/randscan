//! JSON-RPC client for `shrugg-node` on the shielded chain (see fullnode/docs/rpc.md).

use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize};
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

/// JSON-RPC "unknown method": what a node of an earlier phase answers for a method it lacks.
const METHOD_NOT_FOUND: i64 = -32601;

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

    async fn raw(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<Option<serde_json::Value>> {
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
            return Err(RpcFailure {
                method: method.to_string(),
                code: e.code,
                message: e.message,
            }
            .into());
        }
        Ok(match resp.result {
            None | Some(serde_json::Value::Null) => None,
            Some(v) => Some(v),
        })
    }

    /// Send a request; a JSON `null` result is returned as `None`.
    async fn call<T: DeserializeOwned>(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<Option<T>> {
        match self.raw(method, params).await? {
            None => Ok(None),
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

    /// Like `call_required`, but a node that does not serve the method answers `None`
    /// (phase S2 methods on an S1/S3 node).
    async fn call_optional_method<T: DeserializeOwned>(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<Option<T>> {
        match self.call(method, params).await {
            Ok(v) => Ok(v),
            Err(e) => match e.downcast_ref::<RpcFailure>() {
                Some(f) if f.code == METHOD_NOT_FOUND => Ok(None),
                _ => Err(e),
            },
        }
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

    /// The validator register (every entry, active or not).
    pub async fn validators(&self) -> Result<Vec<RpcValidator>> {
        self.call_required("shrugg_getValidators", serde_json::json!([]))
            .await
    }

    /// Phase S2: the epoch schedule. `None` on a node without the method.
    pub async fn epoch(&self) -> Result<Option<RpcEpoch>> {
        self.call_optional_method("shrugg_getEpoch", serde_json::json!([]))
            .await
    }

    /// Phase S2: the supply audit. `None` on a node without the method.
    pub async fn supply(&self) -> Result<Option<randscan_core::Supply>> {
        self.call_optional_method("shrugg_getSupply", serde_json::json!([]))
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

    /// A page of commitment-tree leaves from `from_index`, at most 1000 rows.
    pub async fn commitments(&self, from_index: u64, limit: u64) -> Result<Vec<RpcCommitment>> {
        self.call_required(
            "shrugg_getCommitments",
            serde_json::json!([from_index, limit]),
        )
        .await
    }

    pub async fn tree_info(&self) -> Result<RpcTreeInfo> {
        self.call_required("shrugg_getTreeInfo", serde_json::json!([]))
            .await
    }

    pub async fn bridge_state(&self) -> Result<randscan_core::BridgeState> {
        self.call_required("shrugg_getBridgeState", serde_json::json!([]))
            .await
    }

    pub async fn is_connected(&self) -> bool {
        self.head().await.is_ok()
    }
}

/// An error the node returned for a request.
#[derive(Debug, thiserror::Error)]
#[error("rpc {method} failed: {message} ({code})")]
pub struct RpcFailure {
    pub method: String,
    pub code: i64,
    pub message: String,
}

/// An amount of units as the node serialises it: a JSON integer in a transaction body, a decimal
/// string in the register and the supply audit. Both are accepted; the value is kept as a
/// decimal string (a `u128` stake does not fit an `f64`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Units(pub String);

impl<'de> Deserialize<'de> for Units {
    fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        let v = serde_json::Value::deserialize(d)?;
        match v {
            serde_json::Value::Number(n) => Ok(Units(n.to_string())),
            serde_json::Value::String(s) => {
                if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
                    return Err(serde::de::Error::custom(format!("not an amount: {s:?}")));
                }
                Ok(Units(s))
            }
            other => Err(serde::de::Error::custom(format!("not an amount: {other}"))),
        }
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
    /// Phase S2: whether this node's key is in the set running the current epoch.
    #[serde(default)]
    pub active_validator: Option<bool>,
    #[serde(default)]
    pub faucet: bool,
    #[serde(default)]
    pub confidential: bool,
    #[serde(default)]
    pub fri_profile: String,
    #[serde(default)]
    pub programs: u64,
    /// Leaves in the commitment tree.
    #[serde(default)]
    pub notes: u64,
    /// Nullifiers published.
    #[serde(default)]
    pub nullifiers: u64,
    #[serde(default)]
    pub tree_root: String,
    #[serde(default)]
    pub hc_bundle: String,
    #[serde(default)]
    pub address: Option<String>,
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

/// A transaction as the node serialises it inside a block (`shrugg_getTransaction.tx`).
#[derive(Debug, Clone, Deserialize)]
pub struct RpcTx {
    pub hash: String,
    pub chain_id: u64,
    /// `None` for a validator-signed action (mint, unbond, withdraw).
    #[serde(default)]
    pub bundle: Option<RpcBundle>,
    pub action: RpcAction,
}

/// The public fields of a bundle. The proof and the envelopes come by length only.
#[derive(Debug, Clone, Deserialize)]
pub struct RpcBundle {
    pub anchor: String,
    pub nullifiers: [String; 2],
    pub commitments: [String; 2],
    pub fee: Units,
    #[serde(default)]
    pub burn: Units,
    #[serde(default)]
    pub asset: u32,
    #[serde(default)]
    pub time: u64,
    #[serde(default)]
    pub proof_len: u64,
    #[serde(default)]
    pub envelope_len: [u64; 2],
}

/// Actions as the node serialises them. Kinds this client does not know become
/// [`RpcAction::Unknown`] rather than a parse error, so a node that is newer than the explorer
/// never stalls indexing on an unfamiliar block.
#[derive(Debug, Clone)]
pub enum RpcAction {
    /// A plain shielded transfer.
    None,
    Mint {
        cm: String,
        amount: Units,
        minter: String,
    },
    Deploy {
        program: String,
        words: u64,
    },
    Call {
        program: String,
        proof_len: u64,
        input_envelope_len: Option<u64>,
    },
    Bond {
        validator: String,
        amount: Units,
        registered: bool,
    },
    Unbond {
        validator: String,
        amount: Units,
        nonce: u64,
    },
    Withdraw {
        validator: String,
        amount: Units,
        nonce: u64,
    },
    BridgeAttest {
        attestation_len: u64,
        recipient: String,
        asset_index: Option<u32>,
        amount: Option<Units>,
        time: Option<u64>,
    },
    BridgeBurn {
        asset: u32,
        amount: Units,
        relayer_fee: Units,
        to_chain: u16,
        to: String,
        asset_bundle: Box<RpcBundle>,
    },
    /// A kind this build does not decode; `kind` is the node's tag.
    Unknown {
        kind: String,
    },
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum KnownAction {
    None,
    Mint {
        cm: String,
        amount: Units,
        minter: String,
    },
    Deploy {
        program: String,
        words: u64,
    },
    Call {
        program: String,
        proof_len: u64,
        #[serde(default)]
        input_envelope_len: Option<u64>,
    },
    Bond {
        validator: String,
        amount: Units,
        #[serde(default)]
        registered: bool,
    },
    Unbond {
        validator: String,
        amount: Units,
        #[serde(default)]
        nonce: u64,
    },
    Withdraw {
        validator: String,
        amount: Units,
        #[serde(default)]
        nonce: u64,
    },
    BridgeAttest {
        attestation_len: u64,
        recipient: String,
        #[serde(default)]
        asset_index: Option<u32>,
        #[serde(default)]
        amount: Option<Units>,
        #[serde(default)]
        time: Option<u64>,
    },
    BridgeBurn {
        asset: u32,
        amount: Units,
        #[serde(default)]
        relayer_fee: Units,
        to_chain: u16,
        to: String,
        asset_bundle: Box<RpcBundle>,
    },
}

const KNOWN_TAGS: &[&str] = &[
    "none",
    "mint",
    "deploy",
    "call",
    "bond",
    "unbond",
    "withdraw",
    "bridge_attest",
    "bridge_burn",
];

impl<'de> Deserialize<'de> for RpcAction {
    fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        let value = serde_json::Value::deserialize(d)?;
        let tag = value
            .get("kind")
            .and_then(|t| t.as_str())
            .ok_or_else(|| serde::de::Error::missing_field("kind"))?
            .to_string();
        match serde_json::from_value::<KnownAction>(value) {
            Ok(k) => Ok(match k {
                KnownAction::None => RpcAction::None,
                KnownAction::Mint { cm, amount, minter } => RpcAction::Mint { cm, amount, minter },
                KnownAction::Deploy { program, words } => RpcAction::Deploy { program, words },
                KnownAction::Call {
                    program,
                    proof_len,
                    input_envelope_len,
                } => RpcAction::Call {
                    program,
                    proof_len,
                    input_envelope_len,
                },
                KnownAction::Bond {
                    validator,
                    amount,
                    registered,
                } => RpcAction::Bond {
                    validator,
                    amount,
                    registered,
                },
                KnownAction::Unbond {
                    validator,
                    amount,
                    nonce,
                } => RpcAction::Unbond {
                    validator,
                    amount,
                    nonce,
                },
                KnownAction::Withdraw {
                    validator,
                    amount,
                    nonce,
                } => RpcAction::Withdraw {
                    validator,
                    amount,
                    nonce,
                },
                KnownAction::BridgeAttest {
                    attestation_len,
                    recipient,
                    asset_index,
                    amount,
                    time,
                } => RpcAction::BridgeAttest {
                    attestation_len,
                    recipient,
                    asset_index,
                    amount,
                    time,
                },
                KnownAction::BridgeBurn {
                    asset,
                    amount,
                    relayer_fee,
                    to_chain,
                    to,
                    asset_bundle,
                } => RpcAction::BridgeBurn {
                    asset,
                    amount,
                    relayer_fee,
                    to_chain,
                    to,
                    asset_bundle,
                },
            }),
            // A known tag with a malformed body is a real error; an unknown tag is tolerated.
            Err(e) => {
                if KNOWN_TAGS.contains(&tag.as_str()) {
                    Err(serde::de::Error::custom(e))
                } else {
                    Ok(RpcAction::Unknown { kind: tag })
                }
            }
        }
    }
}

/// One entry of the validator register. The S3 branch serves `{ address, stake, rewards }`; S2
/// adds the unbonding queue, payout address, nonce and the active flag.
#[derive(Debug, Clone, Deserialize)]
pub struct RpcValidator {
    pub address: String,
    pub stake: Units,
    #[serde(default)]
    pub rewards: Units,
    #[serde(default)]
    pub pending: Vec<RpcPending>,
    #[serde(default)]
    pub payout: Option<String>,
    #[serde(default)]
    pub nonce: u64,
    /// Absent before S2, where every register entry is in the set.
    #[serde(default)]
    pub active: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RpcPending {
    pub release_epoch: u64,
    pub amount: Units,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RpcEpoch {
    pub epoch: u64,
    pub epoch_blocks: u64,
    #[serde(default)]
    pub next_set: Vec<String>,
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
    pub height: u64,
    pub index: u32,
    #[serde(default)]
    pub h_in: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RpcProgram {
    pub id: String,
    pub base_pc: u32,
    pub words_len: u32,
    pub code_hash: String,
    pub deployed_at: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RpcCommitment {
    pub index: u64,
    pub cm: String,
    pub height: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RpcTreeInfo {
    pub next_index: u64,
    pub root: String,
    pub nullifiers: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bundle_json() -> serde_json::Value {
        serde_json::json!({
            "anchor": "6b1d", "nullifiers": ["8c04", "5e77"], "commitments": ["2a9f", "b310"],
            "fee": 1000000, "burn": 0, "asset": 0, "time": 5, "proof_len": 302857, "envelope_len": [1348, 1348]
        })
    }

    #[test]
    fn parses_a_block_with_every_kind() {
        let b = bundle_json();
        let json = serde_json::json!({
            "hash": "a9c8", "height": 10, "view": 32, "parent": "a070", "proposer": "2nRd", "timestamp_ms": 1,
            "tx_root": "dc97", "state_root": "b364", "justify_view": 31, "tx_count": 9,
            "transactions": [
                { "hash": "t0", "chain_id": 7, "bundle": b, "action": { "kind": "none" } },
                { "hash": "t1", "chain_id": 7, "bundle": null, "action": { "kind": "mint", "cm": "2a9f", "amount": 100000000000u64, "minter": "2nRd" } },
                { "hash": "t2", "chain_id": 7, "bundle": b, "action": { "kind": "deploy", "program": "675a", "words": 412 } },
                { "hash": "t3", "chain_id": 7, "bundle": b, "action": { "kind": "call", "program": "675a", "proof_len": 268123, "input_envelope_len": 1280 } },
                { "hash": "t4", "chain_id": 7, "bundle": b, "action": { "kind": "bond", "validator": "2nRd", "amount": 500, "registered": false } },
                { "hash": "t5", "chain_id": 7, "bundle": null, "action": { "kind": "unbond", "validator": "2nRd", "amount": 7, "nonce": 2 } },
                { "hash": "t6", "chain_id": 7, "bundle": null, "action": { "kind": "withdraw", "validator": "2nRd", "amount": 9, "nonce": 3 } },
                { "hash": "t7", "chain_id": 7, "bundle": b, "action": { "kind": "bridge_attest", "attestation_len": 520, "recipient": "shrugg1abc", "asset_index": 1, "amount": 1000, "time": 41 } },
                { "hash": "t8", "chain_id": 7, "bundle": b, "action": { "kind": "bridge_burn", "asset": 2, "amount": 400, "relayer_fee": 100, "to_chain": 5, "to": "abab", "asset_bundle": b } }
            ]
        });
        let b: RpcBlock = serde_json::from_value(json).unwrap();
        assert_eq!(b.transactions.len(), 9);
        assert!(matches!(b.transactions[0].action, RpcAction::None));
        assert!(b.transactions[1].bundle.is_none());
        match &b.transactions[1].action {
            RpcAction::Mint { amount, .. } => assert_eq!(amount.0, "100000000000"),
            other => panic!("{other:?}"),
        }
        assert!(matches!(
            &b.transactions[3].action,
            RpcAction::Call {
                input_envelope_len: Some(1280),
                ..
            }
        ));
        assert!(matches!(
            &b.transactions[6].action,
            RpcAction::Withdraw { nonce: 3, .. }
        ));
        match &b.transactions[7].action {
            RpcAction::BridgeAttest {
                asset_index,
                amount,
                time,
                ..
            } => {
                assert_eq!(*asset_index, Some(1));
                assert_eq!(amount.as_ref().unwrap().0, "1000");
                assert_eq!(*time, Some(41));
            }
            other => panic!("{other:?}"),
        }
        match &b.transactions[8].action {
            RpcAction::BridgeBurn {
                asset_bundle,
                relayer_fee,
                ..
            } => {
                assert_eq!(asset_bundle.nullifiers[1], "5e77");
                assert_eq!(relayer_fee.0, "100");
            }
            other => panic!("{other:?}"),
        }
        assert_eq!(b.transactions[0].bundle.as_ref().unwrap().fee.0, "1000000");
    }

    #[test]
    fn a_rotation_attestation_has_no_deposit() {
        let json = r#"{"kind":"bridge_attest","attestation_len":700,"recipient":"shrugg1x","asset_index":null,"amount":null,"time":3}"#;
        let a: RpcAction = serde_json::from_str(json).unwrap();
        assert!(matches!(
            a,
            RpcAction::BridgeAttest {
                asset_index: None,
                amount: None,
                ..
            }
        ));
    }

    #[test]
    fn unknown_kind_does_not_fail_the_block() {
        let json = serde_json::json!({ "hash": "t", "chain_id": 7, "bundle": bundle_json(), "action": { "kind": "slash", "evidence": "…" } });
        let t: RpcTx = serde_json::from_value(json).unwrap();
        assert!(matches!(&t.action, RpcAction::Unknown { kind } if kind == "slash"));
    }

    #[test]
    fn malformed_known_kind_is_an_error() {
        assert!(serde_json::from_str::<RpcAction>(r#"{"kind":"mint","cm":"2a9f"}"#).is_err());
        assert!(serde_json::from_str::<RpcAction>(r#"{"cm":"2a9f"}"#).is_err());
    }

    #[test]
    fn amounts_accept_numbers_and_strings() {
        assert_eq!(serde_json::from_str::<Units>("12").unwrap().0, "12");
        assert_eq!(
            serde_json::from_str::<Units>(r#""340282366920938463463374607431768211455""#)
                .unwrap()
                .0,
            "340282366920938463463374607431768211455"
        );
        assert!(serde_json::from_str::<Units>(r#""1.5""#).is_err());
        assert!(serde_json::from_str::<Units>("true").is_err());
    }

    #[test]
    fn parses_both_register_shapes() {
        let s3: Vec<RpcValidator> =
            serde_json::from_str(r#"[{"address":"2nRd","stake":"100000","rewards":4000000}]"#)
                .unwrap();
        assert_eq!(s3[0].rewards.0, "4000000");
        assert_eq!(s3[0].active, None);
        assert!(s3[0].pending.is_empty());
        let s2: Vec<RpcValidator> = serde_json::from_str(r#"[{"address":"2nRd","stake":"1000000000000","pending":[{"release_epoch":41,"amount":"5000000000"}],"rewards":"4000000","payout":"shrugg1x","nonce":3,"active":true}]"#).unwrap();
        assert_eq!(s2[0].pending[0].release_epoch, 41);
        assert_eq!(s2[0].payout.as_deref(), Some("shrugg1x"));
        assert_eq!(s2[0].active, Some(true));
    }

    #[test]
    fn parses_receipt_and_status() {
        let r: RpcReceipt = serde_json::from_str(r#"{"tx":"d4c7","program":"675a","tier":14,"outputs":[1,0,25,0,0,0,0,0],"height":17,"index":0,"h_in":"9c0e"}"#).unwrap();
        assert_eq!(r.h_in, "9c0e");
        let s: NodeStatus = serde_json::from_str(r#"{"height":1998,"head_hash":"x","view":2251,"high_qc_view":2250,"syncing":false,"sync_target":1998,"peer_count":5,"mempool_size":0,"is_validator":true,"faucet":true,"confidential":true,"fri_profile":"production","programs":2,"notes":41,"nullifiers":12,"tree_root":"6b1d","hc_bundle":"f07a","address":null,"peer_id":"12D3"}"#).unwrap();
        assert_eq!(s.notes, 41);
        assert_eq!(s.address, None);
        assert_eq!(s.active_validator, None);
    }
}
