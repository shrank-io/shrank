use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

use crate::db;
use crate::AppError;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct SyncParams {
    pub since: Option<String>,
    pub limit: Option<i64>,
}

pub async fn delta_sync(
    State(state): State<AppState>,
    Query(params): Query<SyncParams>,
) -> Result<Json<db::sync::SyncResponse>, AppError> {
    let limit = params.limit.unwrap_or(50).min(100);
    let response =
        db::sync::get_documents_since(&state.db, params.since.as_deref(), limit).await?;
    Ok(Json(response))
}

#[derive(Debug, Deserialize)]
pub struct RegisterBody {
    pub client_id: String,
}

pub async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterBody>,
) -> Result<StatusCode, AppError> {
    if body.client_id.is_empty() {
        return Err(AppError::Validation("client_id is required".into()));
    }
    db::sync::register_client(&state.db, &body.client_id).await?;
    Ok(StatusCode::CREATED)
}
