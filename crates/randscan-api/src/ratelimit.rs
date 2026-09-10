//! Fixed-window rate limiting per anonymous IP or API key.

#[derive(Default)]
pub struct RateLimiter;

impl RateLimiter {
    pub fn new() -> Self {
        Self
    }
}
