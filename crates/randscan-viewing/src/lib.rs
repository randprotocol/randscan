//! The open side of the shielded chain's disclosure layer, for the explorer's browser code.
//!
//! A shielded transaction publishes, per created note, an envelope: the note plaintext under a
//! fresh per-transaction key, that key wrapped to the receiver's ML-KEM-768 address and under
//! the sender's outgoing viewing key. This crate re-implements exactly what the fullnode's
//! `crates/shrugg-zkvm/src/{hash,notes,viewing,call_envelope}.rs` do on the *opening* side
//! (commit `01dc23d`, the build chain 6 runs), with no sealing, no randomness and no ledger, so
//! it compiles to WebAssembly and a key pasted into the explorer never leaves the browser.
//!
//! Every ciphertext is ChaCha20-Poly1305 with the on-chain commitment as associated data, so a
//! wrong key fails authentication instead of yielding garbage, and an opened note is checked by
//! recomputing its commitment. Keys, commitments and nullifiers are `Word8` — eight `u32`
//! words, hex-encoded little-endian word by word (64 hex characters).
//!
//! `tests/vectors.json` was produced by the fullnode crate itself (sealing with known keys);
//! the tests here open those vectors, which is what pins this crate to the wire format.

use p3_field::{PrimeCharacteristicRing, PrimeField64};
use p3_goldilocks::{Goldilocks, Poseidon2Goldilocks};
use p3_symmetric::{CryptographicHasher, PaddingFreeSponge, Permutation};
use rand::rngs::StdRng;
use rand::SeedableRng;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use wasm_bindgen::prelude::*;

pub type Word8 = [u32; 8];
type Val = Goldilocks;
type Perm = Poseidon2Goldilocks<8>;

/// `machine::PERM_SEED` in the fullnode: the seed every node draws the Poseidon2 round
/// constants from ("RandZK").
const PERM_SEED: u64 = 0x5261_6e64_5a4b;

fn perm() -> &'static Perm {
    static PERM: OnceLock<Perm> = OnceLock::new();
    PERM.get_or_init(|| Perm::new_from_rng_128(&mut StdRng::seed_from_u64(PERM_SEED)))
}

fn split_digest(elems: [Val; 4]) -> Word8 {
    let mut out = [0u32; 8];
    for (i, e) in elems.iter().enumerate() {
        let v = e.as_canonical_u64();
        out[2 * i] = v as u32;
        out[2 * i + 1] = (v >> 32) as u32;
    }
    out
}

/// The `POSEIDON2` syscall's sponge: rate 4, capacity 4, overwrite mode, no padding.
pub fn sponge_hash(msg: &[u32]) -> Word8 {
    let sponge = PaddingFreeSponge::<_, 8, 4, 4>::new(perm().clone());
    let elems: Vec<Val> = msg.iter().copied().map(Val::from_u32).collect();
    split_digest(sponge.hash_iter(elems))
}

fn permute_state(state: [Val; 8]) -> [Val; 8] {
    let mut s = state;
    perm().permute_mut(&mut s);
    s
}

/// `H_IN`: the salted input commitment of a confidential call (zkVM M4.1), header
/// `[IN, n_in, 0]` in the capacity lanes, the salt as the first absorbed block.
pub fn input_digest(salt: [u32; 4], inputs: &[u32]) -> Word8 {
    let mut state = [Val::ZERO; 8];
    state[4] = Val::from_u32(domain::IN);
    state[5] = Val::from_u32(inputs.len() as u32);
    let mut merged = state;
    for k in 0..4 {
        merged[k] = Val::from_u32(salt[k]);
    }
    state = permute_state(merged);
    for block in inputs.chunks(4) {
        let mut merged = state;
        for (k, w) in block.iter().enumerate() {
            merged[k] = Val::from_u32(*w);
        }
        state = permute_state(merged);
    }
    split_digest([state[0], state[1], state[2], state[3]])
}

pub mod domain {
    pub const NK: u32 = 1;
    pub const PK: u32 = 2;
    pub const NF: u32 = 3;
    pub const CM: u32 = 4;
    pub const OVK: u32 = 5;
    pub const KEM_SEED: u32 = 6;
    pub const IN: u32 = 10;
}

/// `H(domain, msg)`: the domain tag is the first absorbed word.
pub fn hash(domain: u32, msg: &[u32]) -> Word8 {
    let mut full = Vec::with_capacity(1 + msg.len());
    full.push(domain);
    full.extend_from_slice(msg);
    sponge_hash(&full)
}

fn wide_hash(domain: u32, msg: &[u32], out_words: usize) -> Vec<u32> {
    let mut out = Vec::with_capacity(out_words);
    let mut counter = 0u32;
    while out.len() < out_words {
        let mut full = Vec::with_capacity(2 + msg.len());
        full.push(domain);
        full.extend_from_slice(msg);
        full.push(counter);
        let chunk = sponge_hash(&full);
        let take = (out_words - out.len()).min(chunk.len());
        out.extend_from_slice(&chunk[..take]);
        counter += 1;
    }
    out
}

pub fn words_to_bytes(w: &[u32]) -> Vec<u8> {
    w.iter().flat_map(|x| x.to_le_bytes()).collect()
}

pub fn word8_to_hex(w: &Word8) -> String {
    hex::encode(words_to_bytes(w))
}

pub fn word8_from_hex(s: &str) -> Option<Word8> {
    let b = hex::decode(s.trim().trim_start_matches("0x")).ok()?;
    if b.len() != 32 {
        return None;
    }
    let mut w = [0u32; 8];
    for (i, c) in b.chunks(4).enumerate() {
        w[i] = u32::from_le_bytes(c.try_into().unwrap());
    }
    Some(w)
}

fn key32_from_hex(s: &str) -> Option<[u8; 32]> {
    hex::decode(s.trim().trim_start_matches("0x")).ok()?.try_into().ok()
}

/// A party's viewing key: sees everything, spends nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ViewingKey {
    pub nk: Word8,
}

type Dk = ml_kem::ml_kem_768::DecapsulationKey;
type Ek = ml_kem::ml_kem_768::EncapsulationKey;
type KemCt = ml_kem::ml_kem_768::Ciphertext;

impl ViewingKey {
    pub fn from_spend_key(sk: &Word8) -> ViewingKey {
        ViewingKey { nk: hash(domain::NK, sk) }
    }
    pub fn pk(&self) -> Word8 {
        hash(domain::PK, &self.nk)
    }
    pub fn nullifier(&self, cm: &Word8) -> Word8 {
        let mut msg = [0u32; 16];
        msg[..8].copy_from_slice(&self.nk);
        msg[8..].copy_from_slice(cm);
        hash(domain::NF, &msg)
    }
    pub fn ovk(&self) -> [u8; 32] {
        words_to_bytes(&wide_hash(domain::OVK, &self.nk, 8)).try_into().unwrap()
    }
    pub fn kem_seed(&self) -> [u8; 64] {
        words_to_bytes(&wide_hash(domain::KEM_SEED, &self.nk, 16)).try_into().unwrap()
    }
    fn kem_keys(&self) -> (Dk, Ek) {
        use ml_kem::kem::FromSeed;
        ml_kem::MlKem768::from_seed(&ml_kem::Seed::from(self.kem_seed()))
    }
    /// `shrugg1` + base58(pk bytes || ML-KEM-768 encapsulation key).
    pub fn address(&self) -> String {
        use ml_kem::KeyExport;
        let (_, ek) = self.kem_keys();
        let mut raw = words_to_bytes(&self.pk());
        raw.extend_from_slice(&ek.to_bytes());
        format!("shrugg1{}", bs58::encode(raw).into_string())
    }
}

/// What a note records: `pk(8) from(8) amount_lo amount_hi asset time r(8)`, 28 words.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Note {
    pub pk: Word8,
    pub from: Word8,
    pub amount: u64,
    pub asset: u32,
    pub time: u32,
    pub r: Word8,
}

impl Note {
    pub const WORDS: usize = 28;
    pub fn words(&self) -> [u32; Self::WORDS] {
        let mut w = [0u32; Self::WORDS];
        w[0..8].copy_from_slice(&self.pk);
        w[8..16].copy_from_slice(&self.from);
        w[16] = self.amount as u32;
        w[17] = (self.amount >> 32) as u32;
        w[18] = self.asset;
        w[19] = self.time;
        w[20..28].copy_from_slice(&self.r);
        w
    }
    pub fn commitment(&self) -> Word8 {
        hash(domain::CM, &self.words())
    }
    pub fn from_bytes(b: &[u8]) -> Option<Note> {
        if b.len() != 4 * Self::WORDS {
            return None;
        }
        let mut w = [0u32; Self::WORDS];
        for (i, c) in b.chunks(4).enumerate() {
            w[i] = u32::from_le_bytes(c.try_into().unwrap());
        }
        Some(Note {
            pk: w[0..8].try_into().unwrap(),
            from: w[8..16].try_into().unwrap(),
            amount: (w[16] as u64) | ((w[17] as u64) << 32),
            asset: w[18],
            time: w[19],
            r: w[20..28].try_into().unwrap(),
        })
    }
}

fn aead_open(key: &[u8; 32], aad: &[u8], ct: &[u8]) -> Option<Vec<u8>> {
    use chacha20poly1305::aead::{Aead, KeyInit, Payload};
    use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
    if ct.len() < 12 {
        return None;
    }
    let nonce: [u8; 12] = ct[..12].try_into().ok()?;
    ChaCha20Poly1305::new(&Key::from(*key))
        .decrypt(&Nonce::from(nonce), Payload { msg: &ct[12..], aad })
        .ok()
}

fn decapsulate(vk: &ViewingKey, kem_ct: &[u8]) -> Option<[u8; 32]> {
    use ml_kem::Decapsulate;
    let (dk, _) = vk.kem_keys();
    let ct = KemCt::try_from(kem_ct).ok()?;
    Some(dk.decapsulate(&ct).into())
}

/// A note envelope as the node serves it (`shrugg_getCommitments`), hex fields.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnvelopeHex {
    pub kem_ct: String,
    pub to_receiver: String,
    pub to_sender: String,
    pub body: String,
}

pub struct Envelope {
    pub kem_ct: Vec<u8>,
    pub to_receiver: Vec<u8>,
    pub to_sender: Vec<u8>,
    pub body: Vec<u8>,
}

impl Envelope {
    pub fn from_hex(e: &EnvelopeHex) -> Option<Envelope> {
        Some(Envelope {
            kem_ct: hex::decode(&e.kem_ct).ok()?,
            to_receiver: hex::decode(&e.to_receiver).ok()?,
            to_sender: hex::decode(&e.to_sender).ok()?,
            body: hex::decode(&e.body).ok()?,
        })
    }

    fn aad(tag: &[u8], cm: &Word8) -> Vec<u8> {
        [tag, &words_to_bytes(cm)].concat()
    }

    /// The note, if `key` is the per-transaction key this envelope was sealed under and the
    /// plaintext really commits to `cm`.
    pub fn open_with_tx_key(&self, cm: &Word8, key: &[u8; 32]) -> Option<Note> {
        let note = Note::from_bytes(&aead_open(key, &Self::aad(b"rand-envelope-body", cm), &self.body)?)?;
        (note.commitment() == *cm).then_some(note)
    }
    /// As the receiver: decapsulate, unwrap the transaction key, open the body.
    pub fn open_as_receiver(&self, cm: &Word8, vk: &ViewingKey) -> Option<([u8; 32], Note)> {
        let ss = decapsulate(vk, &self.kem_ct)?;
        let key: [u8; 32] = aead_open(&ss, &Self::aad(b"rand-envelope-receiver", cm), &self.to_receiver)?.try_into().ok()?;
        Some((key, self.open_with_tx_key(cm, &key)?))
    }
    /// As the sender, through the outgoing viewing key.
    pub fn open_as_sender(&self, cm: &Word8, vk: &ViewingKey) -> Option<([u8; 32], Note)> {
        let key: [u8; 32] = aead_open(&vk.ovk(), &Self::aad(b"rand-envelope-sender", cm), &self.to_sender)?.try_into().ok()?;
        Some((key, self.open_with_tx_key(cm, &key)?))
    }
}

/// A call's sealed input transcript as the node serves it (`shrugg_getCallEnvelope`).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CallEnvelopeHex {
    pub kem_ct: String,
    pub to_sender: String,
    pub to_auditor: String,
    pub body: String,
}

fn parse_transcript(pt: &[u8]) -> Option<([u32; 4], Vec<u32>)> {
    if pt.len() < 16 || pt.len() % 4 != 0 {
        return None;
    }
    let word = |b: &[u8]| u32::from_le_bytes(b.try_into().unwrap());
    let salt: [u32; 4] = std::array::from_fn(|i| word(&pt[4 * i..4 * i + 4]));
    Some((salt, pt[16..].chunks_exact(4).map(word).collect()))
}

pub fn open_call_with_key(e: &CallEnvelopeHex, h_in: &Word8, key: &[u8; 32]) -> Option<([u32; 4], Vec<u32>)> {
    parse_transcript(&aead_open(key, &words_to_bytes(h_in), &hex::decode(&e.body).ok()?)?)
}

pub fn open_call_as_sender(e: &CallEnvelopeHex, h_in: &Word8, vk: &ViewingKey) -> Option<([u8; 32], [u32; 4], Vec<u32>)> {
    let key: [u8; 32] = aead_open(&vk.ovk(), b"shrugg-call-sender", &hex::decode(&e.to_sender).ok()?)?.try_into().ok()?;
    let (salt, inputs) = open_call_with_key(e, h_in, &key)?;
    Some((key, salt, inputs))
}

pub fn open_call_as_auditor(e: &CallEnvelopeHex, h_in: &Word8, vk: &ViewingKey) -> Option<([u8; 32], [u32; 4], Vec<u32>)> {
    let ss = decapsulate(vk, &hex::decode(&e.kem_ct).ok()?)?;
    let key: [u8; 32] = aead_open(&ss, b"shrugg-call-auditor", &hex::decode(&e.to_auditor).ok()?)?.try_into().ok()?;
    let (salt, inputs) = open_call_with_key(e, h_in, &key)?;
    Some((key, salt, inputs))
}

// ------------------------------------------------------------------ the browser surface

/// What the explorer accepts as a key. A viewing key or a spend key is 64 hex characters
/// (a `Word8`); a spend key file is the wallet's `{ "version": 2, "spend_key": "…" }`; a
/// transaction key or a call key is 32 bytes of hex.
#[derive(Serialize)]
struct KeyInfo {
    /// "viewing" (derived from a spend key when one was given) or "tx" / "call" (32 bytes).
    kind: String,
    /// The viewing key `nk` (never the spend key), when `kind` is "viewing".
    nk: Option<String>,
    pk: Option<String>,
    address: Option<String>,
    /// True when the input was a spend key; the caller should say the derivation happened locally.
    from_spend_key: bool,
}

fn parse_key_input(input: &str) -> Result<(String, Option<ViewingKey>, Option<[u8; 32]>, bool), String> {
    let s = input.trim();
    if s.starts_with('{') {
        let v: serde_json::Value = serde_json::from_str(s).map_err(|_| "not a key file")?;
        let sk = v.get("spend_key").and_then(|x| x.as_str()).ok_or("key file has no spend_key")?;
        let sk = word8_from_hex(sk).ok_or("spend_key must be 64 hex characters")?;
        return Ok(("viewing".into(), Some(ViewingKey::from_spend_key(&sk)), None, true));
    }
    let hex_str = s.trim_start_matches("0x");
    if hex_str.len() != 64 || !hex_str.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("a key is 64 hex characters (or a wallet key file)".into());
    }
    Ok(("hex".into(), None, None, false))
}

/// Interpret a pasted key as `kind` ("viewing", "spend", "tx", "call") and describe it.
#[wasm_bindgen]
pub fn key_info(input: &str, kind: &str) -> Result<String, String> {
    let (_, vk_from_file, _, from_file) = parse_key_input(input)?;
    let s = input.trim().trim_start_matches("0x");
    let info = match (vk_from_file, kind) {
        (Some(vk), _) => KeyInfo { kind: "viewing".into(), nk: Some(word8_to_hex(&vk.nk)), pk: Some(word8_to_hex(&vk.pk())), address: Some(vk.address()), from_spend_key: from_file },
        (None, "spend") => {
            let vk = ViewingKey::from_spend_key(&word8_from_hex(s).ok_or("spend key must be 64 hex characters")?);
            KeyInfo { kind: "viewing".into(), nk: Some(word8_to_hex(&vk.nk)), pk: Some(word8_to_hex(&vk.pk())), address: Some(vk.address()), from_spend_key: true }
        }
        (None, "viewing") => {
            let vk = ViewingKey { nk: word8_from_hex(s).ok_or("viewing key must be 64 hex characters")? };
            KeyInfo { kind: "viewing".into(), nk: Some(word8_to_hex(&vk.nk)), pk: Some(word8_to_hex(&vk.pk())), address: Some(vk.address()), from_spend_key: false }
        }
        (None, "tx") | (None, "call") => {
            key32_from_hex(s).ok_or("a transaction key is 32 bytes of hex")?;
            KeyInfo { kind: kind.into(), nk: None, pk: None, address: None, from_spend_key: false }
        }
        _ => return Err(format!("unknown key kind {kind}")),
    };
    serde_json::to_string(&info).map_err(|e| e.to_string())
}

#[derive(Serialize)]
struct OpenedNote {
    /// "received", "sent" or "tx_key": which key path opened the envelope.
    role: String,
    tx_key: String,
    pk: String,
    from: String,
    /// Units, as a decimal string.
    amount: String,
    asset: u32,
    time: u32,
    r: String,
    /// The nullifier this note publishes when spent, when the receiver's viewing key opened it.
    nullifier: Option<String>,
    /// True when the opened plaintext recomputes to the on-chain commitment (always, or the
    /// opening would have failed): stated so the page can say the row is verified.
    verified: bool,
}

fn opened(role: &str, key: [u8; 32], note: Note, nullifier: Option<Word8>) -> OpenedNote {
    OpenedNote {
        role: role.into(),
        tx_key: hex::encode(key),
        pk: word8_to_hex(&note.pk),
        from: word8_to_hex(&note.from),
        amount: note.amount.to_string(),
        asset: note.asset,
        time: note.time,
        r: word8_to_hex(&note.r),
        nullifier: nullifier.map(|n| word8_to_hex(&n)),
        verified: true,
    }
}

/// Try to open one envelope with a key. `key_kind` is "viewing" (64-hex `nk`), "spend"
/// (64-hex spend key, derived locally), "file" (a key file JSON) or "tx" (32-byte transaction
/// key). Returns the opened note as JSON, or `null` when the key does not open it.
#[wasm_bindgen]
pub fn open_note(cm_hex: &str, envelope_json: &str, key_kind: &str, key: &str) -> Result<String, String> {
    let cm = word8_from_hex(cm_hex).ok_or("commitment must be 64 hex characters")?;
    let e: EnvelopeHex = serde_json::from_str(envelope_json).map_err(|e| format!("envelope: {e}"))?;
    let env = Envelope::from_hex(&e).ok_or("envelope fields must be hex")?;
    let result = match key_kind {
        "tx" => {
            let k = key32_from_hex(key).ok_or("a transaction key is 32 bytes of hex")?;
            env.open_with_tx_key(&cm, &k).map(|n| opened("tx_key", k, n, None))
        }
        _ => {
            let vk = viewing_key_from(key_kind, key)?;
            if let Some((k, n)) = env.open_as_receiver(&cm, &vk) {
                let nf = vk.nullifier(&cm);
                Some(opened("received", k, n, Some(nf)))
            } else if let Some((k, n)) = env.open_as_sender(&cm, &vk) {
                // A change note is both sent and received; the receiver path above wins.
                Some(opened("sent", k, n, None))
            } else {
                None
            }
        }
    };
    match result {
        Some(r) => serde_json::to_string(&r).map_err(|e| e.to_string()),
        None => Ok("null".into()),
    }
}

fn viewing_key_from(key_kind: &str, key: &str) -> Result<ViewingKey, String> {
    let s = key.trim();
    match key_kind {
        "viewing" => Ok(ViewingKey { nk: word8_from_hex(s).ok_or("viewing key must be 64 hex characters")? }),
        "spend" => Ok(ViewingKey::from_spend_key(&word8_from_hex(s).ok_or("spend key must be 64 hex characters")?)),
        "file" => {
            let (_, vk, _, _) = parse_key_input(s)?;
            vk.ok_or_else(|| "not a key file".to_string())
        }
        other => Err(format!("unknown key kind {other}")),
    }
}

#[derive(Serialize)]
struct OpenedCall {
    role: String,
    call_key: String,
    salt: [u32; 4],
    inputs: Vec<u32>,
    /// `input_digest(salt, inputs) == h_in`: the transcript is what the proof committed to.
    faithful: bool,
}

/// Open a call's input transcript with the caller's viewing key ("viewing"/"spend"/"file"),
/// the auditor's viewing key ("auditor", same formats as viewing) or the per-call key ("call").
#[wasm_bindgen]
pub fn open_call(h_in_hex: &str, envelope_json: &str, key_kind: &str, key: &str) -> Result<String, String> {
    let h_in = word8_from_hex(h_in_hex).ok_or("h_in must be 64 hex characters")?;
    let e: CallEnvelopeHex = serde_json::from_str(envelope_json).map_err(|e| format!("call envelope: {e}"))?;
    let result = match key_kind {
        "call" => {
            let k = key32_from_hex(key).ok_or("a call key is 32 bytes of hex")?;
            open_call_with_key(&e, &h_in, &k).map(|(salt, inputs)| ("call_key", k, salt, inputs))
        }
        "auditor" => {
            let vk = viewing_key_from("viewing", key)?;
            open_call_as_auditor(&e, &h_in, &vk).map(|(k, s, i)| ("auditor", k, s, i))
        }
        kind => {
            let vk = viewing_key_from(kind, key)?;
            open_call_as_sender(&e, &h_in, &vk)
                .map(|(k, s, i)| ("caller", k, s, i))
                .or_else(|| open_call_as_auditor(&e, &h_in, &vk).map(|(k, s, i)| ("auditor", k, s, i)))
        }
    };
    match result {
        Some((role, k, salt, inputs)) => {
            let faithful = input_digest(salt, &inputs) == h_in;
            serde_json::to_string(&OpenedCall { role: role.into(), call_key: hex::encode(k), salt, inputs, faithful }).map_err(|e| e.to_string())
        }
        None => Ok("null".into()),
    }
}

/// The nullifier a viewing key's note with commitment `cm` publishes when spent.
#[wasm_bindgen]
pub fn nullifier_of(key_kind: &str, key: &str, cm_hex: &str) -> Result<String, String> {
    let vk = viewing_key_from(key_kind, key)?;
    let cm = word8_from_hex(cm_hex).ok_or("commitment must be 64 hex characters")?;
    Ok(word8_to_hex(&vk.nullifier(&cm)))
}
