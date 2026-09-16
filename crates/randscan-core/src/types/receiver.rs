use serde::{Deserialize, Serialize};

/// A published version of the receiver registry (fullnode spec 2026-09-17 §5, §8): the short
/// shielded address (`id`, `rand1…`) resolves to this record. `GET /receivers/:address` serves
/// the highest version; `GET /receivers/:address/history` serves every version, newest first.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiverRecordView {
    pub id: String,
    pub version: i32,
    /// Hex, 64 characters (the shielded address's spend key, `Word8`).
    pub pk: String,
    /// Hex, 2368 characters (the ML-KEM encapsulation key).
    pub kem_ek: String,
    /// Hex Dilithium2 public key that signed this record.
    pub signing_key: String,
    /// Hex Dilithium2 signature over the record.
    pub signature: String,
    pub tx_hash: String,
    pub height: i64,
}
