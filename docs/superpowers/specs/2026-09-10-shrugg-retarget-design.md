# RandScan retarget to the SHRUGG full node (chain 4)

Date: 2026-09-10. Status: approved for implementation (autonomous session).

## Goal

Make RandScan index and display the real Rand Protocol chain served by `shrugg-node`
(`../fullnode`), deploy it on observer node E (188.166.235.187) next to the node, and serve it at
https://randscan.org (with randscan.com and www.* redirecting there).

## What the chain actually exposes

JSON-RPC 2.0 over HTTP at `127.0.0.1:8545` (see `fullnode/docs/rpc.md`). Relevant methods:
`shrugg_chainId`, `shrugg_tokenInfo`, `shrugg_getHead`, `shrugg_status`, `shrugg_getBlockByHeight`,
`shrugg_getBlockByHash`, `shrugg_getTransaction`, `shrugg_getAccount`, `shrugg_getValidators`,
`shrugg_getPeers`, `shrugg_getProgram`, `shrugg_getReceipt`.

- One token, SHRUGG, 9 decimals. Amounts are strings of units (u128).
- Tx kinds: `transfer {to, amount}`, `mint {to, amount}`, `deploy {base_pc, words_len, program}`,
  `call {program, proof_len, recipients[]}`.
- Blocks: `hash, height, view, parent, proposer, timestamp_ms, tx_root, state_root, justify_view,
  tx_count, transactions[]`. Only committed blocks are served; no finality window.
- Validators: `[{address, stake}]` sorted by address; leader of view v is entry `v mod n`.
- Receipts (calls only): `{tx, program, tier, outputs[8], effect: {to, amount}|null, height, index}`.

## Architecture (unchanged shape, new contents)

```
crates/randscan-core     API/WS wire types (serde), helpers
crates/randscan-db       schema (migrations/001_initial_schema.sql), row models, queries
crates/randscan-indexer  RPC client (shrugg_*), block processor, sync service, broadcaster
crates/randscan-api      axum REST + /ws, binary randscan-api (runs indexer in-process)
crates/randscan-ws       websocket manager/handler (channels: blocks, transactions, stats)
frontend/                Next.js 14 app (client-side SWR against /api/v1, WS at /ws)
deploy/                  VPS install script, systemd units, Caddyfile, push script
```

The Leptos crate (`crates/randscan-frontend`), `Trunk.toml`, `index.html` are removed.

## Database schema

Amounts (fee, amount, balance, stake, supply) are `NUMERIC(40,0)`; bound as text with `::numeric`
and read back with `::text`.

- `blocks(hash PK, height UNIQUE, view, parent, proposer, timestamp_ms, tx_root, state_root,
  justify_view, tx_count)`
- `transactions(hash PK, block_hash FK cascade, height, tx_index, sender, nonce, fee, kind, chain_id,
  timestamp_ms, to_address, amount, program_id, base_pc, words_len, proof_len, recipients TEXT[])`
- `receipts(tx_hash PK FK cascade, program, tier, outputs BIGINT[], effect_to, effect_amount, height, tx_index)`
- `accounts(address PK, balance, nonce, tx_count, first_seen_height, last_seen_height, updated_at)`
- `account_transactions(id, account, tx_hash FK cascade, role sender|recipient, height, tx_index,
  UNIQUE(account, tx_hash, role))`
- `validators(address PK, stake, sort_index, updated_at)` — blocks proposed is computed from `blocks`.
- `programs(id PK, deployer, deploy_tx FK cascade, deployed_at_height, base_pc, words_len, code_hash)`
  — call counts computed from `transactions`.
- `network_stats(id=1, chain_id, symbol, decimals, height, view, total_transactions, total_accounts,
  validator_count, total_stake, total_supply, program_count, avg_block_time_ms, peer_count,
  mempool_size, node_syncing, faucet, confidential, updated_at)`
- `indexer_state(id=1, next_height DEFAULT 0, last_indexed_hash, is_syncing, updated_at)`

## Indexer

Loop every `POLL_INTERVAL_MS` (default 1000): `shrugg_getHead`; for `h in next_height..=head` (at
most `BATCH_SIZE` per pass) fetch the block, verify `parent == stored hash at h-1` (on mismatch delete
blocks `>= h-1`, set `next_height = h-1`, continue), then in one DB transaction insert block, txs,
programs (deploy), receipts (call, via `shrugg_getReceipt`), account_transactions; commit;
then refresh every touched address (sender, to, recipients, effect.to, proposer) via
`shrugg_getAccount`; advance `next_height`; broadcast NewBlock/NewTransaction.
When caught up (and at startup) refresh validators (`shrugg_getValidators`, also seeding their
accounts) and stats (`shrugg_status`, `shrugg_chainId`, `shrugg_tokenInfo`, DB counts, avg block time
over the last 100 blocks). Stats refresh at most every 5 s and broadcast StatsUpdate.

## REST API (`/api/v1`)

All amounts are strings of units. Timestamps are `timestamp_ms` (milliseconds).

| route | result |
|---|---|
| `GET /health` | `Health` |
| `GET /stats` | `NetworkStats` |
| `GET /blocks?page&limit&proposer` | `Paginated<BlockSummary>` (newest first) |
| `GET /blocks/latest?limit=10` | `BlockSummary[]` |
| `GET /blocks/:id` (height or hash) | `BlockDetail` |
| `GET /transactions?page&limit&kind&sender&height` | `Paginated<TransactionSummary>` (newest first) |
| `GET /transactions/latest?limit=10` | `TransactionSummary[]` |
| `GET /transactions/:hash` | `TransactionDetail` |
| `GET /accounts/:address` | `AccountDetail` (404 if never seen) |
| `GET /accounts/:address/transactions?page&limit` | `Paginated<AccountTransaction>` |
| `GET /validators` | `Validator[]` (sorted by sort_index; includes `share_percent`) |
| `GET /validators/:address` | `ValidatorDetail` |
| `GET /programs?page&limit` | `Paginated<ProgramSummary>` |
| `GET /programs/:id` | `ProgramDetail` |
| `GET /search?q=` | `SearchResult[]` |

Types (JSON field names exactly):

```ts
BlockSummary { hash, height, view, parent, proposer, timestamp_ms, tx_count, justify_view }
BlockDetail  = BlockSummary & { tx_root, state_root, transactions: TransactionSummary[] }
TransactionSummary { hash, height, block_hash, tx_index, sender, nonce, fee, kind: "transfer"|"mint"|"deploy"|"call",
                     timestamp_ms, to: string|null, amount: string|null, program: string|null }
TransactionDetail = TransactionSummary & { chain_id, base_pc: number|null, words_len: number|null,
                     proof_len: number|null, recipients: string[], receipt: Receipt|null }
Receipt { tx, program, tier, outputs: number[], effect: {to, amount}|null, height, index }
AccountDetail { address, balance, nonce, tx_count, first_seen_height, last_seen_height, is_validator,
                stake: string|null, programs_deployed: number }
AccountTransaction = TransactionSummary & { role: "sender"|"recipient" }
Validator { address, stake, share_percent, blocks_proposed, last_proposed_height: number|null,
            last_proposed_timestamp_ms: number|null, sort_index }
ValidatorDetail = Validator & { recent_blocks: BlockSummary[] }
ProgramSummary { id, deployer, deploy_tx, deployed_at_height, base_pc, words_len, code_hash,
                 call_count, last_called_height: number|null }
ProgramDetail = ProgramSummary & { recent_calls: TransactionSummary[] }
NetworkStats { chain_id, symbol, decimals, height, view, total_transactions, total_accounts,
               validator_count, total_stake, total_supply, program_count, avg_block_time_ms,
               peer_count, mempool_size, node_syncing, faucet, confidential, current_leader: string|null,
               updated_at }
Health { status: "healthy"|"degraded"|"unhealthy", version, database, indexer: {connected, synced, current_height, node_height, lag} }
Paginated<T> { data: T[], pagination: { page, limit, total, total_pages, has_next, has_prev } }
SearchResult { type: "block"|"transaction"|"account"|"validator"|"program", id, title, subtitle?, url }
Error { error, message, code? }   // 404 -> {"error":"not_found",...}
```

Search: decimal → block by height; 64 hex (optional 0x) → block hash, tx hash, program id;
base58 32–44 chars → account (DB, else live `shrugg_getAccount` with nonzero balance/nonce) and
validator.

## WebSocket (`/ws`)

Client → `{"type":"subscribe","channel":"blocks"|"transactions"|"stats"}`, `unsubscribe`, `ping`.
Server → `{"type":"subscribed","channel","subscription_id"}`, `unsubscribed`, `pong`, `error`,
`{"type":"new_block","block":BlockSummary}`, `{"type":"new_transaction","transaction":TransactionSummary}`,
`{"type":"stats_update","stats":NetworkStats}`.

## Frontend

Pages: `/` dashboard (stats cards, latest blocks, latest txs, live dot), `/blocks`, `/blocks/[id]`,
`/transactions` (kind filter), `/transactions/[hash]` (kind-specific panel + receipt with outputs and
effect), `/account/[address]`, `/validators`, `/validators/[address]`, `/programs`, `/programs/[id]`,
`/search?q=`. Header nav: Dashboard, Blocks, Transactions, Validators, Programs. Amounts shown as
SHRUGG with up to 9 decimals; hashes shortened with copy; timestamps from `timestamp_ms`.
Fetching is client-side (SWR) against same-origin `/api/v1` (so `NEXT_PUBLIC_API_URL` may be empty)
and `NEXT_PUBLIC_WS_URL` (default derived from `window.location`).

## Deployment (node E)

`deploy/push-to-vps.sh <ip>` rsyncs the repo to `/root/randscan` and runs `deploy/vps-setup.sh`,
which installs postgresql, nodejs 20, caddy; creates db/user; `cargo build --release --bin randscan-api`;
`npm ci && npm run build` (standalone); installs `randscan-api.service` (env from
`/etc/randscan/api.env`, RPC_URL=http://127.0.0.1:8545) and `randscan-frontend.service` (port 3001);
writes `/etc/caddy/Caddyfile` (randscan.org → /api/*, /ws → :3000, rest → :3001; other hosts 301 →
randscan.org); opens ufw 80/443. Caddy obtains Let's Encrypt certificates, so DNS records are
created unproxied (the API token cannot set Cloudflare's SSL mode).

## Testing

- `cargo build --release`, `cargo test` (unit tests for RPC deserialisation with captured
  payloads, kind mapping, search classification).
- Local end-to-end: SSH tunnel to node E's RPC, local Postgres, run `randscan-api`, curl each route.
- `npm run build` + `npm run lint` for the frontend.
- After deploy: curl https://randscan.org/api/v1/health and the home page; compare `/stats.height`
  with `shrugg status` on the node.
