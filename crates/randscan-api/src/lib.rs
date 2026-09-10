//! RandScan REST API (axum).

pub mod error;
pub mod handlers;
pub mod routes;
pub mod state;

pub use error::*;
pub use routes::*;
pub use state::*;

use std::net::SocketAddr;

#[derive(Debug, Clone)]
pub struct ApiConfig {
    pub listen_addr: SocketAddr,
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
        }
    }
}
