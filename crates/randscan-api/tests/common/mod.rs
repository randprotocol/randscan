//! Shared harness: a real database (`DATABASE_URL`), a stub indexer, `oneshot` requests.

#![allow(dead_code)]

use axum::{
    body::Body,
    http::{HeaderMap, Request, StatusCode},
    Router,
};
use http_body_util::BodyExt;
use randscan_api::{create_router, mail::MailSender, ratelimit::RateLimiter, ApiConfig, AppState};
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
    test_app_with_mailer(cfg, None).await
}

pub async fn test_app_with_mailer(
    cfg: ApiConfig,
    mailer: Option<Arc<dyn MailSender>>,
) -> Option<(Router, PgPool)> {
    let url = std::env::var("DATABASE_URL").ok()?;
    let pool = PgPoolOptions::new()
        .max_connections(4)
        .connect(&url)
        .await
        .expect("connect to DATABASE_URL");
    run_migrations(&pool).await.expect("migrations");
    let db = DbPool::new(pool.clone());
    let indexer = Arc::new(IndexerService::new(
        IndexerConfig::default(),
        db.clone(),
        Broadcaster::new(),
    ));
    let state = AppState {
        db,
        indexer,
        ws_manager: Arc::new(WsManager::new()),
        config: Arc::new(cfg),
        limiter: Arc::new(RateLimiter::new()),
        mailer,
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

pub fn json_req(
    method: &str,
    uri: &str,
    body: Option<Value>,
    cookie: Option<&str>,
) -> Request<Body> {
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
