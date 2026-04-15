use axum::routing::{get, post};
use axum::Router;
use tower_http::services::ServeDir;

use crate::AppState;

mod chat;
mod documents;
mod graph;
mod images;
mod search;
mod sync;
mod system;

pub fn router(state: AppState) -> Router {
    let api = Router::new()
        // Documents
        .route("/api/documents", post(documents::upload).get(documents::list))
        .route(
            "/api/documents/{id}",
            get(documents::get_one)
                .put(documents::update)
                .delete(documents::delete),
        )
        .route("/api/documents/{id}/reprocess", post(documents::reprocess))
        // Search
        .route("/api/search", get(search::search))
        // Graph
        .route("/api/documents/{id}/related", get(graph::related))
        .route("/api/entities", get(graph::list_entities))
        .route("/api/entities/{id}/documents", get(graph::entity_documents))
        // Sync
        .route("/api/sync", get(sync::delta_sync))
        .route("/api/sync/register", post(sync::register))
        // Images
        .route("/api/images/original/{id}", get(images::original))
        .route("/api/images/thumbnail/{id}", get(images::thumbnail))
        // Chat
        .route("/api/chat", post(chat::chat))
        // System
        .route("/api/health", get(system::health))
        .route("/api/stats", get(system::stats))
        // Auth middleware on all API routes
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            crate::auth::require_auth,
        ))
        .with_state(state.clone());

    // Serve web UI static files as fallback (production mode)
    let static_dir = std::env::current_dir()
        .unwrap_or_default()
        .parent()
        .unwrap_or(&std::path::PathBuf::from("."))
        .join("web")
        .join("dist");

    Router::new()
        .merge(api)
        .fallback_service(ServeDir::new(static_dir))
}
