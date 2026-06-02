//! Anthropic Provider 实现

use async_trait::async_trait;
use serde_json;

use super::types::{FinishReason, LlmError, LlmProvider, LlmRequest, LlmResponse, Usage};

pub struct AnthropicProvider {
    api_key: String,
    base_url: String,
    client: reqwest::Client,
}

impl AnthropicProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            base_url: "https://api.anthropic.com".to_string(),
            client: crate::neotrix::nt_io_http_factory::global_client().clone(),
        }
    }
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    async fn complete(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        let url = format!("{}/v1/messages", self.base_url);

        let system = request.messages.iter()
            .find(|m| m.role == super::types::Role::System)
            .map(|m| m.content.clone());

        let user_messages: Vec<_> = request.messages.iter()
            .filter(|m| m.role == super::types::Role::User)
            .collect();

        let body = serde_json::json!({
            "model": request.model,
            "messages": user_messages,
            "temperature": request.temperature,
            "max_tokens": request.max_tokens,
            "system": system,
        });

        let response = self.client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| LlmError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(LlmError::Server(response.text().await.unwrap_or_default()));
        }

        let resp: serde_json::Value = response.json().await.map_err(|e| LlmError::InvalidRequest(e.to_string()))?;

        let content = resp["content"].as_array()
            .and_then(|arr| arr.first())
            .and_then(|v| v.get("text"))
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .to_string();

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
            content: "Anthropic stream (stub)".to_string(),
            model: "claude-3".to_string(),
            usage: Usage::default(),
            finish_reason: FinishReason::Stop,
        })).await;
        Ok(rx)
    }
}
