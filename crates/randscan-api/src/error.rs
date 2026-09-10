use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use randscan_core::ApiError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("{error}: {message}")]
    Status {
        status: StatusCode,
        error: &'static str,
        message: String,
    },
    #[error("rate limited")]
    TooManyRequests { retry_after: u64 },
    #[error("internal: {0}")]
    Internal(String),
    #[error("database: {0}")]
    Database(#[from] randscan_db::DbError),
}

impl AppError {
    pub fn bad(error: &'static str, message: impl Into<String>) -> Self {
        Self::Status {
            status: StatusCode::BAD_REQUEST,
            error,
            message: message.into(),
        }
    }

    pub fn unauthorized(error: &'static str, message: impl Into<String>) -> Self {
        Self::Status {
            status: StatusCode::UNAUTHORIZED,
            error,
            message: message.into(),
        }
    }

    pub fn conflict(error: &'static str, message: impl Into<String>) -> Self {
        Self::Status {
            status: StatusCode::CONFLICT,
            error,
            message: message.into(),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            AppError::NotFound(what) => (StatusCode::NOT_FOUND, ApiError::not_found(&what)),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, ApiError::bad_request(&msg)),
            AppError::Status {
                status,
                error,
                message,
            } => (status, ApiError::new(error, &message)),
            AppError::TooManyRequests { retry_after } => {
                let body = ApiError::new(
                    "rate_limited",
                    &format!("too many requests; retry in {} seconds", retry_after),
                );
                return (
                    StatusCode::TOO_MANY_REQUESTS,
                    [("retry-after", retry_after.to_string())],
                    Json(body),
                )
                    .into_response();
            }
            AppError::Internal(msg) => {
                tracing::error!("internal error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ApiError::internal("internal error"),
                )
            }
            AppError::Database(e) => {
                tracing::error!("database error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ApiError::internal("database error"),
                )
            }
        };
        (status, Json(body)).into_response()
    }
}

pub type ApiResult<T> = Result<T, AppError>;
