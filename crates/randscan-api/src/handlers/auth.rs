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
    http::{request::Parts, StatusCode},
    response::IntoResponse,
    Json,
};
use axum_extra::extract::CookieJar;
use chrono::Utc;
use randscan_core::{
    ChangePasswordRequest, ForgotPasswordRequest, LoginRequest, ResetPasswordRequest,
    SignupRequest, UserResponse,
};
use randscan_db::{self as db, DbError, UserRow};

/// Where a request came from, for the session row.
struct Origin {
    user_agent: Option<String>,
    ip: Option<String>,
}

impl Origin {
    fn from_parts(state: &AppState, parts: &Parts) -> Self {
        Self {
            user_agent: parts
                .headers
                .get("user-agent")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string()),
            ip: client_ip(
                &parts.headers,
                peer_addr(&parts.extensions),
                state.config.trust_proxy,
            ),
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
            return Err(AppError::conflict(
                "email_taken",
                "an account with this email already exists",
            ))
        }
        Err(e) => return Err(e.into()),
    };
    let origin = Origin::from_parts(&state, &parts);
    let jar = start_session(&state, jar, &user, origin).await?;
    Ok((
        StatusCode::CREATED,
        jar,
        Json(UserResponse {
            user: user_to_wire(&user),
        }),
    ))
}

/// POST /api/v1/auth/login
pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    parts: Parts,
    Json(req): Json<LoginRequest>,
) -> ApiResult<impl IntoResponse> {
    let invalid =
        || AppError::unauthorized("invalid_credentials", "email or password is incorrect");
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
    let user = db::get_user(state.db.inner(), user.id)
        .await?
        .unwrap_or(user);
    let origin = Origin::from_parts(&state, &parts);
    let jar = start_session(&state, jar, &user, origin).await?;
    Ok((
        StatusCode::OK,
        jar,
        Json(UserResponse {
            user: user_to_wire(&user),
        }),
    ))
}

/// POST /api/v1/auth/logout
///
/// Idempotent: it never requires a valid session (no `AuthUser` extractor). With no
/// `randscan_session` cookie, or one that names a session that is already gone, this still
/// clears the cookie and returns 204 — logging out twice, or logging out after the session
/// already expired, is not an error.
pub async fn logout(State(state): State<AppState>, jar: CookieJar) -> ApiResult<impl IntoResponse> {
    if let Some(c) = jar.get(crate::auth::SESSION_COOKIE) {
        // `delete_session` is a no-op (not an error) when no session matches the hash.
        db::delete_session(state.db.inner(), &hash_secret(c.value())).await?;
    }
    let jar = jar.add(clear_session_cookie(state.config.cookie_secure));
    Ok((StatusCode::NO_CONTENT, jar))
}

/// GET /api/v1/auth/me
pub async fn me(AuthUser(user): AuthUser) -> ApiResult<Json<UserResponse>> {
    Ok(Json(UserResponse {
        user: user_to_wire(&user),
    }))
}

// ---------------------------------------------------------------------------
// Password reset and change
// ---------------------------------------------------------------------------

/// Reset links stop working after this long.
pub const RESET_TTL: chrono::Duration = chrono::Duration::hours(1);

/// POST /api/v1/auth/forgot
///
/// Always 202 whether or not the address has an account, so this endpoint cannot be used to
/// probe for registered emails. With no mail provider configured it is 503 `email_disabled`.
/// Delivery failures are logged, not surfaced (that would reveal that the account exists).
pub async fn forgot_password(
    State(state): State<AppState>,
    Json(req): Json<ForgotPasswordRequest>,
) -> ApiResult<StatusCode> {
    let Some(mailer) = state.mailer.clone() else {
        return Err(AppError::Status {
            status: StatusCode::SERVICE_UNAVAILABLE,
            error: "email_disabled",
            message:
                "password reset by email is not enabled on this explorer; contact the operator"
                    .into(),
        });
    };
    let Some(email) = normalize_email(&req.email) else {
        return Ok(StatusCode::ACCEPTED);
    };
    let Some(user) = db::get_user_by_email(state.db.inner(), &email).await? else {
        return Ok(StatusCode::ACCEPTED);
    };
    let token = generate_session_token(); // same 32-byte hex shape as a session token
    db::create_password_reset(
        state.db.inner(),
        user.id,
        &hash_secret(&token),
        Utc::now() + RESET_TTL,
    )
    .await?;
    let mail = crate::mail::reset_email(&user.email, &state.config.public_url, &token);
    tokio::spawn(async move {
        if let Err(e) = mailer.send(mail).await {
            tracing::error!("password reset email failed: {}", e);
        }
    });
    Ok(StatusCode::ACCEPTED)
}

/// POST /api/v1/auth/reset
///
/// Consumes the token (single use), sets the new password, logs the user out everywhere, and
/// signs them in with a fresh session. Unknown, used and expired tokens are all 400
/// `invalid_token`.
pub async fn reset_password(
    State(state): State<AppState>,
    jar: CookieJar,
    parts: Parts,
    Json(req): Json<ResetPasswordRequest>,
) -> ApiResult<impl IntoResponse> {
    let invalid = || {
        AppError::bad(
            "invalid_token",
            "this reset link is invalid or has expired; request a new one",
        )
    };
    let token = req.token.trim();
    if token.len() != 64 || !token.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(invalid());
    }
    validate_password(&req.password).map_err(|m| AppError::bad("weak_password", m))?;
    let user_id = db::consume_password_reset(state.db.inner(), &hash_secret(token))
        .await?
        .ok_or_else(invalid)?;
    let hash = hash_password(req.password).await?;
    db::update_password(state.db.inner(), user_id, &hash).await?;
    db::delete_user_sessions(state.db.inner(), user_id, None).await?;
    db::touch_last_login(state.db.inner(), user_id).await?;
    let user = db::get_user(state.db.inner(), user_id)
        .await?
        .ok_or_else(invalid)?;
    let origin = Origin::from_parts(&state, &parts);
    let jar = start_session(&state, jar, &user, origin).await?;
    Ok((
        StatusCode::OK,
        jar,
        Json(UserResponse {
            user: user_to_wire(&user),
        }),
    ))
}

/// POST /api/v1/auth/password (session only)
///
/// Verifies the current password, stores the new one, and invalidates every other session so a
/// stolen cookie stops working; the caller's own session stays valid.
pub async fn change_password(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    jar: CookieJar,
    Json(req): Json<ChangePasswordRequest>,
) -> ApiResult<StatusCode> {
    validate_password(&req.new_password).map_err(|m| AppError::bad("weak_password", m))?;
    let ok = verify_password(user.password_hash.clone(), req.current_password).await?;
    if !ok {
        return Err(AppError::unauthorized(
            "invalid_credentials",
            "current password is incorrect",
        ));
    }
    let hash = hash_password(req.new_password).await?;
    db::update_password(state.db.inner(), user.id, &hash).await?;
    let keep = jar
        .get(crate::auth::SESSION_COOKIE)
        .map(|c| hash_secret(c.value()));
    db::delete_user_sessions(state.db.inner(), user.id, keep.as_deref()).await?;
    Ok(StatusCode::NO_CONTENT)
}
