//! OpenAI Provider 实现

use async_trait::async_trait;
use serde_json;

use super::types::{FinishReason, LlmError, LlmProvider, LlmRequest, LlmResponse, Usage};

pub struct OpenAiProvider {
    api_key: String,
    base_url: String,
    client: reqwest::Client,
}

impl OpenAiProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            base_url: "https://api.openai.com/v1".to_string(),
            client: crate::neotrix::http_factory::global_client().clone(),
        }
    }

    pub fn with_base_url(mut self, url: &str) -> Self {
        self.base_url = url.to_string();
        self
    }
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    async fn complete(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        let url = format!("{}/chat/completions", self.base_url);

        let mut body = serde_json::json!({
            "model": request.model,
            "messages": request.messages,
            "temperature": request.temperature,
            "max_tokens": request.max_tokens,
        });

        if let Some(image_url) = &request.image_data {
            if let Some(messages) = body["messages"].as_array_mut() {
                if let Some(last_user) = messages.iter_mut().rev().find(|m| {
                    m.get("role").and_then(|r| r.as_str()) == Some("User")
                }) {
                    let text = last_user["content"].as_str().unwrap_or("").to_string();
                    last_user["content"] = serde_json::json!([
                        {"type": "text", "text": text},
                        {"type": "image_url", "image_url": {"url": image_url}}
                    ]);
                }
            }
        }

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| LlmError::Network(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(LlmError::Server(error_text));
        }

        let resp: serde_json::Value = response.json().await.map_err(|e| LlmError::InvalidRequest(e.to_string()))?;

        let content = resp["choices"].as_array()
            .and_then(|arr| arr.first())
            .and_then(|v| v.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .unwrap_or("")
            .to_string();

        let usage = resp.get("usage").map(|u| Usage {
            prompt_tokens: u.get("prompt_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
            completion_tokens: u.get("completion_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
            total_tokens: u.get("total_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
        }).unwrap_or_default();

        let finish_reason = resp["choices"].as_array()
            .and_then(|arr| arr.first())
            .and_then(|v| v.get("finish_reason"))
            .and_then(|f| f.as_str())
            .map(|f| match f {
                "stop" => FinishReason::Stop,
                "length" => FinishReason::Length,
                "tool_calls" => FinishReason::Tool,
                "content_filter" => FinishReason::ContentFilter,
                _ => FinishReason::Unknown,
            })
            .unwrap_or(FinishReason::Unknown);

        Ok(LlmResponse {
            content,
            model: request.model.clone(),
            usage,
            finish_reason,
        })
    }

    async fn stream_complete(&self, _request: &LlmRequest) -> Result<tokio::sync::mpsc::Receiver<Result<LlmResponse, LlmError>>, LlmError> {
        let (tx, rx) = tokio::sync::mpsc::channel(1);
        let _ = tx.send(Ok(LlmResponse {
            content: "OpenAI stream (stub)".to_string(),
            model: "gpt-4".to_string(),
            usage: Usage::default(),
            finish_reason: FinishReason::Stop,
        })).await;
        Ok(rx)
    }
}
