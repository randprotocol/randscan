mod common;

use axum::{body::Body, http::Request};
use common::*;
use serde_json::json;

async fn signed_in(app: &axum::Router) -> String {
    let body = json!({ "email": unique_email(), "password": "correct horse battery" });
    let (status, headers, _) = call(
        app,
        json_req("POST", "/api/v1/auth/signup", Some(body), None),
    )
    .await;
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

    let (status, _, created) = call(
        &app,
        json_req(
            "POST",
            "/api/v1/keys",
            Some(json!({ "name": " exchange bot " })),
            Some(&cookie),
        ),
    )
    .await;
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
    let (status, _, _) = call(
        &app,
        json_req("DELETE", &format!("/api/v1/keys/{id}"), None, Some(&cookie)),
    )
    .await;
    assert_eq!(status, 204);
    let (status, _, _) = call(
        &app,
        json_req("DELETE", &format!("/api/v1/keys/{id}"), None, Some(&cookie)),
    )
    .await;
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
    let (status, _, _) = call(
        &app,
        json_req("DELETE", &format!("/api/v1/keys/{id}"), None, Some(&other)),
    )
    .await;
    assert_eq!(status, 404);
}

#[tokio::test]
async fn key_validation_and_limit() {
    let Some((app, _)) = test_app(test_config()).await else {
        return;
    };
    let cookie = signed_in(&app).await;
    let (status, _, body) = call(
        &app,
        json_req(
            "POST",
            "/api/v1/keys",
            Some(json!({ "name": "   " })),
            Some(&cookie),
        ),
    )
    .await;
    assert_eq!(status, 400);
    assert_eq!(body["error"], "invalid_name");
    let (status, _, _) = call(
        &app,
        json_req(
            "POST",
            "/api/v1/keys",
            Some(json!({ "name": "x".repeat(65) })),
            Some(&cookie),
        ),
    )
    .await;
    assert_eq!(status, 400);
    for i in 0..10 {
        let (status, _, _) = call(
            &app,
            json_req(
                "POST",
                "/api/v1/keys",
                Some(json!({ "name": format!("k{i}") })),
                Some(&cookie),
            ),
        )
        .await;
        assert_eq!(status, 201);
    }
    let (status, _, body) = call(
        &app,
        json_req(
            "POST",
            "/api/v1/keys",
            Some(json!({ "name": "one too many" })),
            Some(&cookie),
        ),
    )
    .await;
    assert_eq!(status, 409);
    assert_eq!(body["error"], "key_limit");
    let (status, _, _) = call(
        &app,
        json_req(
            "POST",
            "/api/v1/keys",
            Some(json!({ "name": "anon" })),
            None,
        ),
    )
    .await;
    assert_eq!(status, 401);
}
