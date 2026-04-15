use std::collections::HashMap;

use rusqlite::params;
use serde::Serialize;

use super::Db;
use crate::AppError;

#[derive(Debug)]
pub struct SearchQuery {
    pub raw: String,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Serialize)]
pub struct SearchResults {
    pub results: Vec<SearchHit>,
    pub facets: Facets,
    pub total: i64,
    pub query_intent: String,
}

#[derive(Debug, Serialize)]
pub struct SearchHit {
    pub document: super::documents::Document,
    pub score: f64,
    pub match_sources: Vec<String>,
    pub highlights: HashMap<String, String>,
}

#[derive(Debug, Serialize)]
pub struct Facets {
    pub senders: Vec<FacetEntry>,
    pub tags: Vec<FacetEntry>,
    pub types: Vec<FacetEntry>,
    pub years: Vec<FacetEntry>,
}

#[derive(Debug, Serialize)]
pub struct FacetEntry {
    pub name: String,
    pub count: i64,
}

#[derive(Debug)]
enum SearchIntent {
    Structured(Vec<Filter>),
    Keyword(String),
    Hybrid(String), // FTS5 + semantic
}

#[derive(Debug)]
struct Filter {
    field: String,
    value: String,
}

const RRF_K: f64 = 60.0;

fn parse_intent(query: &str) -> SearchIntent {
    let mut filters = Vec::new();
    let mut text_parts = Vec::new();

    for token in query.split_whitespace() {
        if let Some((field, value)) = token.split_once(':') {
            match field {
                "sender" | "type" | "tag" | "date" | "language" | "status" => {
                    filters.push(Filter {
                        field: field.to_string(),
                        value: value.trim_matches('"').to_string(),
                    });
                }
                _ => text_parts.push(token.to_string()),
            }
        } else {
            text_parts.push(token.to_string());
        }
    }

    let text = text_parts.join(" ");

    if !filters.is_empty() && text.is_empty() {
        SearchIntent::Structured(filters)
    } else if !text.is_empty() {
        if filters.is_empty() {
            SearchIntent::Hybrid(text)
        } else {
            // Has both structured filters and text — treat as hybrid but apply filters too
            SearchIntent::Hybrid(text)
        }
    } else {
        SearchIntent::Keyword(query.to_string())
    }
}

pub async fn search(
    db: &Db,
    query: &SearchQuery,
    _embedding: Option<&[f32]>,
) -> Result<SearchResults, AppError> {
    let raw = query.raw.clone();
    let limit = query.limit;
    let offset = query.offset;

    let conn = db.read().await?;
    conn.interact(move |conn| {
        let intent = parse_intent(&raw);
        let intent_desc;

        // Collect (doc_id, rank) from each layer
        let mut fts_results: Vec<(String, usize)> = Vec::new();
        let mut structured_results: Vec<(String, usize)> = Vec::new();

        match &intent {
            SearchIntent::Structured(filters) => {
                intent_desc = "structured".to_string();
                structured_results = run_structured(conn, filters)?;
            }
            SearchIntent::Keyword(text) => {
                intent_desc = "keyword".to_string();
                fts_results = run_fts(conn, text)?;
            }
            SearchIntent::Hybrid(text) => {
                intent_desc = "hybrid(keyword, semantic)".to_string();
                fts_results = run_fts(conn, text)?;
                // TODO: semantic layer when sqlite-vec is integrated
            }
        }

        // RRF fusion
        let mut scores: HashMap<String, (f64, Vec<String>)> = HashMap::new();

        for (i, (doc_id, _)) in fts_results.iter().enumerate() {
            let entry = scores.entry(doc_id.clone()).or_insert((0.0, Vec::new()));
            entry.0 += 1.0 / (RRF_K + i as f64 + 1.0);
            if !entry.1.contains(&"fts5".to_string()) {
                entry.1.push("fts5".to_string());
            }
        }
        for (i, (doc_id, _)) in structured_results.iter().enumerate() {
            let entry = scores.entry(doc_id.clone()).or_insert((0.0, Vec::new()));
            entry.0 += 1.0 / (RRF_K + i as f64 + 1.0);
            if !entry.1.contains(&"structured".to_string()) {
                entry.1.push("structured".to_string());
            }
        }

        let total = scores.len() as i64;

        // Sort by fused score
        let mut ranked: Vec<(String, f64, Vec<String>)> = scores
            .into_iter()
            .map(|(id, (score, sources))| (id, score, sources))
            .collect();
        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Paginate
        let page = ranked
            .into_iter()
            .skip(offset as usize)
            .take(limit as usize)
            .collect::<Vec<_>>();

        // Fetch full documents + build highlights
        let mut results = Vec::new();
        for (doc_id, score, sources) in page {
            if let Ok(doc) = conn.query_row(
                "SELECT * FROM documents WHERE id = ?1",
                [&doc_id],
                super::documents::document_from_row,
            ) {
                let mut highlights = HashMap::new();
                // Try to get FTS highlights
                if sources.contains(&"fts5".to_string()) {
                    if let Ok(snippet) = conn.query_row(
                        "SELECT snippet(documents_fts, 2, '<mark>', '</mark>', '...', 32)
                         FROM documents_fts WHERE documents_fts MATCH ?1 AND rowid = (SELECT rowid FROM documents WHERE id = ?2)",
                        params![raw, doc_id],
                        |row| row.get::<_, String>(0),
                    ) {
                        highlights.insert("extracted_text".to_string(), snippet);
                    }
                }

                results.push(SearchHit {
                    document: doc,
                    score,
                    match_sources: sources,
                    highlights,
                });
            }
        }

        // Compute facets from result set doc IDs
        let facets = compute_facets(conn)?;

        Ok::<SearchResults, rusqlite::Error>(SearchResults {
            results,
            facets,
            total,
            query_intent: intent_desc,
        })
    })
    .await?
    .map_err(AppError::from)
}

fn run_fts(
    conn: &rusqlite::Connection,
    query: &str,
) -> rusqlite::Result<Vec<(String, usize)>> {
    // FTS5 needs proper query syntax. Escape special chars for safety.
    let fts_query = query
        .replace('"', "\"\"")
        .split_whitespace()
        .map(|w| format!("\"{w}\""))
        .collect::<Vec<_>>()
        .join(" OR ");

    if fts_query.is_empty() {
        return Ok(Vec::new());
    }

    let mut stmt = conn.prepare(
        "SELECT d.id FROM documents_fts f
         JOIN documents d ON d.rowid = f.rowid
         WHERE documents_fts MATCH ?1
         ORDER BY f.rank
         LIMIT 100",
    )?;

    let results = stmt
        .query_map([&fts_query], |row| {
            Ok(row.get::<_, String>(0)?)
        })?
        .enumerate()
        .filter_map(|(i, r)| r.ok().map(|id| (id, i)))
        .collect();

    Ok(results)
}

fn run_structured(
    conn: &rusqlite::Connection,
    filters: &[Filter],
) -> rusqlite::Result<Vec<(String, usize)>> {
    let mut conditions = Vec::new();
    let mut bind_values: Vec<String> = Vec::new();

    for filter in filters {
        match filter.field.as_str() {
            "sender" => {
                bind_values.push(filter.value.clone());
                conditions.push(format!(
                    "sender_normalized LIKE '%' || ?{} || '%'",
                    bind_values.len()
                ));
            }
            "type" => {
                bind_values.push(filter.value.clone());
                conditions.push(format!("document_type = ?{}", bind_values.len()));
            }
            "tag" => {
                bind_values.push(format!("\"{}\"", filter.value));
                conditions.push(format!("tags LIKE '%' || ?{} || '%'", bind_values.len()));
            }
            "language" => {
                bind_values.push(filter.value.clone());
                conditions.push(format!("language = ?{}", bind_values.len()));
            }
            "status" => {
                bind_values.push(filter.value.clone());
                conditions.push(format!("status = ?{}", bind_values.len()));
            }
            "date" => {
                // Support >YYYY-MM-DD, <YYYY-MM-DD, YYYY-MM-DD
                if let Some(date) = filter.value.strip_prefix('>') {
                    bind_values.push(date.to_string());
                    conditions.push(format!("document_date > ?{}", bind_values.len()));
                } else if let Some(date) = filter.value.strip_prefix('<') {
                    bind_values.push(date.to_string());
                    conditions.push(format!("document_date < ?{}", bind_values.len()));
                } else {
                    bind_values.push(filter.value.clone());
                    conditions.push(format!("document_date = ?{}", bind_values.len()));
                }
            }
            _ => {}
        }
    }

    if conditions.is_empty() {
        return Ok(Vec::new());
    }

    let sql = format!(
        "SELECT id FROM documents WHERE {} ORDER BY document_date DESC NULLS LAST LIMIT 100",
        conditions.join(" AND ")
    );

    let refs: Vec<&dyn rusqlite::types::ToSql> =
        bind_values.iter().map(|v| v as &dyn rusqlite::types::ToSql).collect();

    let mut stmt = conn.prepare(&sql)?;
    let results = stmt
        .query_map(refs.as_slice(), |row| row.get::<_, String>(0))?
        .enumerate()
        .filter_map(|(i, r)| r.ok().map(|id| (id, i)))
        .collect();

    Ok(results)
}

fn compute_facets(conn: &rusqlite::Connection) -> rusqlite::Result<Facets> {
    // Senders
    let mut stmt = conn.prepare(
        "SELECT sender_normalized, COUNT(*) as cnt FROM documents
         WHERE sender_normalized IS NOT NULL
         GROUP BY sender_normalized ORDER BY cnt DESC LIMIT 20",
    )?;
    let senders = stmt
        .query_map([], |row| {
            Ok(FacetEntry {
                name: row.get(0)?,
                count: row.get(1)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    // Types
    let mut stmt = conn.prepare(
        "SELECT document_type, COUNT(*) as cnt FROM documents
         WHERE document_type IS NOT NULL
         GROUP BY document_type ORDER BY cnt DESC",
    )?;
    let types = stmt
        .query_map([], |row| {
            Ok(FacetEntry {
                name: row.get(0)?,
                count: row.get(1)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    // Years
    let mut stmt = conn.prepare(
        "SELECT SUBSTR(document_date, 1, 4) as year, COUNT(*) as cnt FROM documents
         WHERE document_date IS NOT NULL
         GROUP BY year ORDER BY year DESC",
    )?;
    let years = stmt
        .query_map([], |row| {
            Ok(FacetEntry {
                name: row.get(0)?,
                count: row.get(1)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    // Tags — aggregate from JSON arrays
    let mut tag_counts: HashMap<String, i64> = HashMap::new();
    let mut stmt = conn.prepare(
        "SELECT tags FROM documents WHERE tags IS NOT NULL AND tags != '[]'",
    )?;
    let rows = stmt
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<rusqlite::Result<Vec<String>>>()?;
    for row in rows {
        if let Ok(arr) = serde_json::from_str::<Vec<String>>(&row) {
            for tag in arr {
                *tag_counts.entry(tag).or_insert(0) += 1;
            }
        }
    }
    let mut tags: Vec<FacetEntry> = tag_counts
        .into_iter()
        .map(|(name, count)| FacetEntry { name, count })
        .collect();
    tags.sort_by(|a, b| b.count.cmp(&a.count));
    tags.truncate(30);

    Ok(Facets {
        senders,
        tags,
        types,
        years,
    })
}
