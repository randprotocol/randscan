# Chain 18 readiness: the gas section, split authorisation, the memo viewer

**Date:** 2026-09-29. **Fullnode:** `feat/gas-chain18` at `d40fb94` (v0.6.6, constraint set 8,
circuits `18c2627`). **Live when written:** chain 17 (v0.6.3, genesis `d1afefc3…`), which the
explorer already indexes with no code change.

## What chain 18 changes for the explorer

The fullnode plan (`docs/superpowers/plans/2026-09-28-gas-phase1-phase2-chain18.md`, Task C) says:
"ship the clients (core re-vendored at cs8) and randscan's `randscan-viewing`/pv decoding before the
cut — a wallet on cs7 can prove nothing chain 18 accepts". The explorer never verifies a proof or
decodes public values itself — it reads the node's RPC — so "builds against the gas circuits" means
two things here:

1. **The memo viewer** (`crates/randscan-viewing`, the WebAssembly that opens envelopes in the
   browser) must still open what a cs8 wallet seals. The viewing-side files of the fullnode's zkVM
   (`hash.rs`, `notes.rs`, `viewing.rs`, `call_envelope.rs`, `address.rs`) are byte-identical
   between v0.6.3 and `d40fb94`: cs8 changes the STARK verifier key and adds a 35th public value,
   not the sponge, the key derivations, the note layout or the AEAD. Verified, not assumed: a
   throwaway generator built against the `d40fb94` crate re-sealed `tests/vectors.json` and
   `tests/fixtures/memo.json` from the same keys, note, salt and inputs; every deterministic value
   (sponge, `nk`/`pk`/`ovk`, the address, `cm`, the nullifier, `H_IN`) came out identical, and
   all 11 viewing tests pass on the freshly sealed envelopes. The wasm was rebuilt from the crate
   and is byte-identical to the committed one; the provenance pins now name `d40fb94`.
2. **The RPC surface** the explorer stores and shows. Chains 16, 17 and 18 added fields the
   explorer parsed tolerantly and dropped:
   - `rand_getLimits`: `envelope_bytes`, `hardening_v6`, `hc_auth`, `gas_price`, `byte_price`,
     `gas_metering`, `bundle_gas_limit`, `adjust_bps` — now on `ChainLimits` (stored whole in the
     existing `limits` JSONB, served on `/stats`, shown on the landing page).
   - `rand_status`: `hc_auth`, `gas_prices` (the tip's live prices under a dynamic section) —
     new stats columns (migration 011), served on `/stats`.
   - `tx_json.bundle`: `auth_commit`, `auth_proof_bytes` — new transaction columns (migration
     011), served in `TransactionDetail.bundle`, shown on the transaction page.
   - `rand_getSupply`: the four `vesting_*` fields — passed through on `/supply`.

   A call's *declared* gas limit is not on the RPC (it is `pv[GAS]` of the proof; the receipt has
   no such field), so the explorer cannot show it per call. Fullnode follow-up if wanted.

## Issue #64 (the envelope-format rule)

Fullnode #64: a wallet must not take the envelope format from an unauthenticated `rand_getLimits`
reply, because a lying RPC can make it seal 1 860-byte envelopes on a legacy chain and tag its
transactions. The explorer **seals nothing**: `randscan-viewing` only opens, and it reads the body
length off the envelope itself (`Note::BYTES` or `Note::BYTES + 512`). The explorer does display
`envelope_bytes` from `rand_getLimits`, which is a display of what its own node says, not a
decision. Nothing to do here; the rule is the wallets'.

## Tests

- `randscan-indexer`: `parses_the_chain_18_limits_status_and_bundle` (the doc's JSON shapes, the
  Phase 0 node-policy shape, a pre-v3 bundle, the JSONB round trip).
- `randscan-api/tests/mock_node.rs`: the mock node serves chain 18's limits, `hc_auth`, moved tip
  prices and bundles with `auth_commit`; `/stats` and `/transactions/:hash` are asserted.
- `randscan-api/tests/real_node.rs`: a chain-18 build gets chain 18's genesis flags
  (`--hardening-v6 --bundle-guest v3 --auth-guest --envelope-bytes 1860 --gas-price 100
  --byte-price 800 --bundle-gas-limit 20479 --gas-dynamic 10485760,262144,1250`); the explorer is
  checked against the node's `rand_getLimits`, `rand_status.gas_prices`, and a real v3 transfer's
  `auth_commit`, `auth_proof_bytes` and 1 860-byte envelopes.
- `randscan-db/tests/accounts.rs`: the migration list includes 11.

## Deploy

Backward compatible with chain 17 (every new field is `null`/`0` there), so the explorer can ship
before the cut, as the fullnode plan asks. Migration 011 is additive. At the cut the explorer
resets its chain tables itself when node E serves chain 18.
