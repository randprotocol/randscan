//! Task 10 (address-sharing spec, 2026-09-26): the memo field riding in the note body's
//! trailing 512 bytes (fullnode's `viewing::seal_with_memo`, `body_parts`, `memo_text`).
//!
//! `tests/fixtures/memo.json` is produced by a throwaway test in the fullnode worktree
//! (`randprotocol-zkvm`, `seal_note_as(EnvelopeFormat::Memo | Legacy, ..)` and
//! `viewing::Envelope::seal_with_memo` directly for the corrupt case) — the only way to get a
//! real sealed envelope, since this crate is a from-scratch re-implementation with no sealing
//! side of its own. Three fixtures, one receiver viewing key:
//! - `memo`: a 624-byte body (note ‖ memo field), memo text "fixture memo".
//! - `legacy`: a 112-byte body (note alone, no memo field) — the pre-memo wire shape.
//! - `corrupt`: a 624-byte body whose memo field's `len` header is 600 (> the 510-byte max) —
//!   the note must still open; the memo must come back `None`.

use randscan_viewing::*;
use serde_json::Value;

fn fixtures() -> Value {
    serde_json::from_str(include_str!("fixtures/memo.json")).unwrap()
}

fn s<'a>(v: &'a Value, k: &str) -> &'a str {
    v[k].as_str().unwrap()
}

#[test]
fn a_memo_envelope_opens_and_discloses_its_memo() {
    let v = fixtures();
    let f = &v["memo"];
    let cm = s(f, "cm");
    let env = f["envelope"].to_string();
    let vk = s(f, "viewing_key");

    let opened_json = open_note(cm, &env, "viewing", vk).expect("open_note should not error");
    assert_ne!(opened_json, "null", "a 624-byte (note + memo) body must open");
    let opened: Value = serde_json::from_str(&opened_json).unwrap();
    assert_eq!(opened["role"], "received");
    assert_eq!(opened["verified"], true);
    assert_eq!(opened["memo"], "fixture memo");
}

#[test]
fn a_legacy_envelope_opens_with_no_memo() {
    let v = fixtures();
    let f = &v["legacy"];
    let cm = s(f, "cm");
    let env = f["envelope"].to_string();
    let vk = s(f, "viewing_key");

    let opened_json = open_note(cm, &env, "viewing", vk).expect("open_note should not error");
    assert_ne!(opened_json, "null", "a 112-byte (note-only) body must still open");
    let opened: Value = serde_json::from_str(&opened_json).unwrap();
    assert_eq!(opened["role"], "received");
    assert!(opened["memo"].is_null(), "a legacy (pre-memo) body has no memo field at all");
}

#[test]
fn a_corrupted_memo_field_never_costs_the_payee_the_note() {
    let v = fixtures();
    let f = &v["corrupt"];
    let cm = s(f, "cm");
    let env = f["envelope"].to_string();
    let vk = s(f, "viewing_key");

    let opened_json = open_note(cm, &env, "viewing", vk).expect("open_note should not error");
    assert_ne!(opened_json, "null", "a malformed memo field must not fail the note open");
    let opened: Value = serde_json::from_str(&opened_json).unwrap();
    assert_eq!(opened["role"], "received");
    assert_eq!(opened["verified"], true);
    assert!(opened["memo"].is_null(), "len=600 exceeds the 510-byte max: the field is malformed, so memo is null");
}
