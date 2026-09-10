//! Outbound email (password-reset links) through Resend's HTTP API.
//!
//! The API server holds an `Option<Arc<dyn MailSender>>`: `None` when `RESEND_API_KEY` is unset,
//! in which case password reset is reported as disabled. Tests use `MemoryMailer`, which records
//! what would have been sent so the reset link can be followed end to end without a network.

use axum::async_trait;
use std::sync::Mutex;

/// A plain-text message. HTML is deliberately not used: reset mails must survive any client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Email {
    pub to: String,
    pub subject: String,
    pub text: String,
}

#[async_trait]
pub trait MailSender: Send + Sync {
    /// Deliver `email`. The error string must never contain credentials.
    async fn send(&self, email: Email) -> Result<(), String>;
}

/// Sends through `https://api.resend.com/emails`.
pub struct ResendMailer {
    client: reqwest::Client,
    api_key: String,
    from: String,
}

impl ResendMailer {
    pub fn new(api_key: String, from: String) -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(15))
                .build()
                .expect("reqwest client"),
            api_key,
            from,
        }
    }
}

#[async_trait]
impl MailSender for ResendMailer {
    async fn send(&self, email: Email) -> Result<(), String> {
        let body = serde_json::json!({
            "from": self.from,
            "to": [email.to],
            "subject": email.subject,
            "text": email.text,
        });
        let res = self
            .client
            .post("https://api.resend.com/emails")
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("resend request: {}", e))?;
        let status = res.status();
        if status.is_success() {
            return Ok(());
        }
        let text = res.text().await.unwrap_or_default();
        Err(format!(
            "resend responded {}: {}",
            status,
            text.chars().take(300).collect::<String>()
        ))
    }
}

/// Records messages instead of sending them (tests and local development).
#[derive(Default)]
pub struct MemoryMailer {
    sent: Mutex<Vec<Email>>,
}

impl MemoryMailer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn sent(&self) -> Vec<Email> {
        self.sent.lock().unwrap_or_else(|p| p.into_inner()).clone()
    }
}

#[async_trait]
impl MailSender for MemoryMailer {
    async fn send(&self, email: Email) -> Result<(), String> {
        self.sent
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .push(email);
        Ok(())
    }
}

/// The password-reset message. `public_url` is the site origin without a trailing slash.
pub fn reset_email(to: &str, public_url: &str, token: &str) -> Email {
    let link = format!("{}/reset?token={}", public_url.trim_end_matches('/'), token);
    Email {
        to: to.to_string(),
        subject: "Reset your RandScan password".to_string(),
        text: format!(
            "Someone asked to reset the password for the RandScan account {to}.\n\n\
             To choose a new password, open this link within one hour:\n\n{link}\n\n\
             If you did not ask for this, ignore this email; your password will not change.\n\n\
             RandScan — explorer for Rand Protocol\n"
        ),
    }
}

/// Extract the token from a reset email's text (used by tests and the dashboard smoke check).
pub fn token_from_reset_email(text: &str) -> Option<String> {
    text.split("token=")
        .nth(1)
        .map(|rest| rest.chars().take_while(|c| c.is_ascii_hexdigit()).collect())
        .filter(|t: &String| t.len() == 64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn memory_mailer_records_and_email_carries_link() {
        let m = MemoryMailer::new();
        let token = "ab".repeat(32);
        m.send(reset_email(
            "bob@example.com",
            "https://randscan.org/",
            &token,
        ))
        .await
        .unwrap();
        let sent = m.sent();
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0].to, "bob@example.com");
        assert!(sent[0].subject.contains("RandScan"));
        assert!(sent[0]
            .text
            .contains(&format!("https://randscan.org/reset?token={token}")));
        assert_eq!(
            token_from_reset_email(&sent[0].text).as_deref(),
            Some(token.as_str())
        );
        assert_eq!(token_from_reset_email("no link here"), None);
    }
}
