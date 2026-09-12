use serde::{Deserialize, Serialize};

use super::TxKind;

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

/// A tree leaf with the envelope its transaction published: what a viewing key or a
/// per-transaction key opens (in the browser; the explorer holds no keys).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteEnvelope {
    pub leaf_index: i64,
    pub cm: String,
    pub height: i64,
    pub tx_hash: Option<String>,
    /// `{ kem_ct, to_receiver, to_sender, body }` as hex, or `null` for a leaf indexed before
    /// envelopes were stored (re-fetched on the next pass).
    pub envelope: Option<serde_json::Value>,
}

/// Everything a key can be tried against for one transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionEnvelopes {
    pub hash: String,
    pub kind: TxKind,
    pub notes: Vec<NoteEnvelope>,
    /// For a call: the receipt's `h_in` and the sealed input transcript
    /// (`{ kem_ct, to_sender, to_auditor, body }`), when the caller published one.
    pub h_in: Option<String>,
    pub call_envelope: Option<serde_json::Value>,
}

/// A page of leaves with envelopes, oldest first, for a history scan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteEnvelopePage {
    pub from_leaf: i64,
    pub next_leaf: Option<i64>,
    pub total_leaves: i64,
    pub notes: Vec<NoteEnvelope>,
}
