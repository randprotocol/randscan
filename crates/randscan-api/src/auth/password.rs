//! Password policy and argon2id hashing (CPU-bound, so it runs on the blocking pool).

use crate::error::AppError;
use argon2::password_hash::{
    rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
};
use argon2::Argon2;

pub const MIN_PASSWORD: usize = 10;
pub const MAX_PASSWORD: usize = 128;

pub fn validate_password(password: &str) -> Result<(), &'static str> {
    let n = password.chars().count();
    if n < MIN_PASSWORD {
        return Err("password must be at least 10 characters");
    }
    if n > MAX_PASSWORD {
        return Err("password must be at most 128 characters");
    }
    Ok(())
}

pub async fn hash_password(password: String) -> Result<String, AppError> {
    tokio::task::spawn_blocking(move || {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map(|h| h.to_string())
            .map_err(|e| AppError::Internal(format!("hash: {}", e)))
    })
    .await
    .map_err(|e| AppError::Internal(format!("join: {}", e)))?
}

/// True when `password` matches `hash`. A malformed hash counts as no match.
pub async fn verify_password(hash: String, password: String) -> Result<bool, AppError> {
    tokio::task::spawn_blocking(move || {
        let Ok(parsed) = PasswordHash::new(&hash) else {
            return false;
        };
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok()
    })
    .await
    .map_err(|e| AppError::Internal(format!("join: {}", e)))
}

/// A real hash of a fixed value; verifying against it keeps login timing the same for
/// unknown emails and wrong passwords. Computed once per process.
pub fn dummy_hash() -> &'static str {
    static HASH: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    HASH.get_or_init(|| {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(b"randscan-dummy-password", &salt)
            .map(|h| h.to_string())
            .expect("argon2 hash")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn password_policy_bounds() {
        assert!(validate_password("123456789").is_err());
        assert!(validate_password("1234567890").is_ok());
        assert!(validate_password(&"x".repeat(128)).is_ok());
        assert!(validate_password(&"x".repeat(129)).is_err());
    }

    #[tokio::test]
    async fn hash_and_verify_round_trip() {
        let hash = hash_password("correct horse battery".into()).await.unwrap();
        assert!(hash.starts_with("$argon2id$"));
        assert!(
            verify_password(hash.clone(), "correct horse battery".into())
                .await
                .unwrap()
        );
        assert!(!verify_password(hash, "wrong".into()).await.unwrap());
    }

    #[tokio::test]
    async fn dummy_hash_is_real_and_never_matches() {
        assert!(PasswordHash::new(dummy_hash()).is_ok());
        assert!(
            !verify_password(dummy_hash().to_string(), "anything".into())
                .await
                .unwrap()
        );
    }
}
