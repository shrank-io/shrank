use axum::extract::State;
use axum::Json;

use crate::db;
use crate::AppError;
use crate::AppState;

pub async fn health(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Check database
    let db_ok = state.db.read().await.is_ok();

    // Check vllm-mlx
    let inference_status = match state.inference.health().await {
        Ok(status) => status,
        Err(_) => serde_json::json!({ "status": "unavailable" }),
    };

    Ok(Json(serde_json::json!({
        "status": if db_ok { "ok" } else { "degraded" },
        "database": db_ok,
        "inference": inference_status,
        "version": env!("CARGO_PKG_VERSION"),
    })))
}

pub async fn stats(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let status_counts = db::documents::count_by_status(&state.db).await?;
    let tag_counts = db::documents::get_tag_counts(&state.db).await?;
    let senders = db::documents::get_all_senders(&state.db).await?;

    let total: i64 = status_counts.iter().map(|(_, c)| c).sum();
    let status_map: serde_json::Value = status_counts
        .into_iter()
        .map(|(s, c)| (s, serde_json::json!(c)))
        .collect::<serde_json::Map<String, serde_json::Value>>()
        .into();

    let tags: Vec<serde_json::Value> = tag_counts
        .iter()
        .map(|(name, count)| serde_json::json!({ "name": name, "count": count }))
        .collect();

    // Compute storage usage
    let data_dir = state.config.data_dir();
    let storage_bytes = dir_size(&data_dir).await;

    Ok(Json(serde_json::json!({
        "total_documents": total,
        "by_status": status_map,
        "unique_tags": tag_counts.len(),
        "unique_senders": senders.len(),
        "tags": tags,
        "storage_bytes": storage_bytes,
    })))
}

async fn dir_size(path: &std::path::Path) -> u64 {
    let mut total = 0u64;
    if let Ok(mut entries) = tokio::fs::read_dir(path).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            if let Ok(meta) = entry.metadata().await {
                if meta.is_file() {
                    total += meta.len();
                } else if meta.is_dir() {
                    total += Box::pin(dir_size(&entry.path())).await;
                }
            }
        }
    }
    total
}
