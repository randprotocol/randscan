# RandScan

Block explorer for the Rand Protocol SHRUGG chain (the network served by
[`shrugg-node`](../fullnode)). Live at https://randscan.org.

- **Backend**: Rust — axum REST API + WebSocket, SQLx/PostgreSQL, and an in-process indexer that
  follows a `shrugg-node` JSON-RPC endpoint (`shrugg_*` methods).
- **Frontend**: Next.js 14, TypeScript, Tailwind, SWR; Leaflet for the nodes map.
- **Database**: PostgreSQL 16.

## What it indexes

Committed blocks (hash, height, HotStuff view, proposer, roots, `justify_view`), the four transaction
kinds (`transfer`, `mint`, `deploy`, `call`), confidential-call receipts (tier, outputs, effect),
accounts (balance and nonce read back from the node), validators (stake, blocks proposed), deployed
programs, network stats, and the node's libp2p peers with geolocated IPs.

## Run locally

Requirements: Rust 1.75+, Node 20+, PostgreSQL, and a reachable `shrugg-node` RPC
(e.g. `ssh -N -L 8545:127.0.0.1:8545 root@<node>` to tunnel a remote node).

```bash
createdb randscan
cp .env.example .env            # DATABASE_URL, RPC_URL, API_PORT ...
cargo run --release --bin randscan-api          # API + indexer + WebSocket on :3000

cd frontend
npm ci
NEXT_PUBLIC_API_URL=http://localhost:3000 NEXT_PUBLIC_WS_URL=ws://localhost:3000/ws npm run dev   # :3001
```

Set `COOKIE_SECURE=false` in `.env` for local http so sign-in works.

The schema (`migrations/001_initial_schema.sql`) is created automatically on first start. The
indexer catches up from height 0, then polls `shrugg_getHead` every `POLL_INTERVAL_MS`.

### Environment

| Variable | Default | Description |
|---|---|---|
| `DATABASE_URL` | `postgres://randscan:randscan@localhost:5432/randscan` | PostgreSQL connection |
| `RPC_URL` | `http://127.0.0.1:8545` | `shrugg-node` JSON-RPC endpoint |
| `API_HOST` / `API_PORT` | `0.0.0.0` / `3000` | API bind address |
| `POLL_INTERVAL_MS` | `1000` | head polling interval when caught up |
| `BATCH_SIZE` | `200` | blocks per pass while catching up |
| `STATS_INTERVAL_SECS` | `5` | minimum interval between stats refreshes |
| `NODES_INTERVAL_SECS` | `60` | peer list / geolocation refresh interval |
| `NODE_PUBLIC_IP` | auto-detected | public IP of the node the explorer runs on |
| `RUST_LOG` | `info` | log filter |
| `COOKIE_SECURE` | `true` | set the session cookie `Secure` (requires https; use `false` for local http) |
| `TRUST_PROXY` | `false` | read the client IP from `X-Forwarded-For` (true behind Caddy/nginx) |
| `PUBLIC_URL` | `https://randscan.org` | site origin used in password-reset links |
| `MAIL_FROM` | `RandScan <no-reply@randscan.org>` | sender of password-reset email (domain must be verified in Resend) |
| `RESEND_API_KEY` | unset | Resend API key; unset disables password reset by email |
| `ANON_RATE_LIMIT_RPM` | `60` | requests per minute per IP for anonymous traffic |
| `KEY_RATE_LIMIT_RPM` | `600` | requests per minute per API key |
| `AUTH_RATE_LIMIT_RPM` | `10` | sign-in/sign-up attempts per minute per IP |

## API

REST under `/api/v1` (`health`, `stats`, `blocks`, `blocks/latest`, `blocks/:id`, `transactions`,
`transactions/latest`, `transactions/:hash`, `accounts/:address`, `accounts/:address/transactions`,
`validators`, `validators/:address`, `programs`, `programs/:id`, `nodes`, `search?q=`) and a
WebSocket at `/ws` (channels `blocks`, `transactions`, `stats`). Amounts are strings of units
(1 SHRUGG = 10^9 units); timestamps are `timestamp_ms`. Full shapes in
`docs/superpowers/specs/2026-09-10-shrugg-retarget-design.md`; a guide for integrators with
examples in [docs/api.md](docs/api.md).

Accounts and API keys: sign up at `/signup`, create keys at `/dashboard`; keyed requests use
`Authorization: Bearer rsk_...`. Details and quotas in [docs/api.md](docs/api.md).

### Password reset

Users reset their own password from `/forgot`: the API emails a single-use link (valid one hour)
through [Resend](https://resend.com) when `RESEND_API_KEY` is set, and signed-in users can change
their password from `/dashboard`. On a node deployed with `deploy/vps-setup.sh`, put the key in
`/etc/randscan/api.secrets.env` (created empty by the script, never overwritten) and restart
`randscan-api`. Without a key the site reports that reset by email is not enabled and the
runbook below is the fallback.

### Operator runbook: reset a password

```bash
read -rs PW && printf '%s' "$PW" | randscan-api hash-password      # prints $argon2id$...
psql "$DATABASE_URL" -c "UPDATE users SET password_hash = '<paste>' WHERE email = 'user@example.com';" \
                     -c "DELETE FROM sessions WHERE user_id = (SELECT id FROM users WHERE email = 'user@example.com');"
```

`read -rs` prompts for the new password without echoing it, and `printf` (rather than `echo`)
avoids appending a trailing newline — together this keeps the password out of shell history and
process listings.

## Deploy on a node

The explorer runs on the same machine as a synced `shrugg-node` (its RPC is bound to localhost):

```bash
deploy/push-to-vps.sh <ip> [domain]     # rsync, build, Postgres + Node 20 + Caddy, systemd units
```

`deploy/vps-setup.sh` installs `randscan-api` (:3000) and `randscan-frontend` (:3001) as systemd
services and a Caddyfile that serves `<domain>` with automatic HTTPS (`/api/*` and `/ws` to the
API, everything else to Next.js; `www.` and `randscan.com` redirect). Point the domain's A records
at the server before running it so Caddy can obtain certificates.

## Docker

`docker compose up --build` runs Postgres, the API and the frontend (set `RPC_URL` to a reachable
node).

## Project structure

```
crates/randscan-core      wire types, helpers (amount formatting, query classification)
crates/randscan-db        schema, row models, queries
crates/randscan-indexer   RPC client, block processor, sync service, peer tracker, broadcaster
crates/randscan-api       axum routes/handlers, binary randscan-api
crates/randscan-ws        WebSocket manager/handler
frontend/                 Next.js app
migrations/               SQL schema
deploy/                   VPS install script, systemd units, Caddyfile
```

## License

MIT
