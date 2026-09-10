mod common;

use common::*;

#[tokio::test]
async fn anonymous_requests_are_limited_per_ip() {
    let cfg = randscan_api::ApiConfig {
        anon_rpm: 3,
        ..test_config()
    };
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
    assert_eq!(headers["x-ratelimit-limit"], "3");
    assert_eq!(headers["x-ratelimit-remaining"], "0");
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
    let Some((app, _)) = test_app(test_config()).await else {
        return;
    };
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
}
