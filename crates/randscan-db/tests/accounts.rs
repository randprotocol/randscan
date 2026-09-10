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
