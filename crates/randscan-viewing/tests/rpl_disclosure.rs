//! Task S1 item 5: a disclosure test that an RPL token's note (zUSD, registry index 3) opens with
//! its transaction key and reveals its `asset` word — the one place the RPC (and this crate)
//! discloses which asset a transfer moved (`rand_checkTransaction`, `docs/rpc.md`). The chain-14
//! hidden-asset bundle carries no public `asset` field anywhere else, so this is the only path:
//! a key the viewer pastes, opened in the browser, never the public transaction page.
//!
//! The note layout, the AEAD scheme and the AAD tags are unchanged by the 4-slot bundle (only the
//! bundle's own wire shape — nullifier/commitment/envelope counts — changed with chain 14; a
//! note's envelope is sealed exactly as before). This test seals an envelope itself (there is no
//! `randprotocol-zkvm`-produced fixture for an RPL note yet), using this crate's own public
//! primitives on the open side and the same AEAD/AAD construction `randprotocol_zkvm::viewing`
//! uses on the seal side, so it is a real ChaCha20Poly1305 round trip, not a stub.
//!
//! Rendering "12.50 zUSD" from the disclosed `{amount, asset}` is the explorer's job once the
//! asset resolves through the token registry (`GET /api/v1/tokens/:id`): `asset` is the registry
//! index, and `randscan_core::format_token_units(amount, token.decimals)` (tested in
//! `randscan-core`) turns the raw units into "0.0007", paired with the registry's `symbol`.

use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use randscan_viewing::*;

/// Seal `note`'s bytes under `key` for commitment `cm`, exactly as `randprotocol_zkvm::viewing`
/// seals an envelope's `body` (AAD `"rand-envelope-body"` ‖ the commitment, a 12-byte nonce
/// prefix). Only the `body` half is needed for a tx-key disclosure test — `kem_ct`/`to_receiver`/
/// `to_sender` are for the receiver/sender paths, which `tests/vectors.rs` already covers.
fn seal_body(note: &Note, cm: &Word8, key: &[u8; 32], nonce: [u8; 12]) -> String {
    let aad = [b"rand-envelope-body".as_slice(), &words_to_bytes(cm)].concat();
    let pt = words_to_bytes(&note.words());
    let ct = ChaCha20Poly1305::new(&Key::from(*key))
        .encrypt(&Nonce::from(nonce), Payload { msg: &pt, aad: &aad })
        .expect("seal");
    hex::encode([nonce.to_vec(), ct].concat())
}

#[test]
fn a_zusd_note_opens_with_its_transaction_key_and_discloses_asset_index_3() {
    // A note of registry index 3 (zUSD in this fixture): 0.0007 zUSD at 6 decimals (700 base
    // units) sent to some recipient `pk`, minted `from` the chain's RPL mint tag.
    let note = Note {
        pk: [10, 20, 30, 40, 50, 60, 70, 80],
        from: [
            u32::from_le_bytes(*b"rpl-"),
            u32::from_le_bytes(*b"mint"),
            0,
            0,
            0,
            0,
            0,
            0,
        ],
        amount: 700,
        asset: 3,
        time: 41,
        r: [1, 2, 3, 4, 5, 6, 7, 8],
    };
    let cm = note.commitment();
    let tx_key = [0x42u8; 32];
    let body_hex = seal_body(&note, &cm, &tx_key, [0x11; 12]);

    let env = EnvelopeHex {
        kem_ct: String::new(),
        to_receiver: String::new(),
        to_sender: String::new(),
        body: body_hex,
    };
    let env_json = serde_json::to_string(&env).unwrap();

    // The public transaction page never has the key: `open_note` with the wrong key (or none at
    // all) fails closed, and the transaction's own JSON (task S1 item 3) carries no `asset` word
    // for a plain transfer — this is the one path that discloses it.
    let wrong_key = "00".repeat(32);
    let closed = open_note(&word8_to_hex(&cm), &env_json, "tx", &wrong_key).unwrap();
    assert_eq!(closed, "null", "a wrong tx key must not open the note");

    let opened_json = open_note(&word8_to_hex(&cm), &env_json, "tx", &hex::encode(tx_key)).unwrap();
    let opened: serde_json::Value = serde_json::from_str(&opened_json).unwrap();
    assert_eq!(opened["role"], "tx_key");
    assert_eq!(opened["asset"], 3, "asset index 3 (zUSD) is disclosed by the key, not the public page");
    assert_eq!(opened["amount"], "700");
    assert_eq!(opened["verified"], true, "the opened plaintext recomputed to the on-chain commitment");

    // The commitment binds the plaintext: a key lifted onto another commitment opens nothing.
    let mut other_cm = cm;
    other_cm[0] ^= 1;
    assert_eq!(open_note(&word8_to_hex(&other_cm), &env_json, "tx", &hex::encode(tx_key)).unwrap(), "null");
}
