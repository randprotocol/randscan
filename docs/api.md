# RandScan API for early users

RandScan indexes the Rand Protocol SHRUGG chain (chain id 4) and serves what it has indexed over a
plain HTTPS JSON API and a WebSocket feed. This guide is for people who want to read the chain from
a script, a bot, an exchange backend, or a wallet without running their own `shrugg-node`.

Base URL: `https://randscan.org/api/v1`
WebSocket: `wss://randscan.org/ws`

No API key is needed today. Every endpoint is read-only. Please keep polling to a sensible rate
(one request per second per endpoint is plenty; new blocks arrive roughly every 3 to 4 seconds) and
use the WebSocket feed instead of tight polling when you need to react to new blocks.

## Conventions

- **Amounts are strings of units.** 1 SHRUGG = 1,000,000,000 units (9 decimals). Amounts are
  unsigned 128-bit integers and are always serialised as decimal strings, never as JSON numbers.
  `"10000000000"` is 10 SHRUGG.
- **Timestamps** are `timestamp_ms`: Unix milliseconds, set by the block proposer.
- **Hashes and program ids** are 64 lowercase hex characters, no `0x` prefix. Inputs accept an
  optional `0x` and uppercase.
- **Addresses** are base58 strings of 32 to 44 characters (for example
  `2nRdFChBXRmKoe2sQE3ZYDzvdg53QmBZJJ9iweY7hk1v`).
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
  | 404 | `not_found` | the block, transaction, account, validator or program does not exist |
  | 500 | `internal_error` | something failed on our side; retry with backoff |

- CORS is open, so browsers can call the API directly. Responses are gzip compressed when the
  client accepts it.

## Quick start

```bash
# Is the explorer healthy and caught up with the chain?
curl -s https://randscan.org/api/v1/health

# Current chain height, supply, validator count
curl -s https://randscan.org/api/v1/stats

# Balance of an address
curl -s https://randscan.org/api/v1/accounts/2nRdFChBXRmKoe2sQE3ZYDzvdg53QmBZJJ9iweY7hk1v

# A transaction by hash
curl -s https://randscan.org/api/v1/transactions/ba03893d4b5b4f03ab8ab85aa9e8df4b46102986f15ef56653f7450b98bdba34
```

Python:

```python
import requests

API = "https://randscan.org/api/v1"
UNITS = 10**9

acct = requests.get(f"{API}/accounts/2nRdFChBXRmKoe2sQE3ZYDzvdg53QmBZJJ9iweY7hk1v", timeout=10).json()
print("balance:", int(acct["balance"]) / UNITS, "SHRUGG")

page = requests.get(f"{API}/transactions", params={"kind": "transfer", "limit": 50}, timeout=10).json()
for tx in page["data"]:
    print(tx["height"], tx["sender"], "->", tx["to"], int(tx["amount"]) / UNITS)
```

## Endpoints

### Health and network

| method and path | returns |
|---|---|
| `GET /health` | explorer status and indexer lag |
| `GET /stats` | network statistics |
| `GET /nodes` | the explorer node and its peers, with geolocation |

`GET /health`:

```json
{
  "status": "healthy",
  "version": "0.2.0",
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
  "chain_id": 4, "symbol": "SHRUGG", "decimals": 9,
  "height": 20147, "view": 23263,
  "total_transactions": 40, "total_accounts": 37,
  "validator_count": 4, "total_stake": "400000", "total_supply": "800000000000",
  "program_count": 1, "avg_block_time_ms": 3684.35,
  "peer_count": 4, "mempool_size": 0, "node_syncing": false,
  "faucet": true, "confidential": true,
  "current_leader": "F6rYLexPhyMmwPNqbEmyyp5FiTmtQqDgZyqScUqYY4F6",
  "updated_at": "2026-09-10T05:20:38.591804+00:00"
}
```

`total_supply` and `total_stake` are unit strings. `faucet` and `confidential` are the chain's
genesis flags. `current_leader` is the validator expected to propose the next block.

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
| `GET /transactions?page&limit&kind&sender&height` | paginated `TransactionSummary`, newest first |
| `GET /transactions/latest?limit=10` | array of the newest `TransactionSummary` |
| `GET /transactions/:hash` | `TransactionDetail` |

Filters: `kind` is one of `transfer`, `mint`, `deploy`, `call`; `sender` is an address; `height`
restricts to one block.

`TransactionSummary`:

```json
{
  "hash": "ba03893d4b5b4f03ab8ab85aa9e8df4b46102986f15ef56653f7450b98bdba34",
  "height": 19340,
  "block_hash": "890715533e41e9ddbeac0edc0d2722847248e3affba70816fff31ca54bc1b80f",
  "tx_index": 0,
  "sender": "GKpmJAgz12hzoNHCf5xfBceNj1nNaVKxtGMK7px9DvLo",
  "nonce": 31, "fee": "1000", "kind": "transfer", "timestamp_ms": 1789009705401,
  "to": "649ZogseujYfKiXXVYHNoJvzPVaYR1QZcNvqByK59y1R",
  "amount": "10000000000",
  "program": null
}
```

The four kinds and which fields they fill:

| kind | `to` / `amount` | `program` | detail-only fields |
|---|---|---|---|
| `transfer` | recipient and amount | null | |
| `mint` | recipient and amount (testnet faucet) | null | |
| `deploy` | null | the new program id | `base_pc`, `words_len` |
| `call` | null | the called program id | `proof_len`, `recipients`, `receipt` |

`TransactionDetail` adds `chain_id`, `base_pc`, `words_len`, `proof_len`, `recipients` (the
public list of addresses a confidential call may pay, at most 8), and `receipt`. The receipt of a
confidential call is:

```json
{
  "tx": "b08244b044e8d719aa4e2c1bd22a92914924ae7a6cd41c4c363a608e211f09b0",
  "program": "675adeea7e4242d8dc48bf56faedb7bea14a4f832d7c8a973f942fa7dd850065",
  "tier": 10,
  "outputs": [1, 0, 200, 0, 0, 0, 0, 0],
  "effect": { "to": "ByDkxsEfDCR5DrmDufKftvcRsgvufypnZ4SgDQzJAQ7Z", "amount": "200" },
  "height": 105, "index": 0
}
```

`outputs` are the program's eight public output words: word 0 is the effect kind (0 none,
1 transfer), word 1 the index into `recipients`, words 2 and 3 the amount in units (low and high
64-bit halves), words 4 to 7 free data. `effect` is the transfer those words requested, already
applied inside the same transaction, or `null` when the call only recorded data.
The proof itself, the private inputs and the cycle count are not on chain and not in the API.

### Accounts and balances

| method and path | returns |
|---|---|
| `GET /accounts/:address` | `AccountDetail`, 404 if the address has never been seen on chain |
| `GET /accounts/:address/transactions?page&limit` | paginated `TransactionSummary` plus `role` |

```json
{
  "address": "2nRdFChBXRmKoe2sQE3ZYDzvdg53QmBZJJ9iweY7hk1v",
  "balance": "99998007775", "nonce": 4, "tx_count": 4,
  "first_seen_height": 10, "last_seen_height": 18701,
  "is_validator": true, "stake": "100000", "programs_deployed": 1
}
```

`balance` is the committed balance read back from the node after the last block that touched the
account. `nonce` is the number of transactions the account has sent; a transaction must carry the
next nonce. Each entry of the transactions list carries `role`, either `sender` or `recipient`.

For deposit detection, watch `/accounts/:address/transactions` or the WebSocket `transactions`
channel and match `to` against your deposit addresses. Confidential-call payouts show up as a
`call` transaction whose `receipt.effect.to` is your address, so match on both.

### Validators

| method and path | returns |
|---|---|
| `GET /validators` | array of `Validator`, in validator-set order |
| `GET /validators/:address` | `Validator` plus `recent_blocks` (last 10 `BlockSummary`) |

```json
{
  "address": "2nRdFChBXRmKoe2sQE3ZYDzvdg53QmBZJJ9iweY7hk1v",
  "stake": "100000", "share_percent": 25.0, "blocks_proposed": 5372,
  "last_proposed_height": 20151, "last_proposed_timestamp_ms": 1789017639030,
  "sort_index": 0
}
```

The leader of view `v` is the validator at index `v mod validator_count`.

### Programs

| method and path | returns |
|---|---|
| `GET /programs?page&limit` | paginated `ProgramSummary` |
| `GET /programs/:id` | `ProgramSummary` plus `recent_calls` (last 10 `TransactionSummary`) |

```json
{
  "id": "675adeea7e4242d8dc48bf56faedb7bea14a4f832d7c8a973f942fa7dd850065",
  "deployer": "2nRdFChBXRmKoe2sQE3ZYDzvdg53QmBZJJ9iweY7hk1v",
  "deploy_tx": "dc97d2696981f4e49d822dee06143725444752e13fc42abdce60b72056f7342e",
  "deployed_at_height": 10,
  "base_pc": 0, "words_len": 42,
  "code_hash": "675adeea7e4242d8dc48bf56faedb7bea14a4f832d7c8a973f942fa7dd850065",
  "call_count": 3, "last_called_height": 105
}
```

Programs are content addressed and immutable; `id` never changes.

### Search

`GET /search?q=` returns an array of matches across blocks, transactions, accounts, validators and
programs. A decimal query is treated as a height, a 64-hex query as a hash or program id, and a
base58 query as an address.

```json
[{ "type": "account", "id": "2nRd…", "title": "Account", "subtitle": "99.998007775 SHRUGG", "url": "/account/2nRd…" }]
```

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

Node.js example that prints every transfer as it commits:

```js
const ws = new WebSocket("wss://randscan.org/ws");
ws.onopen = () => ws.send(JSON.stringify({ type: "subscribe", channel: "transactions" }));
ws.onmessage = (ev) => {
  const msg = JSON.parse(ev.data);
  if (msg.type === "new_transaction" && msg.transaction.kind === "transfer") {
    const t = msg.transaction;
    console.log(t.height, t.sender, "->", t.to, Number(t.amount) / 1e9, "SHRUGG");
  }
};
```

Send a `ping` every 30 seconds to keep idle connections open through proxies. If the socket
drops, reconnect and backfill with `GET /transactions?height=` for any heights you missed; the
feed has no replay.

## Rate limits and reliability

- There is no enforced rate limit today. Abusive traffic will be blocked at the proxy. Per-key
  quotas for registered users are planned; the read endpoints above will stay public.
- The explorer runs next to a full node. `GET /health` reports `indexer.lag`; during a resync the
  lists are still served from the database, so a nonzero lag means the newest blocks are missing,
  not that older data is wrong.
- Retry 5xx responses with exponential backoff. 4xx responses will not succeed on retry.

## Running your own

Everything in this document is served by the open-source explorer in this repository. To run it
against your own `shrugg-node`, see the README. The indexer needs only the node's JSON-RPC
endpoint (`shrugg_*` methods) and a PostgreSQL database.
