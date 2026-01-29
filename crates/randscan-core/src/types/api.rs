//! API types for RandScan REST API

use serde::{Deserialize, Serialize};

/// Pagination parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_limit")]
    pub limit: u32,
}

fn default_page() -> u32 {
    1
}

fn default_limit() -> u32 {
    20
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            page: 1,
            limit: 20,
        }
    }
}

impl Pagination {
    pub fn offset(&self) -> i64 {
        ((self.page.saturating_sub(1)) * self.limit) as i64
    }

    pub fn limit(&self) -> i64 {
        self.limit.min(100) as i64
    }
}

/// Paginated response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub pagination: PaginationInfo,
}

/// Pagination info in response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationInfo {
    pub page: u32,
    pub limit: u32,
    pub total: i64,
    pub total_pages: u32,
    pub has_next: bool,
    pub has_prev: bool,
}

impl PaginationInfo {
    pub fn new(page: u32, limit: u32, total: i64) -> Self {
        let total_pages = ((total as f64) / (limit as f64)).ceil() as u32;
        Self {
            page,
            limit,
            total,
            total_pages,
            has_next: page < total_pages,
            has_prev: page > 1,
        }
    }
}

/// API error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub error: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

impl ApiError {
    pub fn new(error: &str, message: &str) -> Self {
        Self {
            error: error.to_string(),
            message: message.to_string(),
            code: None,
        }
    }

    pub fn not_found(resource: &str) -> Self {
        Self {
            error: "not_found".to_string(),
            message: format!("{} not found", resource),
            code: Some("NOT_FOUND".to_string()),
        }
    }

    pub fn bad_request(message: &str) -> Self {
        Self {
            error: "bad_request".to_string(),
            message: message.to_string(),
            code: Some("BAD_REQUEST".to_string()),
        }
    }

    pub fn internal(message: &str) -> Self {
        Self {
            error: "internal_error".to_string(),
            message: message.to_string(),
            code: Some("INTERNAL_ERROR".to_string()),
        }
    }
}

/// Search query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub q: String,
    #[serde(default)]
    pub limit: Option<u32>,
}

/// Search result item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    #[serde(rename = "type")]
    pub result_type: SearchResultType,
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,
    pub url: String,
}

/// Search result type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchResultType {
    Block,
    Transaction,
    Account,
    Validator,
    Token,
}

/// Block query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockQuery {
    #[serde(flatten)]
    pub pagination: Pagination,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proposer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub epoch: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finalized: Option<bool>,
}

/// Transaction query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionQuery {
    #[serde(flatten)]
    pub pagination: Pagination,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sender: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_timestamp: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_timestamp: Option<i64>,
}

/// Validator query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorQuery {
    #[serde(flatten)]
    pub pagination: Pagination,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_by: Option<String>, // stake, blocks, uptime
}

/// Stats query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_timestamp: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_timestamp: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interval: Option<String>, // hourly, daily
}

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub database: bool,
    pub indexer: IndexerHealth,
}

/// Indexer health info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexerHealth {
    pub connected: bool,
    pub synced: bool,
    pub current_height: i64,
    pub node_height: i64,
    pub lag: i64,
}
