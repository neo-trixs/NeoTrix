//! Ollama Provider 实现 (本地 LLM)

use async_trait::async_trait;
use serde_json;

use super::types::{FinishReason, LlmError, LlmProvider, LlmRequest, LlmResponse, Usage};

pub struct OllamaProvider {
    base_url: String,
    client: reqwest::Client,
}

impl OllamaProvider {
    pub fn new() -> Self {
        Self {
            base_url: "http://localhost:11434".to_string(),
            client: crate::neotrix::http_factory::build_async_client(),
        }
    }

    pub fn with_base_url(mut self, url: &str) -> Self {
        self.base_url = url.to_string();
        self
    }

}

impl Default for OllamaProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LlmProvider for OllamaProvider {
    async fn complete(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        let url = format!("{}/api/generate", self.base_url);

        let body = serde_json::json!({
            "model": request.model,
            "prompt": request.messages.first().map(|m| m.content.as_str()).unwrap_or(""),
            "stream": false,
        });

        let response = self.client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| LlmError::Network(e.to_string()))?;

        let resp: serde_json::Value = response.json().await.map_err(|e| LlmError::InvalidRequest(e.to_string()))?;

        let content = resp["response"].as_str().unwrap_or("").to_string();

        Ok(LlmResponse {
            content,
            model: request.model.clone(),
            usage: Usage::default(),
            finish_reason: FinishReason::Stop,
        })
    }

    async fn stream_complete(&self, _request: &LlmRequest) -> Result<tokio::sync::mpsc::Receiver<Result<LlmResponse, LlmError>>, LlmError> {
        let (tx, rx) = tokio::sync::mpsc::channel(1);
        let _ = tx.send(Ok(LlmResponse {
            content: "Ollama stream (stub)".to_string(),
            model: "llama2".to_string(),
            usage: Usage::default(),
            finish_reason: FinishReason::Stop,
        })).await;
        Ok(rx)
    }
}
