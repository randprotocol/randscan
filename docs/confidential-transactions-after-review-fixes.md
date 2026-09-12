# Confidential transactions after the fullnode review fixes (M1–M4)

How the September 2026 fullnode review items change what a confidential transaction (a `call`
carrying a zkVM proof) looks like on chain, and what that means for RandScan. The items are the
Medium findings of `concerns/fullnode-review-2026-09-10.md` in the fullnode repository, merged
as commits `1d24bf9`, `d5143a6`, `a44d3f4` on 2026-09-10. They are review findings, not the
circuits roadmap milestones that share the M-numbers.

## What a confidential transaction is

A `call` transaction names a deployed program, carries a proof (up to `MAX_PROOF_BYTES` =
1 MiB, typically about 0.9 MB) and a public recipient list of at most 8 addresses. Every
validator verifies the proof (`Machine::verify`) before voting. The program's eight public output
words become the receipt: word 0 is the effect kind, word 1 an index into the recipient list,
words 2–3 the amount. Inputs, the proof itself and the cycle count never leave the prover.

RandScan indexes a call from `shrugg_getBlockByHeight` (program id, `proof_len`, `recipients`)
and fetches its receipt with `shrugg_getReceipt` (tier, outputs, effect). Deploys store the
program's `code_hash` from `shrugg_getProgram`, which since the zkVM fork is the in-circuit
Poseidon2 digest and no longer equals the content id.

## M1 — block size and transaction count are consensus rules

Before: the 4 MiB / 2000-transaction budget only limited a node's own proposals. A Byzantine
leader could propose up to the 16 MiB gossip cap and every replica would verify and execute it.

After: `apply_block` rejects a block whose transaction bytes exceed `MAX_BLOCK_BYTES` (4 MiB)
or whose count exceeds `MAX_BLOCK_TXS` (2000). Because a proof is about 0.9 MB, **a block holds
at most four confidential calls**; transfers are tiny, so a block can be 2000 transfers or four
calls, not both. Calls that do not fit wait in the mempool for the next block.

The same hardening wave made the node reject cheap-before-expensive everywhere: a call whose fee
is below `CALL_BASE` (1,000,000 units = 0.001 SHRUGG) is refused before proof verification, and
the mempool checks duplicates, pool capacity and replacement pricing before it clones the ledger
to validate. The RPC body limit was raised to `2 × MAX_PROOF_BYTES + 256 KiB` so a call with a
near-maximum proof can be submitted as hex JSON, and `shrugg_estimateFee ["call", tier]` now
rejects tiers outside 10, 12, …, 20 instead of silently truncating.

RandScan: block pages and `GET /transactions?height=` serve full 2000-transaction blocks (covered
by the mock-node integration test). Calls per block are naturally few; `proof_len` on the detail
page shows why. Minimum fees per tier: 0.001 SHRUGG at tier 10, +0.0001 per tier step, so
0.0015 SHRUGG at tier 20.

## M2 — lock promises are not durable (documented, not fixed)

Three liveness choices weaken the HotStuff locking rule: a vote is allowed when the locked block
is unknown, the lock can fall back to the head QC, and `high_qc`/`locked_qc` are not restored
after a restart. With the C1 fix (three consecutive-view commit rule) these are no longer
exploitable by timing alone, but the safety margin under Byzantine faults is thinner than the
whitepaper's. A protocol decision is pending.

Effect on confidential transactions: none specific. RandScan only reads **committed** blocks
(`shrugg_getBlockByHeight` serves nothing else), so what the explorer shows is final under the
same guarantee as every other transaction. If a re-org ever happened, the indexer's
parent-hash check rewinds and re-indexes.

## M3 — sync serving is byte-budgeted

Before: a peer could request 100 committed blocks and a batch of proof-heavy blocks could exceed
the 10 MiB request-response cap, so the response was undeliverable and the requester retried the
same range forever.

After: sync responses are capped at `SYNC_MAX_BYTES` (8 MiB) and always contain at least one
block. With four 0.9 MB calls per block, a response may carry only two blocks; a syncing node
therefore catches up more slowly across confidential-heavy ranges but never stalls. The same
commit made `committed_block` refuse to serve a block whose call receipt is missing from
storage (the receipts column family would be damaged), rather than sending a short receipt list
that peers reject.

RandScan: node E is an observer that syncs from the validators; a proof-heavy history means the
node, and so the explorer, trails the head for longer during a resync. `GET /health` reports the
lag; `stats.node_syncing` mirrors `shrugg_status.syncing`. The indexer itself reads over RPC and
is unaffected by the sync protocol.

## M4 — wallet stale-nonce race (not fixed)

`shrugg call` proves locally (seconds to minutes depending on tier), then signs with the
**committed** account nonce from `shrugg_getAccount`. Two sends in quick succession reuse a nonce;
the mempool rejects or fee-replaces the first, and the user waits 60 s for a transaction that will
never commit. Confidential calls are the most exposed because proving takes long enough that a
transfer sent in between is easy. The clean fix is a mempool-aware `next_nonce` RPC, deferred
because it changes the RPC surface.

RandScan: shows the committed nonce from the node on the account page and the mempool size in
stats. Nothing to change now; if `next_nonce` lands, the account page can show the pending nonce
and the explorer's own integration test against a real node should exercise a double-send.

## Related, not M-numbered

- Proof verification still runs on the consensus event loop (~20 ms warm, ~2 s on a cold verifier
  key). The fee floor and a bounded FIFO verifier-key cache are mitigations; moving verification
  off the loop is the real fix (H3). Blocks with several cold-key calls can delay a view.
- Bridge transactions (`bridge_attest`, `bridge_burn`) arrived with the same main and are
  indexed by RandScan; a bridge attestation is at most 16 KiB and is not confidential.

## What the explorer shows for a call today

| where | fields |
|---|---|
| lists | kind `call`, program id, fee; `to`/`amount` null |
| detail | `proof_len`, `recipients`, `chain_id`, `receipt` |
| receipt | `tier`, `outputs[8]`, `effect { to, amount }` or null, block height and index |

Viewing keys, note envelopes and nullifiers are not exposed by the node yet, so shielded content
cannot be opened in the explorer; see the "Later" section of
`docs/superpowers/specs/2026-09-10-accounts-and-api-keys-design.md`.

## zkVM constraint set 4 (fullnode `dbea18c`, 2026-09-11)

The fullnode re-synced its vendored zkVM to research milestone 4.1 (`f06446a`, merged as
`dbea18c`): `read_input` is now bound to a salted input commitment `H_IN`, published as eight
more public values, and `Proof` gained an `input_log_height` field. This changes proof bytes and
the verifier key, not the RPC surface: no RPC file changed, and `shrugg_getBlockByHeight`,
`shrugg_getReceipt` and `shrugg_getProgram` return the same shapes as before. `H_IN` is not
exposed by the node, so the explorer has nothing new to show.

What matters to RandScan is the hard fork: proofs made under constraint set 3 do not verify
under set 4, so a node built from this commit truncates any chain holding an older call and the
operator starts a new chain id. The explorer already handles that unattended (chain id or
genesis mismatch, or the head dropping below the indexed height, truncates only the chain tables
and re-indexes; users and API keys survive). Node E has run `dbea18c` on chain 5 since
2026-09-11.

Verified against this build on 2026-09-12: the real-node integration test
(`crates/randscan-api/tests/real_node.rs`) now also deploys the `private_payment` guest, proves
and submits a call through the wallet CLI, and checks that the explorer's transaction, receipt,
program `code_hash` and account balance equal the node's. Set `SHRUGG_NODE_BIN` to the node
binary; the wallet is picked up from the sibling `shrugg` binary or `SHRUGG_CLI`.
