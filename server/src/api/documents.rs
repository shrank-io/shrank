use axum::extract::{Multipart, Path, Query, State};
use axum::http::StatusCode;
use axum::Json;

use crate::db;
use crate::images::{processing, storage};
use crate::AppError;
use crate::AppState;

pub async fn upload(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let mut image_bytes: Option<Vec<u8>> = None;
    let mut captured_at: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Validation(format!("multipart error: {e}")))?
    {
        match field.name() {
            Some("image") => {
                image_bytes = Some(
                    field
                        .bytes()
                        .await
                        .map_err(|e| AppError::Validation(format!("failed to read image: {e}")))?
                        .to_vec(),
                );
            }
            Some("captured_at") => {
                captured_at = Some(
                    field
                        .text()
                        .await
                        .map_err(|e| AppError::Validation(format!("failed to read captured_at: {e}")))?,
                );
            }
            _ => {}
        }
    }

    let image_bytes =
        image_bytes.ok_or_else(|| AppError::Validation("missing 'image' field".into()))?;
    let captured_at =
        captured_at.unwrap_or_else(|| chrono::Utc::now().to_rfc3339());

    let doc_id = ulid::Ulid::new().to_string();
    let data_dir = state.config.data_dir();

    // Store original
    let original_path = storage::store_original(&data_dir, &doc_id, &image_bytes).await?;

    // Generate thumbnail
    let orig_abs = storage::original_abs_path(&data_dir, &doc_id);
    let thumb_abs = storage::thumbnail_abs_path(&data_dir, &doc_id);
    processing::generate_thumbnail(&orig_abs, &thumb_abs, &state.config.images).await?;
    let thumbnail_path = storage::thumbnail_rel_path(&doc_id);

    // Insert document record
    let doc = db::documents::insert(
        &state.db,
        db::documents::NewDocument {
            id: doc_id.clone(),
            captured_at,
            original_path,
            thumbnail_path,
        },
    )
    .await?;

    // Queue for background processing
    if let Err(e) = state.process_tx.send(doc_id.clone()).await {
        tracing::error!(doc_id, error = %e, "failed to queue document for processing");
    }

    tracing::info!(doc_id, "document uploaded");

    Ok((
        StatusCode::CREATED,
        Json(serde_json::to_value(&doc).unwrap()),
    ))
}

pub async fn list(
    State(state): State<AppState>,
    Query(params): Query<db::documents::ListParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    let (docs, total) = db::documents::list(&state.db, params).await?;
    Ok(Json(serde_json::json!({
        "documents": docs,
        "total": total,
    })))
}

pub async fn get_one(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let doc = db::documents::get(&state.db, &id).await?;
    Ok(Json(serde_json::to_value(&doc).unwrap()))
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<db::documents::DocumentUpdate>,
) -> Result<Json<serde_json::Value>, AppError> {
    let doc = db::documents::update(&state.db, &id, body).await?;
    Ok(Json(serde_json::to_value(&doc).unwrap()))
}

pub async fn delete(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    // Get the doc first to know its ID for file cleanup
    let _doc = db::documents::get(&state.db, &id).await?;

    // Delete from database (cascades to entities, edges)
    db::documents::delete(&state.db, &id).await?;

    // Clean up image files
    let data_dir = state.config.data_dir();
    storage::delete_images(&data_dir, &id).await?;

    tracing::info!(id, "document deleted");
    Ok(StatusCode::NO_CONTENT)
}

pub async fn reprocess(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    db::documents::reset_for_reprocess(&state.db, &id).await?;

    if let Err(e) = state.process_tx.send(id.clone()).await {
        tracing::error!(id, error = %e, "failed to queue document for reprocessing");
    }

    let doc = db::documents::get(&state.db, &id).await?;
    Ok(Json(serde_json::to_value(&doc).unwrap()))
}
