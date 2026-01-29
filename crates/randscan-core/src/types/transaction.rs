//! Transaction types for RandScan

use serde::{Deserialize, Serialize};
use super::{EncryptedAmount, Id32, Signature64};

/// Transaction identifier - SHA256 hash of transaction
pub type TransactionId = Id32;

/// Transaction payload type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PayloadType {
    Public,
    Private,
    Stealth,
    Stake,
    Unstake,
    Transfer,
    Deploy,
    Invoke,
    PrivateTransfer,
}

impl PayloadType {
    pub fn as_str(&self) -> &'static str {
        match self {
            PayloadType::Public => "public",
            PayloadType::Private => "private",
            PayloadType::Stealth => "stealth",
            PayloadType::Stake => "stake",
            PayloadType::Unstake => "unstake",
            PayloadType::Transfer => "transfer",
            PayloadType::Deploy => "deploy",
            PayloadType::Invoke => "invoke",
            PayloadType::PrivateTransfer => "private_transfer",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "public" => Some(PayloadType::Public),
            "private" => Some(PayloadType::Private),
            "stealth" => Some(PayloadType::Stealth),
            "stake" => Some(PayloadType::Stake),
            "unstake" => Some(PayloadType::Unstake),
            "transfer" => Some(PayloadType::Transfer),
            "deploy" => Some(PayloadType::Deploy),
            "invoke" => Some(PayloadType::Invoke),
            "private_transfer" => Some(PayloadType::PrivateTransfer),
            _ => None,
        }
    }

    pub fn is_private(&self) -> bool {
        matches!(
            self,
            PayloadType::Private | PayloadType::Stealth | PayloadType::PrivateTransfer
        )
    }

    pub fn privacy_level(&self) -> PrivacyLevel {
        match self {
            PayloadType::Public | PayloadType::Stake | PayloadType::Unstake |
            PayloadType::Transfer | PayloadType::Deploy | PayloadType::Invoke => PrivacyLevel::Public,
            PayloadType::Private => PrivacyLevel::Confidential,
            PayloadType::Stealth => PrivacyLevel::Stealth,
            PayloadType::PrivateTransfer => PrivacyLevel::Private,
        }
    }
}

impl std::fmt::Display for PayloadType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Privacy level indicator for UI display
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrivacyLevel {
    Public,
    Confidential,
    Stealth,
    Private,
}

impl PrivacyLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            PrivacyLevel::Public => "public",
            PrivacyLevel::Confidential => "confidential",
            PrivacyLevel::Stealth => "stealth",
            PrivacyLevel::Private => "private",
        }
    }
}

/// Transaction status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Finalized,
    Failed,
}

impl TransactionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TransactionStatus::Pending => "pending",
            TransactionStatus::Confirmed => "confirmed",
            TransactionStatus::Finalized => "finalized",
            TransactionStatus::Failed => "failed",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(TransactionStatus::Pending),
            "confirmed" => Some(TransactionStatus::Confirmed),
            "finalized" => Some(TransactionStatus::Finalized),
            "failed" => Some(TransactionStatus::Failed),
            _ => None,
        }
    }
}

/// Transaction payload variants
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TransactionPayload {
    Public {
        instructions: Vec<u8>,
    },
    Private {
        encrypted_payload: Vec<u8>,
        proof: Vec<u8>,
        nullifiers: Vec<Id32>,
        commitments: Vec<Id32>,
    },
    Stealth {
        ephemeral_pubkey: Id32,
        stealth_address: Id32,
        encrypted_amount: EncryptedAmount,
        proof: Vec<u8>,
    },
    Stake {
        amount: u64,
    },
    Unstake {
        amount: u64,
    },
    Transfer {
        to: Id32,
        amount: u64,
    },
    Deploy {
        code: Vec<u8>,
    },
    Invoke {
        program_id: Id32,
        instruction: Vec<u8>,
    },
    PrivateTransfer {
        proof: Vec<u8>,
        nullifiers: Vec<Id32>,
        commitments: Vec<Id32>,
    },
}

impl TransactionPayload {
    pub fn payload_type(&self) -> PayloadType {
        match self {
            TransactionPayload::Public { .. } => PayloadType::Public,
            TransactionPayload::Private { .. } => PayloadType::Private,
            TransactionPayload::Stealth { .. } => PayloadType::Stealth,
            TransactionPayload::Stake { .. } => PayloadType::Stake,
            TransactionPayload::Unstake { .. } => PayloadType::Unstake,
            TransactionPayload::Transfer { .. } => PayloadType::Transfer,
            TransactionPayload::Deploy { .. } => PayloadType::Deploy,
            TransactionPayload::Invoke { .. } => PayloadType::Invoke,
            TransactionPayload::PrivateTransfer { .. } => PayloadType::PrivateTransfer,
        }
    }

    pub fn get_nullifiers(&self) -> Vec<Id32> {
        match self {
            TransactionPayload::Private { nullifiers, .. } => nullifiers.clone(),
            TransactionPayload::PrivateTransfer { nullifiers, .. } => nullifiers.clone(),
            _ => Vec::new(),
        }
    }

    pub fn get_commitments(&self) -> Vec<Id32> {
        match self {
            TransactionPayload::Private { commitments, .. } => commitments.clone(),
            TransactionPayload::PrivateTransfer { commitments, .. } => commitments.clone(),
            _ => Vec::new(),
        }
    }
}

/// Full transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub nonce: u64,
    pub sender: Id32,
    pub compute_budget: u64,
    pub fee: u64,
    pub payload: TransactionPayload,
    pub signature: Signature64,
    pub timestamp: u64,
}

impl Transaction {
    pub fn id(&self) -> TransactionId {
        use super::sha256;
        let data = bincode::serialize(self).unwrap_or_default();
        TransactionId::new(sha256(&data))
    }

    pub fn payload_type(&self) -> PayloadType {
        self.payload.payload_type()
    }

    pub fn is_private(&self) -> bool {
        self.payload_type().is_private()
    }
}

/// Transaction summary for list views
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionSummary {
    pub tx_id: String,
    pub block_id: Option<String>,
    pub block_height: Option<i64>,
    pub sender: String,
    pub nonce: i64,
    pub fee: i64,
    pub payload_type: String,
    pub status: String,
    pub timestamp: i64,
    pub privacy_level: String,
}

/// Transaction detail with full information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionDetail {
    pub tx_id: String,
    pub block_id: Option<String>,
    pub block_height: Option<i64>,
    pub sender: String,
    pub nonce: i64,
    pub compute_budget: i64,
    pub fee: i64,
    pub payload_type: String,
    pub status: String,
    pub timestamp: i64,
    pub signature: String,
    pub privacy_level: String,
    // Type-specific fields
    pub payload_data: Option<TransactionPayloadData>,
    // Privacy tracking
    pub nullifier_count: i32,
    pub commitment_count: i32,
}

/// Type-specific payload data for display
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TransactionPayloadData {
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
        amount: i64,
        amount_display: f64,
    },
    Unstake {
        amount: i64,
        amount_display: f64,
    },
    Transfer {
        to: String,
        amount: i64,
        amount_display: f64,
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

/// Nullifier record for privacy tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Nullifier {
    pub nullifier: String,
    pub tx_id: String,
    pub created_at: i64,
}

/// Commitment record for privacy tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commitment {
    pub commitment: String,
    pub tx_id: String,
    pub spent: bool,
    pub spent_tx_id: Option<String>,
    pub created_at: i64,
}
