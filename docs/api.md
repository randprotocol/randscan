# RandScan API for early users

RandScan indexes the Rand Protocol SHRUGG chain and serves what it has indexed over a plain HTTPS
JSON API and a WebSocket feed. This guide is for people who want to read the chain from a script,
a bot, an exchange backend, or a wallet without running their own `shrugg-node`.

Base URL: `https://randscan.org/api/v1`
WebSocket: `wss://randscan.org/ws`

Public read endpoints need no key. Anonymous traffic is limited to 60 requests per minute per IP.
For more, create a free account at https://randscan.org/signup and an API key at
https://randscan.org/dashboard: keyed requests get 600 requests per minute. Use the WebSocket feed
instead of tight polling when you need to react to new blocks.

## What is public on a shielded chain

SHRUGG is a fully shielded chain (fullnode `docs/shielded.md`). Every balance is a set of notes
in a commitment tree, and every transaction is a **bundle** (two spent notes, two created notes,
a public fee and a STARK proof) plus an optional public **action**. There are **no accounts, no
balances, no senders and no recipients** anywhere in this API. What the explorer can show is:

- every block, and every transaction's bundle *as published*: the anchor, two nullifiers, two
  commitments, the fee, the burn, the asset index, the target height, and the proof and envelope
  sizes;
- the action a transaction carried and its public fields: a deploy's program, a call's program
  and receipt, a validator's bond / unbond / withdraw amounts, a faucet mint's amount, a bridge
  deposit's amount and a bridge burn's destination — these are the only amounts ever public;
- the commitment tree leaf by leaf, the nullifier set, the validator register (stake, rewards,
  unbonding queue), the supply audit, and the bridge's public state.

To see a balance or a transfer's parties you need the owner's viewing key and a wallet; the
explorer has neither. A future release will let you paste a viewing key to open your own rows.

## Conventions

- **Amounts are strings of units.** 1 SHRUGG = 1,000,000,000 units (9 decimals). Amounts are
  unsigned 128-bit integers and are always serialised as decimal strings, never as JSON numbers.
  `"10000000000"` is 10 SHRUGG. An amount of a **bridged asset** (`asset_index` > 0) is in that
  asset's own smallest unit, which the source chain defines.
- **Timestamps** are `timestamp_ms`: Unix milliseconds, set by the block proposer.
- **Hashes, program ids, commitments, nullifiers and anchors** are 64 lowercase hex characters,
  no `0x` prefix. Inputs accept an optional `0x` and uppercase.
- **Validator addresses** are base58 strings of 32 to 44 characters (for example
  `2nRdFChBXRmKoe2sQE3ZYDzvdg53QmBZJJ9iweY7hk1v`). **Shielded addresses** (`shrugg1…`, about
  1,700 characters) appear only as a bridge deposit's recipient and a validator's payout address.
- **Heights** are decimal integers starting at 0. Only committed blocks are indexed; there is no
  finality window to wait for. A block that appears in the API is final.
- **Pagination.** List endpoints take `page` (from 1) and `limit` (1 to 100, default 20) and return
  `{ "data": [...], "pagination": { page, limit, total, total_pages, has_next, has_prev } }`.
  Lists are newest first.
- **Errors** are JSON with an HTTP status:

  ```json
  { "error": "not_found", "message": "block not found", "code": "NOT_FOUND" }
  ```

  | status | `error` | when |
  |---|---|---|
  | 400 | `bad_request` | malformed hash, unknown `kind`, and so on |
  | 404 | `not_found` | the block, transaction, note, nullifier, validator or program does not exist |
  | 410 | `no_accounts` | `/accounts/...`: this chain has no accounts |
  | 500 | `internal_error` | something failed on our side; retry with backoff |

- CORS is open, so browsers can call the API directly. Responses are gzip compressed when the
  client accepts it.

## Quick start

```bash
# Is the explorer healthy and caught up with the chain?
curl -s https://randscan.org/api/v1/health

# Current chain height, tree size, validator count, supply audit
curl -s https://randscan.org/api/v1/stats

# A transaction by hash
curl -s https://randscan.org/api/v1/transactions/ba03893d4b5b4f03ab8ab85aa9e8df4b46102986f15ef56653f7450b98bdba34

# Which transaction spent a note (by nullifier), or created one (by commitment)
curl -s https://randscan.org/api/v1/nullifiers/8c04…d1
curl -s https://randscan.org/api/v1/notes/2a9f…07
```

Python:

```python
import requests

API = "https://randscan.org/api/v1"
UNITS = 10**9

page = requests.get(f"{API}/transactions", params={"kind": "mint", "limit": 50}, timeout=10).json()
for tx in page["data"]:
    # A mint is one of the few actions with a public amount; a transfer has none.
    print(tx["height"], tx["hash"][:12], int(tx["amount"]) / UNITS, "SHRUGG minted")

validators = requests.get(f"{API}/validators", timeout=10).json()
for v in validators:
    print(v["address"], int(v["stake"]) / UNITS, "SHRUGG staked", "active" if v["active"] else "inactive")
```

## Endpoints

### Health and network

| method and path | returns |
|---|---|
| `GET /health` | explorer status and indexer lag |
| `GET /stats` | network statistics |
| `GET /supply` | the node's supply audit (404 on a node that does not serve it) |
| `GET /bridge` | the bridge's public state and asset registry |
| `GET /nodes` | the explorer node and its peers, with geolocation |

`GET /health`:

```json
{
  "status": "healthy",
  "version": "0.3.0",
  "database": true,
  "indexer": { "connected": true, "synced": true, "current_height": 20147, "node_height": 20147, "lag": 0 }
}
```

`status` is `healthy` when the database is up and the indexer is within 2 blocks of the node,
`degraded` when the indexer is behind or disconnected, and `unhealthy` when the database is
unreachable. Check `indexer.lag` before trusting "latest" data during an outage.

`GET /stats`:

```json
{
  "chain_id": 7, "symbol": "SHRUGG", "decimals": 9,
  "height": 20147, "view": 23263,
  "total_transactions": 40, "notes": 91, "nullifiers": 62,
  "validator_count": 5, "active_validator_count": 4, "total_stake": "400000000000000",
  "total_supply": "1400100000000000", "pool_value": "1000097000000000",
  "program_count": 1, "avg_block_time_ms": 1004.35,
  "peer_count": 4, "mempool_size": 0, "node_syncing": false,
  "faucet": true, "confidential": true,
  "current_leader": "F6rYLexPhyMmwPNqbEmyyp5FiTmtQqDgZyqScUqYY4F6",
  "tree_root": "6b1d…c4", "hc_bundle": "f07a…19",
  "epoch": 20, "epoch_blocks": 1000,
  "updated_at": "2026-09-12T05:20:38.591804+00:00"
}
```

`notes` is every note the chain has ever created (leaves of the commitment tree) and
`nullifiers` every note it has ever spent. `total_stake` sums the *active* set. `total_supply`
and `pool_value` come from the node's supply audit and are `"0"` / `null` on a node that does
not serve it. `tree_root` is the current commitment-tree root and `hc_bundle` the digest of the
bundle guest every proof on this chain is checked against. `current_leader` is the validator
expected to propose the next block.

`GET /supply` returns the audit verbatim (all units strings):

```json
{ "height": 1998,
  "genesis_deposited": "…", "genesis_staked": "…", "faucet_minted": "…",
  "withdraw_deposited": "…", "fees_paid": "…", "burned": "…",
  "pool_value": "…", "register_total": "…", "total_supply": "…", "invariant_holds": true }
```

Note values are hidden, but every crossing of the pool boundary is public, so these are exact.
`invariant_holds` false would be a chain bug, never a legitimate state.

`GET /bridge`:

```json
{
  "enabled": true, "emitter": "01…", "emitters": { "2": "02…" },
  "guardian_set_index": 0, "guardians": ["aabb…"],
  "burn_sequence": 1, "next_index": 2,
  "assets": [{ "index": 1, "chain": 2, "token": "aaaa…", "asset_id": "…" }]
}
```

`assets[].index` is the `asset_index` a bridged note carries; index 0 is SHRUGG. There are no
balances: bridged value is notes. A chain without a bridge reports `{ "enabled": false }`.

`GET /nodes` returns an array of:

```json
{
  "peer_id": "12D3KooW...", "ip": "167.172.65.63", "port": 30303,
  "connected_secs": 86400, "is_self": false, "role": "peer",
  "geo": { "lat": 1.29, "lon": 103.85, "city": "Singapore", "region": null, "country": "Singapore", "country_code": "SG", "org": "DigitalOcean" }
}
```

`role` is `validator` or `observer` for the explorer's own node and `peer` for remote peers, whose
role is not known. `geo` is `null` for private or unresolved addresses.

### Blocks

| method and path | returns |
|---|---|
| `GET /blocks?page&limit&proposer` | paginated `BlockSummary`, newest first |
| `GET /blocks/latest?limit=10` | array of the newest `BlockSummary` (limit 1 to 100) |
| `GET /blocks/:id` | `BlockDetail`; `:id` is a height or a block hash |

`BlockSummary`:

```json
{
  "hash": "b171d72a96923673b6d96793530f942c07d62fa9571315c73187e8b4556215eb",
  "height": 20147, "view": 23256,
  "parent": "5460c3ca5899aa2880986f06b48e7c8d5781d26048b30149cc9d1881905b01f1",
  "proposer": "2nRdFChBXRmKoe2sQE3ZYDzvdg53QmBZJJ9iweY7hk1v",
  "timestamp_ms": 1789017624229, "tx_count": 0, "justify_view": 23255
}
```

`BlockDetail` adds `tx_root`, `state_root`, and `transactions` (an array of `TransactionSummary`
in block order). `view` and `justify_view` are HotStuff consensus views; most integrations only
need `height`.

### Transactions

| method and path | returns |
|---|---|
| `GET /transactions?page&limit&kind&height&validator&program` | paginated `TransactionSummary`, newest first |
| `GET /transactions/latest?limit=10` | array of the newest `TransactionSummary` |
| `GET /transactions/:hash` | `TransactionDetail` |

Filters: `kind` is one of `transfer`, `mint`, `deploy`, `call`, `bond`, `unbond`, `withdraw`,
`bridge_attest`, `bridge_burn`; `height` restricts to one block; `validator` to the staking
actions (and mints) of one validator address; `program` to the deploy and calls of one program.

`TransactionSummary`:

```json
{
  "hash": "ba03893d4b5b4f03ab8ab85aa9e8df4b46102986f15ef56653f7450b98bdba34",
  "height": 19340,
  "block_hash": "890715533e41e9ddbeac0edc0d2722847248e3affba70816fff31ca54bc1b80f",
  "tx_index": 0,
  "kind": "transfer", "fee": "1000000", "timestamp_ms": 1789009705401,
  "has_bundle": true,
  "program": null, "validator": null, "amount": null, "asset_index": null
}
```

There is no sender, recipient, nonce or (for a transfer) amount: a stored transfer has no such
field. `has_bundle` is false for the three validator-signed actions (`mint`, `unbond`,
`withdraw`), which carry no bundle and pay no fee. The kinds and which fields they fill:

| kind | what it is | `amount` | other summary fields | detail-only fields |
|---|---|---|---|---|
| `transfer` | a plain shielded transfer (the node's `none` action) | null | | `bundle` |
| `mint` | testnet faucet deposit, signed by a validator | SHRUGG units of the new note | `validator` = the minter | `cm` |
| `deploy` | a zkVM program deployment, paid by the bundle | null | `program` = the new program id | `words_len` |
| `call` | a confidential call, paid by the bundle | null | `program` | `call_proof_len`, `input_envelope_len`, `receipt` |
| `bond` | stake leaving the pool into a validator's register entry | SHRUGG units | `validator` | `registered` (a first-time registration) |
| `unbond` | stake moved to the unbonding queue (validator-signed) | SHRUGG units | `validator` | `action_nonce` |
| `withdraw` | released stake and rewards deposited as a new note (validator-signed) | SHRUGG units | `validator` | `action_nonce` |
| `bridge_attest` | a guardian-signed inbound message depositing a bridged note | bridged units (null for a guardian-set rotation) | `asset_index` | `attestation_len`, `recipient`, `note_time` |
| `bridge_burn` | a bridged asset burned to another chain | bridged units | `asset_index` | `relayer_fee`, `to_chain`, `bridge_to`, `asset_bundle` |
| `other` | a kind newer than this explorer build | null | | none |

`TransactionDetail` adds `chain_id`, `bundle` and the per-kind fields above (null when they do
not apply). The bundle is the transaction's public face:

```json
"bundle": {
  "anchor": "6b1d…c4",
  "nullifiers": ["8c04…d1", "5e77…20"],
  "commitments": ["2a9f…07", "b310…88"],
  "fee": "1000000", "burn": "0", "asset": 0, "time": 5,
  "proof_len": 302857, "envelope_len": [1380, 1380]
}
```

`anchor` is the tree root the proof was made against; `nullifiers` mark the two spent notes
(a dummy input still publishes one, so every bundle looks alike); `commitments` are the two notes
created; `burn` is value leaving the pool into the action (a bond, a bridge burn), `asset` the
asset the bundle balances (0 = SHRUGG) and `time` the height the sender targeted. The proof and
the two encrypted envelopes are reported by size only; nothing in a bundle names a sender,
receiver or amount. A `bridge_burn` carries a second bundle in `asset_bundle` (same shape) that
burns the bridged asset; `bridge_to` is a 32-byte hex address on `to_chain` (1 Rand, 2 Ethereum,
3 BSC, 4 Tron, 5 Solana; 20-byte addresses left-padded with zeros).

The receipt of a confidential call:

```json
{
  "tx": "b08244b044e8d719aa4e2c1bd22a92914924ae7a6cd41c4c363a608e211f09b0",
  "program": "675adeea7e4242d8dc48bf56faedb7bea14a4f832d7c8a973f942fa7dd850065",
  "tier": 14, "outputs": [1, 0, 200, 0, 0, 0, 0, 0],
  "height": 105, "index": 0, "h_in": "9c0e…7f"
}
```

`outputs` are the program's eight public output words; they no longer move value (the old
effect kind 1 is gone with the accounts). `h_in` is the proof's salted commitment to the call's
private inputs; `input_envelope_len` says whether the caller published a sealed transcript of
those inputs (null when not). The transcript opens only for the caller's viewing key, the
per-call key, or the auditor the caller named; the node serves the bytes
(`shrugg_getCallEnvelope`), the explorer only their size. Proof sizes depend on the zkVM
constraint set the chain runs (about 1.2 MB per call proof at constraint set 5); the explorer
reports what the node reports.

### Notes and nullifiers

| method and path | returns |
|---|---|
| `GET /notes?page&limit` | paginated leaves of the commitment tree, newest first |
| `GET /notes/:id` | one leaf by commitment (64 hex) or by leaf index (decimal) |
| `GET /nullifiers/:nf` | the transaction that published a nullifier |

```json
{ "leaf_index": 40, "cm": "2a9f…07", "height": 37, "tx_hash": "4f2c…e7" }
{ "nullifier": "8c04…d1", "tx_hash": "4f2c…e7", "height": 41, "tx_index": 0 }
```

`tx_hash` of a note is the transaction whose bundle or mint carried the commitment, and `null`
for a genesis deposit, a validator's withdraw deposit or a bridge deposit (their commitment is
computed by the chain and not on the wire). The leaf index is what a wallet uses to ask the node
for a Merkle witness. A wallet that wants to detect payments scans envelopes with its viewing
key; the explorer cannot do that for you.

### Accounts

`GET /accounts/:address` and `GET /accounts/:address/transactions` answer **410**
`{ "error": "no_accounts" }`. There are no accounts on this chain. For deposit detection, run a
wallet with your viewing key (`shrugg sync`); to watch stake, read `/validators`.

### Validators

| method and path | returns |
|---|---|
| `GET /validators` | the register: array of `Validator`, in address order |
| `GET /validators/:address` | `Validator` plus `recent_blocks` (last 10 `BlockSummary`) |

```json
{
  "address": "2nRdFChBXRmKoe2sQE3ZYDzvdg53QmBZJJ9iweY7hk1v",
  "stake": "100000000000000", "rewards": "4000000",
  "pending": [{ "release_epoch": 41, "amount": "5000000000" }],
  "payout": "shrugg1…", "nonce": 3, "active": true,
  "share_percent": 25.0, "blocks_proposed": 5372,
  "last_proposed_height": 20151, "last_proposed_timestamp_ms": 1789017639030,
  "sort_index": 0
}
```

The register is the one place the chain stores amounts in the clear. `stake` is in SHRUGG units,
`rewards` the bundle fees credited to the validator as proposer and not yet withdrawn, `pending`
the unbonding queue (oldest first), `payout` the shielded address a withdraw pays to, `nonce`
what its next signed unbond or withdraw must carry, and `active` whether it is in the set
running the current epoch. The leader of view `v` is the active validator at index
`v mod active_validator_count` in address order. `share_percent` is over the active set's stake
(0 for an inactive entry). On a node before phase S2 `pending` is empty, `payout` null and every
entry active.

### Programs

| method and path | returns |
|---|---|
| `GET /programs?page&limit` | paginated `ProgramSummary` |
| `GET /programs/:id` | `ProgramSummary` plus `recent_calls` (last 10 `TransactionSummary`) |

```json
{
  "id": "675adeea7e4242d8dc48bf56faedb7bea14a4f832d7c8a973f942fa7dd850065",
  "deploy_tx": "dc97d2696981f4e49d822dee06143725444752e13fc42abdce60b72056f7342e",
  "deployed_at_height": 10,
  "base_pc": 0, "words_len": 42,
  "code_hash": "675adeea7e4242d8dc48bf56faedb7bea14a4f832d7c8a973f942fa7dd850065",
  "call_count": 3, "last_called_height": 105
}
```

Programs are content addressed and immutable; `id` never changes. There is no deployer: a deploy
is paid by a shielded bundle, so the chain does not know who deployed it.

### Search

`GET /search?q=` returns an array of matches across blocks, transactions, programs, notes,
nullifiers and validators. A decimal query is treated as a height, a 64-hex query as a hash,
program id, commitment or nullifier, and a base58 query as a validator address.

```json
[{ "type": "nullifier", "id": "8c04…d1", "title": "Nullifier", "subtitle": "note spent at #41", "url": "/transactions/4f2c…e7" }]
```

## Authentication and API keys

1. Sign up at https://randscan.org/signup (email and password; no verification email).
2. On https://randscan.org/dashboard create a key. It looks like `rsk_` followed by 48 letters and
   digits and is shown once. Store it like a password; revoke it from the dashboard if it leaks.
3. Send it on every request, either way:

   ```bash
   curl -H "Authorization: Bearer rsk_..." https://randscan.org/api/v1/stats
   curl -H "X-API-Key: rsk_..." https://randscan.org/api/v1/stats
   ```

A request that carries an unknown or revoked key is refused with 401 `invalid_api_key`; it is not
downgraded to anonymous, so a typo is caught immediately. Keys cannot manage keys or read your
account: those endpoints accept only the browser session.

Each account may hold 10 active keys. The dashboard shows when a key was last used and how many
requests it has made.

### Quotas

| identity | limit |
|---|---|
| anonymous, per IP | 60 requests per minute |
| API key | 600 requests per minute |
| sign-in and sign-up, per IP | 10 attempts per minute |

Limits are fixed 60-second windows. Every response includes `X-RateLimit-Limit` and
`X-RateLimit-Remaining`. Over the limit you get 429 with a `Retry-After` header and body
`{"error":"rate_limited",...}`; wait that many seconds and retry. Need more? Open an issue on the
repository with your use case.

### Account endpoints

These are what the website uses; you can drive them from scripts too, but keys are the intended
way for machines.

| method and path | body | result |
|---|---|---|
| `POST /auth/signup` | `{ "email", "password" }` | 201 `{ "user" }`, sets cookie `randscan_session` |
| `POST /auth/login` | `{ "email", "password" }` | 200 `{ "user" }`, sets cookie |
| `POST /auth/logout` | | 204 |
| `GET /auth/me` | | 200 `{ "user" }` or 401 |
| `POST /auth/forgot` | `{ "email" }` | 202 always (503 `email_disabled` if the explorer has no mail provider) |
| `POST /auth/reset` | `{ "token", "password" }` | 200 `{ "user" }`, sets cookie; 400 `invalid_token` / `weak_password` |
| `POST /auth/password` | `{ "current_password", "new_password" }` | 204; signs out every other session |
| `GET /keys` | | `ApiKey[]` (never includes the secret) |
| `POST /keys` | `{ "name" }` | 201 `ApiKey` plus `key`, once |
| `DELETE /keys/:id` | | 204 |

Password reset: `/auth/forgot` emails a single-use link to `/reset?token=…` that is valid for one
hour and answers 202 whether or not the address is registered. `/auth/reset` logs the user out
everywhere and signs them in with a fresh cookie. Both share the sign-in rate limit.

`user`: `{ id, email, created_at, last_login_at }`. `ApiKey`: `{ id, name, prefix, created_at,
last_used_at, request_count, revoked_at }`. Passwords are 10 to 128 characters.

## WebSocket feed

Connect to `wss://randscan.org/ws` and send JSON messages. Channels: `blocks`, `transactions`,
`stats`.

Client to server:

```json
{ "type": "subscribe", "channel": "blocks" }
{ "type": "unsubscribe", "channel": "blocks" }
{ "type": "ping" }
```

Server to client:

```json
{ "type": "subscribed", "channel": "blocks", "subscription_id": "…" }
{ "type": "unsubscribed", "channel": "blocks" }
{ "type": "pong" }
{ "type": "error", "message": "bad message: …" }
{ "type": "new_block", "block": BlockSummary }
{ "type": "new_transaction", "transaction": TransactionSummary }
{ "type": "stats_update", "stats": NetworkStats }
```

Node.js example that prints every faucet mint and staking move as it commits (the only kinds
with a public amount):

```js
const ws = new WebSocket("wss://randscan.org/ws");
ws.onopen = () => ws.send(JSON.stringify({ type: "subscribe", channel: "transactions" }));
ws.onmessage = (ev) => {
  const msg = JSON.parse(ev.data);
  if (msg.type === "new_transaction" && msg.transaction.amount !== null) {
    const t = msg.transaction;
    console.log(t.height, t.kind, t.validator ?? "", Number(t.amount) / 1e9, "SHRUGG");
  }
};
```

Send a `ping` every 30 seconds to keep idle connections open through proxies. If the socket
drops, reconnect and backfill with `GET /transactions?height=` for any heights you missed; the
feed has no replay.

## Rate limits and reliability

- Rate limits are enforced per IP and per API key; see [Quotas](#quotas) above. Abusive traffic
  will also be blocked at the proxy.
- The explorer runs next to a full node. `GET /health` reports `indexer.lag`; during a resync the
  lists are still served from the database, so a nonzero lag means the newest blocks are missing,
  not that older data is wrong.
- Retry 5xx responses with exponential backoff. 4xx responses will not succeed on retry.

## Running your own

Everything in this document is served by the open-source explorer in this repository. To run it
against your own `shrugg-node`, see the README. The indexer needs only the node's JSON-RPC
endpoint (`shrugg_*` methods) and a PostgreSQL database.
