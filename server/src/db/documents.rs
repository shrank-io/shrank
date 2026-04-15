use rusqlite::{params, Row};
use serde::{Deserialize, Serialize};

use super::Db;
use crate::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub created_at: String,
    pub updated_at: String,
    pub captured_at: String,
    pub synced_at: Option<String>,
    pub original_path: String,
    pub thumbnail_path: String,
    pub status: String,
    pub processing_error: Option<String>,
    pub raw_llm_response: Option<String>,
    pub language: Option<String>,
    pub sender: Option<String>,
    pub sender_normalized: Option<String>,
    pub document_date: Option<String>,
    pub document_type: Option<String>,
    pub subject: Option<String>,
    pub extracted_text: Option<String>,
    pub amounts: Option<serde_json::Value>,
    pub dates: Option<serde_json::Value>,
    pub reference_ids: Option<serde_json::Value>,
    pub tags: Option<serde_json::Value>,
    pub confidence: Option<f64>,
}

pub(super) fn document_from_row(row: &Row<'_>) -> rusqlite::Result<Document> {
    let amounts_str: Option<String> = row.get("amounts")?;
    let dates_str: Option<String> = row.get("dates")?;
    let reference_ids_str: Option<String> = row.get("reference_ids")?;
    let tags_str: Option<String> = row.get("tags")?;

    Ok(Document {
        id: row.get("id")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
        captured_at: row.get("captured_at")?,
        synced_at: row.get("synced_at")?,
        original_path: row.get("original_path")?,
        thumbnail_path: row.get("thumbnail_path")?,
        status: row.get("status")?,
        processing_error: row.get("processing_error")?,
        raw_llm_response: row.get("raw_llm_response")?,
        language: row.get("language")?,
        sender: row.get("sender")?,
        sender_normalized: row.get("sender_normalized")?,
        document_date: row.get("document_date")?,
        document_type: row.get("document_type")?,
        subject: row.get("subject")?,
        extracted_text: row.get("extracted_text")?,
        amounts: amounts_str.and_then(|s| serde_json::from_str(&s).ok()),
        dates: dates_str.and_then(|s| serde_json::from_str(&s).ok()),
        reference_ids: reference_ids_str.and_then(|s| serde_json::from_str(&s).ok()),
        tags: tags_str.and_then(|s| serde_json::from_str(&s).ok()),
        confidence: row.get("confidence")?,
    })
}

pub struct NewDocument {
    pub id: String,
    pub captured_at: String,
    pub original_path: String,
    pub thumbnail_path: String,
}

pub async fn insert(db: &Db, doc: NewDocument) -> Result<Document, AppError> {
    let conn = db.write().await?;
    conn.interact(move |conn| {
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO documents (id, created_at, updated_at, captured_at, synced_at, original_path, thumbnail_path, status)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'pending')",
            params![doc.id, now, now, doc.captured_at, now, doc.original_path, doc.thumbnail_path],
        )?;

        conn.query_row(
            "SELECT * FROM documents WHERE id = ?1",
            [&doc.id],
            document_from_row,
        )
    })
    .await?
    .map_err(AppError::from)
}

pub async fn get(db: &Db, id: &str) -> Result<Document, AppError> {
    let id = id.to_string();
    let conn = db.read().await?;
    conn.interact(move |conn| {
        conn.query_row("SELECT * FROM documents WHERE id = ?1", [&id], document_from_row)
    })
    .await?
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => AppError::NotFound(format!("document not found")),
        other => AppError::Database(other),
    })
}

#[derive(Debug, Deserialize)]
pub struct ListParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub status: Option<String>,
    pub sender: Option<String>,
    pub document_type: Option<String>,
    pub tag: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
}

pub async fn list(db: &Db, params: ListParams) -> Result<(Vec<Document>, i64), AppError> {
    let conn = db.read().await?;
    conn.interact(move |conn| {
        let mut conditions = Vec::new();
        let mut bind_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(ref status) = params.status {
            conditions.push(format!("status = ?{}", bind_values.len() + 1));
            bind_values.push(Box::new(status.clone()));
        }
        if let Some(ref sender) = params.sender {
            conditions.push(format!("sender_normalized = ?{}", bind_values.len() + 1));
            bind_values.push(Box::new(sender.clone()));
        }
        if let Some(ref doc_type) = params.document_type {
            conditions.push(format!("document_type = ?{}", bind_values.len() + 1));
            bind_values.push(Box::new(doc_type.clone()));
        }
        if let Some(ref tag) = params.tag {
            conditions.push(format!("tags LIKE ?{}", bind_values.len() + 1));
            bind_values.push(Box::new(format!("%\"{tag}\"%")));
        }
        if let Some(ref date_from) = params.date_from {
            conditions.push(format!("document_date >= ?{}", bind_values.len() + 1));
            bind_values.push(Box::new(date_from.clone()));
        }
        if let Some(ref date_to) = params.date_to {
            conditions.push(format!("document_date <= ?{}", bind_values.len() + 1));
            bind_values.push(Box::new(date_to.clone()));
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        let limit = params.limit.unwrap_or(50).min(100);
        let offset = params.offset.unwrap_or(0);

        let count_sql = format!("SELECT COUNT(*) FROM documents {where_clause}");
        let refs: Vec<&dyn rusqlite::types::ToSql> = bind_values.iter().map(|b| b.as_ref()).collect();
        let total: i64 = conn.query_row(&count_sql, refs.as_slice(), |row| row.get(0))?;

        let query_sql = format!(
            "SELECT * FROM documents {where_clause} ORDER BY created_at DESC LIMIT ?{} OFFSET ?{}",
            bind_values.len() + 1,
            bind_values.len() + 2
        );
        bind_values.push(Box::new(limit));
        bind_values.push(Box::new(offset));

        let refs: Vec<&dyn rusqlite::types::ToSql> = bind_values.iter().map(|b| b.as_ref()).collect();
        let mut stmt = conn.prepare(&query_sql)?;
        let docs = stmt
            .query_map(refs.as_slice(), document_from_row)?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        Ok::<(Vec<Document>, i64), rusqlite::Error>((docs, total))
    })
    .await?
    .map_err(AppError::from)
}

#[derive(Debug, Deserialize)]
pub struct DocumentUpdate {
    pub sender: Option<String>,
    pub sender_normalized: Option<String>,
    pub document_date: Option<String>,
    pub document_type: Option<String>,
    pub subject: Option<String>,
    pub tags: Option<serde_json::Value>,
    pub language: Option<String>,
}

pub async fn update(db: &Db, id: &str, upd: DocumentUpdate) -> Result<Document, AppError> {
    let id = id.to_string();
    let conn = db.write().await?;
    conn.interact(move |conn| {
        let mut sets = vec!["updated_at = ?1".to_string()];
        let now = chrono::Utc::now().to_rfc3339();
        let mut bind_values: Vec<Box<dyn rusqlite::types::ToSql>> = vec![Box::new(now)];

        macro_rules! maybe_set {
            ($field:ident, $col:expr) => {
                if let Some(ref val) = upd.$field {
                    bind_values.push(Box::new(val.clone()));
                    sets.push(format!("{} = ?{}", $col, bind_values.len()));
                }
            };
        }

        maybe_set!(sender, "sender");
        maybe_set!(sender_normalized, "sender_normalized");
        maybe_set!(document_date, "document_date");
        maybe_set!(document_type, "document_type");
        maybe_set!(subject, "subject");
        maybe_set!(language, "language");

        if let Some(ref tags) = upd.tags {
            let tags_str = serde_json::to_string(tags).unwrap_or_default();
            bind_values.push(Box::new(tags_str));
            sets.push(format!("tags = ?{}", bind_values.len()));
        }

        bind_values.push(Box::new(id.clone()));
        let sql = format!(
            "UPDATE documents SET {} WHERE id = ?{}",
            sets.join(", "),
            bind_values.len()
        );

        let refs: Vec<&dyn rusqlite::types::ToSql> = bind_values.iter().map(|b| b.as_ref()).collect();
        let rows = conn.execute(&sql, refs.as_slice())?;
        if rows == 0 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }

        conn.query_row("SELECT * FROM documents WHERE id = ?1", [&id], document_from_row)
    })
    .await?
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => AppError::NotFound("document not found".into()),
        other => AppError::Database(other),
    })
}

pub async fn delete(db: &Db, id: &str) -> Result<(), AppError> {
    let id = id.to_string();
    let conn = db.write().await?;
    conn.interact(move |conn| {
        let rows = conn.execute("DELETE FROM documents WHERE id = ?1", [&id])?;
        if rows == 0 {
            Err(rusqlite::Error::QueryReturnedNoRows)
        } else {
            Ok(())
        }
    })
    .await?
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => AppError::NotFound("document not found".into()),
        other => AppError::Database(other),
    })
}

pub async fn update_status(
    db: &Db,
    id: &str,
    status: &str,
    error: Option<&str>,
) -> Result<(), AppError> {
    let id = id.to_string();
    let status = status.to_string();
    let error = error.map(|s| s.to_string());
    let conn = db.write().await?;
    conn.interact(move |conn| {
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE documents SET status = ?1, processing_error = ?2, updated_at = ?3 WHERE id = ?4",
            params![status, error, now, id],
        )?;
        Ok::<(), rusqlite::Error>(())
    })
    .await?
    .map_err(AppError::from)
}

pub struct ExtractionResult {
    pub language: Option<String>,
    pub sender: Option<String>,
    pub sender_normalized: Option<String>,
    pub document_date: Option<String>,
    pub document_type: Option<String>,
    pub subject: Option<String>,
    pub extracted_text: Option<String>,
    pub amounts: Option<serde_json::Value>,
    pub dates: Option<serde_json::Value>,
    pub reference_ids: Option<serde_json::Value>,
    pub tags: Option<serde_json::Value>,
    pub confidence: Option<f64>,
    pub raw_response: String,
    pub entities: Vec<ExtractedEntity>,
}

pub struct ExtractedEntity {
    pub entity_type: String,
    pub value: String,
    pub role: String,
}

pub async fn update_extraction(
    db: &Db,
    id: &str,
    ext: &ExtractionResult,
) -> Result<(), AppError> {
    let id = id.to_string();
    let language = ext.language.clone();
    let sender = ext.sender.clone();
    let sender_normalized = ext.sender_normalized.clone();
    let document_date = ext.document_date.clone();
    let document_type = ext.document_type.clone();
    let subject = ext.subject.clone();
    let extracted_text = ext.extracted_text.clone();
    let amounts = ext.amounts.as_ref().map(|v| serde_json::to_string(v).unwrap_or_default());
    let dates = ext.dates.as_ref().map(|v| serde_json::to_string(v).unwrap_or_default());
    let reference_ids = ext.reference_ids.as_ref().map(|v| serde_json::to_string(v).unwrap_or_default());
    let tags = ext.tags.as_ref().map(|v| serde_json::to_string(v).unwrap_or_default());
    let confidence = ext.confidence;
    let raw = ext.raw_response.clone();

    let conn = db.write().await?;
    conn.interact(move |conn| {
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE documents SET
                language = ?1, sender = ?2, sender_normalized = ?3,
                document_date = ?4, document_type = ?5, subject = ?6,
                extracted_text = ?7, amounts = ?8, dates = ?9,
                reference_ids = ?10, tags = ?11, confidence = ?12,
                raw_llm_response = ?13, status = 'complete', updated_at = ?14
             WHERE id = ?15",
            params![
                language, sender, sender_normalized, document_date,
                document_type, subject, extracted_text, amounts,
                dates, reference_ids, tags, confidence, raw, now, id,
            ],
        )?;
        Ok::<(), rusqlite::Error>(())
    })
    .await?
    .map_err(AppError::from)
}

pub async fn get_pending_ids(db: &Db) -> Result<Vec<String>, AppError> {
    let conn = db.read().await?;
    conn.interact(|conn| {
        let mut stmt =
            conn.prepare("SELECT id FROM documents WHERE status IN ('pending', 'processing') ORDER BY created_at")?;
        let ids = stmt
            .query_map([], |row| row.get(0))?
            .collect::<rusqlite::Result<Vec<String>>>()?;
        Ok::<Vec<String>, rusqlite::Error>(ids)
    })
    .await?
    .map_err(AppError::from)
}

pub async fn get_all_tags(db: &Db) -> Result<Vec<String>, AppError> {
    let conn = db.read().await?;
    conn.interact(|conn| {
        let mut stmt = conn.prepare(
            "SELECT DISTINCT tags FROM documents WHERE tags IS NOT NULL AND tags != '[]'",
        )?;
        let rows = stmt
            .query_map([], |row| row.get::<_, String>(0))?
            .collect::<rusqlite::Result<Vec<String>>>()?;

        let mut all_tags = std::collections::BTreeSet::new();
        for row in rows {
            if let Ok(arr) = serde_json::from_str::<Vec<String>>(&row) {
                for tag in arr {
                    all_tags.insert(tag);
                }
            }
        }
        Ok::<Vec<String>, rusqlite::Error>(all_tags.into_iter().collect())
    })
    .await?
    .map_err(AppError::from)
}

pub async fn get_all_senders(db: &Db) -> Result<Vec<String>, AppError> {
    let conn = db.read().await?;
    conn.interact(|conn| {
        let mut stmt = conn.prepare(
            "SELECT DISTINCT sender_normalized FROM documents WHERE sender_normalized IS NOT NULL ORDER BY sender_normalized",
        )?;
        let senders = stmt
            .query_map([], |row| row.get(0))?
            .collect::<rusqlite::Result<Vec<String>>>()?;
        Ok::<Vec<String>, rusqlite::Error>(senders)
    })
    .await?
    .map_err(AppError::from)
}

pub async fn count_by_status(db: &Db) -> Result<Vec<(String, i64)>, AppError> {
    let conn = db.read().await?;
    conn.interact(|conn| {
        let mut stmt = conn.prepare("SELECT status, COUNT(*) FROM documents GROUP BY status")?;
        let counts = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect::<rusqlite::Result<Vec<(String, i64)>>>()?;
        Ok::<Vec<(String, i64)>, rusqlite::Error>(counts)
    })
    .await?
    .map_err(AppError::from)
}

pub async fn reset_for_reprocess(db: &Db, id: &str) -> Result<(), AppError> {
    let id = id.to_string();
    let conn = db.write().await?;
    conn.interact(move |conn| {
        let now = chrono::Utc::now().to_rfc3339();
        let rows = conn.execute(
            "UPDATE documents SET
                status = 'pending', processing_error = NULL,
                language = NULL, sender = NULL, sender_normalized = NULL,
                document_date = NULL, document_type = NULL, subject = NULL,
                extracted_text = NULL, amounts = NULL, dates = NULL,
                reference_ids = NULL, tags = NULL, confidence = NULL,
                raw_llm_response = NULL, updated_at = ?1
             WHERE id = ?2",
            params![now, id],
        )?;
        if rows == 0 {
            Err(rusqlite::Error::QueryReturnedNoRows)
        } else {
            Ok(())
        }
    })
    .await?
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => AppError::NotFound("document not found".into()),
        other => AppError::Database(other),
    })
}
