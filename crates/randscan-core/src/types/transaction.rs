use serde::{Deserialize, Serialize};

/// Transaction kinds of the shielded SHRUGG chain (`fullnode/docs/rpc.md`, `shrugg_getTransaction`).
///
/// Every transaction is a shielded bundle plus an *action*; the kind is the action's. The node's
/// `none` action (a plain shielded transfer) is served as `transfer`. `Other` is any kind the node
/// serves that this build does not decode; the indexer keeps the node's tag in the database so a
/// later build can backfill without re-indexing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TxKind {
    Transfer,
    Mint,
    Deploy,
    Call,
    Bond,
    Unbond,
    Withdraw,
    BridgeAttest,
    BridgeBurn,
    Other,
}

impl TxKind {
    /// Kinds an API client may filter by (`GET /transactions?kind=`).
    pub const FILTERABLE: &'static [TxKind] = &[
        TxKind::Transfer,
        TxKind::Mint,
        TxKind::Deploy,
        TxKind::Call,
        TxKind::Bond,
        TxKind::Unbond,
        TxKind::Withdraw,
        TxKind::BridgeAttest,
        TxKind::BridgeBurn,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            TxKind::Transfer => "transfer",
            TxKind::Mint => "mint",
            TxKind::Deploy => "deploy",
            TxKind::Call => "call",
            TxKind::Bond => "bond",
            TxKind::Unbond => "unbond",
            TxKind::Withdraw => "withdraw",
            TxKind::BridgeAttest => "bridge_attest",
            TxKind::BridgeBurn => "bridge_burn",
            TxKind::Other => "other",
        }
    }

    /// Exact match on a known, filterable kind; `None` for anything else (including `other`).
    pub fn parse(s: &str) -> Option<Self> {
        Self::FILTERABLE.iter().copied().find(|k| k.as_str() == s)
    }

    /// Like [`parse`](Self::parse) but maps unrecognised tags to [`TxKind::Other`].
    pub fn parse_lossy(s: &str) -> Self {
        Self::parse(s).unwrap_or(TxKind::Other)
    }

    /// The node's own action tag for this kind (`none` for a transfer).
    pub fn node_tag(&self) -> &'static str {
        match self {
            TxKind::Transfer => "none",
            other => other.as_str(),
        }
    }

    /// Kinds whose `amount` is public on chain (deposits and staking moves).
    pub fn has_public_amount(&self) -> bool {
        matches!(
            self,
            TxKind::Mint
                | TxKind::Bond
                | TxKind::Unbond
                | TxKind::Withdraw
                | TxKind::BridgeAttest
                | TxKind::BridgeBurn
        )
    }

    /// Kinds whose `amount` is in a bridged asset's own unit rather than SHRUGG units.
    pub fn amount_is_bridged(&self) -> bool {
        matches!(self, TxKind::BridgeAttest | TxKind::BridgeBurn)
    }
}

impl std::fmt::Display for TxKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The public fields of a shielded bundle: what every observer sees of a transfer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bundle {
    pub anchor: String,
    pub nullifiers: [String; 2],
    pub commitments: [String; 2],
    /// Units of SHRUGG, as a decimal string.
    pub fee: String,
    /// Units leaving the pool into the action (bond, bridge burn), decimal string.
    pub burn: String,
    /// 0 = SHRUGG; otherwise the bridge registry index of the balanced asset.
    pub asset: i64,
    /// Block height the sender targeted.
    pub time: i64,
    pub proof_len: i64,
    pub envelope_len: [i64; 2],
}

/// Transaction as shown in lists and pushed over WebSocket. Amounts are strings of units.
///
/// There is no sender, recipient or nonce: a shielded transaction has none. `amount` is the one
/// public amount an action carries (a deposit's note value or a staking move); `validator` and
/// `program` name what the action touched.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionSummary {
    pub hash: String,
    pub height: i64,
    pub block_hash: String,
    pub tx_index: i32,
    pub kind: TxKind,
    /// SHRUGG fee paid by the bundle ("0" for a validator-signed action without one).
    pub fee: String,
    pub timestamp_ms: i64,
    /// False for mint / unbond / withdraw, which are signed by a validator instead.
    pub has_bundle: bool,
    /// deploy: deployed program id; call: called program id
    pub program: Option<String>,
    /// bond / unbond / withdraw: the validator address; mint: the minting validator
    pub validator: Option<String>,
    /// mint, bond, unbond, withdraw: SHRUGG units; bridge_attest, bridge_burn: bridged units
    pub amount: Option<String>,
    /// bridge_attest / bridge_burn: the bridged asset's registry index
    pub asset_index: Option<i64>,
}

/// Receipt of a confidential call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Receipt {
    pub tx: String,
    pub program: String,
    pub tier: i32,
    pub outputs: Vec<i64>,
    pub height: i64,
    pub index: i32,
    /// The proof's salted commitment to the call's private inputs (zkVM M4.1).
    pub h_in: String,
}

/// Full transaction detail.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionDetail {
    #[serde(flatten)]
    pub summary: TransactionSummary,
    pub chain_id: i64,
    /// The fee bundle; `null` for a validator-signed action.
    pub bundle: Option<Bundle>,
    /// deploy: program length in words
    pub words_len: Option<i64>,
    /// call: size of the call's own proof
    pub call_proof_len: Option<i64>,
    /// call: size of the sealed input transcript, `null` when the caller published none
    pub input_envelope_len: Option<i64>,
    pub receipt: Option<Receipt>,
    /// mint: the deposit note's commitment
    pub cm: Option<String>,
    /// bond: whether this bond registered a new validator
    pub registered: Option<bool>,
    /// unbond / withdraw: the register nonce the validator signed
    pub action_nonce: Option<i64>,
    /// bridge_attest: size of the guardian-signed message
    pub attestation_len: Option<i64>,
    /// bridge_attest: the depositor's shielded address (`shrugg1…`)
    pub recipient: Option<String>,
    /// bridge_attest: the deposit note's `time` word
    pub note_time: Option<i64>,
    /// bridge_burn: relayer fee in bridged units
    pub relayer_fee: Option<String>,
    /// bridge_burn: destination chain id (1 Rand, 2 Ethereum, 3 BSC, 4 Tron, 5 Solana)
    pub to_chain: Option<i32>,
    /// bridge_burn: destination address, 32 bytes hex (EVM/Tron addresses left-padded)
    pub bridge_to: Option<String>,
    /// bridge_burn: the second bundle, which burns the bridged asset
    pub asset_bundle: Option<Bundle>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kinds_round_trip_through_serde_and_parse() {
        for k in TxKind::FILTERABLE {
            let json = serde_json::to_string(k).unwrap();
            assert_eq!(json, format!("\"{}\"", k.as_str()));
            assert_eq!(TxKind::parse(k.as_str()), Some(*k));
            assert_eq!(serde_json::from_str::<TxKind>(&json).unwrap(), *k);
        }
        assert_eq!(
            serde_json::to_string(&TxKind::BridgeBurn).unwrap(),
            "\"bridge_burn\""
        );
        assert_eq!(TxKind::Transfer.node_tag(), "none");
        assert_eq!(TxKind::Withdraw.node_tag(), "withdraw");
    }

    #[test]
    fn unknown_tags_are_other_only_when_lossy() {
        assert_eq!(
            TxKind::parse("none"),
            None,
            "the node tag is not an API filter"
        );
        assert_eq!(TxKind::parse("other"), None);
        assert_eq!(TxKind::parse_lossy("shielded"), TxKind::Other);
        assert_eq!(TxKind::parse_lossy("bond"), TxKind::Bond);
    }

    #[test]
    fn detail_flattens_the_summary() {
        let d = TransactionDetail {
            summary: TransactionSummary {
                hash: "ab".into(),
                height: 1,
                block_hash: "cd".into(),
                tx_index: 0,
                kind: TxKind::Transfer,
                fee: "1000000".into(),
                timestamp_ms: 1,
                has_bundle: true,
                program: None,
                validator: None,
                amount: None,
                asset_index: None,
            },
            chain_id: 7,
            bundle: Some(Bundle {
                anchor: "00".into(),
                nullifiers: ["01".into(), "02".into()],
                commitments: ["03".into(), "04".into()],
                fee: "1000000".into(),
                burn: "0".into(),
                asset: 0,
                time: 5,
                proof_len: 302857,
                envelope_len: [1380, 1380],
            }),
            words_len: None,
            call_proof_len: None,
            input_envelope_len: None,
            receipt: None,
            cm: None,
            registered: None,
            action_nonce: None,
            attestation_len: None,
            recipient: None,
            note_time: None,
            relayer_fee: None,
            to_chain: None,
            bridge_to: None,
            asset_bundle: None,
        };
        let v = serde_json::to_value(&d).unwrap();
        assert_eq!(v["kind"], "transfer");
        assert_eq!(v["bundle"]["nullifiers"][1], "02");
        assert!(v.get("sender").is_none() && v.get("nonce").is_none());
    }
}
