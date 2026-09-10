# RandScan accounts, sessions and API keys

Date: 2026-09-10. Status: draft for review.

## Goal

Let exchanges, bots and other integrators register on randscan.org, create API keys, and call the
public API with a higher quota than anonymous traffic. Give the explorer the account layer that the
future viewing-key endpoints (server-side scan, see "Later" below) will require.

Decisions taken with the user on 2026-09-10:

- Build now; do not wait for the circuits M2–M4 milestones. The viewing-key pages stay out of scope
  until the chain exposes envelopes and commitments over RPC.
- Public reads stay keyless. Anonymous requests get a per-IP rate limit; a key raises the quota.
  Nothing changes for existing integrations or the frontend.
- Login is email plus password. No email verification, magic links or OAuth in v1 (no mail
  provider is configured on the node).
- Viewing keys, when they arrive, are scanned server-side and gated behind a session or API key.

## Non-goals (v1)

Email verification, password reset by email, 2FA, account deletion, teams, billing, per-endpoint
quotas, request logs per key, an admin UI. A lost password is reset by an operator with `psql`
(documented in the runbook section).

## Architecture

No new services. Everything lives in the existing `randscan-api` binary and PostgreSQL database;
the Next.js app gains three pages. Caddy already proxies `/api/*` to the API on the same origin, so
cookies are first-party.

```
frontend/src/app/login, /signup, /dashboard      pages (client components, SWR)
frontend/src/lib/api.ts, hooks/useApi.ts         auth + keys client functions and useMe()
crates/randscan-api/src/auth/                    extractors, password hashing, key format, cookie
crates/randscan-api/src/handlers/auth.rs         signup / login / logout / me
crates/randscan-api/src/handlers/keys.rs         list / create / revoke
crates/randscan-api/src/ratelimit.rs             per-IP and per-key limiter middleware
crates/randscan-db/src/queries/users.rs          users, sessions, api_keys
migrations/002_accounts.sql                      new tables + schema_migrations
```

## Data model (`migrations/002_accounts.sql`)

```sql
CREATE TABLE schema_migrations (version INTEGER PRIMARY KEY, applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW());

CREATE TABLE users (
    id             BIGSERIAL PRIMARY KEY,
    email          VARCHAR(254) NOT NULL,           -- stored lowercased and trimmed
    password_hash  TEXT NOT NULL,                   -- argon2id PHC string
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_login_at  TIMESTAMPTZ
);
CREATE UNIQUE INDEX idx_users_email ON users(email);

CREATE TABLE sessions (
    token_hash   CHAR(64) PRIMARY KEY,              -- sha256(hex) of the cookie value
    user_id      BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at   TIMESTAMPTZ NOT NULL,
    last_seen_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    user_agent   VARCHAR(256),
    ip           VARCHAR(64)
);
CREATE INDEX idx_sessions_user ON sessions(user_id);
CREATE INDEX idx_sessions_expires ON sessions(expires_at);

CREATE TABLE api_keys (
    id             BIGSERIAL PRIMARY KEY,
    user_id        BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name           VARCHAR(64) NOT NULL,
    prefix         VARCHAR(16) NOT NULL,            -- first 12 chars after "rsk_", for display
    key_hash       CHAR(64) NOT NULL UNIQUE,        -- sha256(hex) of the full key
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_used_at   TIMESTAMPTZ,
    request_count  BIGINT NOT NULL DEFAULT 0,
    revoked_at     TIMESTAMPTZ
);
CREATE INDEX idx_api_keys_user ON api_keys(user_id, created_at DESC);
```

Secrets are never stored: the session cookie value and the API key are random, and only their
SHA-256 is kept. A leaked database dump does not yield usable credentials.

### Migration runner

`run_migrations` today creates the schema only when `blocks` is absent. It becomes a versioned
runner: ensure `schema_migrations` exists; if `blocks` exists and version 1 is not recorded, record
it (existing deployments); then apply every embedded migration whose version is not recorded, each
inside a transaction, in order. Migrations are embedded with `include_str!` and listed in one
array `MIGRATIONS: &[(i32, &str)]`. The legacy-schema check (`justify_view` column) stays.

## Credentials

**Passwords.** Argon2id with the `argon2` crate defaults (19 MiB, t=2, p=1), PHC string stored.
Policy: 10 to 128 characters, no other rules. Hashing runs in `spawn_blocking`.

**Emails.** Trimmed and lowercased before storing and comparing. Validation: one `@`, non-empty
local and domain parts, domain contains a dot, total length ≤ 254. No deliverability check.

**Sessions.** 32 random bytes from `rand::rngs::OsRng`, hex encoded (64 chars) as the cookie value;
the database stores its SHA-256. Cookie `randscan_session`: `HttpOnly`, `SameSite=Lax`, `Path=/`,
`Max-Age` 30 days, `Secure` unless `COOKIE_SECURE=false` (local dev over http). Lifetime is
sliding: `last_seen_at` is bumped at most once per 5 minutes and `expires_at` moves with it. Login
also deletes the user's expired sessions.

**API keys.** Format `rsk_` + 48 random base62 characters (about 286 bits), for example
`rsk_9fT2...`. Shown once at creation. `prefix` keeps the first 12 characters after `rsk_` so the
dashboard can identify a key. Lookup is by `key_hash`, so comparison is a unique-index hit and
timing is not secret-dependent. A user may hold at most 10 active (non-revoked) keys.

**CSRF.** Cookies are `SameSite=Lax` and every state-changing endpoint accepts only
`Content-Type: application/json` bodies, which cross-site forms cannot send. CORS stays
`allow_origin(Any)` without credentials, so cross-origin `fetch` never carries the cookie.

## API

All under `/api/v1`. Errors use the existing `{ error, message, code }` body.

| method and path | auth | body | result |
|---|---|---|---|
| `POST /auth/signup` | none | `{ email, password }` | 201 `{ user }`, sets cookie; 409 `email_taken`; 400 on policy failure |
| `POST /auth/login` | none | `{ email, password }` | 200 `{ user }`, sets cookie; 401 `invalid_credentials` (same message for unknown email and wrong password) |
| `POST /auth/logout` | session | | 204, session row deleted, cookie cleared |
| `GET /auth/me` | session | | 200 `{ user }`; 401 `unauthorized` |
| `GET /keys` | session | | 200 `ApiKey[]` |
| `POST /keys` | session | `{ name }` | 201 `ApiKey & { key }` (the only time `key` is returned); 400 if name empty or > 64 chars; 409 `key_limit` at 10 active keys |
| `DELETE /keys/:id` | session | | 204; 404 if not the caller's key or already revoked |

```ts
User   { id, email, created_at, last_login_at: string|null }
ApiKey { id, name, prefix, created_at, last_used_at: string|null, request_count, revoked_at: string|null }
```

`GET /auth/me` and `GET /keys` accept only the session cookie, not an API key: a key must not be
able to mint more keys or read the account.

### Authenticating API requests with a key

`Authorization: Bearer rsk_...` (preferred) or `X-API-Key: rsk_...`. The middleware runs on every
`/api/v1` request:

1. No key header: anonymous, limited per client IP.
2. Key header present but unknown or revoked: 401 `invalid_api_key`. It does not fall back to
   anonymous, so a typo is noticed immediately.
3. Valid key: limited per key; `last_used_at` and `request_count` are updated by a detached task
   (fire and forget, one `UPDATE` per request; revisit with batching if key traffic grows).

The WebSocket route is unchanged and unauthenticated.

### Rate limits

Fixed 60-second windows per identity, in memory (`Mutex<HashMap<String, Window>>`, swept every
5 minutes). Restarting the API resets the counters, which is acceptable.

| identity | env | default |
|---|---|---|
| anonymous, per IP | `ANON_RATE_LIMIT_RPM` | 60 |
| API key | `KEY_RATE_LIMIT_RPM` | 600 |
| `/auth/login` and `/auth/signup`, per IP | `AUTH_RATE_LIMIT_RPM` | 10 |

Every response carries `X-RateLimit-Limit` and `X-RateLimit-Remaining`. Over the limit: 429 with
`Retry-After` seconds and body `{ "error": "rate_limited", ... }`. Setting a limit to 0 disables
that limiter (used by tests and local dev).

Client IP: the peer address, unless `TRUST_PROXY=true`, in which case the first entry of
`X-Forwarded-For` (Caddy sets it). `deploy/vps-setup.sh` sets `TRUST_PROXY=true` since the API
binds to localhost behind Caddy. Docker compose does not set it (the API is exposed directly).

## Frontend

Three new pages in the existing visual language (Panel, DetailRow, PageHeader, DataTable):

- `/signup` and `/login`: one form each (email, password), inline error from the API, link to the
  other page. On success, redirect to `/dashboard`.
- `/dashboard`: the account email, a "Sign out" button, and the API keys table (name, prefix,
  created, last used, requests, status) with a "New key" form (name) and a "Revoke" action per
  row. After creation, the full key is shown once in a copyable box with a warning that it will
  not be shown again. A short "Using your key" snippet with a `curl` example links to
  `docs/api.md`.
- Header: when signed out, a "Sign in" link at the right of the nav; when signed in, "Dashboard".
  Driven by `useMe()` (SWR on `/auth/me`, no retry on 401).

The dashboard route is `/dashboard`, not `/account`, because `/account/[address]` is already the
on-chain account page.

`lib/api.ts` gains `signup`, `login`, `logout`, `getMe`, `listKeys`, `createKey`, `revokeKey`, all
sending `credentials: 'same-origin'` (the `request` helper adds it globally; it is harmless for
public reads).

## Configuration

| variable | default | purpose |
|---|---|---|
| `COOKIE_SECURE` | `true` | set `false` for local http |
| `TRUST_PROXY` | `false` | read `X-Forwarded-For` |
| `ANON_RATE_LIMIT_RPM` | `60` | |
| `KEY_RATE_LIMIT_RPM` | `600` | |
| `AUTH_RATE_LIMIT_RPM` | `10` | |

Added to `.env.example`, the README table, `deploy/vps-setup.sh` (`TRUST_PROXY=true`) and
`docker-compose.yml` (`COOKIE_SECURE=false`).

## Error handling

`AppError` gains `Unauthorized(code, msg)`, `Conflict(code, msg)`, `TooManyRequests(retry_secs)`.
Password hashing failures are `Internal`. Unique-violation on `users.email` maps to 409
`email_taken` (the insert is attempted directly; no check-then-insert race).

## Testing

- **Unit** (`randscan-api`): key format and prefix extraction; SHA-256 hashing is deterministic;
  password policy bounds; email normalisation and validation; rate-limit window arithmetic
  (limit reached, window rollover, disabled limiter).
- **Integration** (`randscan-api/tests/auth.rs`, run when `DATABASE_URL` is set, as CI does):
  build the router with a real pool and a stub indexer, drive it with `tower::ServiceExt::oneshot`.
  Cases: signup then `me`; duplicate signup is 409; wrong password is 401 with the same body as an
  unknown email; logout invalidates the cookie; create key returns the secret once and `GET /keys`
  never does; a request with the key succeeds and increments `request_count`; a revoked key is
  401; an eleventh key is 409; an anonymous client over `ANON_RATE_LIMIT_RPM=3` gets 429 with
  `Retry-After`.
- **Frontend**: `npm run build` and `npm run lint` in CI as today; manual pass of signup, key
  creation and a `curl` with the key against the deployed instance.

The stub indexer: `AppState.indexer` is `Arc<IndexerService>`, which tests can construct with
`IndexerService::new(config, pool, Broadcaster::new())` without running it; handlers that call
the RPC are not exercised by these tests.

## Documentation

`docs/api.md` gains an "Authentication and API keys" section (signup on the site, header formats,
quotas, 401/429 semantics) and the rate-limit paragraph is updated. The README lists the new
environment variables and the operator runbook for a password reset:

```sql
UPDATE users SET password_hash = '<argon2 PHC string>' WHERE email = '...';
DELETE FROM sessions WHERE user_id = (SELECT id FROM users WHERE email = '...');
```

with a one-line `randscan-api hash-password` subcommand to produce the PHC string.

## Later: viewing keys (blocked on chain support)

Once `shrugg-node` serves note envelopes, commitments and nullifiers (circuits milestones M2–M4,
`circuits/research/docs/05-roadmap.md`), the explorer adds, behind a session or API key:

- `POST /accounts/:address/shielded` with `{ viewing_key }`: server-side `scan` under a
  `Party` disclosure, returning received and sent rows with `verify_row` status, and the shielded
  balance implied by unspent received notes.
- `POST /transactions/:hash/open` with `{ tx_key }`: the single row of one transaction.

The key is used for the request only, never persisted or logged (request bodies for these routes
are excluded from tracing). That trust boundary, and the tutorial for early users, get their own
spec when the RPC surface exists.
