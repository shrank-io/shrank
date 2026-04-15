use axum::extract::{Path, Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::db;
use crate::AppError;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct RelatedParams {
    pub depth: Option<u32>,
}

#[derive(Serialize)]
pub struct GraphData {
    nodes: Vec<GraphNode>,
    links: Vec<GraphLink>,
}

#[derive(Serialize)]
struct GraphNode {
    id: String,
    label: String,
    #[serde(rename = "type")]
    node_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    document: Option<db::documents::Document>,
}

#[derive(Serialize)]
struct GraphLink {
    source: String,
    target: String,
    relation_type: String,
    confidence: f64,
}

pub async fn related(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(params): Query<RelatedParams>,
) -> Result<Json<GraphData>, AppError> {
    let focus_doc = db::documents::get(&state.db, &id).await?;

    let depth = params.depth.unwrap_or(2);
    let results = db::graph::get_related(&state.db, &id, depth).await?;

    let mut node_ids = std::collections::HashSet::new();
    let mut nodes = Vec::new();
    let mut links = Vec::new();

    // Helper to add a document node (deduped)
    let mut add_doc_node = |doc: &db::documents::Document| {
        if node_ids.insert(doc.id.clone()) {
            let label = doc
                .sender
                .clone()
                .unwrap_or_else(|| doc.id[..8.min(doc.id.len())].to_string());
            nodes.push(GraphNode {
                id: doc.id.clone(),
                label,
                node_type: "document".into(),
                document: Some(doc.clone()),
            });
        }
    };

    // Focus document node
    add_doc_node(&focus_doc);

    // Related document nodes + edges
    for rel in &results {
        add_doc_node(&rel.document);
        links.push(GraphLink {
            source: focus_doc.id.clone(),
            target: rel.document.id.clone(),
            relation_type: rel.relation_type.clone(),
            confidence: 1.0,
        });
    }

    // Collect all document IDs in the graph so far
    let doc_ids: Vec<String> = node_ids.iter().cloned().collect();

    // Add entity nodes and doc→entity links for every document in the graph
    for doc_id in &doc_ids {
        let entities = db::entities::get_entities_for_document(&state.db, doc_id).await?;
        for (entity, role) in entities {
            if node_ids.insert(entity.id.clone()) {
                nodes.push(GraphNode {
                    id: entity.id.clone(),
                    label: entity.display_name.clone().unwrap_or_else(|| entity.value.clone()),
                    node_type: "entity".into(),
                    document: None,
                });
            }
            links.push(GraphLink {
                source: doc_id.clone(),
                target: entity.id.clone(),
                relation_type: role,
                confidence: 1.0,
            });
        }
    }

    Ok(Json(GraphData { nodes, links }))
}

#[derive(Debug, Deserialize)]
pub struct ListParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn list_entities(
    State(state): State<AppState>,
    Query(params): Query<ListParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    let limit = params.limit.unwrap_or(50).min(100);
    let offset = params.offset.unwrap_or(0);

    let (entities, total) = db::entities::list_all(&state.db, limit, offset).await?;
    Ok(Json(serde_json::json!({
        "entities": entities,
        "total": total,
    })))
}

pub async fn entity_documents(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<db::documents::Document>>, AppError> {
    let docs = db::entities::get_documents_for_entity(&state.db, &id).await?;
    Ok(Json(docs))
}
