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

    let (status, headers, json) = call(
        &app,
        json_req("POST", "/api/v1/auth/signup", Some(body.clone()), None),
    )
    .await;
    assert_eq!(status, 201, "{json}");
    assert_eq!(json["user"]["email"], email, "normalised");
    assert!(json["user"]["last_login_at"].is_null());
    let set_cookie = headers["set-cookie"].to_str().unwrap();
    assert!(set_cookie.contains("HttpOnly"), "{set_cookie}");
    assert!(set_cookie.contains("SameSite=Lax"));
    assert!(set_cookie.contains("Path=/"));
    assert!(set_cookie.contains("Max-Age=2592000"));
    assert!(
        !set_cookie.contains("Secure"),
        "cookie_secure=false in tests"
    );
    let cookie = session_cookie_from(&headers);

    let (status, _, json) = call(
        &app,
        json_req("GET", "/api/v1/auth/me", None, Some(&cookie)),
    )
    .await;
    assert_eq!(status, 200);
    assert_eq!(json["user"]["email"], email);

    // Duplicate signup.
    let (status, _, json) = call(
        &app,
        json_req("POST", "/api/v1/auth/signup", Some(body), None),
    )
    .await;
    assert_eq!(status, 409);
    assert_eq!(json["error"], "email_taken");

    // Login with the wrong password and with an unknown email give the same body.
    let (s1, _, j1) = call(
        &app,
        json_req(
            "POST",
            "/api/v1/auth/login",
            Some(json!({ "email": email, "password": "wrong password!" })),
            None,
        ),
    )
    .await;
    let (s2, _, j2) = call(
        &app,
        json_req(
            "POST",
            "/api/v1/auth/login",
            Some(json!({ "email": unique_email(), "password": "wrong password!" })),
            None,
        ),
    )
    .await;
    assert_eq!(s1, 401);
    assert_eq!(s2, 401);
    assert_eq!(j1, j2);
    assert_eq!(j1["error"], "invalid_credentials");

    // Real login sets last_login_at.
    let (status, headers, json) = call(
        &app,
        json_req(
            "POST",
            "/api/v1/auth/login",
            Some(json!({ "email": email, "password": "correct horse battery" })),
            None,
        ),
    )
    .await;
    assert_eq!(status, 200);
    assert!(json["user"]["last_login_at"].is_string());
    let cookie2 = session_cookie_from(&headers);

    // Logout invalidates that session only.
    let (status, headers, _) = call(
        &app,
        json_req("POST", "/api/v1/auth/logout", None, Some(&cookie2)),
    )
    .await;
    assert_eq!(status, 204);
    assert!(headers["set-cookie"]
        .to_str()
        .unwrap()
        .contains("Max-Age=0"));
    let (status, _, _) = call(
        &app,
        json_req("GET", "/api/v1/auth/me", None, Some(&cookie2)),
    )
    .await;
    assert_eq!(status, 401);
    let (status, _, _) = call(
        &app,
        json_req("GET", "/api/v1/auth/me", None, Some(&cookie)),
    )
    .await;
    assert_eq!(status, 200, "first session still valid");

    // Logout is idempotent: repeating it with the same (already-invalidated) cookie still
    // returns 204, and so does logging out with no cookie at all.
    let (status, _, _) = call(
        &app,
        json_req("POST", "/api/v1/auth/logout", None, Some(&cookie2)),
    )
    .await;
    assert_eq!(status, 204, "logging out twice");
    let (status, _, _) = call(&app, json_req("POST", "/api/v1/auth/logout", None, None)).await;
    assert_eq!(status, 204, "logging out with no cookie");
}

#[tokio::test]
async fn signup_validation() {
    let Some((app, _)) = test_app(test_config()).await else {
        return;
    };
    let (status, _, json) = call(
        &app,
        json_req(
            "POST",
            "/api/v1/auth/signup",
            Some(json!({ "email": "nope", "password": "correct horse battery" })),
            None,
        ),
    )
    .await;
    assert_eq!(status, 400);
    assert_eq!(json["error"], "invalid_email");
    let (status, _, json) = call(
        &app,
        json_req(
            "POST",
            "/api/v1/auth/signup",
            Some(json!({ "email": unique_email(), "password": "short" })),
            None,
        ),
    )
    .await;
    assert_eq!(status, 400);
    assert_eq!(json["error"], "weak_password");
    let (status, _, json) = call(&app, json_req("GET", "/api/v1/auth/me", None, None)).await;
    assert_eq!(status, 401);
    assert_eq!(json["error"], "unauthorized");
    let (status, _, _) = call(
        &app,
        json_req(
            "GET",
            "/api/v1/auth/me",
            None,
            Some("randscan_session=deadbeef"),
        ),
    )
    .await;
    assert_eq!(status, 401);
}

#[tokio::test]
async fn login_attempts_are_limited() {
    let cfg = randscan_api::ApiConfig {
        auth_rpm: 2,
        ..test_config()
    };
    let Some((app, _)) = test_app(cfg).await else {
        return;
    };
    let body = json!({ "email": unique_email(), "password": "does not matter" });
    for _ in 0..2 {
        let (status, _, _) = call(
            &app,
            json_req("POST", "/api/v1/auth/login", Some(body.clone()), None),
        )
        .await;
        assert_eq!(status, 401);
    }
    let (status, _, json) = call(
        &app,
        json_req("POST", "/api/v1/auth/login", Some(body), None),
    )
    .await;
    assert_eq!(status, 429);
    assert_eq!(json["error"], "rate_limited");
    // Public reads are not affected by the auth limiter.
    let (status, _, _) = call(&app, json_req("GET", "/api/v1/health", None, None)).await;
    assert_eq!(status, 200);
}
