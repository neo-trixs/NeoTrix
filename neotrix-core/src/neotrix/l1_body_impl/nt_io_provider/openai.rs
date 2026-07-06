use async_trait::async_trait;

use super::types::{FinishReason, LlmError, LlmProvider, LlmRequest, LlmResponse, StructuredOutputConfig, Usage};

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

    fn build_body(&self, request: &LlmRequest, stream: bool) -> serde_json::Value {
        let mut body = serde_json::json!({
            "model": request.model,
            "messages": request.messages,
            "max_tokens": request.max_tokens,
            "stream": stream,
        });

        if let Some(temp) = request.temperature {
            body["temperature"] = serde_json::json!(temp);
        }

        if let Some(so) = &request.structured_output {
            match so {
                StructuredOutputConfig::JsonObject => {
                    body["response_format"] = serde_json::json!({"type": "json_object"});
                }
                StructuredOutputConfig::JsonSchema(schema) => {
                    body["response_format"] = serde_json::json!({
                        "type": "json_schema",
                        "json_schema": {
                            "name": "response",
                            "schema": schema,
                            "strict": true,
                        }
                    });
                }
            }
        }

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
        body
    }
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    async fn complete(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        let url = format!("{}/chat/completions", self.base_url);
        let body = self.build_body(request, false);

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| LlmError::Network(e.to_string()))?;

        let status = response.status();
        let text = response.text().await.unwrap_or_default();

        match status.as_u16() {
            200 => {
                let resp: serde_json::Value = serde_json::from_str(&text)
                    .map_err(|e| LlmError::InvalidRequest(e.to_string()))?;
                let content = resp["choices"][0]["message"]["content"].as_str().unwrap_or("").to_string();
                let usage = resp.get("usage").map(|u| Usage {
                    prompt_tokens: u["prompt_tokens"].as_u64().unwrap_or(0) as u32,
                    completion_tokens: u["completion_tokens"].as_u64().unwrap_or(0) as u32,
                    total_tokens: u["total_tokens"].as_u64().unwrap_or(0) as u32,
                }).unwrap_or_default();
                let finish = match resp["choices"][0]["finish_reason"].as_str() {
                    Some("stop") => FinishReason::Stop,
                    Some("length") => FinishReason::Length,
                    Some("tool_calls") => FinishReason::Tool,
                    Some("content_filter") => FinishReason::ContentFilter,
                    _ => FinishReason::Unknown,
                };
                Ok(LlmResponse { content, model: request.model.clone(), usage, finish_reason: finish })
            }
            401 => Err(LlmError::Authentication(text)),
            429 => Err(LlmError::RateLimit(text)),
            400 => Err(LlmError::InvalidRequest(text)),
            500..=599 => Err(LlmError::Server(text)),
            _ => Err(LlmError::Unknown(text)),
        }
    }

    async fn stream_complete(&self, request: &LlmRequest) -> Result<tokio::sync::mpsc::Receiver<Result<LlmResponse, LlmError>>, LlmError> {
        let url = format!("{}/chat/completions", self.base_url);
        let body = self.build_body(request, true);
        let api_key = self.api_key.clone();
        let (tx, rx) = tokio::sync::mpsc::channel(64);

        tokio::spawn(async move {
            let client = crate::neotrix::nt_io_http_factory::global_client().clone();
            match client.post(&url)
                .header("Authorization", format!("Bearer {}", api_key))
                .json(&body)
                .send().await
            {
                Ok(response) => {
                    let status = response.status();
                    if !status.is_success() {
                        let text = response.text().await.unwrap_or_default();
                        let err = match status.as_u16() {
                            401 => LlmError::Authentication(text),
                            429 => LlmError::RateLimit(text),
                            _ => LlmError::Server(text),
                        };
                        let _ = tx.send(Err(err)).await;
                        return;
                    }
                    let full = response.text().await.unwrap_or_default();
                    for line in full.lines() {
                        let line = line.trim();
                        if line.is_empty() || line == "data: [DONE]" { continue; }
                        if let Some(data) = line.strip_prefix("data: ") {
                            if let Ok(v) = serde_json::from_str::<serde_json::Value>(data) {
                                if let Some(delta) = v["choices"][0]["delta"]["content"].as_str() {
                                    let _ = tx.send(Ok(LlmResponse {
                                        content: delta.to_string(),
                                        model: v["model"].as_str().unwrap_or("").to_string(),
                                        usage: Usage::default(),
                                        finish_reason: FinishReason::Unknown,
                                    })).await;
                                }
                            }
                        }
                    }
                }
                Err(e) => { let _ = tx.send(Err(LlmError::Network(format!("{}", e)))).await; }
            }
        });

        Ok(rx)
    }
}
