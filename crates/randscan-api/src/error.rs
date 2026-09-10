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
    #[error("internal: {0}")]
    Internal(String),
    #[error("database: {0}")]
    Database(#[from] randscan_db::DbError),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            AppError::NotFound(what) => (StatusCode::NOT_FOUND, ApiError::not_found(&what)),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, ApiError::bad_request(&msg)),
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
