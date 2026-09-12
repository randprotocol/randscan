use serde::{Deserialize, Serialize};

/// One leaf of the commitment tree. Its value and owner are hidden; `tx_hash` is the indexed
/// transaction that created it when the commitment was on the wire (bundle outputs, mints), and
/// `null` for genesis, withdraw and bridge deposit notes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub leaf_index: i64,
    pub cm: String,
    pub height: i64,
    pub tx_hash: Option<String>,
}

/// A published nullifier and the transaction that spent the note.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Nullifier {
    pub nullifier: String,
    pub tx_hash: String,
    pub height: i64,
    pub tx_index: i32,
}
