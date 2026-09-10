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
pub async fn list_keys(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> ApiResult<Json<Vec<ApiKey>>> {
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
        return Err(AppError::bad(
            "invalid_name",
            "name must be 1 to 64 characters",
        ));
    }
    let pool = state.db.inner();
    // The 10-key cap is a courtesy limit, not a security boundary: this count-then-insert
    // is not atomic, so two concurrent creates for the same user can both pass this check
    // and briefly leave the user with 11 active keys. That is accepted — nothing relies on
    // the cap being exact, and the user can simply revoke one to get back under it.
    if db::count_active_api_keys(pool, user.id).await? >= MAX_ACTIVE_KEYS {
        return Err(AppError::conflict(
            "key_limit",
            "at most 10 active keys; revoke one first",
        ));
    }
    let key = generate_api_key();
    let row =
        db::create_api_key(pool, user.id, name, &key_prefix(&key), &hash_secret(&key)).await?;
    Ok((
        StatusCode::CREATED,
        Json(CreatedApiKey {
            info: to_wire(row),
            key,
        }),
    ))
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
