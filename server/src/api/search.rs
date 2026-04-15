use axum::extract::{Query, State};
use axum::Json;
use serde::Deserialize;

use crate::db;
use crate::AppError;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct SearchParams {
    pub q: String,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn search(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> Result<Json<db::search::SearchResults>, AppError> {
    let query = db::search::SearchQuery {
        raw: params.q,
        limit: params.limit.unwrap_or(20).min(100),
        offset: params.offset.unwrap_or(0),
    };

    // TODO: generate query embedding for semantic search
    // let embedding = state.inference.embed(&query.raw).await.ok();

    let results = db::search::search(&state.db, &query, None).await?;
    Ok(Json(results))
}
