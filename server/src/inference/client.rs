use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::config::{EmbeddingsConfig, InferenceConfig};
use crate::AppError;

pub struct InferenceClient {
    http: reqwest::Client,
    inference_endpoint: String,
    embeddings_endpoint: String,
    #[allow(dead_code)]
    embeddings_model: String,
}

#[derive(Debug, Serialize)]
struct ExtractRequest {
    image_base64: String,
    existing_tags: Vec<String>,
    existing_senders: Vec<String>,
}

#[derive(Debug, Serialize)]
struct EmbedRequest {
    text: String,
}

#[derive(Debug, Deserialize)]
struct EmbedResponse {
    embedding: Vec<f32>,
    #[allow(dead_code)]
    model: Option<String>,
    #[allow(dead_code)]
    dimensions: Option<usize>,
}

impl InferenceClient {
    pub fn new(inference: &InferenceConfig, embeddings: &EmbeddingsConfig) -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(180)) // LLM inference can be slow
            .connect_timeout(Duration::from_secs(10))
            .build()
            .expect("failed to create HTTP client");

        Self {
            http,
            inference_endpoint: inference.endpoint.clone(),
            embeddings_endpoint: embeddings.endpoint.clone(),
            embeddings_model: embeddings.model.clone(),
        }
    }

    /// Call the sidecar's /extract endpoint with a document image.
    pub async fn extract(
        &self,
        image_bytes: &[u8],
        existing_tags: &[String],
        existing_senders: &[String],
    ) -> Result<serde_json::Value, AppError> {
        use base64::Engine;
        let image_base64 = base64::engine::general_purpose::STANDARD.encode(image_bytes);

        let body = ExtractRequest {
            image_base64,
            existing_tags: existing_tags.to_vec(),
            existing_senders: existing_senders.to_vec(),
        };

        let resp = self
            .http
            .post(format!("{}/extract", self.inference_endpoint))
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::Inference(format!("extract request failed: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(AppError::Inference(format!(
                "sidecar returned {status}: {text}"
            )));
        }

        resp.json::<serde_json::Value>()
            .await
            .map_err(|e| AppError::Inference(format!("failed to parse extract response: {e}")))
    }

    /// Call the sidecar's /embed endpoint to get a text embedding.
    pub async fn embed(&self, text: &str) -> Result<Vec<f32>, AppError> {
        let body = EmbedRequest {
            text: text.to_string(),
        };

        let resp = self
            .http
            .post(format!("{}/embed", self.embeddings_endpoint))
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::Inference(format!("embed request failed: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(AppError::Inference(format!(
                "embed sidecar returned {status}: {text}"
            )));
        }

        let result: EmbedResponse = resp
            .json()
            .await
            .map_err(|e| AppError::Inference(format!("failed to parse embed response: {e}")))?;

        Ok(result.embedding)
    }

    /// Check sidecar health.
    pub async fn health(&self) -> Result<serde_json::Value, AppError> {
        let resp = self
            .http
            .get(format!("{}/health", self.inference_endpoint))
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| AppError::Inference(format!("health check failed: {e}")))?;

        resp.json::<serde_json::Value>()
            .await
            .map_err(|e| AppError::Inference(format!("failed to parse health response: {e}")))
    }
}
