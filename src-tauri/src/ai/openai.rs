use async_trait::async_trait;
use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use super::provider::AiProvider;
use super::types::{AIConfig, AIError, AIProviderType, ChatMessage, ModelInfo, StreamChunk};

pub struct OpenAIProvider {
    client: Client,
}

impl OpenAIProvider {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .expect("Failed to create HTTP client"),
        }
    }
}

impl Default for OpenAIProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessageRequest>,
    stream: bool,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Debug, Serialize)]
struct ChatMessageRequest {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionChunk {
    choices: Vec<ChunkChoice>,
}

#[derive(Debug, Deserialize)]
struct ChunkChoice {
    delta: DeltaContent,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DeltaContent {
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ModelsResponse {
    data: Vec<ModelData>,
}

#[derive(Debug, Deserialize)]
struct ModelData {
    id: String,
}

#[async_trait]
impl AiProvider for OpenAIProvider {
    async fn send_stream(
        &self,
        messages: Vec<ChatMessage>,
        config: &AIConfig,
        tx: mpsc::Sender<Result<StreamChunk, AIError>>,
    ) -> Result<(), AIError> {
        let api_key = config
            .api_key
            .as_ref()
            .ok_or(AIError::AuthenticationFailed)?;

        let request = ChatCompletionRequest {
            model: config.model.clone(),
            messages: messages
                .into_iter()
                .map(|m| ChatMessageRequest {
                    role: m.role,
                    content: m.content,
                })
                .collect(),
            stream: true,
            max_tokens: config.max_tokens,
            temperature: config.temperature,
        };

        let url = format!(
            "{}/chat/completions",
            config.base_url.trim_end_matches('/')
        );

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(AIError::AuthenticationFailed);
        }

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AIError::ApiError(format!("Status {}: {}", status, body)));
        }

        let mut stream = response.bytes_stream();
        let mut buffer = String::new();

        while let Some(chunk_result) = stream.next().await {
            match chunk_result {
                Ok(bytes) => {
                    buffer.push_str(&String::from_utf8_lossy(&bytes));

                    while let Some(idx) = buffer.find('\n') {
                        let mut line: String = buffer.drain(..=idx).collect();
                        while line.ends_with(['\r', '\n']) {
                            line.pop();
                        }
                        if line.is_empty() || line == "data: [DONE]" {
                            continue;
                        }

                        let data = line.strip_prefix("data: ").unwrap_or(&line);
                        if data.is_empty() {
                            continue;
                        }

                        match serde_json::from_str::<ChatCompletionChunk>(data) {
                            Ok(chunk) => {
                                for choice in chunk.choices {
                                    let content = choice.delta.content.unwrap_or_default();
                                    let done = choice.finish_reason.is_some();

                                    if !content.is_empty() || done {
                                        let stream_chunk = StreamChunk { content, done };
                                        if tx.send(Ok(stream_chunk)).await.is_err() {
                                            return Err(AIError::Cancelled);
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                log::warn!("Failed to parse OpenAI chunk: {} - {}", e, data);
                            }
                        }
                    }
                }
                Err(e) => {
                    let _ = tx.send(Err(AIError::from(e))).await;
                    break;
                }
            }
        }

        Ok(())
    }

    async fn list_models(&self, config: &AIConfig) -> Result<Vec<ModelInfo>, AIError> {
        let api_key = config
            .api_key
            .as_ref()
            .ok_or(AIError::AuthenticationFailed)?;

        let url = format!("{}/models", config.base_url.trim_end_matches('/'));

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await?;

        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(AIError::AuthenticationFailed);
        }

        if !response.status().is_success() {
            return Err(AIError::ApiError("Failed to list models".to_string()));
        }

        let models: ModelsResponse = response.json().await.map_err(|e| {
            AIError::ParseError(format!("Failed to parse models response: {}", e))
        })?;

        Ok(models
            .data
            .into_iter()
            .filter(|m| m.id.starts_with("gpt-") || m.id.contains("turbo"))
            .map(|m| ModelInfo {
                id: m.id.clone(),
                name: m.id,
                provider: AIProviderType::OpenAI,
            })
            .collect())
    }

    async fn health_check(&self, config: &AIConfig) -> Result<bool, AIError> {
        if config.api_key.is_none() {
            return Ok(false);
        }

        match self.list_models(config).await {
            Ok(_) => Ok(true),
            Err(AIError::AuthenticationFailed) => Ok(false),
            Err(_) => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires API key
    async fn test_openai_health_check() {
        let provider = OpenAIProvider::new();
        let config = AIConfig {
            provider: AIProviderType::OpenAI,
            base_url: "https://api.openai.com/v1".to_string(),
            model: "gpt-4o-mini".to_string(),
            api_key: std::env::var("OPENAI_API_KEY").ok(),
            max_tokens: 2048,
            temperature: 0.7,
        };
        let result = provider.health_check(&config).await;
        println!("OpenAI health check: {:?}", result);
    }
}
