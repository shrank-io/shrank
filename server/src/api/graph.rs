use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;

use crate::db;
use crate::AppError;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct RelatedParams {
    pub depth: Option<u32>,
}

pub async fn related(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(params): Query<RelatedParams>,
) -> Result<Json<Vec<db::graph::RelatedDocument>>, AppError> {
    // Verify document exists
    let _ = db::documents::get(&state.db, &id).await?;

    let depth = params.depth.unwrap_or(2);
    let results = db::graph::get_related(&state.db, &id, depth).await?;
    Ok(Json(results))
}

#[derive(Debug, Deserialize)]
pub struct ListParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn list_entities(
    State(state): State<AppState>,
    Query(params): Query<ListParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    let limit = params.limit.unwrap_or(50).min(100);
    let offset = params.offset.unwrap_or(0);

    let (entities, total) = db::entities::list_all(&state.db, limit, offset).await?;
    Ok(Json(serde_json::json!({
        "entities": entities,
        "total": total,
    })))
}

pub async fn entity_documents(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<db::documents::Document>>, AppError> {
    let docs = db::entities::get_documents_for_entity(&state.db, &id).await?;
    Ok(Json(docs))
}
