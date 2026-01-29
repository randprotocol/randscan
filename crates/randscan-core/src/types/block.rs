//! Block types for RandScan

use serde::{Deserialize, Serialize};
use super::{Epoch, Height, Id32, Signature64, ViewNumber};

/// Block identifier - SHA256 hash of block header
pub type BlockId = Id32;

/// Validator identifier
pub type ValidatorId = Id32;

/// Vote type in HotStuff consensus
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VoteType {
    Prepare,
    PreCommit,
    Commit,
    NewView,
}

impl std::fmt::Display for VoteType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VoteType::Prepare => write!(f, "prepare"),
            VoteType::PreCommit => write!(f, "pre_commit"),
            VoteType::Commit => write!(f, "commit"),
            VoteType::NewView => write!(f, "new_view"),
        }
    }
}

/// Quorum Certificate - proof of 2f+1 validator agreement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuorumCertificate {
    pub vote_type: VoteType,
    pub view: ViewNumber,
    pub block_id: BlockId,
    pub block_height: Height,
    pub signers: Vec<ValidatorId>,
    pub aggregate_signature: Vec<(ValidatorId, Signature64)>,
}

impl QuorumCertificate {
    pub fn signer_count(&self) -> usize {
        self.signers.len()
    }

    pub fn has_quorum(&self, total_validators: usize) -> bool {
        self.signers.len() >= (2 * total_validators / 3) + 1
    }

    pub fn is_genesis(&self) -> bool {
        self.block_id.is_zero()
    }
}

impl Default for QuorumCertificate {
    fn default() -> Self {
        Self {
            vote_type: VoteType::Commit,
            view: 0,
            block_id: BlockId::zero(),
            block_height: 0,
            signers: Vec::new(),
            aggregate_signature: Vec::new(),
        }
    }
}

/// Block header containing all metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    pub height: Height,
    pub view: ViewNumber,
    pub epoch: Epoch,
    pub parent_id: BlockId,
    pub transactions_root: Id32,
    pub state_root: Id32,
    pub supply_commitment: Id32,
    pub proposer: ValidatorId,
    pub timestamp: u64,
    pub justify: QuorumCertificate,
}

impl BlockHeader {
    pub fn compute_id(&self) -> BlockId {
        use super::sha256;
        let data = bincode::serialize(self).unwrap_or_default();
        BlockId::new(sha256(&data))
    }

    pub fn is_genesis(&self) -> bool {
        self.height == 0 && self.parent_id.is_zero()
    }
}

/// Full block with transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<super::Transaction>,
    pub signature: Signature64,
}

impl Block {
    pub fn id(&self) -> BlockId {
        self.header.compute_id()
    }

    pub fn height(&self) -> Height {
        self.header.height
    }

    pub fn view(&self) -> ViewNumber {
        self.header.view
    }

    pub fn parent_id(&self) -> BlockId {
        self.header.parent_id
    }

    pub fn is_genesis(&self) -> bool {
        self.header.is_genesis()
    }

    pub fn transaction_count(&self) -> usize {
        self.transactions.len()
    }
}

/// Block summary for list views
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockSummary {
    pub block_id: String,
    pub height: Height,
    pub view: ViewNumber,
    pub epoch: Epoch,
    pub parent_id: String,
    pub proposer: String,
    pub timestamp: u64,
    pub transaction_count: i32,
    pub finalized: bool,
}

/// Block detail with full information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockDetail {
    pub block_id: String,
    pub height: Height,
    pub view: ViewNumber,
    pub epoch: Epoch,
    pub parent_id: String,
    pub proposer: String,
    pub timestamp: u64,
    pub transactions_root: String,
    pub state_root: String,
    pub supply_commitment: String,
    pub transaction_count: i32,
    pub transactions: Vec<String>,
    pub finalized: bool,
    pub qc_vote_type: String,
    pub qc_view: ViewNumber,
    pub qc_signers: Vec<String>,
}

/// QC signer record for database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QcSigner {
    pub block_id: String,
    pub validator_id: String,
    pub signature: String,
}
