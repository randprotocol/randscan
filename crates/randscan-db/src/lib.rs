//! RandScan database layer (PostgreSQL via SQLx).

pub mod error;
pub mod models;
pub mod pool;
pub mod queries;

pub use error::*;
pub use models::*;
pub use pool::*;
pub use queries::*;

use sqlx::postgres::PgPoolOptions;
pub use sqlx::PgPool;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout: Duration,
    pub idle_timeout: Duration,
}

impl DatabaseConfig {
    pub fn from_env() -> Self {
        let num = |k: &str, d: u64| {
            std::env::var(k)
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(d)
        };
        Self {
            url: std::env::var("DATABASE_URL").unwrap_or_else(|_| {
                "postgres://randscan:randscan@localhost:5432/randscan".to_string()
            }),
            max_connections: num("DB_MAX_CONNECTIONS", 10) as u32,
            min_connections: num("DB_MIN_CONNECTIONS", 1) as u32,
            connect_timeout: Duration::from_secs(num("DB_CONNECT_TIMEOUT", 10)),
            idle_timeout: Duration::from_secs(num("DB_IDLE_TIMEOUT", 300)),
        }
    }
}

pub async fn create_pool(config: &DatabaseConfig) -> Result<PgPool> {
    PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(config.connect_timeout)
        .idle_timeout(config.idle_timeout)
        .connect(&config.url)
        .await
        .map_err(|e| DbError::Connection(e.to_string()))
}

/// Embedded migrations in order. Add new files here; never edit an applied one.
const MIGRATIONS: &[(i32, &str)] = &[
    (
        1,
        include_str!("../../../migrations/001_initial_schema.sql"),
    ),
    (2, include_str!("../../../migrations/002_accounts.sql")),
];

/// Apply every migration that `schema_migrations` does not record yet, each in its own transaction.
///
/// Databases created before the runner existed have `blocks` but no `schema_migrations`; they are
/// recorded as version 1 first.
pub async fn run_migrations(pool: &PgPool) -> Result<()> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
    )
    .execute(pool)
    .await?;

    let blocks_exist: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_name = 'blocks')",
    )
    .fetch_one(pool)
    .await?;

    if blocks_exist {
        let ok: bool = sqlx::query_scalar(
            "SELECT EXISTS (SELECT FROM information_schema.columns WHERE table_name = 'blocks' AND column_name = 'justify_view')",
        )
        .fetch_one(pool)
        .await?;
        if !ok {
            return Err(DbError::Migration(
                "database has the legacy schema; drop and recreate the database".into(),
            ));
        }
        sqlx::query("INSERT INTO schema_migrations (version) VALUES (1) ON CONFLICT DO NOTHING")
            .execute(pool)
            .await?;
    }

    let applied: Vec<i32> = sqlx::query_scalar("SELECT version FROM schema_migrations")
        .fetch_all(pool)
        .await?;

    for (version, sql) in MIGRATIONS {
        if applied.contains(version) {
            continue;
        }
        tracing::info!("applying migration {}", version);
        let mut tx = pool.begin().await?;
        sqlx::raw_sql(sql)
            .execute(&mut *tx)
            .await
            .map_err(|e| DbError::Migration(format!("migration {}: {}", version, e)))?;
        sqlx::query("INSERT INTO schema_migrations (version) VALUES ($1)")
            .bind(version)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
    }
    Ok(())
}
