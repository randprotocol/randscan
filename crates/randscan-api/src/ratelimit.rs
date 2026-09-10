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
            return Decision::Allowed {
                remaining: u32::MAX,
            };
        }
        let start = now - now % WINDOW_SECS;
        let mut map = self.windows.lock().unwrap_or_else(|p| p.into_inner());
        let entry = map.entry(id.to_string()).or_insert((start, 0));
        if entry.0 != start {
            *entry = (start, 0);
        }
        if entry.1 >= limit {
            return Decision::Limited {
                retry_after: start + WINDOW_SECS - now,
            };
        }
        entry.1 += 1;
        Decision::Allowed {
            remaining: limit - entry.1,
        }
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
    auth::client_ip(
        req.headers(),
        auth::peer_addr(req.extensions()),
        state.config.trust_proxy,
    )
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
                return AppError::unauthorized("invalid_api_key", "malformed API key")
                    .into_response();
            }
            let row = match db::get_active_api_key(state.db.inner(), &auth::hash_secret(&key)).await
            {
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
        Decision::Limited { retry_after } => {
            AppError::TooManyRequests { retry_after }.into_response()
        }
    }
}

/// Login and signup: a tighter per-IP budget on top of `api_limit`.
pub async fn auth_limit(State(state): State<AppState>, req: Request, next: Next) -> Response {
    let id = format!("auth:{}", ip_of(&state, &req));
    match state.limiter.check(&id, state.config.auth_rpm, unix_now()) {
        Decision::Allowed { .. } => next.run(req).await,
        Decision::Limited { retry_after } => {
            AppError::TooManyRequests { retry_after }.into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_within_a_window_and_resets() {
        let l = RateLimiter::new();
        assert!(matches!(
            l.check("a", 2, 100),
            Decision::Allowed { remaining: 1 }
        ));
        assert!(matches!(
            l.check("a", 2, 110),
            Decision::Allowed { remaining: 0 }
        ));
        assert!(matches!(
            l.check("a", 2, 111),
            Decision::Limited { retry_after: 9 }
        ));
        assert!(
            matches!(l.check("b", 2, 111), Decision::Allowed { remaining: 1 }),
            "other id"
        );
        assert!(
            matches!(l.check("a", 2, 125), Decision::Allowed { remaining: 1 }),
            "next window"
        );
    }

    #[test]
    fn zero_disables() {
        let l = RateLimiter::new();
        for _ in 0..1000 {
            assert!(matches!(
                l.check("a", 0, 5),
                Decision::Allowed {
                    remaining: u32::MAX
                }
            ));
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
