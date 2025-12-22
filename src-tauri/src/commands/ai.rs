use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};
use tokio::sync::{mpsc, RwLock};
use std::sync::Arc;
use std::collections::HashMap;

use crate::ai::{
    AIConfig, AIError, AIProviderType, AiProvider, ChatMessage, ModelInfo,
    OllamaProvider, OpenAIProvider, StreamChunk, ActionChip, detect_intent,
};
use crate::privacy::{self};

pub struct AIState {
    ollama: OllamaProvider,
    openai: OpenAIProvider,
    active_requests: RwLock<HashMap<String, tokio::sync::oneshot::Sender<()>>>,
}

impl Default for AIState {
    fn default() -> Self {
        Self {
            ollama: OllamaProvider::new(),
            openai: OpenAIProvider::new(),
            active_requests: RwLock::new(HashMap::new()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AIChunkPayload {
    pub content: String,
    pub done: bool,
    pub request_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AIErrorPayload {
    pub code: String,
    pub message: String,
    pub request_id: String,
}

fn error_to_code(err: &AIError) -> &'static str {
    match err {
        AIError::ConnectionFailed(_) => "CONNECTION_FAILED",
        AIError::AuthenticationFailed => "AUTH_FAILED",
        AIError::Timeout => "TIMEOUT",
        AIError::ModelNotFound(_) => "MODEL_NOT_FOUND",
        AIError::Cancelled => "CANCELLED",
        AIError::ApiError(_) => "API_ERROR",
        AIError::ParseError(_) => "PARSE_ERROR",
    }
}

#[tauri::command]
pub async fn list_local_models(
    state: State<'_, Arc<AIState>>,
) -> Result<Vec<ModelInfo>, String> {
    let config = AIConfig::default();

    state
        .ollama
        .list_models(&config)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn check_ollama_health(
    state: State<'_, Arc<AIState>>,
    base_url: Option<String>,
) -> Result<bool, String> {
    let config = AIConfig {
        base_url: base_url.unwrap_or_else(|| "http://localhost:11434".to_string()),
        ..Default::default()
    };

    state
        .ollama
        .health_check(&config)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn send_ai_request(
    app: AppHandle,
    state: State<'_, Arc<AIState>>,
    prompt: String,
    config: AIConfig,
    request_id: String,
    use_privacy_shield: bool,
) -> Result<(), String> {
    // Privacy shield processing
    let (processed_prompt, mask_result) = if use_privacy_shield
        && config.provider == AIProviderType::OpenAI
    {
        let result = privacy::mask_pii(&prompt);
        (result.masked.clone(), Some(result))
    } else {
        (prompt, None)
    };

    let messages = vec![ChatMessage::user(processed_prompt)];

    let (tx, mut rx) = mpsc::channel::<Result<StreamChunk, AIError>>(100);
    let (cancel_tx, mut cancel_rx) = tokio::sync::oneshot::channel::<()>();

    // Store cancel sender
    {
        let mut active = state.active_requests.write().await;
        active.insert(request_id.clone(), cancel_tx);
    }

    let app_clone = app.clone();
    let request_id_clone = request_id.clone();
    let mapping = mask_result.as_ref().map(|r| r.mapping.clone());

    // Spawn streaming task with cancellation support
    let config_clone = config.clone();
    let state_clone = Arc::clone(&state);
    let tx_for_cancel = tx.clone();

    tokio::spawn(async move {
        let send_future = async {
            match config_clone.provider {
                AIProviderType::Ollama => state_clone.ollama.send_stream(messages, &config_clone, tx).await,
                AIProviderType::OpenAI => state_clone.openai.send_stream(messages, &config_clone, tx).await,
            }
        };

        let result = tokio::select! {
            res = send_future => res,
            _ = &mut cancel_rx => {
                let _ = tx_for_cancel.send(Err(AIError::Cancelled)).await;
                Err(AIError::Cancelled)
            }
        };

        if let Err(e) = result {
            let _ = app_clone.emit("ai:error", AIErrorPayload {
                code: error_to_code(&e).to_string(),
                message: e.to_string(),
                request_id: request_id_clone.clone(),
            });
        }

        // Cleanup
        let mut active = state_clone.active_requests.write().await;
        active.remove(&request_id_clone);
    });

    // Process streaming chunks
    let app_emit = app.clone();
    let request_id_emit = request_id.clone();

    tokio::spawn(async move {
        let mut full_content = String::new();

        loop {
            tokio::select! {
                chunk = rx.recv() => {
                    match chunk {
                        Some(Ok(c)) => {
                            full_content.push_str(&c.content);

                            if c.done {
                                // Restore PII if masked
                                let final_content = if let Some(ref m) = mapping {
                                    privacy::restore_pii(&full_content, m)
                                } else {
                                    full_content.clone()
                                };

                                let _ = app_emit.emit("ai:chunk", AIChunkPayload {
                                    content: final_content,
                                    done: true,
                                    request_id: request_id_emit.clone(),
                                });
                                break;
                            } else {
                                let _ = app_emit.emit("ai:chunk", AIChunkPayload {
                                    content: c.content,
                                    done: false,
                                    request_id: request_id_emit.clone(),
                                });
                            }
                        }
                        Some(Err(e)) => {
                            let _ = app_emit.emit("ai:error", AIErrorPayload {
                                code: error_to_code(&e).to_string(),
                                message: e.to_string(),
                                request_id: request_id_emit.clone(),
                            });
                            break;
                        }
                        None => break,
                    }
                }
                _ = tokio::time::sleep(tokio::time::Duration::from_secs(30)) => {
                    let _ = app_emit.emit("ai:error", AIErrorPayload {
                        code: "TIMEOUT".to_string(),
                        message: "Request timeout".to_string(),
                        request_id: request_id_emit.clone(),
                    });
                    break;
                }
            }
        }
    });

    Ok(())
}

#[tauri::command]
pub async fn cancel_ai_request(
    state: State<'_, Arc<AIState>>,
    request_id: String,
) -> Result<(), String> {
    let mut active = state.active_requests.write().await;
    if let Some(tx) = active.remove(&request_id) {
        let _ = tx.send(());
    }
    Ok(())
}

#[tauri::command]
pub fn detect_content_intent(text: String) -> Vec<ActionChip> {
    detect_intent(&text)
}
