//! Email normalisation. Deliverability is not checked.

pub const MAX_EMAIL: usize = 254;

/// Trim, lowercase and validate. `None` when the address is not acceptable.
pub fn normalize_email(raw: &str) -> Option<String> {
    let email = raw.trim().to_ascii_lowercase();
    if email.is_empty() || email.len() > MAX_EMAIL || email.chars().any(char::is_whitespace) {
        return None;
    }
    let (local, domain) = email.split_once('@')?;
    if local.is_empty() || domain.is_empty() || domain.contains('@') {
        return None;
    }
    if !domain.contains('.') || domain.starts_with('.') || domain.ends_with('.') {
        return None;
    }
    Some(email)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_and_validates() {
        assert_eq!(
            normalize_email("  Bob@Example.COM "),
            Some("bob@example.com".into())
        );
        assert_eq!(normalize_email("bob@example"), None);
        assert_eq!(normalize_email("@example.com"), None);
        assert_eq!(normalize_email("bob@"), None);
        assert_eq!(normalize_email("bob@@example.com"), None);
        assert_eq!(normalize_email("bob@exam ple.com"), None);
        let long = format!("{}@example.com", "a".repeat(250));
        assert_eq!(normalize_email(&long), None);
    }
}
