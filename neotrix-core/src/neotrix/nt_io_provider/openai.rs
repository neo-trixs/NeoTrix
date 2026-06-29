//! OpenAI Provider 实现

use async_trait::async_trait;
use base64::Engine;
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
            client: crate::neotrix::nt_io_http_factory::global_client().clone(),
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

        if let Some(image_bytes) = &request.image_data {
            let encoded = base64::engine::general_purpose::STANDARD.encode(image_bytes);
            let data_uri = format!("data:image/png;base64,{}", encoded);
            if let Some(messages) = body["messages"].as_array_mut() {
                if let Some(last_user) = messages
                    .iter_mut()
                    .rev()
                    .find(|m| m.get("role").and_then(|r| r.as_str()) == Some("User"))
                {
                    let text = last_user["content"].as_str().unwrap_or("").to_string();
                    last_user["content"] = serde_json::json!([
                        {"type": "text", "text": text},
                        {"type": "image_url", "image_url": {"url": data_uri}}
                    ]);
                }
            }
        }

        let response = self
            .client
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

        let resp: serde_json::Value = response
            .json()
            .await
            .map_err(|e| LlmError::InvalidRequest(e.to_string()))?;

        let content = resp["choices"]
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|v| v.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .unwrap_or("")
            .to_string();

        let usage = resp
            .get("usage")
            .map(|u| Usage {
                prompt_tokens: u.get("prompt_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                completion_tokens: u
                    .get("completion_tokens")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u32,
                total_tokens: u.get("total_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
            })
            .unwrap_or_default();

        let finish_reason = resp["choices"]
            .as_array()
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

    async fn stream_complete(
        &self,
        request: &LlmRequest,
    ) -> Result<tokio::sync::mpsc::Receiver<Result<LlmResponse, LlmError>>, LlmError> {
        let url = format!("{}/chat/completions", self.base_url);

        let mut body = serde_json::json!({
            "model": request.model,
            "messages": request.messages,
            "temperature": request.temperature,
            "max_tokens": request.max_tokens,
            "stream": true,
        });

        if let Some(image_bytes) = &request.image_data {
            let encoded = base64::engine::general_purpose::STANDARD.encode(image_bytes);
            let data_uri = format!("data:image/png;base64,{}", encoded);
            if let Some(messages) = body["messages"].as_array_mut() {
                if let Some(last_user) = messages
                    .iter_mut()
                    .rev()
                    .find(|m| m.get("role").and_then(|r| r.as_str()) == Some("User"))
                {
                    let text = last_user["content"].as_str().unwrap_or("").to_string();
                    last_user["content"] = serde_json::json!([
                        {"type": "text", "text": text},
                        {"type": "image_url", "image_url": {"url": data_uri}}
                    ]);
                }
            }
        }

        let response = self
            .client
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

                    if line.is_empty() || line == "data: [DONE]" {
                        continue;
                    }

                    if let Some(data) = line.strip_prefix("data: ") {
                        if let Ok(val) = serde_json::from_str::<serde_json::Value>(data) {
                            if let Some(delta) = val["choices"]
                                .as_array()
                                .and_then(|arr| arr.first())
                                .and_then(|v| v.get("delta"))
                                .and_then(|d| d.get("content"))
                                .and_then(|c| c.as_str())
                            {
                                let _ = tx
                                    .send(Ok(LlmResponse {
                                        content: delta.to_string(),
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
