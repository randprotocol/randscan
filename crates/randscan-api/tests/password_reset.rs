mod common;

use common::*;
use randscan_api::mail::{token_from_reset_email, MemoryMailer};
use serde_json::json;
use std::sync::Arc;

async fn signed_up(app: &axum::Router, email: &str, password: &str) -> String {
    let (status, headers, _) = call(
        app,
        json_req(
            "POST",
            "/api/v1/auth/signup",
            Some(json!({ "email": email, "password": password })),
            None,
        ),
    )
    .await;
    assert_eq!(status, 201);
    session_cookie_from(&headers)
}

async fn login_status(app: &axum::Router, email: &str, password: &str) -> u16 {
    call(
        app,
        json_req(
            "POST",
            "/api/v1/auth/login",
            Some(json!({ "email": email, "password": password })),
            None,
        ),
    )
    .await
    .0
    .as_u16()
}

#[tokio::test]
async fn forgot_and_reset_end_to_end() {
    let mailer = Arc::new(MemoryMailer::new());
    let cfg = randscan_api::ApiConfig {
        public_url: "https://randscan.test".into(),
        ..test_config()
    };
    let Some((app, _)) = test_app_with_mailer(cfg, Some(mailer.clone())).await else {
        eprintln!("skipping: DATABASE_URL unset");
        return;
    };
    let email = unique_email();
    let old_cookie = signed_up(&app, &email, "old password 1").await;

    // Unknown address: same 202, nothing sent.
    let (status, _, _) = call(
        &app,
        json_req(
            "POST",
            "/api/v1/auth/forgot",
            Some(json!({ "email": unique_email() })),
            None,
        ),
    )
    .await;
    assert_eq!(status, 202);
    assert!(mailer.sent().is_empty());

    // Known address: 202 and one email carrying the link.
    let (status, _, _) = call(
        &app,
        json_req(
            "POST",
            "/api/v1/auth/forgot",
            Some(json!({ "email": email.to_uppercase() })),
            None,
        ),
    )
    .await;
    assert_eq!(status, 202);
    // The send is spawned; give it a moment.
    let mut sent = mailer.sent();
    for _ in 0..40 {
        if !sent.is_empty() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(25)).await;
        sent = mailer.sent();
    }
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].to, email);
    assert!(sent[0].text.contains("https://randscan.test/reset?token="));
    let token = token_from_reset_email(&sent[0].text).expect("token in mail");

    // Weak password is rejected before the token is consumed.
    let (status, _, body) = call(
        &app,
        json_req(
            "POST",
            "/api/v1/auth/reset",
            Some(json!({ "token": token, "password": "short" })),
            None,
        ),
    )
    .await;
    assert_eq!(status, 400);
    assert_eq!(body["error"], "weak_password");

    // Reset: signed in with a new cookie, old session dead, new password works, old does not.
    let (status, headers, body) = call(
        &app,
        json_req(
            "POST",
            "/api/v1/auth/reset",
            Some(json!({ "token": token, "password": "new password 2" })),
            None,
        ),
    )
    .await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(body["user"]["email"], email);
    let new_cookie = session_cookie_from(&headers);
    let (status, _, _) = call(
        &app,
        json_req("GET", "/api/v1/auth/me", None, Some(&new_cookie)),
    )
    .await;
    assert_eq!(status, 200);
    let (status, _, _) = call(
        &app,
        json_req("GET", "/api/v1/auth/me", None, Some(&old_cookie)),
    )
    .await;
    assert_eq!(status, 401, "reset logs out every earlier session");
    assert_eq!(login_status(&app, &email, "new password 2").await, 200);
    assert_eq!(login_status(&app, &email, "old password 1").await, 401);

    // The token is single use.
    let (status, _, body) = call(
        &app,
        json_req(
            "POST",
            "/api/v1/auth/reset",
            Some(json!({ "token": token, "password": "another pass 3" })),
            None,
        ),
    )
    .await;
    assert_eq!(status, 400);
    assert_eq!(body["error"], "invalid_token");
}

#[tokio::test]
async fn reset_rejects_bad_and_expired_tokens() {
    let mailer = Arc::new(MemoryMailer::new());
    let Some((app, pool)) = test_app_with_mailer(test_config(), Some(mailer.clone())).await else {
        return;
    };
    for bad in ["", "abc", &"z".repeat(64), &"a".repeat(64)] {
        let (status, _, body) = call(
            &app,
            json_req(
                "POST",
                "/api/v1/auth/reset",
                Some(json!({ "token": bad, "password": "a fine password" })),
                None,
            ),
        )
        .await;
        assert_eq!(status, 400, "token {bad:?}");
        assert_eq!(body["error"], "invalid_token");
    }

    // An expired token: issue one, then age it in the database.
    let email = unique_email();
    signed_up(&app, &email, "old password 1").await;
    let (status, _, _) = call(
        &app,
        json_req(
            "POST",
            "/api/v1/auth/forgot",
            Some(json!({ "email": email })),
            None,
        ),
    )
    .await;
    assert_eq!(status, 202);
    let mut sent = mailer.sent();
    for _ in 0..40 {
        if !sent.is_empty() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(25)).await;
        sent = mailer.sent();
    }
    let token = token_from_reset_email(&sent[0].text).unwrap();
    sqlx::query("UPDATE password_resets SET expires_at = NOW() - INTERVAL '1 minute' WHERE user_id = (SELECT id FROM users WHERE email = $1)")
        .bind(&email)
        .execute(&pool)
        .await
        .unwrap();
    let (status, _, body) = call(
        &app,
        json_req(
            "POST",
            "/api/v1/auth/reset",
            Some(json!({ "token": token, "password": "a fine password" })),
            None,
        ),
    )
    .await;
    assert_eq!(status, 400);
    assert_eq!(body["error"], "invalid_token");
    assert_eq!(
        login_status(&app, &email, "old password 1").await,
        200,
        "unchanged"
    );
}

#[tokio::test]
async fn forgot_is_503_without_a_mailer() {
    let Some((app, _)) = test_app(test_config()).await else {
        return;
    };
    let (status, _, body) = call(
        &app,
        json_req(
            "POST",
            "/api/v1/auth/forgot",
            Some(json!({ "email": unique_email() })),
            None,
        ),
    )
    .await;
    assert_eq!(status, 503);
    assert_eq!(body["error"], "email_disabled");
}

#[tokio::test]
async fn change_password_keeps_current_session_only() {
    let Some((app, _)) = test_app(test_config()).await else {
        return;
    };
    let email = unique_email();
    let cookie_a = signed_up(&app, &email, "old password 1").await;
    // A second session for the same user.
    let (status, headers, _) = call(
        &app,
        json_req(
            "POST",
            "/api/v1/auth/login",
            Some(json!({ "email": email, "password": "old password 1" })),
            None,
        ),
    )
    .await;
    assert_eq!(status, 200);
    let cookie_b = session_cookie_from(&headers);

    // Anonymous and API-key callers cannot change passwords.
    let (status, _, _) = call(
        &app,
        json_req(
            "POST",
            "/api/v1/auth/password",
            Some(json!({ "current_password": "old password 1", "new_password": "new password 2" })),
            None,
        ),
    )
    .await;
    assert_eq!(status, 401);

    // Wrong current password.
    let (status, _, body) = call(
        &app,
        json_req(
            "POST",
            "/api/v1/auth/password",
            Some(
                json!({ "current_password": "wrong password!", "new_password": "new password 2" }),
            ),
            Some(&cookie_a),
        ),
    )
    .await;
    assert_eq!(status, 401);
    assert_eq!(body["error"], "invalid_credentials");
    // Weak new password.
    let (status, _, body) = call(
        &app,
        json_req(
            "POST",
            "/api/v1/auth/password",
            Some(json!({ "current_password": "old password 1", "new_password": "short" })),
            Some(&cookie_a),
        ),
    )
    .await;
    assert_eq!(status, 400);
    assert_eq!(body["error"], "weak_password");

    // Success: 204, session A survives, session B is gone, new password works.
    let (status, _, _) = call(
        &app,
        json_req(
            "POST",
            "/api/v1/auth/password",
            Some(json!({ "current_password": "old password 1", "new_password": "new password 2" })),
            Some(&cookie_a),
        ),
    )
    .await;
    assert_eq!(status, 204);
    let (status, _, _) = call(
        &app,
        json_req("GET", "/api/v1/auth/me", None, Some(&cookie_a)),
    )
    .await;
    assert_eq!(status, 200);
    let (status, _, _) = call(
        &app,
        json_req("GET", "/api/v1/auth/me", None, Some(&cookie_b)),
    )
    .await;
    assert_eq!(status, 401);
    assert_eq!(login_status(&app, &email, "new password 2").await, 200);
    assert_eq!(login_status(&app, &email, "old password 1").await, 401);
}
