use async_trait::async_trait;

use crate::l1_action::nt_io::nt_io_provider::common::types::{FinishReason, LlmError, LlmProvider, LlmRequest, LlmResponse, StructuredOutputConfig, ToolCallFunction, ToolCallInfo, Usage, Role};

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
fn set_proxy(&mut self, proxy_url: &str) {
        self.client = crate::neotrix::nt_io_http_factory::build_async_client_with_proxy(Some(proxy_url));
    }

    async fn complete_raw(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        let model = request.model.trim_start_matches("gemini-");
        let url = format!("{}/models/{}:generateContent?key={}", self.base_url, model, self.api_key);

        let prompt = request.messages.iter()
            .filter(|m| m.role != Role::System)
            .map(|m| m.content.clone())
            .collect::<Vec<_>>()
            .join("\n");

        let mut body = serde_json::json!({
            "contents": [{"parts": [{"text": prompt}]}],
            "generationConfig": {
                "maxOutputTokens": request.max_tokens,
            }
        });

        if let Some(temp) = request.temperature_clean() {
            body["generationConfig"]["temperature"] = serde_json::json!(temp);
        }

        if let Some(so) = &request.structured_output {
            match so {
                StructuredOutputConfig::JsonObject => {
                    body["generationConfig"]["response_mime_type"] = serde_json::json!("application/json");
                }
                StructuredOutputConfig::JsonSchema(schema) => {
                    body["generationConfig"]["response_mime_type"] = serde_json::json!("application/json");
                    body["generationConfig"]["response_schema"] = schema.clone();
                }
            }
        }

        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| LlmError::Network(e.to_string()))?;

        let status = response.status();
        let text = response.text().await
            .map_err(|e| LlmError::Network(format!("Failed to read response body: {}", e)))?;

        match status.as_u16() {
            200 => {
                let resp: serde_json::Value = serde_json::from_str(&text)
                    .map_err(|e| LlmError::InvalidRequest(e.to_string()))?;

                let parts = resp.get("candidates")
                    .and_then(|c| c.as_array())
                    .and_then(|c| c.first())
                    .and_then(|c| c.get("content"))
                    .and_then(|c| c.get("parts"))
                    .and_then(|p| p.as_array());

                let mut content = String::new();
                let mut tool_calls = None;

                if let Some(parts) = parts {
                    let mut calls = Vec::new();
                    for part in parts {
                        if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                            if !content.is_empty() { content.push('\n'); }
                            content.push_str(text);
                        } else if let Some(fc) = part.get("functionCall") {
                            let name = fc.get("name").and_then(|n| n.as_str()).unwrap_or("").to_string();
                            let args = fc.get("args").map(|a| a.to_string()).unwrap_or_else(|| "{}".to_string());
                            calls.push(ToolCallInfo {
                                id: format!("call_{}", uuid::Uuid::new_v4()),
                                call_type: "function".to_string(),
                                function: ToolCallFunction { name, arguments: args },
                            });
                        }
                    }
                    if !calls.is_empty() {
                        tool_calls = Some(calls);
                    }
                }

                let finish_reason = if tool_calls.is_some() {
                    FinishReason::Tool
                } else {
                    FinishReason::Stop
                };

                Ok(LlmResponse { content, model: request.model.clone(), usage: Usage::default(), finish_reason, tool_calls, reasoning: None })
            }
            400 => {
                let msg = serde_json::from_str::<serde_json::Value>(&text)
                    .ok().and_then(|v| v["error"]["message"].as_str().map(|s| s.to_string()))
                    .unwrap_or(text);
                if msg.contains("rate") || msg.contains("quota") { Err(LlmError::RateLimit(msg)) }
                else { Err(LlmError::InvalidRequest(msg)) }
            }
            429 | 403 => Err(LlmError::RateLimit(text)),
            500..=599 => Err(LlmError::Server(text)),
            _ => Err(LlmError::Unknown(text)),
        }
    }

    async fn stream_complete_raw(&self, request: &LlmRequest) -> Result<tokio::sync::mpsc::Receiver<Result<LlmResponse, LlmError>>, LlmError> {
        let model_name = request.model.clone();
        let model = model_name.trim_start_matches("gemini-").to_string();
        let url = format!("{}/models/{}:streamGenerateContent?alt=sse&key={}", self.base_url, model, self.api_key);
        let temperature = request.temperature_clean();
        let max_tokens = request.max_tokens;
        let prompt = request.messages.iter()
            .filter(|m| m.role != Role::System)
            .map(|m| m.content.clone())
            .collect::<Vec<_>>()
            .join("\n");

        let mut body = serde_json::json!({
            "contents": [{"parts": [{"text": prompt}]}],
            "generationConfig": {
                "maxOutputTokens": max_tokens,
            }
        });

        if let Some(temp) = temperature {
            body["generationConfig"]["temperature"] = serde_json::json!(temp);
        }

        let (tx, rx) = tokio::sync::mpsc::channel(64);

        tokio::spawn(async move {
            let client = crate::neotrix::nt_io_http_factory::global_client().clone();
            match client.post(&url)
                .header("Content-Type", "application/json")
                .json(&body)
                .send().await
            {
                Ok(response) => {
                    let status = response.status();
                    if !status.is_success() {
                        let text = response.text().await.unwrap_or_else(|_| status.canonical_reason().unwrap_or("unknown error").to_string());
                        let _ = tx.send(Err(LlmError::Server(text))).await;
                        return;
                    }
                    let full = match response.text().await {
                        Ok(t) => t,
                        Err(_) => return,
                    };
                    for line in full.lines() {
                        let line = line.trim();
                        if line.is_empty() || line == "[DONE]" { continue; }
                        if let Some(data) = line.strip_prefix("data: ") {
                            if let Ok(v) = serde_json::from_str::<serde_json::Value>(data) {
                                let parts = v.get("candidates")
                                    .and_then(|c| c.as_array())
                                    .and_then(|c| c.first())
                                    .and_then(|c| c.get("content"))
                                    .and_then(|c| c.get("parts"))
                                    .and_then(|p| p.as_array());

                                if let Some(parts) = parts {
                                    let mut content = String::new();
                                    let mut tool_calls = None;

                                    for part in parts {
                                        if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                                            if !content.is_empty() { content.push('\n'); }
                                            content.push_str(text);
                                        } else if let Some(fc) = part.get("functionCall") {
                                            let name = fc.get("name").and_then(|n| n.as_str()).unwrap_or("").to_string();
                                            let args = fc.get("args").map(|a| a.to_string()).unwrap_or_else(|| "{}".to_string());
                                            tool_calls.get_or_insert_with(Vec::new).push(ToolCallInfo {
                                                id: format!("call_{}", uuid::Uuid::new_v4()),
                                                call_type: "function".to_string(),
                                                function: ToolCallFunction { name, arguments: args },
                                            });
                                        }
                                    }

                                    let finish_reason = if tool_calls.is_some() {
                                        FinishReason::Tool
                                    } else {
                                        FinishReason::Unknown
                                    };

                                    let _ = tx.send(Ok(LlmResponse {
                                        content,
                                        model: model_name.clone(),
                                        usage: Usage::default(),
                                        finish_reason,
                                        tool_calls,
                                        reasoning: None,
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
