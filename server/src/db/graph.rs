use rusqlite::params;
use serde::Serialize;

use super::Db;
use crate::AppError;

pub async fn create_edge(
    db: &Db,
    source_id: &str,
    target_id: &str,
    relation_type: &str,
    confidence: f64,
    inferred_by: &str,
) -> Result<(), AppError> {
    let source_id = source_id.to_string();
    let target_id = target_id.to_string();
    let relation_type = relation_type.to_string();
    let inferred_by = inferred_by.to_string();

    let conn = db.write().await?;
    conn.interact(move |conn| {
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT OR IGNORE INTO document_edges (source_id, target_id, relation_type, confidence, inferred_by, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![source_id, target_id, relation_type, confidence, inferred_by, now],
        )?;
        Ok::<(), rusqlite::Error>(())
    })
    .await?
    .map_err(AppError::from)
}

#[derive(Debug, Serialize)]
pub struct RelatedDocument {
    pub document: super::documents::Document,
    pub relation_type: String,
    pub depth: i32,
}

pub async fn get_related(
    db: &Db,
    doc_id: &str,
    max_depth: u32,
) -> Result<Vec<RelatedDocument>, AppError> {
    let doc_id = doc_id.to_string();
    let max_depth = max_depth.min(3) as i32; // cap at 3 to prevent expensive queries

    let conn = db.read().await?;
    conn.interact(move |conn| {
        // Two-phase approach: first collect related doc IDs via graph traversal,
        // then fetch full documents.
        let mut stmt = conn.prepare(
            "WITH RECURSIVE related(doc_id, depth, path, rel_type) AS (
                SELECT ?1, 0, ?1, 'self'

                UNION ALL

                -- Follow direct edges (both directions)
                SELECT
                    CASE WHEN de.source_id = r.doc_id THEN de.target_id ELSE de.source_id END,
                    r.depth + 1,
                    r.path || ',' || CASE WHEN de.source_id = r.doc_id THEN de.target_id ELSE de.source_id END,
                    de.relation_type
                FROM related r
                JOIN document_edges de ON de.source_id = r.doc_id OR de.target_id = r.doc_id
                WHERE r.depth < ?2
                  AND INSTR(r.path, CASE WHEN de.source_id = r.doc_id THEN de.target_id ELSE de.source_id END) = 0

                UNION ALL

                -- Follow entity connections (documents sharing entities)
                SELECT
                    de2.document_id,
                    r.depth + 1,
                    r.path || ',' || de2.document_id,
                    'shared_entity'
                FROM related r
                JOIN document_entities de1 ON de1.document_id = r.doc_id
                JOIN document_entities de2 ON de2.entity_id = de1.entity_id AND de2.document_id != r.doc_id
                WHERE r.depth < ?2
                  AND INSTR(r.path, de2.document_id) = 0
            )
            SELECT DISTINCT doc_id, MIN(depth) as depth, rel_type
            FROM related
            WHERE depth > 0
            GROUP BY doc_id
            ORDER BY depth ASC, doc_id DESC",
        )?;

        let related_ids: Vec<(String, i32, String)> = stmt
            .query_map(params![doc_id, max_depth], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        let mut results = Vec::new();
        for (rid, depth, rel_type) in related_ids {
            if let Ok(doc) = conn.query_row(
                "SELECT * FROM documents WHERE id = ?1",
                [&rid],
                super::documents::document_from_row,
            ) {
                results.push(RelatedDocument {
                    document: doc,
                    relation_type: rel_type,
                    depth,
                });
            }
        }

        Ok::<Vec<RelatedDocument>, rusqlite::Error>(results)
    })
    .await?
    .map_err(AppError::from)
}
