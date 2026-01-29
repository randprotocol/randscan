//! Utility functions

use chrono::{TimeZone, Utc};

/// Format a Unix timestamp (milliseconds) to a human-readable string
pub fn format_timestamp(timestamp: u64) -> String {
    let secs = (timestamp / 1000) as i64;
    let dt = Utc.timestamp_opt(secs, 0).single();

    match dt {
        Some(dt) => {
            let now = Utc::now();
            let diff = now.signed_duration_since(dt);

            if diff.num_seconds() < 60 {
                format!("{} seconds ago", diff.num_seconds())
            } else if diff.num_minutes() < 60 {
                format!("{} minutes ago", diff.num_minutes())
            } else if diff.num_hours() < 24 {
                format!("{} hours ago", diff.num_hours())
            } else if diff.num_days() < 7 {
                format!("{} days ago", diff.num_days())
            } else {
                dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
            }
        }
        None => "Unknown".to_string(),
    }
}

/// Format ATLAS/SHRUG amount from lamports
pub fn format_token_amount(lamports: i64) -> String {
    let amount = lamports as f64 / 1_000_000_000.0;
    if amount >= 1_000_000.0 {
        format!("{:.2}M", amount / 1_000_000.0)
    } else if amount >= 1_000.0 {
        format!("{:.2}K", amount / 1_000.0)
    } else {
        format!("{:.4}", amount)
    }
}
