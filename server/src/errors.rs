use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("pool error: {0}")]
    Pool(String),

    #[error("image processing error: {0}")]
    Image(#[from] image::ImageError),

    #[error("inference sidecar error: {0}")]
    Inference(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("validation error: {0}")]
    Validation(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("{0}")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::NotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::Validation(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            other => {
                tracing::error!(error = %other, "internal error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal server error".into(),
                )
            }
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}

impl From<deadpool_sqlite::InteractError> for AppError {
    fn from(e: deadpool_sqlite::InteractError) -> Self {
        AppError::Pool(e.to_string())
    }
}

impl From<deadpool_sqlite::PoolError> for AppError {
    fn from(e: deadpool_sqlite::PoolError) -> Self {
        AppError::Pool(e.to_string())
    }
}
