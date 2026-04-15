use axum::extract::State;
use axum::Json;
use serde::Deserialize;

use crate::AppError;
use crate::AppState;

#[derive(Deserialize)]
pub struct ChatRequest {
    messages: Vec<ChatMessage>,
    #[serde(default = "default_max_tokens")]
    max_tokens: u32,
    #[serde(default = "default_temperature")]
    temperature: f64,
}

#[derive(Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

fn default_max_tokens() -> u32 { 2048 }
fn default_temperature() -> f64 { 0.7 }

pub async fn chat(
    State(state): State<AppState>,
    Json(req): Json<ChatRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let messages: Vec<(String, String)> = req
        .messages
        .into_iter()
        .map(|m| (m.role, m.content))
        .collect();

    let result = state
        .inference
        .chat(messages, req.max_tokens, req.temperature)
        .await?;

    Ok(Json(serde_json::json!({
        "content": result.content,
        "model": result.model,
        "usage": result.usage,
    })))
}
