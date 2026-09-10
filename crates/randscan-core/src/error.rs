//! Error types for RandScan

use thiserror::Error;

#[derive(Error, Debug)]
pub enum RandScanError {
    #[error("Database error: {0}")]
    Database(String),
    #[error("RPC error: {0}")]
    Rpc(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<serde_json::Error> for RandScanError {
    fn from(err: serde_json::Error) -> Self {
        RandScanError::Serialization(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, RandScanError>;
