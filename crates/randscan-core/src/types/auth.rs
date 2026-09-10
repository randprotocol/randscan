use serde::{Deserialize, Serialize};

/// A registered explorer user (never includes the password hash).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub email: String,
    pub created_at: String,
    pub last_login_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse {
    pub user: User,
}

/// An API key as listed in the dashboard. The secret is never included.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: i64,
    pub name: String,
    pub prefix: String,
    pub created_at: String,
    pub last_used_at: Option<String>,
    pub request_count: i64,
    pub revoked_at: Option<String>,
}

/// Returned once, at creation: the key fields plus the full secret.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatedApiKey {
    #[serde(flatten)]
    pub info: ApiKey,
    pub key: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateKeyRequest {
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn created_key_flattens_info_and_adds_key() {
        let created = CreatedApiKey {
            info: ApiKey {
                id: 7,
                name: "bot".into(),
                prefix: "abcdefghijkl".into(),
                created_at: "2026-09-10T00:00:00+00:00".into(),
                last_used_at: None,
                request_count: 0,
                revoked_at: None,
            },
            key: "rsk_secret".into(),
        };
        let v = serde_json::to_value(&created).unwrap();
        assert_eq!(v["id"], 7);
        assert_eq!(v["prefix"], "abcdefghijkl");
        assert_eq!(v["key"], "rsk_secret");
        assert!(v["last_used_at"].is_null());
    }
}
