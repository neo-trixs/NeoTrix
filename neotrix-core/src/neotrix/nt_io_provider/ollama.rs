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
            client: crate::neotrix::nt_io_http_factory::build_async_client(),
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

        let response = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| LlmError::Network(e.to_string()))?;

        let resp: serde_json::Value = response
            .json()
            .await
            .map_err(|e| LlmError::InvalidRequest(e.to_string()))?;

        let content = resp["response"].as_str().unwrap_or("").to_string();

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
        let url = format!("{}/api/generate", self.base_url);

        let body = serde_json::json!({
            "model": request.model,
            "prompt": request.messages.first().map(|m| m.content.as_str()).unwrap_or(""),
            "stream": true,
        });

        let response = self
            .client
            .post(&url)
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

                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&line) {
                        if let Some(token) = val["response"].as_str() {
                            if !val["done"].as_bool().unwrap_or(false) || !token.is_empty() {
                                let _ = tx
                                    .send(Ok(LlmResponse {
                                        content: token.to_string(),
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
