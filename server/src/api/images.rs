use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{header, StatusCode};
use axum::response::Response;

use crate::db;
use crate::AppError;
use crate::AppState;

pub async fn original(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Response, AppError> {
    let doc = db::documents::get(&state.db, &id).await?;
    let data_dir = state.config.data_dir();
    let path = data_dir.join(&doc.original_path);

    if !path.exists() {
        return Err(AppError::NotFound("original image not found".into()));
    }

    let bytes = tokio::fs::read(&path).await?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "image/jpeg")
        .header(
            header::CACHE_CONTROL,
            "public, max-age=31536000, immutable",
        )
        .body(Body::from(bytes))
        .unwrap())
}

pub async fn thumbnail(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Response, AppError> {
    let doc = db::documents::get(&state.db, &id).await?;
    let data_dir = state.config.data_dir();
    let path = data_dir.join(&doc.thumbnail_path);

    if !path.exists() {
        return Err(AppError::NotFound("thumbnail not found".into()));
    }

    let bytes = tokio::fs::read(&path).await?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "image/webp")
        .header(
            header::CACHE_CONTROL,
            "public, max-age=31536000, immutable",
        )
        .body(Body::from(bytes))
        .unwrap())
}
