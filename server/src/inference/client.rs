use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::config::{EmbeddingsConfig, InferenceConfig};
use crate::AppError;

use super::prompt;

pub struct InferenceClient {
    http: reqwest::Client,
    endpoint: String,
    model: String,
    embed_model: String,
}

// ---------------------------------------------------------------------------
// OpenAI-compatible request/response types for vllm-mlx
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    max_tokens: u32,
    temperature: f64,
}

#[derive(Serialize)]
#[serde(untagged)]
enum ChatMessage {
    Text {
        role: String,
        content: String,
    },
    Multimodal {
        role: String,
        content: Vec<ContentPart>,
    },
}

#[derive(Serialize)]
#[serde(tag = "type")]
enum ContentPart {
    #[serde(rename = "image_url")]
    ImageUrl { image_url: ImageUrl },
    #[serde(rename = "text")]
    Text { text: String },
}

#[derive(Serialize)]
struct ImageUrl {
    url: String,
}

#[derive(Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<Choice>,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    usage: Option<serde_json::Value>,
}

#[derive(Deserialize)]
struct Choice {
    message: ChoiceMessage,
}

#[derive(Deserialize)]
struct ChoiceMessage {
    content: String,
}

#[derive(Serialize)]
struct EmbeddingRequest {
    model: String,
    input: Vec<String>,
}

#[derive(Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
}

#[derive(Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
}

impl InferenceClient {
    pub fn new(inference: &InferenceConfig, embeddings: &EmbeddingsConfig) -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(180))
            .connect_timeout(Duration::from_secs(10))
            .build()
            .expect("failed to create HTTP client");

        // Both inference and embeddings go through the same vllm-mlx instance
        // but we allow separate endpoints for flexibility.
        let endpoint = inference.endpoint.trim_end_matches('/').to_string();
        let _embed_endpoint = embeddings.endpoint.trim_end_matches('/').to_string();

        Self {
            http,
            endpoint,
            model: inference.model.clone(),
            embed_model: embeddings.model.clone(),
        }
    }

    /// Extract structured data from a document image via vllm-mlx vision.
    pub async fn extract(
        &self,
        image_bytes: &[u8],
        existing_tags: &[String],
        existing_senders: &[String],
    ) -> Result<serde_json::Value, AppError> {
        use base64::Engine;
        let image_base64 = base64::engine::general_purpose::STANDARD.encode(image_bytes);

        let user_prompt = prompt::build_extraction_prompt(existing_tags, existing_senders);

        let body = ChatCompletionRequest {
            model: self.model.clone(),
            messages: vec![
                ChatMessage::Text {
                    role: "system".into(),
                    content: prompt::EXTRACTION_SYSTEM_PROMPT.into(),
                },
                ChatMessage::Multimodal {
                    role: "user".into(),
                    content: vec![
                        ContentPart::ImageUrl {
                            image_url: ImageUrl {
                                url: format!("data:image/jpeg;base64,{image_base64}"),
                            },
                        },
                        ContentPart::Text { text: user_prompt },
                    ],
                },
            ],
            max_tokens: 4096,
            temperature: 0.1,
        };

        let resp = self
            .http
            .post(format!("{}/v1/chat/completions", self.endpoint))
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::Inference(format!("vllm-mlx extract request failed: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(AppError::Inference(format!(
                "vllm-mlx returned {status}: {text}"
            )));
        }

        let completion: ChatCompletionResponse = resp
            .json()
            .await
            .map_err(|e| AppError::Inference(format!("failed to parse vllm-mlx response: {e}")))?;

        let raw_text = completion
            .choices
            .first()
            .map(|c| c.message.content.as_str())
            .ok_or_else(|| AppError::Inference("vllm-mlx returned no choices".into()))?;

        // Parse LLM output into clean JSON (handles fences, truncation, etc.)
        let parsed = super::extraction::parse_llm_json(raw_text)?;
        Ok(parsed)
    }

    /// Get a text embedding via vllm-mlx /v1/embeddings.
    pub async fn embed(&self, text: &str) -> Result<Vec<f32>, AppError> {
        let body = EmbeddingRequest {
            model: self.embed_model.clone(),
            input: vec![text.to_string()],
        };

        let resp = self
            .http
            .post(format!("{}/v1/embeddings", self.endpoint))
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::Inference(format!("vllm-mlx embed request failed: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(AppError::Inference(format!(
                "vllm-mlx embed returned {status}: {text}"
            )));
        }

        let result: EmbeddingResponse = resp
            .json()
            .await
            .map_err(|e| AppError::Inference(format!("failed to parse embed response: {e}")))?;

        result
            .data
            .into_iter()
            .next()
            .map(|d| d.embedding)
            .ok_or_else(|| AppError::Inference("vllm-mlx returned no embedding data".into()))
    }

    /// Simple text chat via vllm-mlx.
    pub async fn chat(
        &self,
        messages: Vec<(String, String)>,
        max_tokens: u32,
        temperature: f64,
    ) -> Result<ChatResult, AppError> {
        let chat_messages: Vec<ChatMessage> = messages
            .into_iter()
            .map(|(role, content)| ChatMessage::Text { role, content })
            .collect();

        let body = ChatCompletionRequest {
            model: self.model.clone(),
            messages: chat_messages,
            max_tokens,
            temperature,
        };

        let resp = self
            .http
            .post(format!("{}/v1/chat/completions", self.endpoint))
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::Inference(format!("vllm-mlx chat request failed: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(AppError::Inference(format!(
                "vllm-mlx chat returned {status}: {text}"
            )));
        }

        let completion: ChatCompletionResponse = resp
            .json()
            .await
            .map_err(|e| AppError::Inference(format!("failed to parse chat response: {e}")))?;

        let content = completion
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        Ok(ChatResult {
            content,
            model: completion.model.unwrap_or_else(|| self.model.clone()),
            usage: completion.usage,
        })
    }

    /// Check vllm-mlx health via /v1/models.
    pub async fn health(&self) -> Result<serde_json::Value, AppError> {
        let resp = self
            .http
            .get(format!("{}/v1/models", self.endpoint))
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| AppError::Inference(format!("vllm-mlx health check failed: {e}")))?;

        let data: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| AppError::Inference(format!("failed to parse models response: {e}")))?;

        let models: Vec<String> = data
            .get("data")
            .and_then(|d| d.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|m| m.get("id").and_then(|id| id.as_str()))
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default();

        Ok(serde_json::json!({
            "status": "ready",
            "backend": "vllm-mlx",
            "models": models,
        }))
    }
}

pub struct ChatResult {
    pub content: String,
    pub model: String,
    pub usage: Option<serde_json::Value>,
}
