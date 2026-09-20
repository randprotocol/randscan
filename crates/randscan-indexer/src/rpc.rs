//! JSON-RPC client for `rand-node` on the shielded chain (see fullnode/docs/rpc.md).

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
        self.call_required("rand_chainId", serde_json::json!([]))
            .await
    }

    pub async fn token_info(&self) -> Result<TokenInfo> {
        self.call_required("rand_tokenInfo", serde_json::json!([]))
            .await
    }

    pub async fn head(&self) -> Result<Head> {
        self.call_required("rand_getHead", serde_json::json!([]))
            .await
    }

    pub async fn status(&self) -> Result<NodeStatus> {
        self.call_required("rand_status", serde_json::json!([]))
            .await
    }

    pub async fn block_by_height(&self, height: u64) -> Result<Option<RpcBlock>> {
        self.call("rand_getBlockByHeight", serde_json::json!([height]))
            .await
    }

    pub async fn block_by_hash(&self, hash: &str) -> Result<Option<RpcBlock>> {
        self.call("rand_getBlockByHash", serde_json::json!([hash]))
            .await
    }

    /// The validator register (every entry, active or not).
    pub async fn validators(&self) -> Result<Vec<RpcValidator>> {
        self.call_required("rand_getValidators", serde_json::json!([]))
            .await
    }

    /// Phase S2: the epoch schedule. `None` on a node without the method.
    pub async fn epoch(&self) -> Result<Option<RpcEpoch>> {
        self.call_optional_method("rand_getEpoch", serde_json::json!([]))
            .await
    }

    /// Phase S2: the supply audit. `None` on a node without the method.
    pub async fn supply(&self) -> Result<Option<randscan_core::Supply>> {
        self.call_optional_method("rand_getSupply", serde_json::json!([]))
            .await
    }

    pub async fn peers(&self) -> Result<Vec<RpcPeer>> {
        self.call_required("rand_getPeers", serde_json::json!([]))
            .await
    }

    pub async fn receipt(&self, tx_hash: &str) -> Result<Option<RpcReceipt>> {
        self.call("rand_getReceipt", serde_json::json!([tx_hash]))
            .await
    }

    pub async fn program(&self, id: &str) -> Result<Option<RpcProgram>> {
        self.call("rand_getProgram", serde_json::json!([id]))
            .await
    }

    /// A page of commitment-tree leaves from `from_index`, at most 1000 rows.
    pub async fn commitments(&self, from_index: u64, limit: u64) -> Result<Vec<RpcCommitment>> {
        self.call_required(
            "rand_getCommitments",
            serde_json::json!([from_index, limit]),
        )
        .await
    }

    pub async fn tree_info(&self) -> Result<RpcTreeInfo> {
        self.call_required("rand_getTreeInfo", serde_json::json!([]))
            .await
    }

    /// A call's sealed input transcript (`null` when the call published none).
    pub async fn call_envelope(&self, tx_hash: &str) -> Result<Option<serde_json::Value>> {
        self.call("rand_getCallEnvelope", serde_json::json!([tx_hash]))
            .await
    }

    pub async fn bridge_state(&self) -> Result<randscan_core::BridgeState> {
        self.call_required("rand_getBridgeState", serde_json::json!([]))
            .await
    }

    /// One page of the RPL token registry, `[from_index, limit]`. `None` on a node without the
    /// method (a chain older than the token RPC task).
    pub async fn tokens_page(&self, from_index: u64, limit: u64) -> Result<Option<randscan_core::TokenList>> {
        self.call_optional_method("rand_getTokens", serde_json::json!([from_index, limit]))
            .await
    }

    /// The whole RPL token registry, paged until a short page ends it. `enabled: false, tokens:
    /// []` on a chain without one (or a node predating the method) — indistinguishable, which is
    /// the right answer for a caller that only wants to know what to show.
    pub async fn tokens_all(&self) -> Result<randscan_core::TokenList> {
        const PAGE: u64 = 1000;
        let mut out = randscan_core::TokenList::default();
        let mut from = 0u64;
        loop {
            let Some(page) = self.tokens_page(from, PAGE).await? else {
                return Ok(out);
            };
            out.enabled = page.enabled;
            out.registration_fee = page.registration_fee;
            out.next_index = page.next_index;
            let got = page.tokens.len() as u64;
            out.tokens.extend(page.tokens);
            if got < PAGE {
                return Ok(out);
            }
            from += got;
        }
    }

    /// One token by registry index, 64-hex id or `rpl1…` text form. `None` when there is no such
    /// token (or on a chain without any). PRIVACY: a per-token lookup tells the node which token
    /// the caller cares about — used for a token detail page read, never for resolving a wallet's
    /// send (`tokens_all` instead).
    pub async fn token(&self, key: &str) -> Result<Option<randscan_core::TokenInfo>> {
        self.call_optional_method("rand_getToken", serde_json::json!([key]))
            .await
    }

    /// A token's supply and each backing's locked amount. Same key forms and privacy note as
    /// [`token`](Self::token).
    pub async fn token_supply(&self, key: &str) -> Result<Option<randscan_core::TokenSupply>> {
        self.call_optional_method("rand_getTokenSupply", serde_json::json!([key]))
            .await
    }

    /// The chain's call limits, from its genesis (v0.4). `None` on a node without the method.
    pub async fn limits(&self) -> Result<Option<randscan_core::ChainLimits>> {
        self.call_optional_method("rand_getLimits", serde_json::json!([]))
            .await
    }

    /// The node's build (v0.3). `None` on a node without the method.
    pub async fn version(&self) -> Result<Option<RpcVersion>> {
        self.call_optional_method("rand_getVersion", serde_json::json!([]))
            .await
    }

    /// The genesis hash, hex (v0.3). `None` on a node without the method.
    pub async fn genesis_hash(&self) -> Result<Option<String>> {
        self.call_optional_method("rand_getGenesisHash", serde_json::json!([]))
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

/// A transaction as the node serialises it inside a block (`rand_getTransaction.tx`).
#[derive(Debug, Clone, Deserialize)]
pub struct RpcTx {
    pub hash: String,
    pub chain_id: u64,
    /// `None` for a validator-signed action (mint, unbond, withdraw).
    #[serde(default)]
    pub bundle: Option<RpcBundle>,
    pub action: RpcAction,
}

/// The public fields of the chain-14 hidden-asset bundle: four input and four output slots
/// (dummies included), and no `asset` field at all — slots 0-1 carry a private asset, slots 2-3
/// always RAND, and nothing public says which asset slots 0-1 moved. The proof and the envelopes
/// come by length only.
#[derive(Debug, Clone, Deserialize)]
pub struct RpcBundle {
    pub anchor: String,
    pub nullifiers: [String; 4],
    pub commitments: [String; 4],
    pub fee: Units,
    /// The private asset burned (a `token_burn` or a `bridge_burn`), units of that asset.
    #[serde(default)]
    pub burn_a: Units,
    /// RAND burned (a `bond` or a `register_aggregator`).
    #[serde(default)]
    pub burn_r: Units,
    /// 0 when nothing was burned; otherwise the registry index `burn_a` names.
    #[serde(default)]
    pub burn_asset: u32,
    #[serde(default)]
    pub time: u64,
    #[serde(default)]
    pub proof_len: u64,
    #[serde(default)]
    pub envelope_len: [u64; 4],
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
        /// The deploy-time public input's length (0 without one, and on a pre-v0.4 node).
        public_words_len: u64,
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
        /// The deposit note's `time` word, chosen by the withdrawing node (chain 8 onward).
        time: Option<u64>,
    },
    BridgeAttest {
        attestation_len: u64,
        recipient: String,
        /// The index the *action* names (never `null` on a committed attest); `asset_index`
        /// (the registry's own resolution) is `null` on a rotation, which deposits nothing.
        asset: Option<u32>,
        asset_index: Option<u32>,
        amount: Option<Units>,
        time: Option<u64>,
        /// The deposit note's blinding — public, a field of the action.
        r: Option<String>,
        /// The leaf the chain appended for the deposit; `null` for a rotation.
        commitment: Option<String>,
        /// B3: the PQ guardians who co-signed, by index.
        pq_signers: Vec<i64>,
    },
    BridgeBurn {
        asset: u32,
        amount: Units,
        relayer_fee: Units,
        to_chain: u16,
        /// The backing being redeemed: a source-chain token address, 32 bytes hex.
        token: String,
        to: String,
    },
    /// RPL (spec §4/§6): a token's registration and mints are public by design, as a bridge
    /// deposit is — only a later *transfer* of the token's notes is shielded.
    /// RPL (spec §4/§6): a token's registration and mints are public by design, as a bridge
    /// deposit is — only a later *transfer* of the token's notes is shielded.
    RegisterToken {
        name: String,
        symbol: String,
        decimals: u8,
        /// `"none"` | `"key"` | `"bridge"` | `"program"`.
        authority: String,
        index: u32,
        initial_amount: Option<Units>,
        initial: Option<RpcInitialMint>,
    },
    TokenMint {
        asset: u32,
        amount: Units,
        recipient: String,
        time: u64,
        r: String,
        nonce: u64,
    },
    SetAuthority {
        asset: u32,
        nonce: u64,
        new_authority: Option<String>,
    },
    /// A holder burn: public by design (it audits `total_supply`); a transfer of the same token
    /// is a plain `none` bundle, its asset private.
    TokenBurn {
        asset: u32,
        amount: Units,
    },
    /// Bridge hardening B1: the genesis pause key's own signature, no PQ quorum.
    PauseMints { nonce: u64 },
    /// Bridge hardening B1: lifting the pause needs the PQ guardian quorum.
    UnpauseMints { nonce: u64, pq_signers: Vec<i64> },
    /// Bridge hardening B4: a new `Bridge`-authority token, registered after genesis by the PQ
    /// guardian quorum.
    RegisterBridgedToken {
        name: String,
        symbol: String,
        salt: String,
        chain: i32,
        token: String,
        decimals: u8,
        nonce: u64,
        asset_id: String,
        pq_signers: Vec<i64>,
    },
    /// Bridge hardening B4: another backing added to an already-listed bridged token.
    ListBacking {
        token_index: u32,
        chain: i32,
        token: String,
        decimals: u8,
        nonce: u64,
        pq_signers: Vec<i64>,
    },
    /// A kind this build does not decode; `kind` is the node's tag.
    Unknown {
        kind: String,
    },
}

/// `register_token`'s optional `initial` mint: every word of the note the chain computes for it.
#[derive(Debug, Clone, Deserialize)]
pub struct RpcInitialMint {
    pub amount: Units,
    pub recipient: String,
    pub time: u64,
    pub r: String,
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
        #[serde(default)]
        public_words_len: u64,
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
        #[serde(default)]
        time: Option<u64>,
    },
    BridgeAttest {
        attestation_len: u64,
        recipient: String,
        #[serde(default)]
        asset: Option<u32>,
        #[serde(default)]
        asset_index: Option<u32>,
        #[serde(default)]
        amount: Option<Units>,
        #[serde(default)]
        time: Option<u64>,
        #[serde(default)]
        r: Option<String>,
        #[serde(default)]
        commitment: Option<String>,
        #[serde(default)]
        pq_signers: Vec<i64>,
    },
    BridgeBurn {
        asset: u32,
        amount: Units,
        #[serde(default)]
        relayer_fee: Units,
        to_chain: u16,
        #[serde(default)]
        token: String,
        to: String,
    },
    RegisterToken {
        name: String,
        symbol: String,
        decimals: u8,
        authority: String,
        index: u32,
        #[serde(default)]
        initial_amount: Option<Units>,
        #[serde(default)]
        initial: Option<RpcInitialMint>,
    },
    TokenMint {
        asset: u32,
        amount: Units,
        recipient: String,
        time: u64,
        r: String,
        #[serde(default)]
        nonce: u64,
    },
    SetAuthority {
        asset: u32,
        #[serde(default)]
        nonce: u64,
        #[serde(default)]
        new_authority: Option<String>,
    },
    TokenBurn {
        asset: u32,
        amount: Units,
    },
    PauseMints {
        nonce: u64,
    },
    UnpauseMints {
        nonce: u64,
        #[serde(default)]
        pq_signers: Vec<i64>,
    },
    RegisterBridgedToken {
        name: String,
        symbol: String,
        salt: String,
        chain: i32,
        token: String,
        decimals: u8,
        nonce: u64,
        asset_id: String,
        #[serde(default)]
        pq_signers: Vec<i64>,
    },
    ListBacking {
        token_index: u32,
        chain: i32,
        token: String,
        decimals: u8,
        nonce: u64,
        #[serde(default)]
        pq_signers: Vec<i64>,
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
    "register_token",
    "token_mint",
    "set_authority",
    "token_burn",
    "pause_mints",
    "unpause_mints",
    "register_bridged_token",
    "list_backing",
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
                KnownAction::Deploy {
                    program,
                    words,
                    public_words_len,
                } => RpcAction::Deploy {
                    program,
                    words,
                    public_words_len,
                },
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
                    time,
                } => RpcAction::Withdraw {
                    validator,
                    amount,
                    nonce,
                    time,
                },
                KnownAction::BridgeAttest {
                    attestation_len,
                    recipient,
                    asset,
                    asset_index,
                    amount,
                    time,
                    r,
                    commitment,
                    pq_signers,
                } => RpcAction::BridgeAttest {
                    attestation_len,
                    recipient,
                    asset,
                    asset_index,
                    amount,
                    time,
                    r,
                    commitment,
                    pq_signers,
                },
                KnownAction::BridgeBurn {
                    asset,
                    amount,
                    relayer_fee,
                    to_chain,
                    token,
                    to,
                } => RpcAction::BridgeBurn {
                    asset,
                    amount,
                    relayer_fee,
                    to_chain,
                    token,
                    to,
                },
                KnownAction::RegisterToken {
                    name,
                    symbol,
                    decimals,
                    authority,
                    index,
                    initial_amount,
                    initial,
                } => RpcAction::RegisterToken {
                    name,
                    symbol,
                    decimals,
                    authority,
                    index,
                    initial_amount,
                    initial,
                },
                KnownAction::TokenMint { asset, amount, recipient, time, r, nonce } => {
                    RpcAction::TokenMint { asset, amount, recipient, time, r, nonce }
                }
                KnownAction::SetAuthority { asset, nonce, new_authority } => {
                    RpcAction::SetAuthority { asset, nonce, new_authority }
                }
                KnownAction::TokenBurn { asset, amount } => RpcAction::TokenBurn { asset, amount },
                KnownAction::PauseMints { nonce } => RpcAction::PauseMints { nonce },
                KnownAction::UnpauseMints { nonce, pq_signers } => {
                    RpcAction::UnpauseMints { nonce, pq_signers }
                }
                KnownAction::RegisterBridgedToken {
                    name,
                    symbol,
                    salt,
                    chain,
                    token,
                    decimals,
                    nonce,
                    asset_id,
                    pq_signers,
                } => RpcAction::RegisterBridgedToken {
                    name,
                    symbol,
                    salt,
                    chain,
                    token,
                    decimals,
                    nonce,
                    asset_id,
                    pq_signers,
                },
                KnownAction::ListBacking { token_index, chain, token, decimals, nonce, pq_signers } => {
                    RpcAction::ListBacking { token_index, chain, token, decimals, nonce, pq_signers }
                }
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
    /// The program's deploy-time public digest the proof was checked against; `None` when that
    /// was the digest of the empty public input (and on a pre-v0.4 node).
    #[serde(default)]
    pub h_pub: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RpcProgram {
    pub id: String,
    pub base_pc: u32,
    pub words_len: u32,
    pub code_hash: String,
    pub deployed_at: u64,
    #[serde(default)]
    pub public_words_len: u64,
    /// `None` for a program deployed without a public input.
    #[serde(default)]
    pub public_digest: Option<String>,
}

/// `rand_getVersion`: which build the node runs.
#[derive(Debug, Clone, Deserialize)]
pub struct RpcVersion {
    pub version: String,
    #[serde(default)]
    pub git_sha: String,
    #[serde(default)]
    pub fri_profile: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RpcCommitment {
    pub index: u64,
    pub cm: String,
    pub height: u64,
    /// `{ kem_ct, to_receiver, to_sender, body }`, hex; public, only a key opens it.
    #[serde(default)]
    pub envelope: Option<serde_json::Value>,
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

    #[test]
    fn parses_the_public_input_fields() {
        // A deploy without the field (a pre-v0.4 node) is a program without a public input.
        let old: RpcAction =
            serde_json::from_str(r#"{"kind":"deploy","program":"675a","words":412}"#).unwrap();
        assert!(matches!(
            old,
            RpcAction::Deploy {
                public_words_len: 0,
                ..
            }
        ));
        let p: RpcProgram = serde_json::from_str(r#"{"id":"7402","base_pc":0,"words_len":9100,"code_hash":"ab12","deployed_at":40,"public_words_len":27151,"public_digest":"5d20"}"#).unwrap();
        assert_eq!(p.public_words_len, 27151);
        assert_eq!(p.public_digest.as_deref(), Some("5d20"));
        let bare: RpcProgram = serde_json::from_str(r#"{"id":"675a","base_pc":0,"words_len":412,"code_hash":"ab12","deployed_at":17,"public_words_len":0,"public_digest":null}"#).unwrap();
        assert_eq!((bare.public_words_len, bare.public_digest), (0, None));
        let r: RpcReceipt = serde_json::from_str(r#"{"tx":"d4c7","program":"7402","tier":16,"outputs":[1,0,0,0,0,0,0,0],"height":41,"index":0,"h_in":"9c0e","h_pub":"5d20"}"#).unwrap();
        assert_eq!(r.h_pub.as_deref(), Some("5d20"));
    }

    #[test]
    fn parses_the_live_chain_14_version_and_limits() {
        let v: RpcVersion = serde_json::from_str(r#"{"chain_id":14,"fri_profile":"production","git_sha":"b3c594cd5872bbc132cfafcd7314614436e6131a","hc_bundle":"83d3","version":"0.1.0"}"#).unwrap();
        assert_eq!(v.version, "0.1.0");
        assert_eq!(v.fri_profile, "production");
        assert!(v.git_sha.starts_with("b3c594c"));
        let l: randscan_core::ChainLimits = serde_json::from_str(r#"{"max_block_bytes":20971520,"max_call_envelope_bytes":65536,"max_program_public_words":32768,"max_program_words":65535,"max_proof_bytes":8388608}"#).unwrap();
        assert_eq!(l.max_proof_bytes, 8 << 20);
        assert_eq!(l.max_block_bytes, 20 << 20);
        assert_eq!(l.max_program_public_words, 32768);
    }

    fn bundle_json() -> serde_json::Value {
        serde_json::json!({
            "anchor": "6b1d", "nullifiers": ["8c04", "5e77", "03aa", "e19b"],
            "commitments": ["2a9f", "b310", "77c1", "5d20"],
            "fee": 1000000, "burn_a": 0, "burn_r": 0, "burn_asset": 0, "time": 5, "proof_len": 302857,
            "envelope_len": [1348, 1348, 1348, 1348]
        })
    }

    #[test]
    fn parses_a_block_with_every_kind() {
        let b = bundle_json();
        let json = serde_json::json!({
            "hash": "a9c8", "height": 10, "view": 32, "parent": "a070", "proposer": "2nRd", "timestamp_ms": 1,
            "tx_root": "dc97", "state_root": "b364", "justify_view": 31, "tx_count": 17,
            "transactions": [
                { "hash": "t0", "chain_id": 14, "bundle": b, "action": { "kind": "none" } },
                { "hash": "t1", "chain_id": 14, "bundle": null, "action": { "kind": "mint", "cm": "2a9f", "amount": 100000000000u64, "minter": "2nRd" } },
                { "hash": "t2", "chain_id": 14, "bundle": b, "action": { "kind": "deploy", "program": "675a", "words": 412, "public_words_len": 27151 } },
                { "hash": "t3", "chain_id": 14, "bundle": b, "action": { "kind": "call", "program": "675a", "proof_len": 268123, "input_envelope_len": 1280 } },
                { "hash": "t4", "chain_id": 14, "bundle": b, "action": { "kind": "bond", "validator": "2nRd", "amount": 500, "registered": false } },
                { "hash": "t5", "chain_id": 14, "bundle": null, "action": { "kind": "unbond", "validator": "2nRd", "amount": 7, "nonce": 2 } },
                { "hash": "t6", "chain_id": 14, "bundle": null, "action": { "kind": "withdraw", "validator": "2nRd", "amount": "9", "nonce": 3, "time": 1994 } },
                { "hash": "t7", "chain_id": 14, "bundle": b, "action": { "kind": "bridge_attest", "attestation_len": 520, "recipient": "rand1abc", "asset": 1, "asset_index": 1, "amount": 1000, "time": 41, "r": "aa", "commitment": "cc", "pq_signers": [0, 1] } },
                { "hash": "t8", "chain_id": 14, "bundle": b, "action": { "kind": "bridge_burn", "asset": 2, "amount": 400, "relayer_fee": 100, "to_chain": 5, "token": "cdcd", "to": "abab" } },
                { "hash": "t9", "chain_id": 14, "bundle": b, "action": { "kind": "register_token", "name": "zUSD", "symbol": "zUSD", "decimals": 6, "authority": "bridge", "index": 3, "initial_amount": 5000, "initial": { "amount": 5000, "recipient": "rand1abc", "time": 40, "r": "bb" } } },
                { "hash": "t10", "chain_id": 14, "bundle": b, "action": { "kind": "token_mint", "asset": 3, "amount": 700, "recipient": "rand1abc", "time": 41, "r": "cc", "nonce": 0 } },
                { "hash": "t11", "chain_id": 14, "bundle": b, "action": { "kind": "set_authority", "asset": 3, "nonce": 1, "new_authority": "2nRd" } },
                { "hash": "t12", "chain_id": 14, "bundle": b, "action": { "kind": "token_burn", "asset": 3, "amount": 400 } },
                { "hash": "t13", "chain_id": 14, "bundle": null, "action": { "kind": "pause_mints", "nonce": 4 } },
                { "hash": "t14", "chain_id": 14, "bundle": null, "action": { "kind": "unpause_mints", "nonce": 5, "pq_signers": [0, 2] } },
                { "hash": "t15", "chain_id": 14, "bundle": b, "action": { "kind": "register_bridged_token", "name": "zUSD", "symbol": "zUSD", "salt": "ee", "chain": 3, "token": "ff", "decimals": 6, "nonce": 6, "asset_id": "aa", "pq_signers": [1] } },
                { "hash": "t16", "chain_id": 14, "bundle": b, "action": { "kind": "list_backing", "token_index": 3, "chain": 4, "token": "11", "decimals": 6, "nonce": 7, "pq_signers": [2] } }
            ]
        });
        let b: RpcBlock = serde_json::from_value(json).unwrap();
        assert_eq!(b.transactions.len(), 17);
        assert!(matches!(b.transactions[0].action, RpcAction::None));
        assert_eq!(b.transactions[0].bundle.as_ref().unwrap().nullifiers.len(), 4);
        assert_eq!(b.transactions[0].bundle.as_ref().unwrap().commitments.len(), 4);
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
            RpcAction::Withdraw {
                nonce: 3,
                time: Some(1994),
                ..
            }
        ));
        match &b.transactions[7].action {
            RpcAction::BridgeAttest {
                asset_index,
                amount,
                time,
                r,
                commitment,
                pq_signers,
                ..
            } => {
                assert_eq!(*asset_index, Some(1));
                assert_eq!(amount.as_ref().unwrap().0, "1000");
                assert_eq!(*time, Some(41));
                assert_eq!(r.as_deref(), Some("aa"));
                assert_eq!(commitment.as_deref(), Some("cc"));
                assert_eq!(pq_signers, &vec![0, 1]);
            }
            other => panic!("{other:?}"),
        }
        match &b.transactions[8].action {
            RpcAction::BridgeBurn { token, relayer_fee, .. } => {
                assert_eq!(token, "cdcd");
                assert_eq!(relayer_fee.0, "100");
            }
            other => panic!("{other:?}"),
        }
        match &b.transactions[9].action {
            RpcAction::RegisterToken { name, index, initial, .. } => {
                assert_eq!(name, "zUSD");
                assert_eq!(*index, 3);
                assert_eq!(initial.as_ref().unwrap().amount.0, "5000");
            }
            other => panic!("{other:?}"),
        }
        match &b.transactions[10].action {
            RpcAction::TokenMint { asset, amount, recipient, .. } => {
                assert_eq!(*asset, 3);
                assert_eq!(amount.0, "700");
                assert_eq!(recipient, "rand1abc");
            }
            other => panic!("{other:?}"),
        }
        assert!(matches!(&b.transactions[11].action, RpcAction::SetAuthority { asset: 3, nonce: 1, .. }));
        match &b.transactions[12].action {
            RpcAction::TokenBurn { asset, amount } => {
                assert_eq!(*asset, 3);
                assert_eq!(amount.0, "400");
            }
            other => panic!("{other:?}"),
        }
        assert!(matches!(&b.transactions[13].action, RpcAction::PauseMints { nonce: 4 }));
        assert!(b.transactions[13].bundle.is_none());
        match &b.transactions[14].action {
            RpcAction::UnpauseMints { nonce, pq_signers } => {
                assert_eq!(*nonce, 5);
                assert_eq!(pq_signers, &vec![0, 2]);
            }
            other => panic!("{other:?}"),
        }
        match &b.transactions[15].action {
            RpcAction::RegisterBridgedToken { name, chain, pq_signers, .. } => {
                assert_eq!(name, "zUSD");
                assert_eq!(*chain, 3);
                assert_eq!(pq_signers, &vec![1]);
            }
            other => panic!("{other:?}"),
        }
        match &b.transactions[16].action {
            RpcAction::ListBacking { token_index, chain, pq_signers, .. } => {
                assert_eq!(*token_index, 3);
                assert_eq!(*chain, 4);
                assert_eq!(pq_signers, &vec![2]);
            }
            other => panic!("{other:?}"),
        }
        assert_eq!(b.transactions[0].bundle.as_ref().unwrap().fee.0, "1000000");
    }

    #[test]
    fn a_rotation_attestation_has_no_deposit() {
        let json = r#"{"kind":"bridge_attest","attestation_len":700,"recipient":"rand1x","asset":null,"asset_index":null,"amount":null,"time":3,"r":"00","commitment":null}"#;
        let a: RpcAction = serde_json::from_str(json).unwrap();
        assert!(matches!(
            a,
            RpcAction::BridgeAttest {
                asset_index: None,
                amount: None,
                commitment: None,
                ..
            }
        ));
    }

    #[test]
    fn there_is_no_token_transfer_kind_a_transfer_is_none() {
        let json = r#"{"kind":"none"}"#;
        assert!(matches!(serde_json::from_str::<RpcAction>(json).unwrap(), RpcAction::None));
        // A node tag this build has never heard of (e.g. a future kind) is tolerated, not fatal.
        assert!(serde_json::from_str::<RpcAction>(r#"{"kind":"token_transfer"}"#).is_ok());
    }

    #[test]
    fn unknown_kind_does_not_fail_the_block() {
        let json = serde_json::json!({ "hash": "t", "chain_id": 14, "bundle": bundle_json(), "action": { "kind": "slash", "evidence": "…" } });
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
        let s2: Vec<RpcValidator> = serde_json::from_str(r#"[{"address":"2nRd","stake":"1000000000000","pending":[{"release_epoch":41,"amount":"5000000000"}],"rewards":"4000000","payout":"rand1x","nonce":3,"active":true}]"#).unwrap();
        assert_eq!(s2[0].pending[0].release_epoch, 41);
        assert_eq!(s2[0].payout.as_deref(), Some("rand1x"));
        assert_eq!(s2[0].active, Some(true));
    }

    #[test]
    fn parses_receipt_and_status() {
        let r: RpcReceipt = serde_json::from_str(r#"{"tx":"d4c7","program":"675a","tier":14,"outputs":[1,0,25,0,0,0,0,0],"height":17,"index":0,"h_in":"9c0e"}"#).unwrap();
        assert_eq!(r.h_in, "9c0e");
        assert_eq!(r.h_pub, None);
        let s: NodeStatus = serde_json::from_str(r#"{"height":1998,"head_hash":"x","view":2251,"high_qc_view":2250,"syncing":false,"sync_target":1998,"peer_count":5,"mempool_size":0,"is_validator":true,"faucet":true,"confidential":true,"fri_profile":"production","programs":2,"notes":41,"nullifiers":12,"tree_root":"6b1d","hc_bundle":"f07a","address":null,"peer_id":"12D3"}"#).unwrap();
        assert_eq!(s.notes, 41);
        assert_eq!(s.address, None);
        assert_eq!(s.active_validator, None);
    }
}
