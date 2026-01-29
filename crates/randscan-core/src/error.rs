//! Error types for RandScan

use thiserror::Error;

#[derive(Error, Debug)]
pub enum RandScanError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("RPC error: {0}")]
    Rpc(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Invalid hex: {0}")]
    InvalidHex(String),

    #[error("Invalid block: {0}")]
    InvalidBlock(String),

    #[error("Invalid transaction: {0}")]
    InvalidTransaction(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("WebSocket error: {0}")]
    WebSocket(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<hex::FromHexError> for RandScanError {
    fn from(err: hex::FromHexError) -> Self {
        RandScanError::InvalidHex(err.to_string())
    }
}

impl From<serde_json::Error> for RandScanError {
    fn from(err: serde_json::Error) -> Self {
        RandScanError::Serialization(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, RandScanError>;
