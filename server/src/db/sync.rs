use rusqlite::params;
use serde::Serialize;

use super::Db;
use crate::AppError;

#[derive(Debug, Serialize)]
pub struct SyncResponse {
    pub documents: Vec<SyncDocument>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SyncDocument {
    pub id: String,
    pub created_at: String,
    pub updated_at: String,
    pub captured_at: String,
    pub status: String,
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
    pub thumbnail_url: String,
}

pub async fn register_client(db: &Db, client_id: &str) -> Result<(), AppError> {
    let client_id = client_id.to_string();
    let conn = db.write().await?;
    conn.interact(move |conn| {
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT OR REPLACE INTO sync_cursors (client_id, last_sync_at, last_document_id)
             VALUES (?1, ?2, NULL)",
            params![client_id, now],
        )?;
        Ok::<(), rusqlite::Error>(())
    })
    .await?
    .map_err(AppError::from)
}

pub async fn get_documents_since(
    db: &Db,
    since: Option<&str>,
    limit: i64,
) -> Result<SyncResponse, AppError> {
    let since = since.map(|s| s.to_string());
    let limit = limit.min(100);

    let conn = db.read().await?;
    conn.interact(move |conn| {
        let (sql, bind) = if let Some(ref cursor) = since {
            (
                "SELECT * FROM documents WHERE updated_at > ?1 ORDER BY updated_at ASC LIMIT ?2",
                vec![cursor.clone(), limit.to_string()],
            )
        } else {
            (
                "SELECT * FROM documents ORDER BY updated_at ASC LIMIT ?1",
                vec![limit.to_string()],
            )
        };

        let mut stmt = conn.prepare(sql)?;
        let refs: Vec<&dyn rusqlite::types::ToSql> =
            bind.iter().map(|v| v as &dyn rusqlite::types::ToSql).collect();

        let docs = stmt
            .query_map(refs.as_slice(), |row| {
                let amounts_str: Option<String> = row.get("amounts")?;
                let dates_str: Option<String> = row.get("dates")?;
                let reference_ids_str: Option<String> = row.get("reference_ids")?;
                let tags_str: Option<String> = row.get("tags")?;
                let id: String = row.get("id")?;

                Ok(SyncDocument {
                    thumbnail_url: format!("/api/images/thumbnail/{}", &id),
                    id,
                    created_at: row.get("created_at")?,
                    updated_at: row.get("updated_at")?,
                    captured_at: row.get("captured_at")?,
                    status: row.get("status")?,
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
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        let next_cursor = if docs.len() as i64 == limit {
            docs.last().map(|d| d.updated_at.clone())
        } else {
            None
        };

        Ok::<SyncResponse, rusqlite::Error>(SyncResponse {
            documents: docs,
            next_cursor,
        })
    })
    .await?
    .map_err(AppError::from)
}
