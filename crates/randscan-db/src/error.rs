//! Database error types

use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Query error: {0}")]
    Query(String),

    #[error("Migration error: {0}")]
    Migration(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Constraint violation: {0}")]
    Constraint(String),

    #[error("Serialization error: {0}")]
    Serialization(String),
}

impl From<sqlx::Error> for DbError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => DbError::NotFound("Row not found".to_string()),
            sqlx::Error::Database(e) => {
                if e.is_unique_violation() {
                    DbError::Constraint(e.to_string())
                } else {
                    DbError::Query(e.to_string())
                }
            }
            _ => DbError::Query(err.to_string()),
        }
    }
}

pub type Result<T> = std::result::Result<T, DbError>;
