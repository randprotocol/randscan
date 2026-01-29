//! RandScan Database Layer
//!
//! PostgreSQL database access using SQLx with async support.

pub mod pool;
pub mod models;
pub mod queries;
pub mod error;

pub use pool::*;
pub use models::*;
pub use queries::*;
pub use error::*;

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::time::Duration;

/// Database configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout: Duration,
    pub idle_timeout: Duration,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "postgres://localhost/randscan".to_string(),
            max_connections: 10,
            min_connections: 1,
            connect_timeout: Duration::from_secs(10),
            idle_timeout: Duration::from_secs(300),
        }
    }
}

impl DatabaseConfig {
    pub fn from_env() -> Self {
        Self {
            url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://localhost/randscan".to_string()),
            max_connections: std::env::var("DB_MAX_CONNECTIONS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(10),
            min_connections: std::env::var("DB_MIN_CONNECTIONS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(1),
            connect_timeout: Duration::from_secs(
                std::env::var("DB_CONNECT_TIMEOUT")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(10),
            ),
            idle_timeout: Duration::from_secs(
                std::env::var("DB_IDLE_TIMEOUT")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(300),
            ),
        }
    }
}

/// Create a database connection pool
pub async fn create_pool(config: &DatabaseConfig) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(config.connect_timeout)
        .idle_timeout(config.idle_timeout)
        .connect(&config.url)
        .await
        .map_err(|e| DbError::Connection(e.to_string()))?;

    Ok(pool)
}

/// Run database migrations
/// Note: In production, run migrations manually with sqlx-cli
pub async fn run_migrations(_pool: &PgPool) -> Result<()> {
    // Migrations are run via sqlx-cli in production
    // sqlx migrate run
    Ok(())
}
