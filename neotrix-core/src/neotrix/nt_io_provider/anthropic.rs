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

        let system = request
            .messages
            .iter()
            .find(|m| m.role == super::types::Role::System)
            .map(|m| m.content.clone());

        let user_messages: Vec<_> = request
            .messages
            .iter()
            .filter(|m| m.role == super::types::Role::User)
            .collect();

        let body = serde_json::json!({
            "model": request.model,
            "messages": user_messages,
            "temperature": request.temperature,
            "max_tokens": request.max_tokens,
            "system": system,
        });

        let response = self
            .client
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

        let resp: serde_json::Value = response
            .json()
            .await
            .map_err(|e| LlmError::InvalidRequest(e.to_string()))?;

        let content = resp["content"]
            .as_array()
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

    async fn stream_complete(
        &self,
        request: &LlmRequest,
    ) -> Result<tokio::sync::mpsc::Receiver<Result<LlmResponse, LlmError>>, LlmError> {
        let url = format!("{}/v1/messages", self.base_url);

        let system = request
            .messages
            .iter()
            .find(|m| m.role == super::types::Role::System)
            .map(|m| m.content.clone());

        let user_messages: Vec<_> = request
            .messages
            .iter()
            .filter(|m| m.role == super::types::Role::User)
            .collect();

        let body = serde_json::json!({
            "model": request.model,
            "messages": user_messages,
            "temperature": request.temperature,
            "max_tokens": request.max_tokens,
            "system": system,
            "stream": true,
        });

        let response = self
            .client
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

        let (tx, rx) = tokio::sync::mpsc::channel(64);
        let model_name = request.model.clone();

        tokio::spawn(async move {
            use futures_util::StreamExt;

            let mut stream = response.bytes_stream();
            let mut buf = String::new();
            let mut current_event = String::new();

            while let Some(chunk_result) = stream.next().await {
                let chunk = match chunk_result {
                    Ok(c) => c,
                    Err(e) => {
                        let _ = tx.send(Err(LlmError::Network(e.to_string()))).await;
                        break;
                    }
                };
                buf.push_str(&String::from_utf8_lossy(&chunk));

                while let Some(line_end) = buf.find('\n') {
                    let line = buf[..line_end].trim().to_string();
                    buf = buf[line_end + 1..].to_string();

                    if line.is_empty() {
                        continue;
                    }

                    if let Some(event_name) = line.strip_prefix("event: ") {
                        current_event = event_name.to_string();
                        continue;
                    }

                    if let Some(data) = line.strip_prefix("data: ") {
                        if current_event == "content_block_delta" {
                            if let Ok(val) = serde_json::from_str::<serde_json::Value>(data) {
                                if let Some(text) = val["delta"]["text"].as_str() {
                                    let _ = tx
                                        .send(Ok(LlmResponse {
                                            content: text.to_string(),
                                            model: model_name.clone(),
                                            usage: Usage::default(),
                                            finish_reason: FinishReason::Stop,
                                        }))
                                        .await;
                                }
                            }
                        }
                    }
                }
            }
        });

        Ok(rx)
    }
}
