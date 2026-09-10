//! Users, cookie sessions and API keys. Hashes in, never secrets.

use crate::{ApiKeyRow, Result, SessionRow, UserRow};
use chrono::{DateTime, Utc};
use sqlx::PgPool;

const USER_COLS: &str = "id, email, password_hash, created_at, last_login_at";
const KEY_COLS: &str =
    "id, user_id, name, prefix, created_at, last_used_at, request_count, revoked_at";

/// Joined `sessions` + `users` columns as returned by `get_session_user`'s query.
type SessionUserRow = (
    String,
    i64,
    DateTime<Utc>,
    DateTime<Utc>,
    String,
    String,
    DateTime<Utc>,
    Option<DateTime<Utc>>,
);

pub async fn create_user(pool: &PgPool, email: &str, password_hash: &str) -> Result<UserRow> {
    let sql =
        format!("INSERT INTO users (email, password_hash) VALUES ($1, $2) RETURNING {USER_COLS}");
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
    .bind(ip.map(|s| s.chars().take(64).collect::<String>()))
    .execute(pool)
    .await?;
    Ok(())
}

/// The session and its user, only while the session is unexpired.
pub async fn get_session_user(
    pool: &PgPool,
    token_hash: &str,
) -> Result<Option<(SessionRow, UserRow)>> {
    let row: Option<SessionUserRow> = sqlx::query_as(
        "SELECT s.token_hash, s.user_id, s.expires_at, s.last_seen_at,
                u.email, u.password_hash, u.created_at, u.last_login_at
         FROM sessions s JOIN users u ON u.id = s.user_id
         WHERE s.token_hash = $1 AND s.expires_at > NOW()",
    )
    .bind(token_hash)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(
        |(
            token_hash,
            user_id,
            expires_at,
            last_seen_at,
            email,
            password_hash,
            created_at,
            last_login_at,
        )| {
            (
                SessionRow {
                    token_hash,
                    user_id,
                    expires_at,
                    last_seen_at,
                },
                UserRow {
                    id: user_id,
                    email,
                    password_hash,
                    created_at,
                    last_login_at,
                },
            )
        },
    ))
}

pub async fn touch_session(
    pool: &PgPool,
    token_hash: &str,
    expires_at: DateTime<Utc>,
) -> Result<()> {
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

/// Delete every expired session across all users. Used by the background reaper task.
/// Returns the number of rows deleted.
pub async fn delete_all_expired_sessions(pool: &PgPool) -> Result<u64> {
    let res = sqlx::query("DELETE FROM sessions WHERE expires_at <= NOW()")
        .execute(pool)
        .await?;
    Ok(res.rows_affected())
}

pub async fn list_api_keys(pool: &PgPool, user_id: i64) -> Result<Vec<ApiKeyRow>> {
    let sql = format!(
        "SELECT {KEY_COLS} FROM api_keys WHERE user_id = $1 ORDER BY created_at DESC, id DESC"
    );
    Ok(sqlx::query_as::<_, ApiKeyRow>(&sql)
        .bind(user_id)
        .fetch_all(pool)
        .await?)
}

pub async fn count_active_api_keys(pool: &PgPool, user_id: i64) -> Result<i64> {
    Ok(sqlx::query_scalar(
        "SELECT COUNT(*) FROM api_keys WHERE user_id = $1 AND revoked_at IS NULL",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?)
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
    sqlx::query(
        "UPDATE api_keys SET last_used_at = NOW(), request_count = request_count + 1 WHERE id = $1",
    )
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}
