use serde::{Deserialize, Serialize};

/// Transaction kinds of the SHRUGG chain (`fullnode/docs/rpc.md`).
///
/// `Other` is any kind the node serves that this build does not decode; the indexer keeps the
/// node's tag in the database so a later build can backfill without re-indexing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TxKind {
    Transfer,
    Mint,
    Deploy,
    Call,
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
        TxKind::BridgeAttest,
        TxKind::BridgeBurn,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            TxKind::Transfer => "transfer",
            TxKind::Mint => "mint",
            TxKind::Deploy => "deploy",
            TxKind::Call => "call",
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

    /// Whether the transaction moves SHRUGG to `to` (so `to` is a chain address, not foreign).
    pub fn has_native_recipient(&self) -> bool {
        matches!(self, TxKind::Transfer | TxKind::Mint)
    }
}

impl std::fmt::Display for TxKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Transaction as shown in lists and pushed over WebSocket. Amounts are strings of units.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionSummary {
    pub hash: String,
    pub height: i64,
    pub block_hash: String,
    pub tx_index: i32,
    pub sender: String,
    pub nonce: i64,
    pub fee: String,
    pub kind: TxKind,
    pub timestamp_ms: i64,
    /// transfer / mint recipient
    pub to: Option<String>,
    /// transfer / mint amount (units)
    pub amount: Option<String>,
    /// deploy: deployed program id; call: called program id
    pub program: Option<String>,
}

/// Receipt of a confidential call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Receipt {
    pub tx: String,
    pub program: String,
    pub tier: i32,
    pub outputs: Vec<i64>,
    pub effect: Option<ReceiptEffect>,
    pub height: i64,
    pub index: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptEffect {
    pub to: String,
    pub amount: String,
}

/// Full transaction detail.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionDetail {
    #[serde(flatten)]
    pub summary: TransactionSummary,
    pub chain_id: i64,
    pub base_pc: Option<i64>,
    pub words_len: Option<i64>,
    pub proof_len: Option<i64>,
    pub recipients: Vec<String>,
    pub receipt: Option<Receipt>,
    /// bridge_burn: bridged asset id (hex)
    #[serde(default)]
    pub asset: Option<String>,
    /// bridge_burn: amount burned, in bridged units (8 decimals, not SHRUGG's 9)
    #[serde(default)]
    pub bridge_amount: Option<String>,
    /// bridge_burn: destination chain id (1 Rand, 2 Ethereum, 3 BSC, 4 Tron, 5 Solana)
    #[serde(default)]
    pub to_chain: Option<i32>,
    /// bridge_burn: destination address, 32 bytes hex (EVM/Tron addresses left-padded)
    #[serde(default)]
    pub bridge_to: Option<String>,
    /// bridge_burn: relayer fee in bridged units
    #[serde(default)]
    pub bridge_fee: Option<String>,
    /// bridge_attest: the guardian-signed message, hex
    #[serde(default)]
    pub attestation: Option<String>,
}

/// A transaction as seen from one account's history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountTransaction {
    #[serde(flatten)]
    pub summary: TransactionSummary,
    pub role: String,
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
        assert_eq!(serde_json::to_string(&TxKind::BridgeBurn).unwrap(), "\"bridge_burn\"");
    }

    #[test]
    fn unknown_tags_are_other_only_when_lossy() {
        assert_eq!(TxKind::parse("shielded"), None);
        assert_eq!(TxKind::parse("other"), None);
        assert_eq!(TxKind::parse_lossy("shielded"), TxKind::Other);
        assert_eq!(TxKind::parse_lossy("bridge_attest"), TxKind::BridgeAttest);
    }
}
