use std::sync::Arc;

use tokio::sync::mpsc;
use tower_http::cors::CorsLayer;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

mod api;
mod auth;
mod config;
mod db;
mod errors;
mod images;
mod inference;
mod processing;

pub use errors::AppError;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<db::Db>,
    pub inference: Arc<inference::InferenceClient>,
    pub process_tx: mpsc::Sender<String>,
    pub config: Arc<config::Config>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("shrank_server=debug,tower_http=debug")),
        )
        .init();

    let config = config::load()?;
    let data_dir = config.data_dir();
    tokio::fs::create_dir_all(&data_dir).await?;

    tracing::info!(data_dir = %data_dir.display(), "starting shrank server");

    let db = db::Db::new(&data_dir).await?;

    let inference_client =
        inference::InferenceClient::new(&config.inference, &config.embeddings);

    let (process_tx, process_rx) = mpsc::channel::<String>(100);

    let state = AppState {
        db: Arc::new(db),
        inference: Arc::new(inference_client),
        process_tx,
        config: Arc::new(config),
    };

    // Spawn background document processor
    let processor_state = state.clone();
    tokio::spawn(async move {
        processing::run_processor(processor_state, process_rx).await;
    });

    // Re-enqueue any documents that were pending/processing before shutdown
    processing::requeue_pending(&state).await?;

    let app = api::router(state.clone())
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .layer(RequestBodyLimitLayer::new(50 * 1024 * 1024));

    let addr = format!("{}:{}", state.config.server.host, state.config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("shrank server listening on {}", addr);
    axum::serve(listener, app).await?;

    Ok(())
}
