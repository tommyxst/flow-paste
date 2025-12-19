use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AIConfig {
    pub provider: AIProviderType,
    pub base_url: String,
    pub model: String,
    pub api_key: Option<String>,
    pub max_tokens: u32,
    pub temperature: f32,
}

impl Default for AIConfig {
    fn default() -> Self {
        Self {
            provider: AIProviderType::Ollama,
            base_url: "http://localhost:11434".to_string(),
            model: "llama3.2".to_string(),
            api_key: None,
            max_tokens: 2048,
            temperature: 0.7,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AIProviderType {
    OpenAI,
    Ollama,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub provider: AIProviderType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamChunk {
    pub content: String,
    pub done: bool,
}

#[derive(Debug, Error)]
pub enum AIError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Authentication failed: invalid API key")]
    AuthenticationFailed,

    #[error("Request timeout")]
    Timeout,

    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("Request cancelled")]
    Cancelled,

    #[error("API error: {0}")]
    ApiError(String),

    #[error("Parse error: {0}")]
    ParseError(String),
}

impl From<reqwest::Error> for AIError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            AIError::Timeout
        } else if err.is_connect() {
            AIError::ConnectionFailed(err.to_string())
        } else {
            AIError::ApiError(err.to_string())
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

impl ChatMessage {
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
        }
    }

    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: content.into(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into(),
        }
    }
}
