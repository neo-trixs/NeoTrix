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
        let url = format!(
            "{}/models/{}:generateContent?key={}",
            self.base_url, model, self.api_key
        );

        let prompt = request
            .messages
            .iter()
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

        let response = self
            .client
            .post(&url)
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

    async fn stream_complete(
        &self,
        request: &LlmRequest,
    ) -> Result<tokio::sync::mpsc::Receiver<Result<LlmResponse, LlmError>>, LlmError> {
        let model = request.model.trim_start_matches("gemini-");
        let url = format!(
            "{}/models/{}:streamGenerateContent?alt=sse&key={}",
            self.base_url, model, self.api_key
        );

        let prompt = request
            .messages
            .iter()
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

        let response = self
            .client
            .post(&url)
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

                    if let Some(data) = line.strip_prefix("data: ") {
                        if let Ok(val) = serde_json::from_str::<serde_json::Value>(data) {
                            if let Some(text) = val["candidates"]
                                .as_array()
                                .and_then(|arr| arr.first())
                                .and_then(|c| c.get("content"))
                                .and_then(|c| c.get("parts"))
                                .and_then(|p| p.as_array())
                                .and_then(|parts| parts.first())
                                .and_then(|p| p.get("text"))
                                .and_then(|t| t.as_str())
                            {
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
        });

        Ok(rx)
    }
}
