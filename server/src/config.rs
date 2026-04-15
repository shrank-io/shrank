use std::path::PathBuf;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub server: ServerConfig,
    pub auth: AuthConfig,
    #[serde(default)]
    pub inference: InferenceConfig,
    #[serde(default)]
    pub embeddings: EmbeddingsConfig,
    #[serde(default)]
    pub images: ImageConfig,
    #[serde(default)]
    pub processing: ProcessingConfig,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default)]
    pub data_dir: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AuthConfig {
    pub api_key: String,
}

#[derive(Debug, Deserialize)]
pub struct InferenceConfig {
    #[serde(default = "default_inference_backend")]
    pub backend: String,
    #[serde(default = "default_inference_endpoint")]
    pub endpoint: String,
    #[serde(default = "default_inference_model")]
    pub model: String,
}

#[derive(Debug, Deserialize)]
pub struct EmbeddingsConfig {
    #[serde(default = "default_embeddings_backend")]
    pub backend: String,
    #[serde(default = "default_embeddings_model")]
    pub model: String,
    #[serde(default = "default_embeddings_endpoint")]
    pub endpoint: String,
    #[serde(default = "default_dimensions")]
    pub dimensions: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ImageConfig {
    #[serde(default = "default_thumbnail_width")]
    pub thumbnail_width: u32,
    #[serde(default = "default_thumbnail_format")]
    pub thumbnail_format: String,
    #[serde(default = "default_thumbnail_quality")]
    pub thumbnail_quality: u8,
}

#[derive(Debug, Deserialize)]
pub struct ProcessingConfig {
    #[serde(default = "default_workers")]
    pub workers: usize,
    #[serde(default)]
    pub auto_reprocess: bool,
    #[serde(default = "default_confidence_threshold")]
    pub confidence_threshold: f64,
}

// Defaults
fn default_host() -> String { "0.0.0.0".into() }
fn default_port() -> u16 { 3420 }
fn default_inference_backend() -> String { "vllm-mlx".into() }
fn default_inference_endpoint() -> String { "http://127.0.0.1:8000".into() }
fn default_inference_model() -> String { "mlx-community/gemma-4-26b-a4b-it-4bit".into() }
fn default_embeddings_backend() -> String { "vllm-mlx".into() }
fn default_embeddings_model() -> String { "mlx-community/all-MiniLM-L6-v2-4bit".into() }
fn default_embeddings_endpoint() -> String { "http://127.0.0.1:8000".into() }
fn default_dimensions() -> usize { 384 }
fn default_thumbnail_width() -> u32 { 400 }
fn default_thumbnail_format() -> String { "webp".into() }
fn default_thumbnail_quality() -> u8 { 80 }
fn default_workers() -> usize { 1 }
fn default_confidence_threshold() -> f64 { 0.7 }

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            data_dir: None,
        }
    }
}

impl Default for InferenceConfig {
    fn default() -> Self {
        Self {
            backend: default_inference_backend(),
            endpoint: default_inference_endpoint(),
            model: default_inference_model(),
        }
    }
}

impl Default for EmbeddingsConfig {
    fn default() -> Self {
        Self {
            backend: default_embeddings_backend(),
            model: default_embeddings_model(),
            endpoint: default_embeddings_endpoint(),
            dimensions: default_dimensions(),
        }
    }
}

impl Default for ImageConfig {
    fn default() -> Self {
        Self {
            thumbnail_width: default_thumbnail_width(),
            thumbnail_format: default_thumbnail_format(),
            thumbnail_quality: default_thumbnail_quality(),
        }
    }
}

impl Default for ProcessingConfig {
    fn default() -> Self {
        Self {
            workers: default_workers(),
            auto_reprocess: false,
            confidence_threshold: default_confidence_threshold(),
        }
    }
}

impl Config {
    pub fn data_dir(&self) -> PathBuf {
        if let Some(ref dir) = self.server.data_dir {
            let expanded = shellexpand(dir);
            PathBuf::from(expanded)
        } else {
            dirs::data_local_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("shrank")
        }
    }
}

fn shellexpand(path: &str) -> String {
    if path.starts_with("~/") {
        if let Some(home) = dirs::home_dir() {
            return format!("{}{}", home.display(), &path[1..]);
        }
    }
    path.to_string()
}

pub fn load() -> Result<Config, Box<dyn std::error::Error>> {
    let config_path = config_path();

    if !config_path.exists() {
        return Err(format!(
            "Config file not found at {}. Create it with at least:\n\n\
             [auth]\n\
             api_key = \"your-secret-key\"\n",
            config_path.display()
        )
        .into());
    }

    let contents = std::fs::read_to_string(&config_path)?;
    let config: Config = toml::from_str(&contents)?;

    tracing::info!(path = %config_path.display(), "loaded config");
    Ok(config)
}

fn config_path() -> PathBuf {
    if let Ok(path) = std::env::var("SHRANK_CONFIG") {
        return PathBuf::from(path);
    }

    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("shrank")
        .join("config.toml")
}
