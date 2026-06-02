//! Gemini Provider 实现

use async_trait::async_trait;
use serde_json;

use super::types::{FinishReason, LlmError, LlmProvider, LlmRequest, LlmResponse, Usage};

pub struct GeminiProvider {
    api_key: String,
    base_url: String,
    client: reqwest::Client,
}

impl GeminiProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            base_url: "https://generativelanguage.googleapis.com/v1beta".to_string(),
            client: crate::neotrix::nt_io_http_factory::global_client().clone(),
        }
    }

    pub fn with_base_url(mut self, url: &str) -> Self {
        self.base_url = url.to_string();
        self
    }
}

#[async_trait]
impl LlmProvider for GeminiProvider {
    async fn complete(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        let model = request.model.trim_start_matches("gemini-");
        let url = format!("{}/models/{}:generateContent?key={}", self.base_url, model, self.api_key);

        let prompt = request.messages.iter()
            .filter(|m| m.role != super::types::Role::System)
            .map(|m| m.content.clone())
            .collect::<Vec<_>>()
            .join("\n");

        let body = serde_json::json!({
            "contents": [{
                "parts": [{"text": prompt}]
            }],
            "generationConfig": {
                "temperature": request.temperature,
                "maxOutputTokens": request.max_tokens,
            }
        });

        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| LlmError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(LlmError::Server(response.text().await.unwrap_or_default()));
        }

        let resp: serde_json::Value = response.json().await.map_err(|e| LlmError::InvalidRequest(e.to_string()))?;

        let mut content = String::new();
        if let Some(candidates) = resp["candidates"].as_array() {
            if let Some(candidate) = candidates.first() {
                if let Some(c) = candidate.get("content") {
                    if let Some(parts_val) = c.get("parts") {
                        if let Some(parts) = parts_val.as_array() {
                            if let Some(part) = parts.first() {
                                if let Some(text_val) = part.get("text") {
                                    if let Some(s) = text_val.as_str() {
                                        content = s.to_string();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

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
            content: "Gemini stream (stub)".to_string(),
            model: "gemini-pro".to_string(),
            usage: Usage::default(),
            finish_reason: FinishReason::Stop,
        })).await;
        Ok(rx)
    }
}
