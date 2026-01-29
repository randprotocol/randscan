//! API error handling

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use randscan_core::ApiError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Database error: {0}")]
    Database(#[from] randscan_db::DbError),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, ApiError::not_found(&msg)),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, ApiError::bad_request(&msg)),
            AppError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, ApiError::internal(&msg))
            }
            AppError::Database(e) => {
                tracing::error!("Database error: {}", e);
                match e {
                    randscan_db::DbError::NotFound(msg) => {
                        (StatusCode::NOT_FOUND, ApiError::not_found(&msg))
                    }
                    _ => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        ApiError::internal("Database error"),
                    ),
                }
            }
        };

        (status, Json(error)).into_response()
    }
}

pub type ApiResult<T> = Result<T, AppError>;
