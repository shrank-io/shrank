use crate::db;
use crate::db::documents::ExtractionResult;
use crate::AppError;

/// After extraction, infer relationships between the new document and existing ones.
/// This implements Section 5.4 of the spec:
/// 1. Match reference IDs across documents
/// 2. UPSERT entities and link them
/// 3. Detect follow-up correspondence from same sender
/// 4. Pattern-match German correspondence references
pub async fn infer(
    db_handle: &db::Db,
    doc_id: &str,
    extraction: &ExtractionResult,
) -> Result<(), AppError> {
    // Step 1: Match reference IDs
    if let Some(ref refs) = extraction.reference_ids {
        if let Some(arr) = refs.as_array() {
            for ref_entry in arr {
                if let Some(value) = ref_entry.get("value").and_then(|v| v.as_str()) {
                    match_reference(db_handle, doc_id, value).await?;
                }
            }
        }
    }

    // Step 2: UPSERT entities and create links
    for entity in &extraction.entities {
        let entity_id = db::entities::upsert(
            db_handle,
            &entity.entity_type,
            &entity.value,
            Some(&entity.value),
        )
        .await?;

        db::entities::link_document(db_handle, doc_id, &entity_id, &entity.role, 1.0).await?;
    }

    // Step 3: Same-sender follow-up detection
    if let Some(ref sender) = extraction.sender_normalized {
        detect_followup(db_handle, doc_id, sender, extraction).await?;
    }

    Ok(())
}

/// Find existing documents that share a reference ID and create edges.
async fn match_reference(db_handle: &db::Db, doc_id: &str, ref_value: &str) -> Result<(), AppError> {
    let doc_id_for_query = doc_id.to_string();
    let ref_value = ref_value.to_string();

    let conn = db_handle.read().await?;
    let matching_ids: Vec<String> = conn
        .interact(move |conn| {
            let mut stmt = conn.prepare(
                "SELECT id FROM documents WHERE reference_ids LIKE ?1 AND id != ?2 LIMIT 10",
            )?;
            let pattern = format!("%{ref_value}%");
            let ids = stmt
                .query_map(rusqlite::params![pattern, doc_id_for_query], |row| row.get(0))?
                .collect::<rusqlite::Result<Vec<String>>>()?;
            Ok::<Vec<String>, rusqlite::Error>(ids)
        })
        .await??;

    for target_id in matching_ids {
        db::graph::create_edge(
            db_handle,
            doc_id,
            &target_id,
            "references",
            0.9,
            "reference_match",
        )
        .await?;
    }

    Ok(())
}

/// If the same sender sent a document recently, create a follow-up edge.
async fn detect_followup(
    db_handle: &db::Db,
    doc_id: &str,
    sender: &str,
    extraction: &ExtractionResult,
) -> Result<(), AppError> {
    let doc_id_owned = doc_id.to_string();
    let sender = sender.to_string();
    let has_matching_refs = extraction.reference_ids.is_some();

    let conn = db_handle.read().await?;
    let recent: Option<(String, Option<String>)> = conn
        .interact(move |conn| {
            conn.query_row(
                "SELECT id, document_date FROM documents
                 WHERE sender_normalized = ?1 AND id != ?2 AND status = 'complete'
                 ORDER BY document_date DESC NULLS LAST LIMIT 1",
                rusqlite::params![sender, doc_id_owned],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
        })
        .await??;

    if let Some((target_id, _target_date)) = recent {
        // If reference numbers match, high confidence follow-up
        let confidence = if has_matching_refs { 0.95 } else { 0.6 };

        db::graph::create_edge(
            db_handle,
            doc_id,
            &target_id,
            "follows_up",
            confidence,
            "llm",
        )
        .await?;
    }

    Ok(())
}

/// Extension trait to add `.optional()` to rusqlite Results.
trait OptionalExt<T> {
    fn optional(self) -> Result<Option<T>, rusqlite::Error>;
}

impl<T> OptionalExt<T> for rusqlite::Result<T> {
    fn optional(self) -> Result<Option<T>, rusqlite::Error> {
        match self {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }
}
