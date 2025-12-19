use async_trait::async_trait;
use tokio::sync::mpsc;

use super::types::{AIConfig, AIError, ChatMessage, ModelInfo, StreamChunk};

#[async_trait]
pub trait AiProvider: Send + Sync {
    async fn send_stream(
        &self,
        messages: Vec<ChatMessage>,
        config: &AIConfig,
        tx: mpsc::Sender<Result<StreamChunk, AIError>>,
    ) -> Result<(), AIError>;

    async fn list_models(&self, config: &AIConfig) -> Result<Vec<ModelInfo>, AIError>;

    async fn health_check(&self, config: &AIConfig) -> Result<bool, AIError>;
}
