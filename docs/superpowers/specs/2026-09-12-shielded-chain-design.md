# RandScan on the shielded chain — design

Date: 2026-09-12. Status: implemented with this spec. Adapts the explorer to the fullnode's
shielded-pool plan (phases S1–S3, `fullnode/docs/superpowers/specs/2026-09-11-shielded-pool-design.md`,
branches `shielded-s1/s2/s3`) and to the zkVM milestone line (M4.1 on `main`, M4.2 / constraint
set 5 on `revendor-cs5`). Supersedes the account-chain parts of
`2026-09-10-shrugg-retarget-design.md`; accounts/API keys (`2026-09-10-accounts-and-api-keys-design.md`)
are unchanged.

## 1. What changed on the node

- **No accounts.** `shrugg_getBalance`, `shrugg_getAccount`, `shrugg_getAssetBalance` are gone.
  A transaction has no `from`, `nonce` or signature.
- **Every transaction is `{ hash, chain_id, bundle, action }`.** `bundle` is the shielded
  2-in-2-out transfer that pays the fee: `anchor`, `nullifiers[2]`, `commitments[2]`, `fee`
  (units, JSON integer), `burn`, `asset` (0 = SHRUGG, else the bridge registry index), `time`,
  `proof_len`, `envelope_len[2]`. `bundle` is `null` for validator-signed actions (`mint`,
  `unbond`, `withdraw`).
- **`action.kind`** is one of `none` (plain transfer), `mint`, `deploy`, `call`, `bond`,
  `unbond`, `withdraw`, `bridge_attest`, `bridge_burn`; the fields are in
  `fullnode/docs/rpc.md` (`shrugg_getTransaction`). `bridge_burn` carries a second bundle
  (`asset_bundle`).
- **Receipts** lose `effect` and gain `h_in`. Programs lose `deployer`.
  `shrugg_getCallEnvelope` serves a call's sealed input transcript.
- **New reads**: `shrugg_getCommitments(from_index, limit)`, `shrugg_getNullifiers(from_height,
  limit)`, `shrugg_getAnchor`, `shrugg_getWitness`, `shrugg_getTreeInfo`, `shrugg_getBridgeState`,
  `shrugg_getAssets`, `shrugg_getBridgeBurn`; on S2 also `shrugg_getEpoch` and `shrugg_getSupply`.
- **Validators** (`shrugg_getValidators`) are the register: `address`, `stake` (decimal string),
  and on S2 `pending[]`, `rewards`, `payout`, `nonce`, `active`. On the S3 branch alone the row
  is `{ address, stake, rewards: number }`. The explorer accepts both.
- **`shrugg_status`** gains `notes`, `nullifiers`, `tree_root`, `hc_bundle` (and on S2
  `active_validator`).
- **Plan M** (zkVM 4.1/4.2): only proof bytes change. Constraint set 5 raises a call proof to
  about 1.2 MB and `MAX_PROOF_BYTES` to 2 MiB; a bundle proof is ~300 KB under the test profile.
  The explorer stores proof sizes as reported and assumes nothing about them.

Each phase is a hard fork; S1 is a new chain. The explorer's chain-switch reset (chain id or
genesis mismatch truncates chain tables, keeps users/API keys) covers the switch, but the
**database schema of the chain tables is rebuilt by migration 005**, so the new explorer build
must go live together with the shielded node: it cannot index an account-chain node.

## 2. Explorer data model (migration 005)

Chain tables are dropped and recreated (they hold no data worth keeping across the fork).
`users`, `sessions`, `api_keys`, `password_resets`, `node_geo` are untouched.

`transactions`: `hash, block_hash, height, tx_index, chain_id, timestamp_ms, kind, fee`
plus the bundle (`has_bundle, anchor, nullifier_1, nullifier_2, commitment_1, commitment_2,
burn, asset, bundle_time, proof_len, envelope_len_1, envelope_len_2`) and one set of
action columns shared by kinds: `program_id, words_len, call_proof_len, input_envelope_len`
(deploy/call), `amount` (mint, bond, unbond, withdraw, bridge_attest, bridge_burn), `cm, minter`
(mint), `validator, registered, action_nonce` (staking), `attestation_len, recipient, asset_index,
note_time` (bridge_attest; `asset_index` also bridge_burn's asset), `relayer_fee, to_chain,
bridge_to, asset_bundle JSONB` (bridge_burn). `kind` stores the explorer kind; unknown node
kinds keep the node's tag verbatim (served as `other`), as before.

`nullifiers (nullifier PK, tx_hash, height, tx_index)`: both nullifiers of every bundle,
including a burn's asset bundle. `notes (leaf_index PK, cm UNIQUE, height, tx_hash NULL)`: the
commitment tree, paged from `shrugg_getCommitments` (cursor `indexer_state.next_leaf`); `tx_hash`
is filled when the commitment appears in an indexed transaction (bundle outputs, mint). Genesis
deposit notes, withdraw and bridge deposit notes stay unlinked (their commitment is not on the
wire).

`receipts`: `tx_hash, program, tier, outputs, height, tx_index, h_in`. `programs`: no
`deployer`. `validators`: `address, stake, rewards, pending JSONB, payout, nonce, active,
sort_index`. `network_stats`: `total_accounts` becomes `notes` and `nullifiers`; adds
`tree_root, hc_bundle, epoch, epoch_blocks, pool_value` (nullable; from `shrugg_getSupply`
when served). `indexer_state` adds `next_leaf`.

## 3. API contract (`/api/v1`)

Removed: `GET /accounts/:address` and `/accounts/:address/transactions` answer **410 Gone**
`{ "error": "no_accounts", ... }` so old integrations get a clear message.

`TransactionSummary`: `hash, height, block_hash, tx_index, kind, fee, timestamp_ms, has_bundle,
program, validator, amount, asset_index`. `kind` is `transfer | mint | deploy | call | bond |
unbond | withdraw | bridge_attest | bridge_burn | other`. `amount` is the public amount of a
deposit or a staking action (units of SHRUGG, or of the bridged asset for `bridge_attest` /
`bridge_burn`); null for `transfer`, `deploy`, `call`.

`TransactionDetail` adds `chain_id`, `bundle` (object or null: `anchor, nullifiers[2],
commitments[2], fee, burn, asset, time, proof_len, envelope_len[2]`), `words_len`,
`call_proof_len`, `input_envelope_len`, `receipt` (`tx, program, tier, outputs, height, index,
h_in`), `cm`, `minter`, `registered`, `action_nonce`, `attestation_len`, `recipient`,
`note_time`, `relayer_fee`, `to_chain`, `bridge_to`, `asset_bundle` (same shape as `bundle`).

Filters: `GET /transactions?kind&height&validator&program`. `sender` is gone.

New: `GET /notes?page&limit` (`{ leaf_index, cm, height, tx_hash }`, newest first),
`GET /notes/:cm`, `GET /nullifiers/:nf` (`{ nullifier, tx_hash, height, tx_index }`),
`GET /bridge` (the node's `shrugg_getBridgeState`, cached by the indexer),
`GET /supply` (the node's `shrugg_getSupply`, or 404 on a node without it).

`Validator`: `address, stake, rewards, pending[], payout, nonce, active, share_percent,
blocks_proposed, last_proposed_height, last_proposed_timestamp_ms, sort_index`. `share_percent`
is over the active set's stake. `ProgramSummary` loses `deployer`.

`NetworkStats`: `total_accounts` is gone; adds `notes, nullifiers, tree_root, hc_bundle, epoch,
epoch_blocks, pool_value`; `total_supply` is the supply audit's total (or `"0"` when the node
has no `shrugg_getSupply`). `current_leader` is `active validators[view mod n]`.

Search: a 64-hex query also matches a note commitment and a nullifier; a base58 query matches a
validator only.

## 4. Frontend

- Transactions list: hash, kind, height, "action" cell (program / validator / amount), fee, age.
  No from/to columns.
- Transaction detail: overview; a **Bundle** panel (or "no bundle: signed by validator …");
  an action panel per kind; the receipt panel with `h_in` and whether an input transcript was
  published.
- `/account/[address]` becomes a short page explaining that the chain has no accounts and
  linking to the validator page when the address is one.
- Validators: register columns (stake, rewards, pending, active); detail shows payout and nonce.
- New `/notes` page (tree size, spent count, latest leaves). Dashboard cards: notes and
  nullifiers replace accounts; tree root and epoch in the secondary row.
- Search help text and kind labels updated.

## 5. Out of scope

Viewing-key scanning (server-side decryption of envelopes for a user-supplied `nk` or per-call
key) is the next design; it needs envelopes stored, which this spec does not do. Slashing,
recursion and the aggregation design are node-side and invisible here.
