use async_trait::async_trait;

use super::types::{FinishReason, LlmError, LlmProvider, LlmRequest, LlmResponse, StructuredOutputConfig, Usage, Role};

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
    fn set_proxy(&mut self, proxy_url: &str) {
        self.client = crate::neotrix::nt_io_http_factory::build_async_client_with_proxy(Some(proxy_url));
    }

    async fn complete(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        let url = format!("{}/v1/messages", self.base_url);

        let system = request.messages.iter()
            .find(|m| m.role == Role::System)
            .map(|m| m.content.clone());

        let messages: Vec<serde_json::Value> = request.messages.iter()
            .filter(|m| m.role != Role::System)
            .map(|m| serde_json::json!({
                "role": match m.role {
                    Role::User => "user",
                    Role::Assistant => "assistant",
                    Role::Tool => "tool",
                    _ => "user",
                },
                "content": m.content,
            }))
            .collect();

        let mut body = serde_json::json!({
            "model": request.model,
            "messages": messages,
            "max_tokens": request.max_tokens,
        });

        if let Some(temp) = request.temperature {
            body["temperature"] = serde_json::json!(temp);
        }

        if let Some(budget) = request.thinking_budget {
            if budget > 0 {
                body["thinking"] = serde_json::json!({
                    "type": "enabled",
                    "budget_tokens": budget,
                });
            } else {
                body["thinking"] = serde_json::json!({"type": "disabled"});
            }
        }

        if let Some(s) = system {
            body["system"] = serde_json::json!(s);
        }

        if let Some(so) = &request.structured_output {
            match so {
                StructuredOutputConfig::JsonObject => {
                    body["output_config"] = serde_json::json!({
                        "format": {"type": "json_object"}
                    });
                }
                StructuredOutputConfig::JsonSchema(schema) => {
                    body["output_config"] = serde_json::json!({
                        "format": {
                            "type": "json_schema",
                            "json_schema": {"name": "response", "schema": schema}
                        }
                    });
                }
            }
        }

        // Apply provider-specific overrides
        for (key, val) in &request.provider_params {
            body[key] = val.clone();
        }

        let response = self.client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
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
                let content = resp["content"].as_array()
                    .and_then(|arr| arr.first())
                    .and_then(|v| v["text"].as_str())
                    .unwrap_or("")
                    .to_string();
                let usage = Usage {
                    prompt_tokens: resp["usage"]["input_tokens"].as_u64().unwrap_or(0) as u32,
                    completion_tokens: resp["usage"]["output_tokens"].as_u64().unwrap_or(0) as u32,
                    total_tokens: (resp["usage"]["input_tokens"].as_u64().unwrap_or(0) + resp["usage"]["output_tokens"].as_u64().unwrap_or(0)) as u32,
                };
                Ok(LlmResponse { content, model: request.model.clone(), usage, finish_reason: FinishReason::Stop })
            }
            401 => Err(LlmError::Authentication(text)),
            429 => Err(LlmError::RateLimit(text)),
            400 => Err(LlmError::InvalidRequest(text)),
            500..=599 => Err(LlmError::Server(text)),
            _ => Err(LlmError::Unknown(text)),
        }
    }

    async fn stream_complete(&self, request: &LlmRequest) -> Result<tokio::sync::mpsc::Receiver<Result<LlmResponse, LlmError>>, LlmError> {
        let url = format!("{}/v1/messages", self.base_url);
        let api_key = self.api_key.clone();

        let model_name = request.model.clone();
        let max_tokens = request.max_tokens;
        let thinking_budget = request.thinking_budget;
        let temperature = request.temperature;
        let provider_params = request.provider_params.clone();

        let system = request.messages.iter()
            .find(|m| m.role == Role::System)
            .map(|m| m.content.clone());

        let messages: Vec<serde_json::Value> = request.messages.iter()
            .filter(|m| m.role != Role::System)
            .map(|m| serde_json::json!({
                "role": match m.role {
                    Role::User => "user",
                    Role::Assistant => "assistant",
                    Role::Tool => "tool",
                    _ => "user",
                },
                "content": m.content,
            }))
            .collect();

        let mut body = serde_json::json!({
            "model": model_name,
            "messages": messages,
            "max_tokens": max_tokens,
            "stream": true,
        });

        if let Some(temp) = temperature {
            body["temperature"] = serde_json::json!(temp);
        }

        if let Some(budget) = thinking_budget {
            if budget > 0 {
                body["thinking"] = serde_json::json!({
                    "type": "enabled",
                    "budget_tokens": budget,
                });
            } else {
                body["thinking"] = serde_json::json!({"type": "disabled"});
            }
        }

        for (key, val) in &provider_params {
            body[key] = val.clone();
        }

        if let Some(s) = system { body["system"] = serde_json::json!(s); }

        let (tx, rx) = tokio::sync::mpsc::channel(64);

        tokio::spawn(async move {
            let client = crate::neotrix::nt_io_http_factory::global_client().clone();
            match client.post(&url)
                .header("x-api-key", &api_key)
                .header("anthropic-version", "2023-06-01")
                .json(&body)
                .send().await
            {
                Ok(response) => {
                    let status = response.status();
                    if !status.is_success() {
                        let text = response.text().await.unwrap_or_default();
                        let _ = tx.send(Err(match status.as_u16() {
                            401 => LlmError::Authentication(text),
                            429 => LlmError::RateLimit(text),
                            _ => LlmError::Server(text),
                        })).await;
                        return;
                    }
                    let full = response.text().await.unwrap_or_default();
                    for line in full.lines() {
                        let line = line.trim();
                        if line.is_empty() { continue; }
                        if let Some(data) = line.strip_prefix("data: ") {
                            if let Ok(v) = serde_json::from_str::<serde_json::Value>(data) {
                                match v["type"].as_str() {
                                    Some("content_block_delta") => {
                                        if let Some(text) = v["delta"]["text"].as_str() {
                                            let _ = tx.send(Ok(LlmResponse {
                                                content: text.to_string(),
                                                model: model_name.clone(),
                                                usage: Usage::default(),
                                                finish_reason: FinishReason::Unknown,
                                            })).await;
                                        }
                                    }
                                    Some("message_stop") => {
                                        let _ = tx.send(Ok(LlmResponse {
                                            content: String::new(),
                                                model: model_name.clone(),
                                            usage: Usage::default(),
                                            finish_reason: FinishReason::Stop,
                                        })).await;
                                    }
                                    _ => {}
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
