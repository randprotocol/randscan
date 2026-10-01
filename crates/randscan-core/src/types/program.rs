use serde::{Deserialize, Serialize};

use super::TransactionSummary;

/// A deployed zkVM program. There is no deployer: a deploy is paid by a shielded bundle, so the
/// chain does not know who deployed it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramSummary {
    pub id: String,
    pub deploy_tx: String,
    pub deployed_at_height: i64,
    pub base_pc: i64,
    pub words_len: i64,
    pub code_hash: String,
    /// The length of the public input fixed at deploy (0 without one).
    pub public_words_len: i64,
    /// Its digest (Word8 hex): what every call's proof is checked against, and a call receipt's
    /// `h_pub`. `None` for a program deployed without a public input.
    pub public_digest: Option<String>,
    pub call_count: i64,
    pub last_called_height: Option<i64>,
    /// RPL-2: how many `invoke`s moved this program's state, and the last height one did.
    pub invoke_count: i64,
    pub last_invoked_height: Option<i64>,
}

/// One row of a program's vault (RPL-2, `rand_getProgramVault`): what the program holds of one
/// asset, in that asset's own units. A row at zero does not exist.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VaultRow {
    /// registry index (0 is RAND)
    pub asset: i64,
    #[serde(deserialize_with = "crate::amount::amount")]
    pub amount: String,
}

/// A program's public state on a chain with the `program_state` section (RPL-2): its vault and
/// the first page of its cells, read live from the node when the program page is served.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProgramState {
    pub vault: Vec<VaultRow>,
    /// the program's cells in key order (unsigned comparison of the eight words), first page
    pub cells: Vec<super::ProgramCell>,
    /// the last key served when more cells follow (`GET /programs/:id/cells?after=`), else `null`
    pub cells_next: Option<String>,
}

/// A page of a program's cells (`GET /programs/:id/cells`, `rand_getProgramCells`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProgramCellsPage {
    pub cells: Vec<super::ProgramCell>,
    #[serde(default)]
    pub next: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramDetail {
    #[serde(flatten)]
    pub program: ProgramSummary,
    /// the latest calls and invokes of this program
    pub recent_calls: Vec<TransactionSummary>,
    /// `null` on a chain without a `program_state` section (and on a node predating RPL-2).
    pub program_state: Option<ProgramState>,
}
