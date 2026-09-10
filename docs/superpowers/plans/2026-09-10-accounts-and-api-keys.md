# Accounts, Sessions and API Keys Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let integrators sign up on randscan.org, create API keys, and call `/api/v1` with a per-key quota while anonymous reads stay open under a per-IP limit.

**Architecture:** Everything runs inside the existing `randscan-api` binary and PostgreSQL. A versioned migration runner adds `users`, `sessions`, `api_keys`. Cookie sessions (random token, SHA-256 stored) and API keys (`rsk_` + 48 base62, SHA-256 stored) are checked by an axum extractor and a rate-limit middleware. Next.js gains `/login`, `/signup`, `/dashboard`.

**Tech Stack:** Rust 1.91, axum 0.7, axum-extra 0.9 (cookie), sqlx 0.7 (postgres, chrono), argon2 0.5, sha2 0.10, rand 0.8, hex 0.4, time 0.3; Next.js 15, React 19, SWR 2, Tailwind 3.

**Spec:** `docs/superpowers/specs/2026-09-10-accounts-and-api-keys-design.md`

## Global Constraints

- Public read endpoints remain keyless; the frontend never needs a key.
- Secrets are never stored or logged: only SHA-256 of session tokens and API keys is persisted. Never `tracing` a password, token or key.
- API key format: `rsk_` + 48 base62 chars. `prefix` column = first 12 chars after `rsk_`.
- Password policy: 10 to 128 characters, no other rules. Argon2id with `argon2` crate defaults, hashing in `spawn_blocking`.
- Emails: trimmed and lowercased; one `@`, non-empty local and domain, domain contains a dot, total ≤ 254.
- Cookie `randscan_session`: HttpOnly, SameSite=Lax, Path=/, Max-Age 30 days, Secure unless `COOKIE_SECURE=false`.
- State-changing endpoints accept only `Content-Type: application/json` (axum `Json` extractor enforces this).
- `GET /auth/me` and `/keys*` accept sessions only, never API keys.
- Rate limits: fixed 60 s windows. Env: `ANON_RATE_LIMIT_RPM` (60), `KEY_RATE_LIMIT_RPM` (600), `AUTH_RATE_LIMIT_RPM` (10). 0 disables that limiter. Headers `X-RateLimit-Limit`, `X-RateLimit-Remaining`; 429 carries `Retry-After` and body `{"error":"rate_limited",...}`.
- Client IP = peer address unless `TRUST_PROXY=true`, then first entry of `X-Forwarded-For`.
- Unknown or revoked key: 401 `invalid_api_key`, never fall back to anonymous.
- At most 10 active keys per user (409 `key_limit`).
- Error body stays `{ error, message, code }` with `code` = upper-snake of `error`.
- Dashboard route is `/dashboard` (not `/account`, which is the on-chain account page).
- Every Rust commit must pass `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test --all` (integration tests need `DATABASE_URL`; they skip themselves when it is unset). Frontend commits must pass `npm run lint` and `npm run build`.
- Commit messages end with the two attribution lines used by this repo's recent commits (`Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>` and the `Claude-Session:` line).

---

## File structure

| path | responsibility |
|---|---|
| `migrations/002_accounts.sql` | users, sessions, api_keys tables |
| `crates/randscan-db/src/lib.rs` | versioned migration runner (`MIGRATIONS`, `run_migrations`) |
| `crates/randscan-db/src/models.rs` | `UserRow`, `SessionRow`, `ApiKeyRow` (appended) |
| `crates/randscan-db/src/queries/users.rs` | user, session and API-key queries |
| `crates/randscan-db/tests/accounts.rs` | migration idempotence and query tests (need `DATABASE_URL`) |
| `crates/randscan-core/src/types/auth.rs` | wire types: `User`, `ApiKey`, `CreatedApiKey`, request bodies |
| `crates/randscan-api/src/lib.rs` | `ApiConfig` gains auth and rate-limit settings |
| `crates/randscan-api/src/state.rs` | `AppState` gains `config` and `limiter` |
| `crates/randscan-api/src/error.rs` | `AppError::Status`, `AppError::TooManyRequests` and constructors |
| `crates/randscan-api/src/auth/mod.rs` | module root, re-exports |
| `crates/randscan-api/src/auth/password.rs` | policy, argon2 hash/verify |
| `crates/randscan-api/src/auth/email.rs` | normalisation/validation |
| `crates/randscan-api/src/auth/keys.rs` | key generation, prefix, SHA-256, header extraction |
| `crates/randscan-api/src/auth/session.rs` | token generation, cookie builders, `AuthUser` extractor |
| `crates/randscan-api/src/ratelimit.rs` | `RateLimiter` + `api_limit` / `auth_limit` middleware |
| `crates/randscan-api/src/handlers/auth.rs` | signup, login, logout, me |
| `crates/randscan-api/src/handlers/keys.rs` | list, create, revoke |
| `crates/randscan-api/src/routes.rs` | wiring |
| `crates/randscan-api/src/main.rs` | `hash-password` subcommand, ConnectInfo, sweeper task |
| `crates/randscan-api/tests/common/mod.rs` | test app builder and request helpers |
| `crates/randscan-api/tests/auth.rs`, `tests/keys.rs`, `tests/ratelimit.rs` | integration tests |
| `frontend/src/types/index.ts` | `User`, `ApiKey`, `CreatedApiKey` |
| `frontend/src/lib/api.ts` | auth/keys client functions, same-origin requests |
| `frontend/src/hooks/useApi.ts` | `useMe`, `useApiKeys` |
| `frontend/src/components/AuthForm.tsx` | shared login/signup form |
| `frontend/src/app/login/page.tsx`, `signup/page.tsx`, `dashboard/page.tsx` | pages |
| `frontend/src/components/Header.tsx` | Sign in / API keys link |
| `frontend/src/app/globals.css` | `.input` class |
| `.env.example`, `README.md`, `docs/api.md`, `deploy/vps-setup.sh`, `docker-compose.yml`, `frontend/Dockerfile`, `frontend/next.config.js` | config and docs |

---

### Task 1: Versioned migration runner and `002_accounts.sql`

**Files:**
- Create: `migrations/002_accounts.sql`
- Modify: `crates/randscan-db/src/lib.rs` (replace `SCHEMA_SQL` and `run_migrations`)
- Modify: `crates/randscan-db/Cargo.toml` (dev-dependency `tokio` already present as a dependency; nothing to add)
- Test: `crates/randscan-db/tests/accounts.rs`

**Interfaces:**
- Produces: `randscan_db::run_migrations(pool: &PgPool) -> Result<()>` (same signature, now versioned) and the three tables below.

- [ ] **Step 1: Write the migration**

`migrations/002_accounts.sql`:

```sql
-- RandScan accounts: users, cookie sessions and API keys. Secrets are stored as SHA-256 hex only.

CREATE TABLE users (
    id             BIGSERIAL PRIMARY KEY,
    email          VARCHAR(254) NOT NULL,
    password_hash  TEXT NOT NULL,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_login_at  TIMESTAMPTZ
);

CREATE UNIQUE INDEX idx_users_email ON users(email);

CREATE TABLE sessions (
    token_hash    CHAR(64) PRIMARY KEY,
    user_id       BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at    TIMESTAMPTZ NOT NULL,
    last_seen_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    user_agent    VARCHAR(256),
    ip            VARCHAR(64)
);

CREATE INDEX idx_sessions_user ON sessions(user_id);
CREATE INDEX idx_sessions_expires ON sessions(expires_at);

CREATE TABLE api_keys (
    id             BIGSERIAL PRIMARY KEY,
    user_id        BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name           VARCHAR(64) NOT NULL,
    prefix         VARCHAR(16) NOT NULL,
    key_hash       CHAR(64) NOT NULL UNIQUE,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_used_at   TIMESTAMPTZ,
    request_count  BIGINT NOT NULL DEFAULT 0,
    revoked_at     TIMESTAMPTZ
);

CREATE INDEX idx_api_keys_user ON api_keys(user_id, created_at DESC);
```

- [ ] **Step 2: Write the failing test**

`crates/randscan-db/tests/accounts.rs`:

```rust
//! Integration tests against a real PostgreSQL (`DATABASE_URL`). They skip when it is unset.

use randscan_db::run_migrations;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

pub async fn test_pool() -> Option<PgPool> {
    let url = std::env::var("DATABASE_URL").ok()?;
    let pool = PgPoolOptions::new()
        .max_connections(4)
        .connect(&url)
        .await
        .expect("connect to DATABASE_URL");
    run_migrations(&pool).await.expect("migrations");
    Some(pool)
}

#[tokio::test]
async fn migrations_are_versioned_and_idempotent() {
    let Some(pool) = test_pool().await else {
        eprintln!("skipping: DATABASE_URL unset");
        return;
    };
    // Running again must be a no-op.
    run_migrations(&pool).await.expect("second run");

    let versions: Vec<i32> = sqlx::query_scalar("SELECT version FROM schema_migrations ORDER BY version")
        .fetch_all(&pool)
        .await
        .unwrap();
    assert_eq!(versions, vec![1, 2]);

    for table in ["users", "sessions", "api_keys"] {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_name = $1)",
        )
        .bind(table)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert!(exists, "table {table} missing");
    }
}
```

- [ ] **Step 3: Run the test to verify it fails**

Run: `cd crates/randscan-db && DATABASE_URL=postgres://randscan:randscan@localhost:5432/randscan cargo test --test accounts`
Expected: FAIL (`relation "schema_migrations" does not exist`). If no local Postgres is available, run `docker run -d --name randscan-pg -e POSTGRES_USER=randscan -e POSTGRES_PASSWORD=randscan -e POSTGRES_DB=randscan -p 5432:5432 postgres:16-alpine` first.

- [ ] **Step 4: Implement the runner**

In `crates/randscan-db/src/lib.rs` replace everything from `/// Schema version string ...` to the end of `run_migrations` with:

```rust
/// Embedded migrations in order. Add new files here; never edit an applied one.
const MIGRATIONS: &[(i32, &str)] = &[
    (1, include_str!("../../../migrations/001_initial_schema.sql")),
    (2, include_str!("../../../migrations/002_accounts.sql")),
];

/// Apply every migration that `schema_migrations` does not record yet, each in its own transaction.
///
/// Databases created before the runner existed have `blocks` but no `schema_migrations`; they are
/// recorded as version 1 first.
pub async fn run_migrations(pool: &PgPool) -> Result<()> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
    )
    .execute(pool)
    .await?;

    let blocks_exist: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_name = 'blocks')",
    )
    .fetch_one(pool)
    .await?;

    if blocks_exist {
        let ok: bool = sqlx::query_scalar(
            "SELECT EXISTS (SELECT FROM information_schema.columns WHERE table_name = 'blocks' AND column_name = 'justify_view')",
        )
        .fetch_one(pool)
        .await?;
        if !ok {
            return Err(DbError::Migration(
                "database has the legacy schema; drop and recreate the database".into(),
            ));
        }
        sqlx::query("INSERT INTO schema_migrations (version) VALUES (1) ON CONFLICT DO NOTHING")
            .execute(pool)
            .await?;
    }

    let applied: Vec<i32> = sqlx::query_scalar("SELECT version FROM schema_migrations")
        .fetch_all(pool)
        .await?;

    for (version, sql) in MIGRATIONS {
        if applied.contains(version) {
            continue;
        }
        tracing::info!("applying migration {}", version);
        let mut tx = pool.begin().await?;
        sqlx::raw_sql(sql)
            .execute(&mut *tx)
            .await
            .map_err(|e| DbError::Migration(format!("migration {}: {}", version, e)))?;
        sqlx::query("INSERT INTO schema_migrations (version) VALUES ($1)")
            .bind(version)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
    }
    Ok(())
}
```

- [ ] **Step 5: Run the test to verify it passes**

Run: `cd crates/randscan-db && DATABASE_URL=... cargo test --test accounts`
Expected: PASS. Also run it against a database that already had the old schema (any existing dev DB): version 1 gets recorded, version 2 applied.

- [ ] **Step 6: Commit**

```bash
git add migrations/002_accounts.sql crates/randscan-db/src/lib.rs crates/randscan-db/tests/accounts.rs
git commit -m "db: versioned migration runner and accounts schema"
```

---

### Task 2: User, session and API-key queries

**Files:**
- Modify: `crates/randscan-db/src/models.rs` (append)
- Create: `crates/randscan-db/src/queries/users.rs`
- Modify: `crates/randscan-db/src/queries/mod.rs`
- Test: `crates/randscan-db/tests/accounts.rs` (append)

**Interfaces:**
- Produces (all `pub async fn` in `randscan_db`, `Result<T>` = `randscan_db::Result`):
  - `create_user(pool: &PgPool, email: &str, password_hash: &str) -> Result<UserRow>` (unique violation → `DbError::Constraint`)
  - `get_user_by_email(pool, email: &str) -> Result<Option<UserRow>>`
  - `get_user(pool, id: i64) -> Result<Option<UserRow>>`
  - `touch_last_login(pool, id: i64) -> Result<()>`
  - `create_session(pool, token_hash: &str, user_id: i64, expires_at: DateTime<Utc>, user_agent: Option<&str>, ip: Option<&str>) -> Result<()>`
  - `get_session_user(pool, token_hash: &str) -> Result<Option<(SessionRow, UserRow)>>` (only unexpired)
  - `touch_session(pool, token_hash: &str, expires_at: DateTime<Utc>) -> Result<()>`
  - `delete_session(pool, token_hash: &str) -> Result<()>`
  - `delete_expired_sessions(pool, user_id: i64) -> Result<()>`
  - `list_api_keys(pool, user_id: i64) -> Result<Vec<ApiKeyRow>>`
  - `count_active_api_keys(pool, user_id: i64) -> Result<i64>`
  - `create_api_key(pool, user_id: i64, name: &str, prefix: &str, key_hash: &str) -> Result<ApiKeyRow>`
  - `revoke_api_key(pool, user_id: i64, id: i64) -> Result<bool>` (true if a row was revoked)
  - `get_active_api_key(pool, key_hash: &str) -> Result<Option<ApiKeyRow>>`
  - `record_api_key_use(pool, id: i64) -> Result<()>`
  - Rows: `UserRow { id: i64, email: String, password_hash: String, created_at: DateTime<Utc>, last_login_at: Option<DateTime<Utc>> }`, `SessionRow { token_hash: String, user_id: i64, expires_at: DateTime<Utc>, last_seen_at: DateTime<Utc> }`, `ApiKeyRow { id: i64, user_id: i64, name: String, prefix: String, created_at, last_used_at: Option<_>, request_count: i64, revoked_at: Option<_> }`.

- [ ] **Step 1: Write the failing tests**

Append to `crates/randscan-db/tests/accounts.rs`:

```rust
use chrono::{Duration, Utc};
use randscan_db as db;

fn unique_email() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("t{nanos}@example.com")
}

#[tokio::test]
async fn users_are_unique_by_email() {
    let Some(pool) = test_pool().await else { return };
    let email = unique_email();
    let u = db::create_user(&pool, &email, "hash").await.unwrap();
    assert_eq!(u.email, email);
    assert!(u.last_login_at.is_none());
    let dup = db::create_user(&pool, &email, "hash2").await;
    assert!(matches!(dup, Err(db::DbError::Constraint(_))));
    let found = db::get_user_by_email(&pool, &email).await.unwrap().unwrap();
    assert_eq!(found.id, u.id);
    assert_eq!(found.password_hash, "hash");
    db::touch_last_login(&pool, u.id).await.unwrap();
    assert!(db::get_user(&pool, u.id).await.unwrap().unwrap().last_login_at.is_some());
}

#[tokio::test]
async fn sessions_expire_and_delete() {
    let Some(pool) = test_pool().await else { return };
    let u = db::create_user(&pool, &unique_email(), "hash").await.unwrap();
    let live = format!("{:0>64}", format!("{:x}", u.id * 7 + 1));
    let dead = format!("{:0>64}", format!("{:x}", u.id * 7 + 2));
    db::create_session(&pool, &live, u.id, Utc::now() + Duration::days(1), Some("ua"), Some("1.2.3.4"))
        .await
        .unwrap();
    db::create_session(&pool, &dead, u.id, Utc::now() - Duration::days(1), None, None)
        .await
        .unwrap();

    let (s, user) = db::get_session_user(&pool, &live).await.unwrap().unwrap();
    assert_eq!(s.user_id, u.id);
    assert_eq!(user.id, u.id);
    assert!(db::get_session_user(&pool, &dead).await.unwrap().is_none());

    db::delete_expired_sessions(&pool, u.id).await.unwrap();
    let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sessions WHERE user_id = $1")
        .bind(u.id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(n, 1);

    let later = Utc::now() + Duration::days(30);
    db::touch_session(&pool, &live, later).await.unwrap();
    let (s, _) = db::get_session_user(&pool, &live).await.unwrap().unwrap();
    assert!((s.expires_at - later).num_seconds().abs() < 2);

    db::delete_session(&pool, &live).await.unwrap();
    assert!(db::get_session_user(&pool, &live).await.unwrap().is_none());
}

#[tokio::test]
async fn api_keys_lifecycle() {
    let Some(pool) = test_pool().await else { return };
    let u = db::create_user(&pool, &unique_email(), "hash").await.unwrap();
    let hash = format!("{:0>64}", format!("{:x}", u.id * 11 + 3));
    let k = db::create_api_key(&pool, u.id, "bot", "abcdefghijkl", &hash).await.unwrap();
    assert_eq!(k.name, "bot");
    assert_eq!(k.request_count, 0);
    assert_eq!(db::count_active_api_keys(&pool, u.id).await.unwrap(), 1);

    let active = db::get_active_api_key(&pool, &hash).await.unwrap().unwrap();
    assert_eq!(active.id, k.id);
    db::record_api_key_use(&pool, k.id).await.unwrap();
    let listed = db::list_api_keys(&pool, u.id).await.unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].request_count, 1);
    assert!(listed[0].last_used_at.is_some());

    // Another user cannot revoke it.
    assert!(!db::revoke_api_key(&pool, u.id + 1_000_000, k.id).await.unwrap());
    assert!(db::revoke_api_key(&pool, u.id, k.id).await.unwrap());
    assert!(!db::revoke_api_key(&pool, u.id, k.id).await.unwrap(), "already revoked");
    assert!(db::get_active_api_key(&pool, &hash).await.unwrap().is_none());
    assert_eq!(db::count_active_api_keys(&pool, u.id).await.unwrap(), 0);
    assert!(db::list_api_keys(&pool, u.id).await.unwrap()[0].revoked_at.is_some());
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd crates/randscan-db && DATABASE_URL=... cargo test --test accounts`
Expected: compile error (`create_user` not found).

- [ ] **Step 3: Add the row models**

Append to `crates/randscan-db/src/models.rs`:

```rust
#[derive(Debug, Clone, FromRow)]
pub struct UserRow {
    pub id: i64,
    pub email: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, FromRow)]
pub struct SessionRow {
    pub token_hash: String,
    pub user_id: i64,
    pub expires_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct ApiKeyRow {
    pub id: i64,
    pub user_id: i64,
    pub name: String,
    pub prefix: String,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub request_count: i64,
    pub revoked_at: Option<DateTime<Utc>>,
}
```

- [ ] **Step 4: Write the queries**

`crates/randscan-db/src/queries/users.rs`:

```rust
//! Users, cookie sessions and API keys. Hashes in, never secrets.

use crate::{ApiKeyRow, Result, SessionRow, UserRow};
use chrono::{DateTime, Utc};
use sqlx::PgPool;

const USER_COLS: &str = "id, email, password_hash, created_at, last_login_at";
const KEY_COLS: &str = "id, user_id, name, prefix, created_at, last_used_at, request_count, revoked_at";

pub async fn create_user(pool: &PgPool, email: &str, password_hash: &str) -> Result<UserRow> {
    let sql = format!("INSERT INTO users (email, password_hash) VALUES ($1, $2) RETURNING {USER_COLS}");
    Ok(sqlx::query_as::<_, UserRow>(&sql)
        .bind(email)
        .bind(password_hash)
        .fetch_one(pool)
        .await?)
}

pub async fn get_user_by_email(pool: &PgPool, email: &str) -> Result<Option<UserRow>> {
    let sql = format!("SELECT {USER_COLS} FROM users WHERE email = $1");
    Ok(sqlx::query_as::<_, UserRow>(&sql)
        .bind(email)
        .fetch_optional(pool)
        .await?)
}

pub async fn get_user(pool: &PgPool, id: i64) -> Result<Option<UserRow>> {
    let sql = format!("SELECT {USER_COLS} FROM users WHERE id = $1");
    Ok(sqlx::query_as::<_, UserRow>(&sql)
        .bind(id)
        .fetch_optional(pool)
        .await?)
}

pub async fn touch_last_login(pool: &PgPool, id: i64) -> Result<()> {
    sqlx::query("UPDATE users SET last_login_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn create_session(
    pool: &PgPool,
    token_hash: &str,
    user_id: i64,
    expires_at: DateTime<Utc>,
    user_agent: Option<&str>,
    ip: Option<&str>,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO sessions (token_hash, user_id, expires_at, user_agent, ip) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(token_hash)
    .bind(user_id)
    .bind(expires_at)
    .bind(user_agent.map(|s| s.chars().take(256).collect::<String>()))
    .bind(ip)
    .execute(pool)
    .await?;
    Ok(())
}

/// The session and its user, only while the session is unexpired.
pub async fn get_session_user(pool: &PgPool, token_hash: &str) -> Result<Option<(SessionRow, UserRow)>> {
    let row: Option<(String, i64, DateTime<Utc>, DateTime<Utc>, String, String, DateTime<Utc>, Option<DateTime<Utc>>)> =
        sqlx::query_as(
            "SELECT s.token_hash, s.user_id, s.expires_at, s.last_seen_at,
                    u.email, u.password_hash, u.created_at, u.last_login_at
             FROM sessions s JOIN users u ON u.id = s.user_id
             WHERE s.token_hash = $1 AND s.expires_at > NOW()",
        )
        .bind(token_hash)
        .fetch_optional(pool)
        .await?;
    Ok(row.map(|(token_hash, user_id, expires_at, last_seen_at, email, password_hash, created_at, last_login_at)| {
        (
            SessionRow { token_hash, user_id, expires_at, last_seen_at },
            UserRow { id: user_id, email, password_hash, created_at, last_login_at },
        )
    }))
}

pub async fn touch_session(pool: &PgPool, token_hash: &str, expires_at: DateTime<Utc>) -> Result<()> {
    sqlx::query("UPDATE sessions SET last_seen_at = NOW(), expires_at = $2 WHERE token_hash = $1")
        .bind(token_hash)
        .bind(expires_at)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete_session(pool: &PgPool, token_hash: &str) -> Result<()> {
    sqlx::query("DELETE FROM sessions WHERE token_hash = $1")
        .bind(token_hash)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete_expired_sessions(pool: &PgPool, user_id: i64) -> Result<()> {
    sqlx::query("DELETE FROM sessions WHERE user_id = $1 AND expires_at <= NOW()")
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn list_api_keys(pool: &PgPool, user_id: i64) -> Result<Vec<ApiKeyRow>> {
    let sql = format!("SELECT {KEY_COLS} FROM api_keys WHERE user_id = $1 ORDER BY created_at DESC, id DESC");
    Ok(sqlx::query_as::<_, ApiKeyRow>(&sql)
        .bind(user_id)
        .fetch_all(pool)
        .await?)
}

pub async fn count_active_api_keys(pool: &PgPool, user_id: i64) -> Result<i64> {
    Ok(
        sqlx::query_scalar("SELECT COUNT(*) FROM api_keys WHERE user_id = $1 AND revoked_at IS NULL")
            .bind(user_id)
            .fetch_one(pool)
            .await?,
    )
}

pub async fn create_api_key(
    pool: &PgPool,
    user_id: i64,
    name: &str,
    prefix: &str,
    key_hash: &str,
) -> Result<ApiKeyRow> {
    let sql = format!(
        "INSERT INTO api_keys (user_id, name, prefix, key_hash) VALUES ($1, $2, $3, $4) RETURNING {KEY_COLS}"
    );
    Ok(sqlx::query_as::<_, ApiKeyRow>(&sql)
        .bind(user_id)
        .bind(name)
        .bind(prefix)
        .bind(key_hash)
        .fetch_one(pool)
        .await?)
}

/// Revoke one of the user's active keys. Returns false when no such active key exists.
pub async fn revoke_api_key(pool: &PgPool, user_id: i64, id: i64) -> Result<bool> {
    let res = sqlx::query(
        "UPDATE api_keys SET revoked_at = NOW() WHERE id = $1 AND user_id = $2 AND revoked_at IS NULL",
    )
    .bind(id)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(res.rows_affected() > 0)
}

pub async fn get_active_api_key(pool: &PgPool, key_hash: &str) -> Result<Option<ApiKeyRow>> {
    let sql = format!("SELECT {KEY_COLS} FROM api_keys WHERE key_hash = $1 AND revoked_at IS NULL");
    Ok(sqlx::query_as::<_, ApiKeyRow>(&sql)
        .bind(key_hash)
        .fetch_optional(pool)
        .await?)
}

pub async fn record_api_key_use(pool: &PgPool, id: i64) -> Result<()> {
    sqlx::query("UPDATE api_keys SET last_used_at = NOW(), request_count = request_count + 1 WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}
```

Add `mod users;` and `pub use users::*;` to `crates/randscan-db/src/queries/mod.rs` (alphabetical, after `transactions`).

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cd crates/randscan-db && DATABASE_URL=... cargo test --test accounts`
Expected: 4 tests PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/randscan-db
git commit -m "db: user, session and API key queries"
```

---

### Task 3: Wire types in `randscan-core`

**Files:**
- Create: `crates/randscan-core/src/types/auth.rs`
- Modify: `crates/randscan-core/src/types/mod.rs`

**Interfaces:**
- Produces (in `randscan_core`): `User { id: i64, email: String, created_at: String, last_login_at: Option<String> }`, `UserResponse { user: User }`, `ApiKey { id: i64, name: String, prefix: String, created_at: String, last_used_at: Option<String>, request_count: i64, revoked_at: Option<String> }`, `CreatedApiKey { #[serde(flatten)] info: ApiKey, key: String }`, `SignupRequest { email: String, password: String }`, `LoginRequest { email, password }`, `CreateKeyRequest { name: String }`. Timestamps are RFC 3339 strings.

- [ ] **Step 1: Write the failing test**

In `crates/randscan-core/src/types/auth.rs` (create the file with the test only first):

```rust
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn created_key_flattens_info_and_adds_key() {
        let created = CreatedApiKey {
            info: ApiKey {
                id: 7,
                name: "bot".into(),
                prefix: "abcdefghijkl".into(),
                created_at: "2026-09-10T00:00:00+00:00".into(),
                last_used_at: None,
                request_count: 0,
                revoked_at: None,
            },
            key: "rsk_secret".into(),
        };
        let v = serde_json::to_value(&created).unwrap();
        assert_eq!(v["id"], 7);
        assert_eq!(v["prefix"], "abcdefghijkl");
        assert_eq!(v["key"], "rsk_secret");
        assert!(v["last_used_at"].is_null());
    }
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p randscan-core auth`
Expected: compile error (`CreatedApiKey` not found).

- [ ] **Step 3: Add the types**

Insert above the test module in `auth.rs`:

```rust
/// A registered explorer user (never includes the password hash).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub email: String,
    pub created_at: String,
    pub last_login_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse {
    pub user: User,
}

/// An API key as listed in the dashboard. The secret is never included.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: i64,
    pub name: String,
    pub prefix: String,
    pub created_at: String,
    pub last_used_at: Option<String>,
    pub request_count: i64,
    pub revoked_at: Option<String>,
}

/// Returned once, at creation: the key fields plus the full secret.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatedApiKey {
    #[serde(flatten)]
    pub info: ApiKey,
    pub key: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateKeyRequest {
    pub name: String,
}
```

In `types/mod.rs` add `mod auth;` (first, alphabetical) and `pub use auth::*;`.

- [ ] **Step 4: Run the test to verify it passes**

Run: `cargo test -p randscan-core auth`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/randscan-core
git commit -m "core: auth and API key wire types"
```

---

### Task 4: API config, state, errors and the `auth` module

**Files:**
- Modify: `Cargo.toml` (workspace deps), `crates/randscan-api/Cargo.toml`
- Modify: `crates/randscan-api/src/lib.rs`, `state.rs`, `error.rs`, `main.rs`
- Create: `crates/randscan-api/src/auth/mod.rs`, `password.rs`, `email.rs`, `keys.rs`, `session.rs`

**Interfaces:**
- Consumes: `randscan_db::{get_session_user, touch_session, UserRow}` (Task 2).
- Produces:
  - `ApiConfig { listen_addr, cookie_secure: bool, trust_proxy: bool, anon_rpm: u32, key_rpm: u32, auth_rpm: u32 }` with `ApiConfig::from_env()` and `Default`.
  - `AppState { db, indexer, ws_manager, config: Arc<ApiConfig>, limiter: Arc<RateLimiter> }` (the `RateLimiter` type is created in Task 5; this task adds a placeholder `pub struct RateLimiter` in `ratelimit.rs` with `new()` so the state compiles).
  - `AppError::Status { status: StatusCode, error: &'static str, message: String }`, `AppError::TooManyRequests { retry_after: u64 }`, constructors `AppError::bad(error, msg)`, `AppError::unauthorized(error, msg)`, `AppError::conflict(error, msg)`.
  - `auth::password::{validate_password(&str) -> Result<(), &'static str>, hash_password(String) -> Result<String, AppError>, verify_password(String, String) -> Result<bool, AppError>}`
  - `auth::email::normalize_email(&str) -> Option<String>`
  - `auth::keys::{KEY_PREFIX, generate_api_key() -> String, key_prefix(&str) -> String, hash_secret(&str) -> String, is_key_shaped(&str) -> bool, extract_api_key(&HeaderMap) -> Option<String>}`
  - `auth::session::{SESSION_COOKIE, SESSION_TTL: Duration, generate_session_token() -> String, session_cookie(&str, bool) -> Cookie<'static>, clear_session_cookie(bool) -> Cookie<'static>, AuthUser(pub UserRow) extractor, client_ip(&HeaderMap, Option<SocketAddr>, bool) -> Option<String>, user_to_wire(&UserRow) -> User}`

- [ ] **Step 1: Add dependencies**

Workspace `Cargo.toml` `[workspace.dependencies]` (add after `uuid`):

```toml
# Auth
argon2 = "0.5"
sha2 = "0.10"
rand = "0.8"
hex = "0.4"
axum-extra = { version = "0.9", features = ["cookie"] }
time = "0.3"
```

`crates/randscan-api/Cargo.toml` `[dependencies]` add:

```toml
argon2 = { workspace = true }
sha2 = { workspace = true }
rand = { workspace = true }
hex = { workspace = true }
axum-extra = { workspace = true }
time = { workspace = true }
chrono = { workspace = true }
```

and a new section:

```toml
[dev-dependencies]
tower = { version = "0.5", features = ["util"] }
http-body-util = "0.1"
```

Run `cargo build -p randscan-api` to fetch and confirm versions resolve.

- [ ] **Step 2: Write the failing unit tests**

Create `crates/randscan-api/src/auth/mod.rs`:

```rust
//! Sessions, passwords, API keys.

pub mod email;
pub mod keys;
pub mod password;
pub mod session;

pub use email::*;
pub use keys::*;
pub use password::*;
pub use session::*;
```

Create `crates/randscan-api/src/auth/password.rs` with only:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn password_policy_bounds() {
        assert!(validate_password("123456789").is_err());
        assert!(validate_password("1234567890").is_ok());
        assert!(validate_password(&"x".repeat(128)).is_ok());
        assert!(validate_password(&"x".repeat(129)).is_err());
    }

    #[tokio::test]
    async fn hash_and_verify_round_trip() {
        let hash = hash_password("correct horse battery".into()).await.unwrap();
        assert!(hash.starts_with("$argon2id$"));
        assert!(verify_password(hash.clone(), "correct horse battery".into()).await.unwrap());
        assert!(!verify_password(hash, "wrong".into()).await.unwrap());
    }
}
```

Create `crates/randscan-api/src/auth/email.rs` with only:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_and_validates() {
        assert_eq!(normalize_email("  Bob@Example.COM "), Some("bob@example.com".into()));
        assert_eq!(normalize_email("bob@example"), None);
        assert_eq!(normalize_email("@example.com"), None);
        assert_eq!(normalize_email("bob@"), None);
        assert_eq!(normalize_email("bob@@example.com"), None);
        assert_eq!(normalize_email("bob@exam ple.com"), None);
        let long = format!("{}@example.com", "a".repeat(250));
        assert_eq!(normalize_email(&long), None);
    }
}
```

Create `crates/randscan-api/src/auth/keys.rs` with only:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderMap;

    #[test]
    fn keys_have_the_documented_shape() {
        let k = generate_api_key();
        assert_eq!(k.len(), 4 + 48);
        assert!(k.starts_with(KEY_PREFIX));
        assert!(k[4..].chars().all(|c| c.is_ascii_alphanumeric()));
        assert_ne!(k, generate_api_key());
        assert_eq!(key_prefix(&k), k[4..16]);
        assert!(is_key_shaped(&k));
        assert!(!is_key_shaped("rsk_short"));
        assert!(!is_key_shaped("abc"));
    }

    #[test]
    fn hashing_is_sha256_hex() {
        assert_eq!(
            hash_secret("abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn extracts_from_either_header() {
        let mut h = HeaderMap::new();
        assert_eq!(extract_api_key(&h), None);
        h.insert("x-api-key", "rsk_x".parse().unwrap());
        assert_eq!(extract_api_key(&h).as_deref(), Some("rsk_x"));
        h.insert("authorization", "Bearer rsk_y".parse().unwrap());
        assert_eq!(extract_api_key(&h).as_deref(), Some("rsk_y"), "bearer wins");
        let mut h = HeaderMap::new();
        h.insert("authorization", "Basic abc".parse().unwrap());
        assert_eq!(extract_api_key(&h), None);
    }
}
```

Create `crates/randscan-api/src/auth/session.rs` with only:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderMap;
    use std::net::SocketAddr;

    #[test]
    fn tokens_are_32_bytes_hex() {
        let t = generate_session_token();
        assert_eq!(t.len(), 64);
        assert!(t.chars().all(|c| c.is_ascii_hexdigit()));
        assert_ne!(t, generate_session_token());
    }

    #[test]
    fn cookie_attributes() {
        let c = session_cookie("tok", true);
        assert_eq!(c.name(), SESSION_COOKIE);
        assert_eq!(c.value(), "tok");
        assert_eq!(c.http_only(), Some(true));
        assert_eq!(c.secure(), Some(true));
        assert_eq!(c.path(), Some("/"));
        assert_eq!(c.max_age(), Some(time::Duration::days(30)));
        let cleared = clear_session_cookie(false);
        assert_eq!(cleared.value(), "");
        assert_eq!(cleared.max_age(), Some(time::Duration::ZERO));
        assert_eq!(cleared.secure(), Some(false));
    }

    #[test]
    fn client_ip_honours_trust_proxy() {
        let peer: SocketAddr = "10.0.0.5:1234".parse().unwrap();
        let mut h = HeaderMap::new();
        h.insert("x-forwarded-for", "203.0.113.9, 10.0.0.1".parse().unwrap());
        assert_eq!(client_ip(&h, Some(peer), false).as_deref(), Some("10.0.0.5"));
        assert_eq!(client_ip(&h, Some(peer), true).as_deref(), Some("203.0.113.9"));
        assert_eq!(client_ip(&HeaderMap::new(), Some(peer), true).as_deref(), Some("10.0.0.5"));
        assert_eq!(client_ip(&HeaderMap::new(), None, true), None);
    }
}
```

Add `pub mod auth;` and `pub mod ratelimit;` to `crates/randscan-api/src/lib.rs`.

- [ ] **Step 3: Run the tests to verify they fail**

Run: `cargo test -p randscan-api --lib`
Expected: compile errors for the missing functions.

- [ ] **Step 4: Config, state, error, placeholder limiter**

`crates/randscan-api/src/lib.rs`: replace the `ApiConfig` struct and impl with:

```rust
#[derive(Debug, Clone)]
pub struct ApiConfig {
    pub listen_addr: SocketAddr,
    /// `Secure` attribute on the session cookie (disable for local http).
    pub cookie_secure: bool,
    /// Read the client IP from `X-Forwarded-For` (set when behind Caddy).
    pub trust_proxy: bool,
    /// Requests per minute per anonymous IP (0 disables).
    pub anon_rpm: u32,
    /// Requests per minute per API key (0 disables).
    pub key_rpm: u32,
    /// Login/signup attempts per minute per IP (0 disables).
    pub auth_rpm: u32,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            listen_addr: ([0, 0, 0, 0], 3000).into(),
            cookie_secure: true,
            trust_proxy: false,
            anon_rpm: 60,
            key_rpm: 600,
            auth_rpm: 10,
        }
    }
}

impl ApiConfig {
    pub fn from_env() -> Self {
        let d = Self::default();
        let host = std::env::var("API_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port: u16 = std::env::var("API_PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(3000);
        let flag = |k: &str, d: bool| {
            std::env::var(k)
                .ok()
                .map(|v| !matches!(v.trim().to_ascii_lowercase().as_str(), "0" | "false" | "no" | "off"))
                .unwrap_or(d)
        };
        let num = |k: &str, d: u32| std::env::var(k).ok().and_then(|s| s.parse().ok()).unwrap_or(d);
        Self {
            listen_addr: format!("{}:{}", host, port)
                .parse()
                .unwrap_or(d.listen_addr),
            cookie_secure: flag("COOKIE_SECURE", d.cookie_secure),
            trust_proxy: flag("TRUST_PROXY", d.trust_proxy),
            anon_rpm: num("ANON_RATE_LIMIT_RPM", d.anon_rpm),
            key_rpm: num("KEY_RATE_LIMIT_RPM", d.key_rpm),
            auth_rpm: num("AUTH_RATE_LIMIT_RPM", d.auth_rpm),
        }
    }
}
```

`crates/randscan-api/src/state.rs`:

```rust
use crate::{ratelimit::RateLimiter, ApiConfig};
use randscan_db::DbPool;
use randscan_indexer::IndexerService;
use randscan_ws::WsManager;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
    pub indexer: Arc<IndexerService>,
    pub ws_manager: Arc<WsManager>,
    pub config: Arc<ApiConfig>,
    pub limiter: Arc<RateLimiter>,
}
```

Create `crates/randscan-api/src/ratelimit.rs` placeholder (Task 5 fills it in):

```rust
//! Fixed-window rate limiting per anonymous IP or API key.

#[derive(Default)]
pub struct RateLimiter;

impl RateLimiter {
    pub fn new() -> Self {
        Self
    }
}
```

`crates/randscan-api/src/error.rs`: replace the enum and `IntoResponse` impl with:

```rust
#[derive(Error, Debug)]
pub enum AppError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("{error}: {message}")]
    Status {
        status: StatusCode,
        error: &'static str,
        message: String,
    },
    #[error("rate limited")]
    TooManyRequests { retry_after: u64 },
    #[error("internal: {0}")]
    Internal(String),
    #[error("database: {0}")]
    Database(#[from] randscan_db::DbError),
}

impl AppError {
    pub fn bad(error: &'static str, message: impl Into<String>) -> Self {
        Self::Status { status: StatusCode::BAD_REQUEST, error, message: message.into() }
    }

    pub fn unauthorized(error: &'static str, message: impl Into<String>) -> Self {
        Self::Status { status: StatusCode::UNAUTHORIZED, error, message: message.into() }
    }

    pub fn conflict(error: &'static str, message: impl Into<String>) -> Self {
        Self::Status { status: StatusCode::CONFLICT, error, message: message.into() }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            AppError::NotFound(what) => (StatusCode::NOT_FOUND, ApiError::not_found(&what)),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, ApiError::bad_request(&msg)),
            AppError::Status { status, error, message } => (status, ApiError::new(error, &message)),
            AppError::TooManyRequests { retry_after } => {
                let body = ApiError::new(
                    "rate_limited",
                    &format!("too many requests; retry in {} seconds", retry_after),
                );
                return (
                    StatusCode::TOO_MANY_REQUESTS,
                    [("retry-after", retry_after.to_string())],
                    Json(body),
                )
                    .into_response();
            }
            AppError::Internal(msg) => {
                tracing::error!("internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, ApiError::internal("internal error"))
            }
            AppError::Database(e) => {
                tracing::error!("database error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, ApiError::internal("database error"))
            }
        };
        (status, Json(body)).into_response()
    }
}
```

Add to `randscan_core::ApiError` (`crates/randscan-core/src/types/api.rs`, inside `impl ApiError`):

```rust
    /// Generic error with `code` derived from `error` (`invalid_credentials` -> `INVALID_CREDENTIALS`).
    pub fn new(error: &str, message: &str) -> Self {
        Self {
            error: error.into(),
            message: message.into(),
            code: Some(error.to_ascii_uppercase()),
        }
    }
```

`crates/randscan-api/src/main.rs`: build the state with the new fields:

```rust
    let api_config = Arc::new(ApiConfig::from_env());
    let state = AppState {
        db: db_pool,
        indexer,
        ws_manager,
        config: api_config.clone(),
        limiter: Arc::new(randscan_api::ratelimit::RateLimiter::new()),
    };
    let app = create_router(state);

    info!("listening on {}", api_config.listen_addr);
    let listener = tokio::net::TcpListener::bind(api_config.listen_addr).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await?;
```

(Remove the old `let api_config = ApiConfig::from_env();` line.)

- [ ] **Step 5: Implement the auth modules**

`password.rs` (above the tests):

```rust
//! Password policy and argon2id hashing (CPU-bound, so it runs on the blocking pool).

use crate::error::AppError;
use argon2::password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;

pub const MIN_PASSWORD: usize = 10;
pub const MAX_PASSWORD: usize = 128;

pub fn validate_password(password: &str) -> Result<(), &'static str> {
    let n = password.chars().count();
    if n < MIN_PASSWORD {
        return Err("password must be at least 10 characters");
    }
    if n > MAX_PASSWORD {
        return Err("password must be at most 128 characters");
    }
    Ok(())
}

pub async fn hash_password(password: String) -> Result<String, AppError> {
    tokio::task::spawn_blocking(move || {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map(|h| h.to_string())
            .map_err(|e| AppError::Internal(format!("hash: {}", e)))
    })
    .await
    .map_err(|e| AppError::Internal(format!("join: {}", e)))?
}

/// True when `password` matches `hash`. A malformed hash counts as no match.
pub async fn verify_password(hash: String, password: String) -> Result<bool, AppError> {
    tokio::task::spawn_blocking(move || {
        let Ok(parsed) = PasswordHash::new(&hash) else {
            return false;
        };
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok()
    })
    .await
    .map_err(|e| AppError::Internal(format!("join: {}", e)))
}

/// A real hash of a random value; verifying against it keeps login timing the same for
/// unknown emails and wrong passwords.
pub fn dummy_hash() -> &'static str {
    "$argon2id$v=19$m=19456,t=2,p=1$c2FsdHNhbHRzYWx0c2FsdA$dGhpcyBpcyBub3QgYSByZWFsIGhhc2ggYXQgYWxs"
}
```

`email.rs`:

```rust
//! Email normalisation. Deliverability is not checked.

pub const MAX_EMAIL: usize = 254;

/// Trim, lowercase and validate. `None` when the address is not acceptable.
pub fn normalize_email(raw: &str) -> Option<String> {
    let email = raw.trim().to_ascii_lowercase();
    if email.is_empty() || email.len() > MAX_EMAIL || email.chars().any(char::is_whitespace) {
        return None;
    }
    let (local, domain) = email.split_once('@')?;
    if local.is_empty() || domain.is_empty() || domain.contains('@') {
        return None;
    }
    if !domain.contains('.') || domain.starts_with('.') || domain.ends_with('.') {
        return None;
    }
    Some(email)
}
```

`keys.rs`:

```rust
//! API key format: `rsk_` + 48 base62 characters. Stored and looked up as SHA-256 hex.

use axum::http::HeaderMap;
use rand::{distributions::Alphanumeric, rngs::OsRng, Rng};
use sha2::{Digest, Sha256};

pub const KEY_PREFIX: &str = "rsk_";
pub const KEY_RANDOM_LEN: usize = 48;
pub const KEY_DISPLAY_PREFIX_LEN: usize = 12;

pub fn generate_api_key() -> String {
    let mut rng = OsRng;
    let body: String = (0..KEY_RANDOM_LEN)
        .map(|_| rng.sample(Alphanumeric) as char)
        .collect();
    format!("{KEY_PREFIX}{body}")
}

/// The first 12 characters after `rsk_`, for display in the dashboard.
pub fn key_prefix(key: &str) -> String {
    key.strip_prefix(KEY_PREFIX)
        .unwrap_or(key)
        .chars()
        .take(KEY_DISPLAY_PREFIX_LEN)
        .collect()
}

/// SHA-256 hex of a secret (session token or API key).
pub fn hash_secret(secret: &str) -> String {
    hex::encode(Sha256::digest(secret.as_bytes()))
}

pub fn is_key_shaped(s: &str) -> bool {
    s.len() == KEY_PREFIX.len() + KEY_RANDOM_LEN
        && s.starts_with(KEY_PREFIX)
        && s[KEY_PREFIX.len()..].chars().all(|c| c.is_ascii_alphanumeric())
}

/// `Authorization: Bearer rsk_...` wins over `X-API-Key`. Any other Authorization scheme is ignored.
pub fn extract_api_key(headers: &HeaderMap) -> Option<String> {
    if let Some(v) = headers.get("authorization").and_then(|v| v.to_str().ok()) {
        if let Some(rest) = v.strip_prefix("Bearer ").or_else(|| v.strip_prefix("bearer ")) {
            let t = rest.trim();
            if !t.is_empty() {
                return Some(t.to_string());
            }
        }
    }
    headers
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}
```

`session.rs`:

```rust
//! Cookie sessions: a random token in the cookie, its SHA-256 in the database.

use crate::{auth::hash_secret, error::AppError, state::AppState};
use axum::{
    async_trait,
    extract::{ConnectInfo, FromRequestParts},
    http::{request::Parts, HeaderMap},
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use chrono::Utc;
use rand::{rngs::OsRng, RngCore};
use randscan_core::User;
use randscan_db::{self as db, UserRow};
use std::net::SocketAddr;

pub const SESSION_COOKIE: &str = "randscan_session";
pub const SESSION_TTL: chrono::Duration = chrono::Duration::days(30);
/// Sliding expiry is refreshed at most this often.
const TOUCH_INTERVAL: chrono::Duration = chrono::Duration::minutes(5);

pub fn generate_session_token() -> String {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    hex::encode(bytes)
}

fn base_cookie(value: String, secure: bool) -> Cookie<'static> {
    Cookie::build((SESSION_COOKIE, value))
        .http_only(true)
        .secure(secure)
        .same_site(SameSite::Lax)
        .path("/")
        .build()
}

pub fn session_cookie(token: &str, secure: bool) -> Cookie<'static> {
    let mut c = base_cookie(token.to_string(), secure);
    c.set_max_age(time::Duration::days(30));
    c
}

pub fn clear_session_cookie(secure: bool) -> Cookie<'static> {
    let mut c = base_cookie(String::new(), secure);
    c.set_max_age(time::Duration::ZERO);
    c
}

/// Client address for logging and rate limiting.
pub fn client_ip(headers: &HeaderMap, peer: Option<SocketAddr>, trust_proxy: bool) -> Option<String> {
    if trust_proxy {
        if let Some(first) = headers
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.split(',').next())
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            return Some(first.to_string());
        }
    }
    peer.map(|p| p.ip().to_string())
}

pub fn user_to_wire(u: &UserRow) -> User {
    User {
        id: u.id,
        email: u.email.clone(),
        created_at: u.created_at.to_rfc3339(),
        last_login_at: u.last_login_at.map(|t| t.to_rfc3339()),
    }
}

/// The signed-in user, from the session cookie. Rejects with 401 `unauthorized`.
pub struct AuthUser(pub UserRow);

#[async_trait]
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, AppError> {
        let jar = CookieJar::from_request_parts(parts, state)
            .await
            .map_err(|_| AppError::Internal("cookie jar".into()))?;
        let token = jar
            .get(SESSION_COOKIE)
            .map(|c| c.value().to_string())
            .filter(|t| t.len() == 64 && t.chars().all(|c| c.is_ascii_hexdigit()))
            .ok_or_else(|| AppError::unauthorized("unauthorized", "sign in required"))?;
        let token_hash = hash_secret(&token);
        let (session, user) = db::get_session_user(state.db.inner(), &token_hash)
            .await?
            .ok_or_else(|| AppError::unauthorized("unauthorized", "sign in required"))?;

        if Utc::now() - session.last_seen_at > TOUCH_INTERVAL {
            let pool = state.db.inner().clone();
            tokio::spawn(async move {
                if let Err(e) = db::touch_session(&pool, &token_hash, Utc::now() + SESSION_TTL).await {
                    tracing::warn!("touch session: {}", e);
                }
            });
        }
        Ok(AuthUser(user))
    }
}

/// Peer address recorded by `into_make_service_with_connect_info`, if any.
pub fn peer_addr(parts_or_req_extensions: &axum::http::Extensions) -> Option<SocketAddr> {
    parts_or_req_extensions.get::<ConnectInfo<SocketAddr>>().map(|c| c.0)
}
```

- [ ] **Step 6: Add the `hash-password` subcommand**

At the top of `main()` in `crates/randscan-api/src/main.rs`, before `dotenvy`:

```rust
    if std::env::args().nth(1).as_deref() == Some("hash-password") {
        return hash_password_cli().await;
    }
```

and add the function:

```rust
/// `randscan-api hash-password` reads one line from stdin and prints the argon2id PHC string.
/// Operators use it with the password-reset runbook in the README.
async fn hash_password_cli() -> Result<()> {
    let mut line = String::new();
    std::io::stdin().read_line(&mut line)?;
    let password = line.trim_end_matches(['\r', '\n']).to_string();
    randscan_api::auth::validate_password(&password).map_err(|e| anyhow::anyhow!(e))?;
    let hash = randscan_api::auth::hash_password(password)
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;
    println!("{}", hash);
    Ok(())
}
```

- [ ] **Step 7: Run the tests to verify they pass**

Run: `cargo test -p randscan-api --lib && cargo clippy -p randscan-api --all-targets -- -D warnings && cargo fmt --all`
Expected: all unit tests PASS, no clippy warnings. Then `echo 'correct horse battery' | cargo run -q --bin randscan-api -- hash-password` prints a `$argon2id$` string.

- [ ] **Step 8: Commit**

```bash
git add Cargo.toml Cargo.lock crates/randscan-api crates/randscan-core/src/types/api.rs
git commit -m "api: config, errors and auth primitives (passwords, sessions, keys)"
```

---

### Task 5: Rate limiter and middleware

**Files:**
- Modify: `crates/randscan-api/src/ratelimit.rs` (replace placeholder)
- Modify: `crates/randscan-api/src/routes.rs`
- Modify: `crates/randscan-api/src/main.rs` (sweeper task)
- Create: `crates/randscan-api/tests/common/mod.rs`, `crates/randscan-api/tests/ratelimit.rs`

**Interfaces:**
- Consumes: `auth::{extract_api_key, is_key_shaped, hash_secret, client_ip, peer_addr}`, `db::{get_active_api_key, record_api_key_use}`, `AppState`, `AppError::TooManyRequests`.
- Produces:
  - `RateLimiter::new()`, `RateLimiter::check(&self, id: &str, limit: u32, now: u64) -> Decision`, `RateLimiter::sweep(&self, now: u64)`, `RateLimiter::spawn_sweeper(self: Arc<Self>)`.
  - `enum Decision { Allowed { remaining: u32 }, Limited { retry_after: u64 } }`
  - middleware `api_limit(State<AppState>, Request, Next) -> Response` and `auth_limit(...)`.
  - Request extension `ApiKeyId(pub i64)` inserted when a valid key was used.
  - Test helpers: `common::test_app(cfg: ApiConfig) -> Option<(Router, PgPool)>`, `common::call(app: &Router, req: Request<Body>) -> (StatusCode, HeaderMap, serde_json::Value)`, `common::json_req(method: &str, uri: &str, body: Option<Value>, cookie: Option<&str>) -> Request<Body>`, `common::unique_email() -> String`, `common::session_cookie_from(headers: &HeaderMap) -> String`.

- [ ] **Step 1: Write the failing unit tests**

Replace `ratelimit.rs` with just the tests for now:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_within_a_window_and_resets() {
        let l = RateLimiter::new();
        assert!(matches!(l.check("a", 2, 100), Decision::Allowed { remaining: 1 }));
        assert!(matches!(l.check("a", 2, 130), Decision::Allowed { remaining: 0 }));
        assert!(matches!(l.check("a", 2, 131), Decision::Limited { retry_after: 49 }));
        assert!(matches!(l.check("b", 2, 131), Decision::Allowed { remaining: 1 }), "other id");
        assert!(matches!(l.check("a", 2, 180), Decision::Allowed { remaining: 1 }), "next window");
    }

    #[test]
    fn zero_disables() {
        let l = RateLimiter::new();
        for _ in 0..1000 {
            assert!(matches!(l.check("a", 0, 5), Decision::Allowed { remaining: u32::MAX }));
        }
    }

    #[test]
    fn sweep_drops_old_windows() {
        let l = RateLimiter::new();
        l.check("old", 5, 100);
        l.check("new", 5, 500);
        l.sweep(500);
        assert_eq!(l.len(), 1);
    }
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p randscan-api --lib ratelimit`
Expected: compile error (`Decision` not found).

- [ ] **Step 3: Implement the limiter and middleware**

Above the tests in `ratelimit.rs`:

```rust
//! Fixed-window rate limiting per anonymous IP or API key, in memory.

use crate::{auth, error::AppError, state::AppState};
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::{IntoResponse, Response},
};
use randscan_db as db;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const WINDOW_SECS: u64 = 60;

#[derive(Debug, PartialEq, Eq)]
pub enum Decision {
    Allowed { remaining: u32 },
    Limited { retry_after: u64 },
}

#[derive(Default)]
pub struct RateLimiter {
    /// id -> (window start, count)
    windows: Mutex<HashMap<String, (u64, u32)>>,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Count one request for `id` at unix time `now`. `limit == 0` disables limiting.
    pub fn check(&self, id: &str, limit: u32, now: u64) -> Decision {
        if limit == 0 {
            return Decision::Allowed { remaining: u32::MAX };
        }
        let start = now - now % WINDOW_SECS;
        let mut map = self.windows.lock().unwrap_or_else(|p| p.into_inner());
        let entry = map.entry(id.to_string()).or_insert((start, 0));
        if entry.0 != start {
            *entry = (start, 0);
        }
        if entry.1 >= limit {
            return Decision::Limited { retry_after: start + WINDOW_SECS - now };
        }
        entry.1 += 1;
        Decision::Allowed { remaining: limit - entry.1 }
    }

    /// Drop windows that ended before `now`.
    pub fn sweep(&self, now: u64) {
        let mut map = self.windows.lock().unwrap_or_else(|p| p.into_inner());
        map.retain(|_, (start, _)| *start + WINDOW_SECS > now);
    }

    pub fn len(&self) -> usize {
        self.windows.lock().unwrap_or_else(|p| p.into_inner()).len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Sweep every 5 minutes for the life of the process.
    pub fn spawn_sweeper(self: Arc<Self>) {
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(300)).await;
                self.sweep(unix_now());
            }
        });
    }
}

pub fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Marker inserted into the request when a valid API key authenticated it.
#[derive(Clone, Copy, Debug)]
pub struct ApiKeyId(pub i64);

fn ip_of(state: &AppState, req: &Request) -> String {
    auth::client_ip(req.headers(), auth::peer_addr(req.extensions()), state.config.trust_proxy)
        .unwrap_or_else(|| "unknown".to_string())
}

fn with_headers(mut res: Response, limit: u32, remaining: u32) -> Response {
    if limit > 0 {
        let h = res.headers_mut();
        if let Ok(v) = limit.to_string().parse() {
            h.insert("x-ratelimit-limit", v);
        }
        if let Ok(v) = remaining.to_string().parse() {
            h.insert("x-ratelimit-remaining", v);
        }
    }
    res
}

/// Every `/api/v1` request: per-key when a key is presented (401 if it is unknown or revoked),
/// otherwise per-IP.
pub async fn api_limit(State(state): State<AppState>, mut req: Request, next: Next) -> Response {
    let (id, limit) = match auth::extract_api_key(req.headers()) {
        Some(key) => {
            if !auth::is_key_shaped(&key) {
                return AppError::unauthorized("invalid_api_key", "malformed API key").into_response();
            }
            let row = match db::get_active_api_key(state.db.inner(), &auth::hash_secret(&key)).await {
                Ok(Some(row)) => row,
                Ok(None) => {
                    return AppError::unauthorized("invalid_api_key", "unknown or revoked API key")
                        .into_response()
                }
                Err(e) => return AppError::from(e).into_response(),
            };
            req.extensions_mut().insert(ApiKeyId(row.id));
            let pool = state.db.inner().clone();
            let key_id = row.id;
            tokio::spawn(async move {
                if let Err(e) = db::record_api_key_use(&pool, key_id).await {
                    tracing::warn!("record api key use: {}", e);
                }
            });
            (format!("key:{}", row.id), state.config.key_rpm)
        }
        None => (format!("ip:{}", ip_of(&state, &req)), state.config.anon_rpm),
    };

    match state.limiter.check(&id, limit, unix_now()) {
        Decision::Allowed { remaining } => with_headers(next.run(req).await, limit, remaining),
        Decision::Limited { retry_after } => AppError::TooManyRequests { retry_after }.into_response(),
    }
}

/// Login and signup: a tighter per-IP budget on top of `api_limit`.
pub async fn auth_limit(State(state): State<AppState>, req: Request, next: Next) -> Response {
    let id = format!("auth:{}", ip_of(&state, &req));
    match state.limiter.check(&id, state.config.auth_rpm, unix_now()) {
        Decision::Allowed { .. } => next.run(req).await,
        Decision::Limited { retry_after } => AppError::TooManyRequests { retry_after }.into_response(),
    }
}
```

Wire the middleware in `routes.rs`: add `use axum::middleware;` and change the `api` router to end with `.layer(middleware::from_fn_with_state(state.clone(), crate::ratelimit::api_limit))` before it is nested (the auth routes are added in Task 6). `create_router` already takes `state` by value; clone it for the layer.

In `main.rs`, after building the state: `state.limiter.clone().spawn_sweeper();`.

- [ ] **Step 4: Run unit tests**

Run: `cargo test -p randscan-api --lib ratelimit`
Expected: 3 PASS.

- [ ] **Step 5: Write the integration test harness and the 429 test**

`crates/randscan-api/tests/common/mod.rs`:

```rust
//! Shared harness: a real database (`DATABASE_URL`), a stub indexer, `oneshot` requests.

#![allow(dead_code)]

use axum::{
    body::Body,
    http::{HeaderMap, Request, StatusCode},
    Router,
};
use http_body_util::BodyExt;
use randscan_api::{create_router, ratelimit::RateLimiter, ApiConfig, AppState};
use randscan_db::{run_migrations, DbPool};
use randscan_indexer::{Broadcaster, IndexerConfig, IndexerService};
use randscan_ws::WsManager;
use serde_json::Value;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::sync::Arc;
use tower::ServiceExt;

/// Limits off, proxy trusted so tests can pick their own client IP, insecure cookies.
pub fn test_config() -> ApiConfig {
    ApiConfig {
        cookie_secure: false,
        trust_proxy: true,
        anon_rpm: 0,
        key_rpm: 0,
        auth_rpm: 0,
        ..ApiConfig::default()
    }
}

pub async fn test_app(cfg: ApiConfig) -> Option<(Router, PgPool)> {
    let url = std::env::var("DATABASE_URL").ok()?;
    let pool = PgPoolOptions::new()
        .max_connections(4)
        .connect(&url)
        .await
        .expect("connect to DATABASE_URL");
    run_migrations(&pool).await.expect("migrations");
    let db = DbPool::new(pool.clone());
    let indexer = Arc::new(IndexerService::new(IndexerConfig::default(), db.clone(), Broadcaster::new()));
    let state = AppState {
        db,
        indexer,
        ws_manager: Arc::new(WsManager::new()),
        config: Arc::new(cfg),
        limiter: Arc::new(RateLimiter::new()),
    };
    Some((create_router(state), pool))
}

pub fn unique_email() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("t{nanos}@example.com")
}

pub fn json_req(method: &str, uri: &str, body: Option<Value>, cookie: Option<&str>) -> Request<Body> {
    let mut b = Request::builder()
        .method(method)
        .uri(uri)
        .header("x-forwarded-for", "198.51.100.7");
    if let Some(c) = cookie {
        b = b.header("cookie", c);
    }
    match body {
        Some(v) => b
            .header("content-type", "application/json")
            .body(Body::from(v.to_string()))
            .unwrap(),
        None => b.body(Body::empty()).unwrap(),
    }
}

pub async fn call(app: &Router, req: Request<Body>) -> (StatusCode, HeaderMap, Value) {
    let res = app.clone().oneshot(req).await.unwrap();
    let status = res.status();
    let headers = res.headers().clone();
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let json = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap_or(Value::Null)
    };
    (status, headers, json)
}

/// `name=value` of the session cookie from a Set-Cookie header.
pub fn session_cookie_from(headers: &HeaderMap) -> String {
    headers
        .get_all("set-cookie")
        .iter()
        .filter_map(|v| v.to_str().ok())
        .find(|v| v.starts_with("randscan_session="))
        .and_then(|v| v.split(';').next())
        .expect("session cookie")
        .to_string()
}
```

`crates/randscan-api/tests/ratelimit.rs`:

```rust
mod common;

use common::*;
use serde_json::json;

#[tokio::test]
async fn anonymous_requests_are_limited_per_ip() {
    let cfg = randscan_api::ApiConfig { anon_rpm: 3, ..test_config() };
    let Some((app, _)) = test_app(cfg).await else {
        eprintln!("skipping: DATABASE_URL unset");
        return;
    };
    for i in 0..3 {
        let (status, headers, _) = call(&app, json_req("GET", "/api/v1/health", None, None)).await;
        assert_eq!(status, 200, "request {i}");
        assert_eq!(headers["x-ratelimit-limit"], "3");
        assert_eq!(headers["x-ratelimit-remaining"], (2 - i).to_string());
    }
    let (status, headers, body) = call(&app, json_req("GET", "/api/v1/health", None, None)).await;
    assert_eq!(status, 429);
    assert!(headers.contains_key("retry-after"));
    assert_eq!(body["error"], "rate_limited");

    // A different client IP has its own budget.
    let req = axum::http::Request::builder()
        .uri("/api/v1/health")
        .header("x-forwarded-for", "203.0.113.1")
        .body(axum::body::Body::empty())
        .unwrap();
    let (status, _, _) = call(&app, req).await;
    assert_eq!(status, 200);
}

#[tokio::test]
async fn unknown_api_key_is_rejected_not_downgraded() {
    let Some((app, _)) = test_app(test_config()).await else { return };
    let key = format!("rsk_{}", "A".repeat(48));
    let req = axum::http::Request::builder()
        .uri("/api/v1/health")
        .header("authorization", format!("Bearer {key}"))
        .body(axum::body::Body::empty())
        .unwrap();
    let (status, _, body) = call(&app, req).await;
    assert_eq!(status, 401);
    assert_eq!(body["error"], "invalid_api_key");

    let req = axum::http::Request::builder()
        .uri("/api/v1/health")
        .header("x-api-key", "rsk_tooshort")
        .body(axum::body::Body::empty())
        .unwrap();
    let (status, _, _) = call(&app, req).await;
    assert_eq!(status, 401);

    let _ = json!({});
}
```

- [ ] **Step 6: Run the integration tests**

Run: `DATABASE_URL=... cargo test -p randscan-api --test ratelimit`
Expected: 2 PASS. Then `cargo clippy -p randscan-api --all-targets -- -D warnings && cargo fmt --all`.

- [ ] **Step 7: Commit**

```bash
git add crates/randscan-api
git commit -m "api: per-IP and per-key rate limiting"
```

---

### Task 6: Signup, login, logout, me

**Files:**
- Create: `crates/randscan-api/src/handlers/auth.rs`
- Modify: `crates/randscan-api/src/handlers/mod.rs`, `crates/randscan-api/src/routes.rs`
- Test: `crates/randscan-api/tests/auth.rs`

**Interfaces:**
- Consumes: Task 4 auth module, Task 2 queries, Task 3 types, Task 5 `auth_limit`.
- Produces: `handlers::{signup, login, logout, me}` and routes `POST /api/v1/auth/signup`, `POST /api/v1/auth/login`, `POST /api/v1/auth/logout`, `GET /api/v1/auth/me`.

- [ ] **Step 1: Write the failing integration tests**

`crates/randscan-api/tests/auth.rs`:

```rust
mod common;

use common::*;
use serde_json::json;

#[tokio::test]
async fn signup_login_me_logout() {
    let Some((app, _)) = test_app(test_config()).await else {
        eprintln!("skipping: DATABASE_URL unset");
        return;
    };
    let email = unique_email();
    let body = json!({ "email": format!("  {}  ", email.to_uppercase()), "password": "correct horse battery" });

    let (status, headers, json) = call(&app, json_req("POST", "/api/v1/auth/signup", Some(body.clone()), None)).await;
    assert_eq!(status, 201, "{json}");
    assert_eq!(json["user"]["email"], email, "normalised");
    assert!(json["user"]["last_login_at"].is_null());
    let set_cookie = headers["set-cookie"].to_str().unwrap();
    assert!(set_cookie.contains("HttpOnly"), "{set_cookie}");
    assert!(set_cookie.contains("SameSite=Lax"));
    assert!(set_cookie.contains("Path=/"));
    assert!(set_cookie.contains("Max-Age=2592000"));
    assert!(!set_cookie.contains("Secure"), "cookie_secure=false in tests");
    let cookie = session_cookie_from(&headers);

    let (status, _, json) = call(&app, json_req("GET", "/api/v1/auth/me", None, Some(&cookie))).await;
    assert_eq!(status, 200);
    assert_eq!(json["user"]["email"], email);

    // Duplicate signup.
    let (status, _, json) = call(&app, json_req("POST", "/api/v1/auth/signup", Some(body), None)).await;
    assert_eq!(status, 409);
    assert_eq!(json["error"], "email_taken");

    // Login with the wrong password and with an unknown email give the same body.
    let (s1, _, j1) = call(&app, json_req("POST", "/api/v1/auth/login", Some(json!({ "email": email, "password": "wrong password!" })), None)).await;
    let (s2, _, j2) = call(&app, json_req("POST", "/api/v1/auth/login", Some(json!({ "email": unique_email(), "password": "wrong password!" })), None)).await;
    assert_eq!((s1, s2), (401, 401));
    assert_eq!(j1, j2);
    assert_eq!(j1["error"], "invalid_credentials");

    // Real login sets last_login_at.
    let (status, headers, json) = call(&app, json_req("POST", "/api/v1/auth/login", Some(json!({ "email": email, "password": "correct horse battery" })), None)).await;
    assert_eq!(status, 200);
    assert!(json["user"]["last_login_at"].is_string());
    let cookie2 = session_cookie_from(&headers);

    // Logout invalidates that session only.
    let (status, headers, _) = call(&app, json_req("POST", "/api/v1/auth/logout", None, Some(&cookie2))).await;
    assert_eq!(status, 204);
    assert!(headers["set-cookie"].to_str().unwrap().contains("Max-Age=0"));
    let (status, _, _) = call(&app, json_req("GET", "/api/v1/auth/me", None, Some(&cookie2))).await;
    assert_eq!(status, 401);
    let (status, _, _) = call(&app, json_req("GET", "/api/v1/auth/me", None, Some(&cookie))).await;
    assert_eq!(status, 200, "first session still valid");
}

#[tokio::test]
async fn signup_validation() {
    let Some((app, _)) = test_app(test_config()).await else { return };
    let (status, _, json) = call(&app, json_req("POST", "/api/v1/auth/signup", Some(json!({ "email": "nope", "password": "correct horse battery" })), None)).await;
    assert_eq!(status, 400);
    assert_eq!(json["error"], "invalid_email");
    let (status, _, json) = call(&app, json_req("POST", "/api/v1/auth/signup", Some(json!({ "email": unique_email(), "password": "short" })), None)).await;
    assert_eq!(status, 400);
    assert_eq!(json["error"], "weak_password");
    let (status, _, json) = call(&app, json_req("GET", "/api/v1/auth/me", None, None)).await;
    assert_eq!(status, 401);
    assert_eq!(json["error"], "unauthorized");
    let (status, _, _) = call(&app, json_req("GET", "/api/v1/auth/me", None, Some("randscan_session=deadbeef"))).await;
    assert_eq!(status, 401);
}

#[tokio::test]
async fn login_attempts_are_limited() {
    let cfg = randscan_api::ApiConfig { auth_rpm: 2, ..test_config() };
    let Some((app, _)) = test_app(cfg).await else { return };
    let body = json!({ "email": unique_email(), "password": "does not matter" });
    for _ in 0..2 {
        let (status, _, _) = call(&app, json_req("POST", "/api/v1/auth/login", Some(body.clone()), None)).await;
        assert_eq!(status, 401);
    }
    let (status, _, json) = call(&app, json_req("POST", "/api/v1/auth/login", Some(body), None)).await;
    assert_eq!(status, 429);
    assert_eq!(json["error"], "rate_limited");
    // Public reads are not affected by the auth limiter.
    let (status, _, _) = call(&app, json_req("GET", "/api/v1/health", None, None)).await;
    assert_eq!(status, 200);
}
```

- [ ] **Step 2: Run to verify failure**

Run: `DATABASE_URL=... cargo test -p randscan-api --test auth`
Expected: FAIL with 404 on `/api/v1/auth/signup` (route missing).

- [ ] **Step 3: Implement the handlers**

`crates/randscan-api/src/handlers/auth.rs`:

```rust
//! Signup, login, logout, me. Passwords never leave this module unhashed.

use crate::{
    auth::{
        clear_session_cookie, client_ip, dummy_hash, generate_session_token, hash_password,
        hash_secret, normalize_email, peer_addr, session_cookie, user_to_wire, validate_password,
        verify_password, AuthUser, SESSION_TTL,
    },
    error::AppError,
    state::AppState,
    ApiResult,
};
use axum::{
    extract::State,
    http::{request::Parts, HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use axum_extra::extract::CookieJar;
use chrono::Utc;
use randscan_core::{LoginRequest, SignupRequest, UserResponse};
use randscan_db::{self as db, DbError, UserRow};

/// Where a request came from, for the session row.
struct Origin {
    user_agent: Option<String>,
    ip: Option<String>,
}

impl Origin {
    fn from_parts(state: &AppState, headers: &HeaderMap, parts: &Parts) -> Self {
        Self {
            user_agent: headers
                .get("user-agent")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string()),
            ip: client_ip(headers, peer_addr(&parts.extensions), state.config.trust_proxy),
        }
    }
}

async fn start_session(
    state: &AppState,
    jar: CookieJar,
    user: &UserRow,
    origin: Origin,
) -> ApiResult<CookieJar> {
    let token = generate_session_token();
    db::create_session(
        state.db.inner(),
        &hash_secret(&token),
        user.id,
        Utc::now() + SESSION_TTL,
        origin.user_agent.as_deref(),
        origin.ip.as_deref(),
    )
    .await?;
    Ok(jar.add(session_cookie(&token, state.config.cookie_secure)))
}

/// POST /api/v1/auth/signup
pub async fn signup(
    State(state): State<AppState>,
    jar: CookieJar,
    parts: Parts,
    Json(req): Json<SignupRequest>,
) -> ApiResult<impl IntoResponse> {
    let email = normalize_email(&req.email)
        .ok_or_else(|| AppError::bad("invalid_email", "enter a valid email address"))?;
    validate_password(&req.password).map_err(|m| AppError::bad("weak_password", m))?;
    let hash = hash_password(req.password).await?;
    let user = match db::create_user(state.db.inner(), &email, &hash).await {
        Ok(u) => u,
        Err(DbError::Constraint(_)) => {
            return Err(AppError::conflict("email_taken", "an account with this email already exists"))
        }
        Err(e) => return Err(e.into()),
    };
    let origin = Origin::from_parts(&state, &parts.headers, &parts);
    let jar = start_session(&state, jar, &user, origin).await?;
    Ok((StatusCode::CREATED, jar, Json(UserResponse { user: user_to_wire(&user) })))
}

/// POST /api/v1/auth/login
pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    parts: Parts,
    Json(req): Json<LoginRequest>,
) -> ApiResult<impl IntoResponse> {
    let invalid = || AppError::unauthorized("invalid_credentials", "email or password is incorrect");
    let email = normalize_email(&req.email).ok_or_else(invalid)?;
    let user = db::get_user_by_email(state.db.inner(), &email).await?;
    // Always run one verification so unknown emails take as long as wrong passwords.
    let hash = user
        .as_ref()
        .map(|u| u.password_hash.clone())
        .unwrap_or_else(|| dummy_hash().to_string());
    let ok = verify_password(hash, req.password).await?;
    let user = match user {
        Some(u) if ok => u,
        _ => return Err(invalid()),
    };
    db::delete_expired_sessions(state.db.inner(), user.id).await?;
    db::touch_last_login(state.db.inner(), user.id).await?;
    let user = db::get_user(state.db.inner(), user.id).await?.unwrap_or(user);
    let origin = Origin::from_parts(&state, &parts.headers, &parts);
    let jar = start_session(&state, jar, &user, origin).await?;
    Ok((StatusCode::OK, jar, Json(UserResponse { user: user_to_wire(&user) })))
}

/// POST /api/v1/auth/logout
pub async fn logout(
    State(state): State<AppState>,
    AuthUser(_user): AuthUser,
    jar: CookieJar,
) -> ApiResult<impl IntoResponse> {
    if let Some(c) = jar.get(crate::auth::SESSION_COOKIE) {
        db::delete_session(state.db.inner(), &hash_secret(c.value())).await?;
    }
    let jar = jar.add(clear_session_cookie(state.config.cookie_secure));
    Ok((StatusCode::NO_CONTENT, jar))
}

/// GET /api/v1/auth/me
pub async fn me(AuthUser(user): AuthUser) -> ApiResult<Json<UserResponse>> {
    Ok(Json(UserResponse { user: user_to_wire(&user) }))
}
```

Note on extractor order: axum requires the body extractor (`Json`) last; `Parts` is `axum::http::request::Parts`, which implements `FromRequestParts` and gives access to headers and extensions. If the compiler rejects `Parts` alongside `CookieJar`, replace `parts: Parts` with `headers: HeaderMap, ConnectInfo(peer): ConnectInfo<SocketAddr>` is NOT possible in tests (no connect info); keep `Parts`.

Add `mod auth;` and `pub use auth::*;` to `handlers/mod.rs`.

`routes.rs`, inside `create_router` before the `api` router:

```rust
    let auth_public = Router::new()
        .route("/auth/signup", post(handlers::signup))
        .route("/auth/login", post(handlers::login))
        .layer(middleware::from_fn_with_state(state.clone(), crate::ratelimit::auth_limit));
```

and add to the `api` router: `.route("/auth/logout", post(handlers::logout))`, `.route("/auth/me", get(handlers::me))`, `.merge(auth_public)` (before the `api_limit` layer). Import `routing::{get, post}`.

- [ ] **Step 4: Run tests**

Run: `DATABASE_URL=... cargo test -p randscan-api --test auth && cargo clippy -p randscan-api --all-targets -- -D warnings && cargo fmt --all`
Expected: 3 PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/randscan-api
git commit -m "api: signup, login, logout and me"
```

---

### Task 7: API key management endpoints

**Files:**
- Create: `crates/randscan-api/src/handlers/keys.rs`
- Modify: `crates/randscan-api/src/handlers/mod.rs`, `crates/randscan-api/src/routes.rs`
- Test: `crates/randscan-api/tests/keys.rs`

**Interfaces:**
- Produces: `GET /api/v1/keys`, `POST /api/v1/keys`, `DELETE /api/v1/keys/:id`; constant `MAX_ACTIVE_KEYS = 10`.

- [ ] **Step 1: Write the failing tests**

`crates/randscan-api/tests/keys.rs`:

```rust
mod common;

use axum::{body::Body, http::Request};
use common::*;
use serde_json::json;

async fn signed_in(app: &axum::Router) -> String {
    let body = json!({ "email": unique_email(), "password": "correct horse battery" });
    let (status, headers, _) = call(app, json_req("POST", "/api/v1/auth/signup", Some(body), None)).await;
    assert_eq!(status, 201);
    session_cookie_from(&headers)
}

#[tokio::test]
async fn create_use_list_revoke() {
    let Some((app, _)) = test_app(test_config()).await else {
        eprintln!("skipping: DATABASE_URL unset");
        return;
    };
    let cookie = signed_in(&app).await;

    let (status, _, created) = call(&app, json_req("POST", "/api/v1/keys", Some(json!({ "name": " exchange bot " })), Some(&cookie))).await;
    assert_eq!(status, 201, "{created}");
    let key = created["key"].as_str().unwrap().to_string();
    assert!(key.starts_with("rsk_"));
    assert_eq!(key.len(), 52);
    assert_eq!(created["name"], "exchange bot");
    assert_eq!(created["prefix"], &key[4..16]);
    assert_eq!(created["request_count"], 0);
    let id = created["id"].as_i64().unwrap();

    // The list never contains the secret.
    let (status, _, list) = call(&app, json_req("GET", "/api/v1/keys", None, Some(&cookie))).await;
    assert_eq!(status, 200);
    assert_eq!(list.as_array().unwrap().len(), 1);
    assert!(list[0].get("key").is_none());

    // Use the key on a public endpoint.
    let req = Request::builder()
        .uri("/api/v1/stats")
        .header("authorization", format!("Bearer {key}"))
        .body(Body::empty())
        .unwrap();
    let (status, _, _) = call(&app, req).await;
    assert_eq!(status, 200);
    tokio::time::sleep(std::time::Duration::from_millis(200)).await; // usage is recorded asynchronously
    let (_, _, list) = call(&app, json_req("GET", "/api/v1/keys", None, Some(&cookie))).await;
    assert_eq!(list[0]["request_count"], 1);
    assert!(list[0]["last_used_at"].is_string());

    // A key cannot manage keys.
    let req = Request::builder()
        .uri("/api/v1/keys")
        .header("authorization", format!("Bearer {key}"))
        .body(Body::empty())
        .unwrap();
    let (status, _, _) = call(&app, req).await;
    assert_eq!(status, 401);

    // Revoke, then the key is rejected and cannot be revoked twice.
    let (status, _, _) = call(&app, json_req("DELETE", &format!("/api/v1/keys/{id}"), None, Some(&cookie))).await;
    assert_eq!(status, 204);
    let (status, _, _) = call(&app, json_req("DELETE", &format!("/api/v1/keys/{id}"), None, Some(&cookie))).await;
    assert_eq!(status, 404);
    let req = Request::builder()
        .uri("/api/v1/stats")
        .header("x-api-key", &key)
        .body(Body::empty())
        .unwrap();
    let (status, _, body) = call(&app, req).await;
    assert_eq!(status, 401);
    assert_eq!(body["error"], "invalid_api_key");
    let (_, _, list) = call(&app, json_req("GET", "/api/v1/keys", None, Some(&cookie))).await;
    assert!(list[0]["revoked_at"].is_string());

    // Another user cannot revoke it either.
    let other = signed_in(&app).await;
    let (status, _, _) = call(&app, json_req("DELETE", &format!("/api/v1/keys/{id}"), None, Some(&other))).await;
    assert_eq!(status, 404);
}

#[tokio::test]
async fn key_validation_and_limit() {
    let Some((app, _)) = test_app(test_config()).await else { return };
    let cookie = signed_in(&app).await;
    let (status, _, body) = call(&app, json_req("POST", "/api/v1/keys", Some(json!({ "name": "   " })), Some(&cookie))).await;
    assert_eq!(status, 400);
    assert_eq!(body["error"], "invalid_name");
    let (status, _, _) = call(&app, json_req("POST", "/api/v1/keys", Some(json!({ "name": "x".repeat(65) })), Some(&cookie))).await;
    assert_eq!(status, 400);
    for i in 0..10 {
        let (status, _, _) = call(&app, json_req("POST", "/api/v1/keys", Some(json!({ "name": format!("k{i}") })), Some(&cookie))).await;
        assert_eq!(status, 201);
    }
    let (status, _, body) = call(&app, json_req("POST", "/api/v1/keys", Some(json!({ "name": "one too many" })), Some(&cookie))).await;
    assert_eq!(status, 409);
    assert_eq!(body["error"], "key_limit");
    let (status, _, _) = call(&app, json_req("POST", "/api/v1/keys", Some(json!({ "name": "anon" })), None)).await;
    assert_eq!(status, 401);
}
```

- [ ] **Step 2: Run to verify failure**

Run: `DATABASE_URL=... cargo test -p randscan-api --test keys`
Expected: FAIL with 404 on `/api/v1/keys`.

- [ ] **Step 3: Implement**

`crates/randscan-api/src/handlers/keys.rs`:

```rust
//! API keys: list, create (secret shown once), revoke. Sessions only.

use crate::{
    auth::{generate_api_key, hash_secret, key_prefix, AuthUser},
    error::AppError,
    state::AppState,
    ApiResult,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use randscan_core::{ApiKey, CreateKeyRequest, CreatedApiKey};
use randscan_db::{self as db, ApiKeyRow};

pub const MAX_ACTIVE_KEYS: i64 = 10;
pub const MAX_KEY_NAME: usize = 64;

fn to_wire(k: ApiKeyRow) -> ApiKey {
    ApiKey {
        id: k.id,
        name: k.name,
        prefix: k.prefix,
        created_at: k.created_at.to_rfc3339(),
        last_used_at: k.last_used_at.map(|t| t.to_rfc3339()),
        request_count: k.request_count,
        revoked_at: k.revoked_at.map(|t| t.to_rfc3339()),
    }
}

/// GET /api/v1/keys
pub async fn list_keys(State(state): State<AppState>, AuthUser(user): AuthUser) -> ApiResult<Json<Vec<ApiKey>>> {
    let rows = db::list_api_keys(state.db.inner(), user.id).await?;
    Ok(Json(rows.into_iter().map(to_wire).collect()))
}

/// POST /api/v1/keys
pub async fn create_key(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(req): Json<CreateKeyRequest>,
) -> ApiResult<(StatusCode, Json<CreatedApiKey>)> {
    let name = req.name.trim();
    if name.is_empty() || name.chars().count() > MAX_KEY_NAME {
        return Err(AppError::bad("invalid_name", "name must be 1 to 64 characters"));
    }
    let pool = state.db.inner();
    if db::count_active_api_keys(pool, user.id).await? >= MAX_ACTIVE_KEYS {
        return Err(AppError::conflict("key_limit", "at most 10 active keys; revoke one first"));
    }
    let key = generate_api_key();
    let row = db::create_api_key(pool, user.id, name, &key_prefix(&key), &hash_secret(&key)).await?;
    Ok((StatusCode::CREATED, Json(CreatedApiKey { info: to_wire(row), key })))
}

/// DELETE /api/v1/keys/:id
pub async fn revoke_key(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<i64>,
) -> ApiResult<StatusCode> {
    if db::revoke_api_key(state.db.inner(), user.id, id).await? {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound("api key".into()))
    }
}
```

Add `mod keys;` / `pub use keys::*;` to `handlers/mod.rs`. In `routes.rs` add to the `api` router: `.route("/keys", get(handlers::list_keys).post(handlers::create_key))` and `.route("/keys/:id", delete(handlers::revoke_key))` (import `delete`).

- [ ] **Step 4: Run all API tests**

Run: `DATABASE_URL=... cargo test -p randscan-api && cargo clippy --all-targets -- -D warnings && cargo fmt --all -- --check`
Expected: all PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/randscan-api
git commit -m "api: API key management endpoints"
```

---

### Task 8: Frontend types, client, hooks and form styles

**Files:**
- Modify: `frontend/src/types/index.ts`, `frontend/src/lib/api.ts`, `frontend/src/hooks/useApi.ts`, `frontend/src/app/globals.css`

**Interfaces:**
- Produces (TypeScript): types `User`, `ApiKey`, `CreatedApiKey`; `api.signup(email, password): Promise<User>`, `api.login(email, password): Promise<User>`, `api.logout(): Promise<void>`, `api.getMe(): Promise<User>`, `api.listKeys(): Promise<ApiKey[]>`, `api.createKey(name): Promise<CreatedApiKey>`, `api.revokeKey(id): Promise<void>`; hooks `useMe(): SWRResponse<User | null>`, `useApiKeys(enabled: boolean): SWRResponse<ApiKey[]>`; CSS class `.input`.

- [ ] **Step 1: Types**

Append to `frontend/src/types/index.ts`:

```ts
// ---------------------------------------------------------------------------
// Accounts and API keys
// ---------------------------------------------------------------------------

export interface User {
  id: number;
  email: string;
  created_at: string;
  last_login_at: string | null;
}

export interface ApiKey {
  id: number;
  name: string;
  prefix: string;
  created_at: string;
  last_used_at: string | null;
  request_count: number;
  revoked_at: string | null;
}

/** Returned once at creation; `key` is the full secret. */
export interface CreatedApiKey extends ApiKey {
  key: string;
}
```

- [ ] **Step 2: Client**

In `frontend/src/lib/api.ts`:

1. Change the base URL so the browser always talks same-origin (cookies are first-party and the Next.js rewrite proxies to the API when `NEXT_PUBLIC_API_URL` is set):

```ts
/**
 * Requests always go to the same origin (`/api/v1/...`). In production Caddy proxies them;
 * in development `next.config.js` rewrites them to `NEXT_PUBLIC_API_URL`. Same-origin is what
 * lets the session cookie work.
 */
export const API_BASE_URL = '';
```

2. In `request()`, add `credentials: 'same-origin'` to the `fetch` options.

3. Add a JSON helper and the functions at the end of the file:

```ts
// ---------------------------------------------------------------------------
// Accounts and API keys (cookie session; same-origin only)
// ---------------------------------------------------------------------------

function jsonInit(method: string, body?: unknown): RequestInit {
  return {
    method,
    headers: body === undefined ? {} : { 'Content-Type': 'application/json' },
    body: body === undefined ? undefined : JSON.stringify(body),
  };
}

async function requestVoid(path: string, init: RequestInit): Promise<void> {
  const response = await fetch(`${API_BASE_URL}${API_PREFIX}${path}`, {
    ...init,
    credentials: 'same-origin',
    headers: { Accept: 'application/json', ...init.headers },
  });
  if (!response.ok) {
    let body: Partial<ApiErrorBody> = {};
    try {
      body = (await response.json()) as Partial<ApiErrorBody>;
    } catch {
      // no JSON body
    }
    throw new ApiError(response.status, body);
  }
}

export async function signup(email: string, password: string): Promise<User> {
  const res = await request<{ user: User }>('/auth/signup', jsonInit('POST', { email, password }));
  return res.user;
}

export async function login(email: string, password: string): Promise<User> {
  const res = await request<{ user: User }>('/auth/login', jsonInit('POST', { email, password }));
  return res.user;
}

export function logout(): Promise<void> {
  return requestVoid('/auth/logout', jsonInit('POST'));
}

export async function getMe(): Promise<User> {
  const res = await request<{ user: User }>('/auth/me');
  return res.user;
}

export function listKeys(): Promise<ApiKey[]> {
  return request<ApiKey[]>('/keys');
}

export function createKey(name: string): Promise<CreatedApiKey> {
  return request<CreatedApiKey>('/keys', jsonInit('POST', { name }));
}

export function revokeKey(id: number): Promise<void> {
  return requestVoid(`/keys/${id}`, jsonInit('DELETE'));
}
```

Add `ApiKey`, `CreatedApiKey`, `User` to the type import at the top. Run `grep -rn API_BASE_URL frontend/src` and confirm nothing else depends on it being absolute (the WebSocket client uses `NEXT_PUBLIC_WS_URL`).

- [ ] **Step 3: Hooks**

Append to `frontend/src/hooks/useApi.ts` (add `ApiKey`, `User` to the type import and `ApiError` from `@/lib/api`):

```ts
// ---------------------------------------------------------------------------
// Accounts and API keys
// ---------------------------------------------------------------------------

/** The signed-in user, or `null` when there is no session. Never retries a 401. */
export function useMe(config?: SWRConfiguration): SWRResponse<User | null> {
  return useSWR<User | null>(
    'me',
    async () => {
      try {
        return await api.getMe();
      } catch (err) {
        if (err instanceof api.ApiError && err.status === 401) return null;
        throw err;
      }
    },
    { ...defaultConfig, shouldRetryOnError: false, ...config }
  );
}

export function useApiKeys(enabled: boolean, config?: SWRConfiguration): SWRResponse<ApiKey[]> {
  return useSWR<ApiKey[]>(enabled ? 'keys' : null, api.listKeys, {
    ...defaultConfig,
    shouldRetryOnError: false,
    ...config,
  });
}
```

- [ ] **Step 4: Form styles**

In `frontend/src/app/globals.css`, inside `@layer components` after the `.btn-icon:hover` rule:

```css
  /* -- forms ------------------------------------------------------------ */

  .input {
    @apply w-full rounded border border-border bg-surface px-3 py-2 text-sm text-text placeholder-mute transition-colors;
  }

  .input:focus {
    border-color: var(--color-accent);
    outline: none;
  }

  .field-label {
    @apply mb-1.5 block text-xs font-medium uppercase tracking-wide text-mute;
  }

  .form-error {
    @apply rounded border px-3 py-2 text-sm;
    border-color: var(--color-accent-3);
    color: var(--color-accent-3);
  }
```

- [ ] **Step 5: Verify**

Run: `cd frontend && npm run lint && npm run build`
Expected: no errors (nothing uses the new code yet).

- [ ] **Step 6: Commit**

```bash
git add frontend/src/types/index.ts frontend/src/lib/api.ts frontend/src/hooks/useApi.ts frontend/src/app/globals.css
git commit -m "frontend: auth and API key client, hooks and form styles"
```

---

### Task 9: Login and signup pages, header link

**Files:**
- Create: `frontend/src/components/AuthForm.tsx`, `frontend/src/app/login/page.tsx`, `frontend/src/app/signup/page.tsx`
- Modify: `frontend/src/components/Header.tsx`

**Interfaces:**
- Consumes: `api.login`, `api.signup`, `useMe`, `ApiError`.
- Produces: `<AuthForm mode="login" | "signup" />`; routes `/login`, `/signup`.

- [ ] **Step 1: AuthForm**

`frontend/src/components/AuthForm.tsx`:

```tsx
'use client';

import Link from 'next/link';
import { useRouter } from 'next/navigation';
import { useState } from 'react';
import { useSWRConfig } from 'swr';
import * as api from '@/lib/api';

interface AuthFormProps {
  mode: 'login' | 'signup';
}

const copy = {
  login: {
    title: 'Sign in',
    button: 'Sign in',
    alt: 'No account yet?',
    altLink: '/signup',
    altLabel: 'Create one',
  },
  signup: {
    title: 'Create an account',
    button: 'Create account',
    alt: 'Already registered?',
    altLink: '/login',
    altLabel: 'Sign in',
  },
} as const;

/** Email + password form used by /login and /signup. Redirects to /dashboard on success. */
export function AuthForm({ mode }: AuthFormProps) {
  const router = useRouter();
  const { mutate } = useSWRConfig();
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const text = copy[mode];

  const onSubmit = async (event: React.FormEvent) => {
    event.preventDefault();
    setError(null);
    setBusy(true);
    try {
      const user = mode === 'login' ? await api.login(email, password) : await api.signup(email, password);
      await mutate('me', user, { revalidate: false });
      router.push('/dashboard');
    } catch (err) {
      setError(err instanceof api.ApiError ? err.message : 'Request failed');
      setBusy(false);
    }
  };

  return (
    <div className="mx-auto max-w-md">
      <h1 className="font-serif text-3xl font-medium tracking-tight text-strong">{text.title}</h1>
      <p className="mt-2 text-sm text-soft">
        An account lets you create API keys with a higher request quota.
      </p>
      <form onSubmit={onSubmit} className="card-padded mt-6 space-y-4">
        <div>
          <label htmlFor="email" className="field-label">
            Email
          </label>
          <input
            id="email"
            type="email"
            autoComplete="email"
            required
            value={email}
            onChange={(e) => setEmail(e.target.value)}
            className="input"
          />
        </div>
        <div>
          <label htmlFor="password" className="field-label">
            Password
          </label>
          <input
            id="password"
            type="password"
            autoComplete={mode === 'login' ? 'current-password' : 'new-password'}
            required
            minLength={10}
            maxLength={128}
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            className="input"
          />
          {mode === 'signup' && <p className="mt-1.5 text-xs text-mute">At least 10 characters.</p>}
        </div>
        {error && <div className="form-error">{error}</div>}
        <button type="submit" disabled={busy} className="btn-primary w-full justify-center disabled:opacity-60">
          {busy ? 'Please wait…' : text.button}
        </button>
        <p className="text-center text-sm text-soft">
          {text.alt}{' '}
          <Link href={text.altLink} className="link">
            {text.altLabel}
          </Link>
        </p>
      </form>
    </div>
  );
}
```

- [ ] **Step 2: Pages**

`frontend/src/app/login/page.tsx`:

```tsx
import type { Metadata } from 'next';
import { AuthForm } from '@/components/AuthForm';

export const metadata: Metadata = { title: 'Sign in — RandScan' };

export default function LoginPage() {
  return <AuthForm mode="login" />;
}
```

`frontend/src/app/signup/page.tsx`:

```tsx
import type { Metadata } from 'next';
import { AuthForm } from '@/components/AuthForm';

export const metadata: Metadata = { title: 'Create account — RandScan' };

export default function SignupPage() {
  return <AuthForm mode="signup" />;
}
```

- [ ] **Step 3: Header link**

In `frontend/src/components/Header.tsx`: import `useMe` from `@/hooks/useApi`; inside `Header()` add `const { data: me } = useMe();` and a small component rendered in both the desktop right-hand cluster (before the GitHub link) and the mobile menu (after the nav links):

```tsx
function AccountLink({ signedIn, className }: { signedIn: boolean; className?: string }) {
  return (
    <Link href={signedIn ? '/dashboard' : '/login'} className={className}>
      {signedIn ? 'API keys' : 'Sign in'}
    </Link>
  );
}
```

Desktop: `<AccountLink signedIn={!!me} className="text-sm text-soft transition-colors hover:text-strong" />`. Mobile: `<AccountLink signedIn={!!me} className="px-1 py-2 text-sm text-soft transition-colors hover:text-strong" />` with `onClick` closing the menu is not needed (navigation unmounts the menu state on route change only if the component remounts; add `onClick={() => setMobileOpen(false)}` by wrapping in a `<div onClick={...}>`).

- [ ] **Step 4: Verify**

Run: `cd frontend && npm run lint && npm run build`
Expected: `/login` and `/signup` appear as static routes. Then, with the API running locally (`COOKIE_SECURE=false cargo run --bin randscan-api`) and `NEXT_PUBLIC_API_URL=http://localhost:3000 npm run dev`, sign up at http://localhost:3001/signup and confirm the redirect to `/dashboard` (404 until Task 10, but the header must now read "API keys").

- [ ] **Step 5: Commit**

```bash
git add frontend/src
git commit -m "frontend: login and signup pages"
```

---

### Task 10: Dashboard page

**Files:**
- Create: `frontend/src/app/dashboard/page.tsx`

**Interfaces:**
- Consumes: `useMe`, `useApiKeys`, `api.createKey`, `api.revokeKey`, `api.logout`, `copyToClipboard`, `formatDateTime` (takes ms; convert with `Date.parse`).

- [ ] **Step 1: Page**

```tsx
'use client';

import Link from 'next/link';
import { useRouter } from 'next/navigation';
import { useEffect, useState } from 'react';
import { useSWRConfig } from 'swr';
import { DetailSkeleton } from '@/components/Loading';
import { DetailRow, ErrorState, PageHeader, Panel } from '@/components/States';
import { useApiKeys, useMe } from '@/hooks/useApi';
import * as api from '@/lib/api';
import { copyToClipboard, formatDateTime, formatNumber } from '@/lib/utils';
import type { ApiKey, CreatedApiKey } from '@/types';

function when(iso: string | null): string {
  return iso ? formatDateTime(Date.parse(iso)) : '—';
}

export default function DashboardPage() {
  const router = useRouter();
  const { mutate } = useSWRConfig();
  const { data: me, isLoading: meLoading, error: meError } = useMe();
  const { data: keys, error: keysError, mutate: refreshKeys } = useApiKeys(!!me);

  const [name, setName] = useState('');
  const [created, setCreated] = useState<CreatedApiKey | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    if (!meLoading && me === null) router.replace('/login');
  }, [me, meLoading, router]);

  if (meError) return <ErrorState onRetry={() => void mutate('me')} />;
  if (!me) return <DetailSkeleton />;

  const onCreate = async (event: React.FormEvent) => {
    event.preventDefault();
    setError(null);
    setBusy(true);
    try {
      const key = await api.createKey(name);
      setCreated(key);
      setCopied(false);
      setName('');
      await refreshKeys();
    } catch (err) {
      setError(err instanceof api.ApiError ? err.message : 'Request failed');
    } finally {
      setBusy(false);
    }
  };

  const onRevoke = async (key: ApiKey) => {
    setError(null);
    try {
      await api.revokeKey(key.id);
      if (created?.id === key.id) setCreated(null);
      await refreshKeys();
    } catch (err) {
      setError(err instanceof api.ApiError ? err.message : 'Request failed');
    }
  };

  const onLogout = async () => {
    await api.logout();
    await mutate('me', null, { revalidate: false });
    router.push('/');
  };

  const active = (keys ?? []).filter((k) => !k.revoked_at);
  const revoked = (keys ?? []).filter((k) => k.revoked_at);

  return (
    <div className="space-y-8">
      <PageHeader
        label="Account"
        title="API keys"
        subtitle={me.email}
        actions={
          <button type="button" onClick={onLogout} className="btn-secondary">
            Sign out
          </button>
        }
      />

      {created && (
        <Panel title="New key">
          <div className="py-3">
            <p className="text-sm text-soft">
              Copy your key now. For security it is shown only once; if you lose it, revoke it and
              create another.
            </p>
            <div className="mt-3 flex flex-wrap items-center gap-3">
              <code className="break-all rounded border border-border bg-surface-2 px-3 py-2 font-mono text-sm text-strong">
                {created.key}
              </code>
              <button
                type="button"
                className="btn-secondary"
                onClick={async () => setCopied(await copyToClipboard(created.key))}
              >
                {copied ? 'Copied' : 'Copy'}
              </button>
            </div>
            <pre className="mt-4 overflow-x-auto rounded border border-border bg-surface-2 p-3 font-mono text-xs text-soft">
{`curl -H "Authorization: Bearer ${created.key}" https://randscan.org/api/v1/stats`}
            </pre>
          </div>
        </Panel>
      )}

      <Panel title="Create a key">
        <form onSubmit={onCreate} className="flex flex-wrap items-end gap-3 py-3">
          <div className="min-w-[16rem] flex-1">
            <label htmlFor="key-name" className="field-label">
              Name
            </label>
            <input
              id="key-name"
              className="input"
              placeholder="e.g. exchange deposit watcher"
              maxLength={64}
              required
              value={name}
              onChange={(e) => setName(e.target.value)}
            />
          </div>
          <button type="submit" disabled={busy || active.length >= 10} className="btn-primary disabled:opacity-60">
            New key
          </button>
        </form>
        {error && <div className="form-error mb-3">{error}</div>}
        <p className="pb-3 text-xs text-mute">
          Up to 10 active keys. Keyed requests get 600 requests per minute; anonymous traffic gets
          60 per IP. See the{' '}
          <Link href="https://github.com/randprotocol/randscan/blob/main/docs/api.md" className="link">
            API guide
          </Link>
          .
        </p>
      </Panel>

      <Panel title={`Active keys (${active.length})`}>
        {keysError && <ErrorState message="Could not load your keys." onRetry={() => void refreshKeys()} />}
        {!keysError && active.length === 0 && (
          <p className="py-4 text-sm text-mute">No active keys yet.</p>
        )}
        {active.map((k) => (
          <DetailRow key={k.id} label={k.name}>
            <div className="flex flex-wrap items-center justify-between gap-3">
              <div className="text-sm">
                <span className="font-mono text-strong">rsk_{k.prefix}…</span>
                <span className="ml-3 text-mute">
                  created {when(k.created_at)} · last used {when(k.last_used_at)} ·{' '}
                  {formatNumber(k.request_count)} requests
                </span>
              </div>
              <button type="button" onClick={() => onRevoke(k)} className="btn-secondary text-accent-3">
                Revoke
              </button>
            </div>
          </DetailRow>
        ))}
      </Panel>

      {revoked.length > 0 && (
        <Panel title={`Revoked keys (${revoked.length})`}>
          {revoked.map((k) => (
            <DetailRow key={k.id} label={k.name}>
              <span className="font-mono text-mute">rsk_{k.prefix}…</span>
              <span className="ml-3 text-sm text-mute">
                revoked {when(k.revoked_at)} · {formatNumber(k.request_count)} requests
              </span>
            </DetailRow>
          ))}
        </Panel>
      )}

      <Panel title="Account">
        <DetailRow label="Email">{me.email}</DetailRow>
        <DetailRow label="Member since">{when(me.created_at)}</DetailRow>
        <DetailRow label="Last sign-in">{when(me.last_login_at)}</DetailRow>
      </Panel>
    </div>
  );
}
```

- [ ] **Step 2: Verify**

Run: `cd frontend && npm run lint && npm run build`. Then in the browser (API + `npm run dev` as in Task 9): create a key, copy it, run the printed `curl` against `http://localhost:3000/api/v1/stats` and confirm `x-ratelimit-limit: 600`; revoke the key and confirm the curl now returns 401; sign out and confirm `/dashboard` redirects to `/login`.

- [ ] **Step 3: Commit**

```bash
git add frontend/src/app/dashboard
git commit -m "frontend: API keys dashboard"
```

---

### Task 11: Configuration, deployment and documentation

**Files:**
- Modify: `.env.example`, `README.md`, `docs/api.md`, `deploy/vps-setup.sh`, `docker-compose.yml`, `frontend/Dockerfile`, `frontend/next.config.js`

- [ ] **Step 1: `.env.example`**

Append under the `# API server` block:

```
# Sessions and rate limits
COOKIE_SECURE=false            # true in production (https)
TRUST_PROXY=false              # true when behind Caddy/nginx (reads X-Forwarded-For)
ANON_RATE_LIMIT_RPM=60
KEY_RATE_LIMIT_RPM=600
AUTH_RATE_LIMIT_RPM=10
```

- [ ] **Step 2: `deploy/vps-setup.sh`**

In the `cat > $ENV_DIR/api.env <<ENV` block add after `API_PORT=3000`:

```
COOKIE_SECURE=true
TRUST_PROXY=true
```

- [ ] **Step 3: Docker**

`docker-compose.yml`: under `api.environment` add `COOKIE_SECURE: "false"`. Under `frontend.build` add:

```yaml
      args:
        NEXT_PUBLIC_API_URL: http://api:3000
```

`frontend/Dockerfile`: in the builder stage, before the `npm run build` line, add:

```dockerfile
ARG NEXT_PUBLIC_API_URL=
ENV NEXT_PUBLIC_API_URL=$NEXT_PUBLIC_API_URL
```

(Rewrites are evaluated at build time, so the destination must be baked in.)

`frontend/next.config.js`: update the comment above `rewrites()` to say that the browser always calls same-origin `/api`, and the rewrite forwards to `NEXT_PUBLIC_API_URL` when set (development and Docker); in production Caddy handles it.

- [ ] **Step 4: README**

- Environment table: add the five variables with their defaults and one-line purposes.
- "Run locally": change the frontend line to `NEXT_PUBLIC_API_URL=http://localhost:3000 NEXT_PUBLIC_WS_URL=ws://localhost:3000/ws npm run dev` (unchanged) and add a sentence: "Set `COOKIE_SECURE=false` in `.env` for local http so sign-in works."
- API section: add "Accounts and API keys: sign up at `/signup`, create keys at `/dashboard`; keyed requests use `Authorization: Bearer rsk_...`. Details and quotas in [docs/api.md](docs/api.md)."
- New subsection "Operator runbook: reset a password":

```bash
echo 'new password here' | randscan-api hash-password      # prints $argon2id$...
psql "$DATABASE_URL" -c "UPDATE users SET password_hash = '<paste>' WHERE email = 'user@example.com';" \
                     -c "DELETE FROM sessions WHERE user_id = (SELECT id FROM users WHERE email = 'user@example.com');"
```

- [ ] **Step 5: `docs/api.md`**

Replace the paragraph "No API key is needed today..." with:

> Public read endpoints need no key. Anonymous traffic is limited to 60 requests per minute per IP. For more, create a free account at https://randscan.org/signup and an API key at https://randscan.org/dashboard: keyed requests get 600 requests per minute. Use the WebSocket feed instead of tight polling when you need to react to new blocks.

Add a section before "WebSocket feed":

```markdown
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
| `GET /keys` | | `ApiKey[]` (never includes the secret) |
| `POST /keys` | `{ "name" }` | 201 `ApiKey` plus `key`, once |
| `DELETE /keys/:id` | | 204 |

`user`: `{ id, email, created_at, last_login_at }`. `ApiKey`: `{ id, name, prefix, created_at,
last_used_at, request_count, revoked_at }`. Passwords are 10 to 128 characters.
```

Update the "Rate limits and reliability" section's first bullet to point at the quotas table.

- [ ] **Step 6: Full verification**

Run from the repo root:

```bash
cargo fmt --all -- --check && cargo clippy --all-targets -- -D warnings && DATABASE_URL=... cargo test --all
cd frontend && npm run lint && npm run build
```

Expected: everything green.

- [ ] **Step 7: Commit and push**

```bash
git add .env.example README.md docs/api.md deploy/vps-setup.sh docker-compose.yml frontend/Dockerfile frontend/next.config.js
git commit -m "docs, deploy: accounts and API keys configuration"
git push origin main
```

---

### Task 12: Deploy to randscan.org and smoke test

**Files:** none (operations)

- [ ] **Step 1: Deploy**

Run `deploy/push-to-vps.sh <node-E-ip> randscan.org` (the script rsyncs, rebuilds and restarts the services; the migration runner applies `002_accounts.sql` on start). Watch `journalctl -u randscan-api -f` for `applying migration 2`.

- [ ] **Step 2: Smoke test**

```bash
curl -si https://randscan.org/api/v1/health | grep -i x-ratelimit          # limit 60
curl -si -X POST https://randscan.org/api/v1/auth/login -H 'content-type: application/json' -d '{"email":"nobody@example.com","password":"xxxxxxxxxx"}' | head -1   # 401
```

Then in a browser: sign up, create a key, run the printed `curl`, confirm `x-ratelimit-limit: 600`, revoke it, confirm 401.

- [ ] **Step 3: Record**

Note the deployed commit and date in the memory file `randscan-privacy-and-api-keys-roadmap.md` (auth shipped; viewing keys still blocked on chain work).
