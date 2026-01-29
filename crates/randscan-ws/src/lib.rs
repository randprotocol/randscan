//! RandScan WebSocket Server
//!
//! Real-time updates via WebSocket subscriptions.

mod handler;
mod manager;

pub use handler::*;
pub use manager::*;

use std::time::Duration;

/// WebSocket configuration
#[derive(Debug, Clone)]
pub struct WsConfig {
    /// Ping interval
    pub ping_interval: Duration,
    /// Pong timeout
    pub pong_timeout: Duration,
    /// Max message size
    pub max_message_size: usize,
    /// Max connections per IP
    pub max_connections_per_ip: usize,
}

impl Default for WsConfig {
    fn default() -> Self {
        Self {
            ping_interval: Duration::from_secs(30),
            pong_timeout: Duration::from_secs(10),
            max_message_size: 64 * 1024, // 64KB
            max_connections_per_ip: 10,
        }
    }
}
