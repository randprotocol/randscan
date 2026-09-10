//! Integration tests against a real PostgreSQL (`DATABASE_URL`). They skip when it is unset.

use randscan_db::run_migrations;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

pub async fn test_pool() -> Option<PgPool> {
    let url = std::env::var("DATABASE_URL").ok()?;
    let pool = PgPoolOptions::new()
        .max_connections(4)
        .connect(&url)
        .await
        .expect("connect to DATABASE_URL");
    run_migrations(&pool).await.expect("migrations");
    Some(pool)
}

#[tokio::test]
async fn migrations_are_versioned_and_idempotent() {
    let Some(pool) = test_pool().await else {
        eprintln!("skipping: DATABASE_URL unset");
        return;
    };
    // Running again must be a no-op.
    run_migrations(&pool).await.expect("second run");

    let versions: Vec<i32> =
        sqlx::query_scalar("SELECT version FROM schema_migrations ORDER BY version")
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(versions, vec![1, 2]);

    for table in ["users", "sessions", "api_keys"] {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_name = $1)",
        )
        .bind(table)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert!(exists, "table {table} missing");
    }
}

use chrono::{Duration, Utc};
use randscan_db as db;

fn unique_email() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("t{nanos}@example.com")
}

#[tokio::test]
async fn users_are_unique_by_email() {
    let Some(pool) = test_pool().await else {
        return;
    };
    let email = unique_email();
    let u = db::create_user(&pool, &email, "hash").await.unwrap();
    assert_eq!(u.email, email);
    assert!(u.last_login_at.is_none());
    let dup = db::create_user(&pool, &email, "hash2").await;
    assert!(matches!(dup, Err(db::DbError::Constraint(_))));
    let found = db::get_user_by_email(&pool, &email).await.unwrap().unwrap();
    assert_eq!(found.id, u.id);
    assert_eq!(found.password_hash, "hash");
    db::touch_last_login(&pool, u.id).await.unwrap();
    assert!(db::get_user(&pool, u.id)
        .await
        .unwrap()
        .unwrap()
        .last_login_at
        .is_some());
}

#[tokio::test]
async fn sessions_expire_and_delete() {
    let Some(pool) = test_pool().await else {
        return;
    };
    let u = db::create_user(&pool, &unique_email(), "hash")
        .await
        .unwrap();
    let live = format!("{:0>64}", format!("{:x}", u.id * 7 + 1));
    let dead = format!("{:0>64}", format!("{:x}", u.id * 7 + 2));
    db::create_session(
        &pool,
        &live,
        u.id,
        Utc::now() + Duration::days(1),
        Some("ua"),
        Some("1.2.3.4"),
    )
    .await
    .unwrap();
    db::create_session(
        &pool,
        &dead,
        u.id,
        Utc::now() - Duration::days(1),
        None,
        None,
    )
    .await
    .unwrap();

    let (s, user) = db::get_session_user(&pool, &live).await.unwrap().unwrap();
    assert_eq!(s.user_id, u.id);
    assert_eq!(user.id, u.id);
    assert!(db::get_session_user(&pool, &dead).await.unwrap().is_none());

    db::delete_expired_sessions(&pool, u.id).await.unwrap();
    let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sessions WHERE user_id = $1")
        .bind(u.id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(n, 1);

    let later = Utc::now() + Duration::days(30);
    db::touch_session(&pool, &live, later).await.unwrap();
    let (s, _) = db::get_session_user(&pool, &live).await.unwrap().unwrap();
    assert!((s.expires_at - later).num_seconds().abs() < 2);

    db::delete_session(&pool, &live).await.unwrap();
    assert!(db::get_session_user(&pool, &live).await.unwrap().is_none());
}

#[tokio::test]
async fn api_keys_lifecycle() {
    let Some(pool) = test_pool().await else {
        return;
    };
    let u = db::create_user(&pool, &unique_email(), "hash")
        .await
        .unwrap();
    let hash = format!("{:0>64}", format!("{:x}", u.id * 11 + 3));
    let k = db::create_api_key(&pool, u.id, "bot", "abcdefghijkl", &hash)
        .await
        .unwrap();
    assert_eq!(k.name, "bot");
    assert_eq!(k.request_count, 0);
    assert_eq!(db::count_active_api_keys(&pool, u.id).await.unwrap(), 1);

    let active = db::get_active_api_key(&pool, &hash).await.unwrap().unwrap();
    assert_eq!(active.id, k.id);
    db::record_api_key_use(&pool, k.id).await.unwrap();
    let listed = db::list_api_keys(&pool, u.id).await.unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].request_count, 1);
    assert!(listed[0].last_used_at.is_some());

    // Another user cannot revoke it.
    assert!(!db::revoke_api_key(&pool, u.id + 1_000_000, k.id)
        .await
        .unwrap());
    assert!(db::revoke_api_key(&pool, u.id, k.id).await.unwrap());
    assert!(
        !db::revoke_api_key(&pool, u.id, k.id).await.unwrap(),
        "already revoked"
    );
    assert!(db::get_active_api_key(&pool, &hash)
        .await
        .unwrap()
        .is_none());
    assert_eq!(db::count_active_api_keys(&pool, u.id).await.unwrap(), 0);
    assert!(db::list_api_keys(&pool, u.id).await.unwrap()[0]
        .revoked_at
        .is_some());
}
