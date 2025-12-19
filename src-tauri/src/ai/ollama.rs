use async_trait::async_trait;
use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use super::provider::AiProvider;
use super::types::{AIConfig, AIError, AIProviderType, ChatMessage, ModelInfo, StreamChunk};

pub struct OllamaProvider {
    client: Client,
}

impl OllamaProvider {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .expect("Failed to create HTTP client"),
        }
    }
}

impl Default for OllamaProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Serialize)]
struct OllamaGenerateRequest {
    model: String,
    prompt: String,
    stream: bool,
    options: OllamaOptions,
}

#[derive(Debug, Serialize)]
struct OllamaOptions {
    temperature: f32,
    num_predict: u32,
}

#[derive(Debug, Deserialize)]
struct OllamaGenerateResponse {
    response: String,
    done: bool,
}

#[derive(Debug, Deserialize)]
struct OllamaTagsResponse {
    models: Vec<OllamaModel>,
}

#[derive(Debug, Deserialize)]
struct OllamaModel {
    name: String,
    #[serde(default)]
    modified_at: String,
}

#[async_trait]
impl AiProvider for OllamaProvider {
    async fn send_stream(
        &self,
        messages: Vec<ChatMessage>,
        config: &AIConfig,
        tx: mpsc::Sender<Result<StreamChunk, AIError>>,
    ) -> Result<(), AIError> {
        let prompt = messages
            .iter()
            .map(|m| format!("{}: {}", m.role, m.content))
            .collect::<Vec<_>>()
            .join("\n\n");

        let request = OllamaGenerateRequest {
            model: config.model.clone(),
            prompt,
            stream: true,
            options: OllamaOptions {
                temperature: config.temperature,
                num_predict: config.max_tokens,
            },
        };

        let url = format!("{}/api/generate", config.base_url.trim_end_matches('/'));

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await?;

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
                        if line.is_empty() {
                            continue;
                        }

                        match serde_json::from_str::<OllamaGenerateResponse>(&line) {
                            Ok(resp) => {
                                let chunk = StreamChunk {
                                    content: resp.response,
                                    done: resp.done,
                                };
                                if tx.send(Ok(chunk)).await.is_err() {
                                    return Err(AIError::Cancelled);
                                }
                            }
                            Err(e) => {
                                log::warn!("Failed to parse Ollama response: {}", e);
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
        let url = format!("{}/api/tags", config.base_url.trim_end_matches('/'));

        let response = self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(AIError::ConnectionFailed("Failed to list models".to_string()));
        }

        let tags: OllamaTagsResponse = response.json().await.map_err(|e| {
            AIError::ParseError(format!("Failed to parse models response: {}", e))
        })?;

        Ok(tags
            .models
            .into_iter()
            .map(|m| ModelInfo {
                id: m.name.clone(),
                name: m.name,
                provider: AIProviderType::Ollama,
            })
            .collect())
    }

    async fn health_check(&self, config: &AIConfig) -> Result<bool, AIError> {
        let url = format!("{}/api/tags", config.base_url.trim_end_matches('/'));

        let response = self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await;

        Ok(response.is_ok() && response.unwrap().status().is_success())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires running Ollama
    async fn test_ollama_health_check() {
        let provider = OllamaProvider::new();
        let config = AIConfig::default();
        let result = provider.health_check(&config).await;
        println!("Ollama health check: {:?}", result);
    }

    #[tokio::test]
    #[ignore] // Requires running Ollama
    async fn test_ollama_list_models() {
        let provider = OllamaProvider::new();
        let config = AIConfig::default();
        let result = provider.list_models(&config).await;
        println!("Ollama models: {:?}", result);
    }
}
