use tokio::sync::mpsc;

use crate::db;
use crate::inference;
use crate::AppError;
use crate::AppState;

/// Background task that processes documents from the channel.
pub async fn run_processor(state: AppState, mut rx: mpsc::Receiver<String>) {
    tracing::info!("document processor started");

    while let Some(doc_id) = rx.recv().await {
        tracing::info!(doc_id, "processing document");

        if let Err(e) = process_document(&state, &doc_id).await {
            tracing::error!(doc_id, error = %e, "document processing failed");
            if let Err(e2) =
                db::documents::update_status(&state.db, &doc_id, "error", Some(&e.to_string()))
                    .await
            {
                tracing::error!(doc_id, error = %e2, "failed to update error status");
            }
        }
    }

    tracing::info!("document processor stopped");
}

/// Re-enqueue documents that were pending/processing before a restart.
pub async fn requeue_pending(state: &AppState) -> Result<(), AppError> {
    let pending = db::documents::get_pending_ids(&state.db).await?;
    if !pending.is_empty() {
        tracing::info!(count = pending.len(), "re-enqueuing pending documents");
        for id in pending {
            if let Err(e) = state.process_tx.send(id.clone()).await {
                tracing::error!(id, error = %e, "failed to re-enqueue document");
            }
        }
    }
    Ok(())
}

async fn process_document(state: &AppState, doc_id: &str) -> Result<(), AppError> {
    // 1. Set status to processing
    db::documents::update_status(&state.db, doc_id, "processing", None).await?;

    let doc = db::documents::get(&state.db, doc_id).await?;

    // ---------------------------------------------------------------
    // Pass 1: OCR (image → markdown)
    // Skip if ocr_markdown already exists (e.g. reprocess only re-runs pass 2)
    // ---------------------------------------------------------------
    let ocr_markdown = if let Some(ref existing) = doc.ocr_markdown {
        tracing::info!(doc_id, "skipping OCR pass — using existing markdown");
        existing.clone()
    } else {
        tracing::info!(doc_id, "pass 1: OCR");
        let img_path = state.config.data_dir().join(&doc.original_path);
        let img_bytes = tokio::fs::read(&img_path).await?;

        let markdown = state.inference.ocr(&img_bytes).await?;

        // Store immediately so it survives crashes
        db::documents::update_ocr_markdown(&state.db, doc_id, &markdown).await?;

        tracing::info!(doc_id, chars = markdown.len(), "pass 1 complete");
        markdown
    };

    // ---------------------------------------------------------------
    // Pass 2: Extraction (markdown → structured JSON)
    // ---------------------------------------------------------------
    tracing::info!(doc_id, "pass 2: extraction");

    let tags = db::documents::get_all_tags(&state.db).await?;
    let senders = db::documents::get_all_senders(&state.db).await?;

    let raw_response = state
        .inference
        .extract(&ocr_markdown, &tags, &senders)
        .await?;

    // Parse and validate
    let extraction = inference::extraction::parse(&raw_response)?;

    // Update document with extracted metadata
    db::documents::update_extraction(&state.db, doc_id, &extraction).await?;

    tracing::info!(doc_id, "pass 2 complete");

    // ---------------------------------------------------------------
    // Post-processing: relationships + embedding
    // ---------------------------------------------------------------
    inference::relationships::infer(&state.db, doc_id, &extraction).await?;

    if let Some(ref text) = extraction.extracted_text {
        if !text.is_empty() {
            match state.inference.embed(text).await {
                Ok(_embedding) => {
                    tracing::debug!(doc_id, "embedding generated");
                }
                Err(e) => {
                    // Embedding failure is non-fatal
                    tracing::warn!(doc_id, error = %e, "failed to generate embedding");
                }
            }
        }
    }

    tracing::info!(
        doc_id,
        sender = extraction.sender.as_deref().unwrap_or("unknown"),
        confidence = extraction.confidence.unwrap_or(0.0),
        "document processing complete"
    );

    Ok(())
}
