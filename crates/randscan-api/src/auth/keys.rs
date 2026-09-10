//! API key format: `rsk_` + 48 base62 characters. Stored and looked up as SHA-256 hex.

use axum::http::HeaderMap;
use rand::{distributions::Alphanumeric, rngs::OsRng, Rng};
use sha2::{Digest, Sha256};

pub const KEY_PREFIX: &str = "rsk_";
pub const KEY_RANDOM_LEN: usize = 48;
pub const KEY_DISPLAY_PREFIX_LEN: usize = 12;

pub fn generate_api_key() -> String {
    let mut rng = OsRng;
    let body: String = (0..KEY_RANDOM_LEN)
        .map(|_| rng.sample(Alphanumeric) as char)
        .collect();
    format!("{KEY_PREFIX}{body}")
}

/// The first 12 characters after `rsk_`, for display in the dashboard.
pub fn key_prefix(key: &str) -> String {
    key.strip_prefix(KEY_PREFIX)
        .unwrap_or(key)
        .chars()
        .take(KEY_DISPLAY_PREFIX_LEN)
        .collect()
}

/// SHA-256 hex of a secret (session token or API key).
pub fn hash_secret(secret: &str) -> String {
    hex::encode(Sha256::digest(secret.as_bytes()))
}

pub fn is_key_shaped(s: &str) -> bool {
    s.len() == KEY_PREFIX.len() + KEY_RANDOM_LEN
        && s.starts_with(KEY_PREFIX)
        && s[KEY_PREFIX.len()..]
            .chars()
            .all(|c| c.is_ascii_alphanumeric())
}

/// `Authorization: Bearer rsk_...` wins over `X-API-Key`. Any other Authorization scheme is ignored.
pub fn extract_api_key(headers: &HeaderMap) -> Option<String> {
    if let Some(v) = headers.get("authorization").and_then(|v| v.to_str().ok()) {
        if let Some(rest) = v
            .strip_prefix("Bearer ")
            .or_else(|| v.strip_prefix("bearer "))
        {
            let t = rest.trim();
            if !t.is_empty() {
                return Some(t.to_string());
            }
        }
    }
    headers
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_have_the_documented_shape() {
        let k = generate_api_key();
        assert_eq!(k.len(), 4 + 48);
        assert!(k.starts_with(KEY_PREFIX));
        assert!(k[4..].chars().all(|c| c.is_ascii_alphanumeric()));
        assert_ne!(k, generate_api_key());
        assert_eq!(key_prefix(&k), &k[4..16]);
        assert!(is_key_shaped(&k));
        assert!(!is_key_shaped("rsk_short"));
        assert!(!is_key_shaped("abc"));
    }

    #[test]
    fn hashing_is_sha256_hex() {
        assert_eq!(
            hash_secret("abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn extracts_from_either_header() {
        let mut h = HeaderMap::new();
        assert_eq!(extract_api_key(&h), None);
        h.insert("x-api-key", "rsk_x".parse().unwrap());
        assert_eq!(extract_api_key(&h).as_deref(), Some("rsk_x"));
        h.insert("authorization", "Bearer rsk_y".parse().unwrap());
        assert_eq!(extract_api_key(&h).as_deref(), Some("rsk_y"), "bearer wins");
        let mut h = HeaderMap::new();
        h.insert("authorization", "Basic abc".parse().unwrap());
        assert_eq!(extract_api_key(&h), None);
    }
}
