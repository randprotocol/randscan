//! RandScan REST API
//!
//! Axum-based REST API for the RandProtocol blockchain explorer.

pub mod handlers;
pub mod routes;
pub mod state;
pub mod error;

pub use handlers::*;
pub use routes::*;
pub use state::*;
pub use error::*;

use std::net::SocketAddr;
use std::time::Duration;

/// API configuration
#[derive(Debug, Clone)]
pub struct ApiConfig {
    /// Listen address
    pub listen_addr: SocketAddr,
    /// Enable CORS
    pub enable_cors: bool,
    /// CORS allowed origins
    pub cors_origins: Vec<String>,
    /// Rate limit requests per minute
    pub rate_limit_rpm: u32,
    /// Request timeout
    pub request_timeout: Duration,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            listen_addr: ([0, 0, 0, 0], 3000).into(),
            enable_cors: true,
            cors_origins: vec!["*".to_string()],
            rate_limit_rpm: 100,
            request_timeout: Duration::from_secs(30),
        }
    }
}

impl ApiConfig {
    pub fn from_env() -> Self {
        let host = std::env::var("API_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port: u16 = std::env::var("API_PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(3000);

        Self {
            listen_addr: format!("{}:{}", host, port)
                .parse()
                .unwrap_or_else(|_| ([0, 0, 0, 0], 3000).into()),
            enable_cors: std::env::var("ENABLE_CORS")
                .map(|s| s == "true" || s == "1")
                .unwrap_or(true),
            cors_origins: std::env::var("CORS_ORIGINS")
                .map(|s| s.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_else(|_| vec!["*".to_string()]),
            rate_limit_rpm: std::env::var("RATE_LIMIT_RPM")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(100),
            request_timeout: Duration::from_secs(
                std::env::var("REQUEST_TIMEOUT")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(30),
            ),
        }
    }
}
