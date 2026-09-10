//! RandScan REST API (axum).

pub mod auth;
pub mod error;
pub mod handlers;
pub mod ratelimit;
pub mod routes;
pub mod state;

pub use error::*;
pub use routes::*;
pub use state::*;

use std::net::SocketAddr;

#[derive(Debug, Clone)]
pub struct ApiConfig {
    pub listen_addr: SocketAddr,
    /// `Secure` attribute on the session cookie (disable for local http).
    pub cookie_secure: bool,
    /// Read the client IP from `X-Forwarded-For` (set when behind Caddy).
    pub trust_proxy: bool,
    /// Requests per minute per anonymous IP (0 disables).
    pub anon_rpm: u32,
    /// Requests per minute per API key (0 disables).
    pub key_rpm: u32,
    /// Login/signup attempts per minute per IP (0 disables).
    pub auth_rpm: u32,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            listen_addr: ([0, 0, 0, 0], 3000).into(),
            cookie_secure: true,
            trust_proxy: false,
            anon_rpm: 60,
            key_rpm: 600,
            auth_rpm: 10,
        }
    }
}

impl ApiConfig {
    pub fn from_env() -> Self {
        let d = Self::default();
        let host = std::env::var("API_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port: u16 = std::env::var("API_PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(3000);
        let flag = |k: &str, d: bool| {
            std::env::var(k)
                .ok()
                .map(|v| {
                    !matches!(
                        v.trim().to_ascii_lowercase().as_str(),
                        "0" | "false" | "no" | "off"
                    )
                })
                .unwrap_or(d)
        };
        let num = |k: &str, d: u32| {
            std::env::var(k)
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(d)
        };
        Self {
            listen_addr: format!("{}:{}", host, port)
                .parse()
                .unwrap_or(d.listen_addr),
            cookie_secure: flag("COOKIE_SECURE", d.cookie_secure),
            trust_proxy: flag("TRUST_PROXY", d.trust_proxy),
            anon_rpm: num("ANON_RATE_LIMIT_RPM", d.anon_rpm),
            key_rpm: num("KEY_RATE_LIMIT_RPM", d.key_rpm),
            auth_rpm: num("AUTH_RATE_LIMIT_RPM", d.auth_rpm),
        }
    }
}
