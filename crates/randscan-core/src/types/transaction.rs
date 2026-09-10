use serde::{Deserialize, Serialize};

/// The four transaction kinds of the SHRUGG chain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TxKind {
    Transfer,
    Mint,
    Deploy,
    Call,
}

impl TxKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            TxKind::Transfer => "transfer",
            TxKind::Mint => "mint",
            TxKind::Deploy => "deploy",
            TxKind::Call => "call",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "transfer" => Some(TxKind::Transfer),
            "mint" => Some(TxKind::Mint),
            "deploy" => Some(TxKind::Deploy),
            "call" => Some(TxKind::Call),
            _ => None,
        }
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
}

/// A transaction as seen from one account's history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountTransaction {
    #[serde(flatten)]
    pub summary: TransactionSummary,
    pub role: String,
}
