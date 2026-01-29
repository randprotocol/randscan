//! Database pool management

use sqlx::PgPool;
use std::sync::Arc;

/// Database pool wrapper for dependency injection
#[derive(Clone)]
pub struct DbPool {
    inner: Arc<PgPool>,
}

impl DbPool {
    pub fn new(pool: PgPool) -> Self {
        Self {
            inner: Arc::new(pool),
        }
    }

    pub fn inner(&self) -> &PgPool {
        &self.inner
    }
}

impl std::ops::Deref for DbPool {
    type Target = PgPool;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<PgPool> for DbPool {
    fn from(pool: PgPool) -> Self {
        Self::new(pool)
    }
}
