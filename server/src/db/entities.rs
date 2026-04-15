use rusqlite::params;
use serde::Serialize;

use super::Db;
use crate::AppError;

#[derive(Debug, Clone, Serialize)]
pub struct Entity {
    pub id: String,
    pub entity_type: String,
    pub value: String,
    pub display_name: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: String,
}

/// Insert or find an existing entity. Returns the entity ID.
pub async fn upsert(
    db: &Db,
    entity_type: &str,
    value: &str,
    display_name: Option<&str>,
) -> Result<String, AppError> {
    let entity_type = entity_type.to_string();
    let value = value.to_string();
    let display_name = display_name.map(|s| s.to_string());

    let conn = db.write().await?;
    conn.interact(move |conn| {
        let now = chrono::Utc::now().to_rfc3339();
        let id = ulid::Ulid::new().to_string();

        conn.execute(
            "INSERT INTO entities (id, entity_type, value, display_name, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(entity_type, value) DO UPDATE SET
                display_name = COALESCE(excluded.display_name, entities.display_name)",
            params![id, entity_type, value, display_name, now],
        )?;

        // Return the actual ID (might be existing row)
        let actual_id: String = conn.query_row(
            "SELECT id FROM entities WHERE entity_type = ?1 AND value = ?2",
            params![entity_type, value],
            |row| row.get(0),
        )?;

        Ok::<String, rusqlite::Error>(actual_id)
    })
    .await?
    .map_err(AppError::from)
}

pub async fn link_document(
    db: &Db,
    document_id: &str,
    entity_id: &str,
    role: &str,
    confidence: f64,
) -> Result<(), AppError> {
    let document_id = document_id.to_string();
    let entity_id = entity_id.to_string();
    let role = role.to_string();

    let conn = db.write().await?;
    conn.interact(move |conn| {
        conn.execute(
            "INSERT OR IGNORE INTO document_entities (document_id, entity_id, role, confidence)
             VALUES (?1, ?2, ?3, ?4)",
            params![document_id, entity_id, role, confidence],
        )?;
        Ok::<(), rusqlite::Error>(())
    })
    .await?
    .map_err(AppError::from)
}

pub async fn list_all(db: &Db, limit: i64, offset: i64) -> Result<(Vec<Entity>, i64), AppError> {
    let conn = db.read().await?;
    conn.interact(move |conn| {
        let total: i64 =
            conn.query_row("SELECT COUNT(*) FROM entities", [], |row| row.get(0))?;

        let mut stmt = conn.prepare(
            "SELECT id, entity_type, value, display_name, metadata, created_at
             FROM entities ORDER BY created_at DESC LIMIT ?1 OFFSET ?2",
        )?;
        let entities = stmt
            .query_map(params![limit, offset], |row| {
                let metadata_str: Option<String> = row.get("metadata")?;
                Ok(Entity {
                    id: row.get("id")?,
                    entity_type: row.get("entity_type")?,
                    value: row.get("value")?,
                    display_name: row.get("display_name")?,
                    metadata: metadata_str.and_then(|s| serde_json::from_str(&s).ok()),
                    created_at: row.get("created_at")?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        Ok::<(Vec<Entity>, i64), rusqlite::Error>((entities, total))
    })
    .await?
    .map_err(AppError::from)
}

/// Get all entities linked to a specific document, with their roles.
pub async fn get_entities_for_document(
    db: &Db,
    document_id: &str,
) -> Result<Vec<(Entity, String)>, AppError> {
    let document_id = document_id.to_string();
    let conn = db.read().await?;
    conn.interact(move |conn| {
        let mut stmt = conn.prepare(
            "SELECT e.id, e.entity_type, e.value, e.display_name, e.metadata, e.created_at, de.role
             FROM entities e
             JOIN document_entities de ON de.entity_id = e.id
             WHERE de.document_id = ?1",
        )?;
        let results = stmt
            .query_map([&document_id], |row| {
                let metadata_str: Option<String> = row.get("metadata")?;
                Ok((
                    Entity {
                        id: row.get("id")?,
                        entity_type: row.get("entity_type")?,
                        value: row.get("value")?,
                        display_name: row.get("display_name")?,
                        metadata: metadata_str.and_then(|s| serde_json::from_str(&s).ok()),
                        created_at: row.get("created_at")?,
                    },
                    row.get::<_, String>("role")?,
                ))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok::<Vec<(Entity, String)>, rusqlite::Error>(results)
    })
    .await?
    .map_err(AppError::from)
}

pub async fn get_documents_for_entity(
    db: &Db,
    entity_id: &str,
) -> Result<Vec<super::documents::Document>, AppError> {
    let entity_id = entity_id.to_string();
    let conn = db.read().await?;
    conn.interact(move |conn| {
        let mut stmt = conn.prepare(
            "SELECT d.* FROM documents d
             JOIN document_entities de ON de.document_id = d.id
             WHERE de.entity_id = ?1
             ORDER BY d.document_date DESC NULLS LAST",
        )?;
        let docs = stmt
            .query_map([&entity_id], super::documents::document_from_row)?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok::<Vec<super::documents::Document>, rusqlite::Error>(docs)
    })
    .await?
    .map_err(AppError::from)
}
