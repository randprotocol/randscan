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
pub fn client_ip(
    headers: &HeaderMap,
    peer: Option<SocketAddr>,
    trust_proxy: bool,
) -> Option<String> {
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
                if let Err(e) =
                    db::touch_session(&pool, &token_hash, Utc::now() + SESSION_TTL).await
                {
                    tracing::warn!("touch session: {}", e);
                }
            });
        }
        Ok(AuthUser(user))
    }
}

/// Peer address recorded by `into_make_service_with_connect_info`, if any.
pub fn peer_addr(parts_or_req_extensions: &axum::http::Extensions) -> Option<SocketAddr> {
    parts_or_req_extensions
        .get::<ConnectInfo<SocketAddr>>()
        .map(|c| c.0)
}

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
        assert_eq!(
            client_ip(&h, Some(peer), false).as_deref(),
            Some("10.0.0.5")
        );
        assert_eq!(
            client_ip(&h, Some(peer), true).as_deref(),
            Some("203.0.113.9")
        );
        assert_eq!(
            client_ip(&HeaderMap::new(), Some(peer), true).as_deref(),
            Some("10.0.0.5")
        );
        assert_eq!(client_ip(&HeaderMap::new(), None, true), None);
    }
}
