use async_trait::async_trait;

use crate::l1_action::nt_io::nt_io_provider::health::context_budget::estimate_tokens;
use crate::l1_action::nt_io::nt_io_provider::common::types::{FinishReason, LlmError, LlmProvider, LlmRequest, LlmResponse, Message, StructuredOutputConfig, Usage, Role};

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

/// 计算稳定前缀边界消息索引 (在非 System 消息序列上): 从头累积
/// `estimate_tokens` (含每消息 ~4 token 协议开销), 累计越过 `prefix_tokens`
/// 时返回该消息索引 — 在此消息上打 Anthropic `cache_control` 断点, 使
/// ReAct 每轮重发时该稳定前缀命中 provider 缓存 (对标 E1/E10 prefix caching)。
fn prefix_boundary_index(messages: &[&Message], prefix_tokens: usize) -> Option<usize> {
    let mut acc = 0usize;
    for (i, m) in messages.iter().enumerate() {
        acc += estimate_tokens(&m.content) + 4;
        if acc >= prefix_tokens {
            return Some(i);
        }
    }
    None
}

/// 序列化单条非 System 消息; `cache` 为 true 时把 content 转为带
/// `cache_control` 断点的文本块数组 (Anthropic 语义要求)。
fn serialize_message(m: &Message, cache: bool) -> serde_json::Value {
    let content = if cache {
        serde_json::json!([
            {"type": "text", "text": m.content, "cache_control": {"type": "ephemeral"}}
        ])
    } else {
        serde_json::json!(m.content)
    };
    serde_json::json!({
        "role": match m.role {
            Role::User => "user",
            Role::Assistant => "assistant",
            Role::Tool => "tool",
            _ => "user",
        },
        "content": content,
    })
}

/// 序列化 system 提示: `cache` 为 true 时转文本块数组并打 `cache_control`
/// (system 在会话内恒定, 是最高价值的缓存前缀)。
fn serialize_system(s: &str, cache: bool) -> serde_json::Value {
    if cache {
        serde_json::json!([
            {"type": "text", "text": s, "cache_control": {"type": "ephemeral"}}
        ])
    } else {
        serde_json::json!(s)
    }
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
fn set_proxy(&mut self, proxy_url: &str) {
        self.client = crate::neotrix::nt_io_http_factory::build_async_client_with_proxy(Some(proxy_url));
    }

    async fn complete_raw(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        let url = format!("{}/v1/messages", self.base_url);

        let system_msg = request.messages.iter().find(|m| m.role == Role::System);
        let system = system_msg.map(|m| m.content.clone());
        let cache_prefix = request.cacheable_prefix_tokens;

        let non_system: Vec<&Message> = request
            .messages
            .iter()
            .filter(|m| m.role != Role::System)
            .collect();
        // P0-4 prefix caching: 在稳定前缀边界消息上打 cache_control 断点。
        let boundary = cache_prefix.and_then(|n| prefix_boundary_index(&non_system, n));
        let messages: Vec<serde_json::Value> = non_system
            .iter()
            .enumerate()
            .map(|(i, m)| serialize_message(m, Some(i) == boundary))
            .collect();

        let mut body = serde_json::json!({
            "model": request.model,
            "messages": messages,
            "max_tokens": request.max_tokens,
        });

        if let Some(temp) = request.temperature_clean() {
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
            body["system"] = serialize_system(&s, cache_prefix.is_some());
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
        let text = response.text().await
            .map_err(|e| LlmError::Network(format!("Failed to read response body: {}", e)))?;

        match status.as_u16() {
            200 => {
                let resp: serde_json::Value = serde_json::from_str(&text)
                    .map_err(|e| LlmError::InvalidRequest(e.to_string()))?;

                // Extract text content from content blocks
                let text_content = resp["content"].as_array()
                    .map(|blocks| {
                        blocks.iter()
                            .filter_map(|b| b["text"].as_str())
                            .collect::<Vec<_>>()
                            .join("")
                    })
                    .unwrap_or_default();

                // Parse tool_use blocks from content array
                let tool_calls = resp["content"].as_array()
                    .and_then(|blocks| {
                        let calls: Vec<super::types::ToolCallInfo> = blocks.iter()
                            .filter_map(|block| {
                                if block["type"].as_str() == Some("tool_use") {
                                    Some(super::types::ToolCallInfo {
                                        id: block["id"].as_str().unwrap_or("").to_string(),
                                        call_type: "function".to_string(),
                                        function: super::types::ToolCallFunction {
                                            name: block["name"].as_str().unwrap_or("").to_string(),
                                            arguments: block["input"].to_string(),
                                        },
                                    })
                                } else {
                                    None
                                }
                            })
                            .collect();
                        if calls.is_empty() { None } else { Some(calls) }
                    });

                let finish_reason = if tool_calls.is_some() {
                    FinishReason::Tool
                } else {
                    FinishReason::Stop
                };

                let usage = Usage {
                    prompt_tokens: resp["usage"]["input_tokens"].as_u64().unwrap_or(0) as u32,
                    completion_tokens: resp["usage"]["output_tokens"].as_u64().unwrap_or(0) as u32,
                    total_tokens: (resp["usage"]["input_tokens"].as_u64().unwrap_or(0) + resp["usage"]["output_tokens"].as_u64().unwrap_or(0)) as u32,
                };
                Ok(LlmResponse { content: text_content, model: request.model.clone(), usage, finish_reason, tool_calls, reasoning: None })
            }
            401 => Err(LlmError::Authentication(text)),
            429 => Err(LlmError::RateLimit(text)),
            400 => Err(LlmError::InvalidRequest(text)),
            500..=599 => Err(LlmError::Server(text)),
            _ => Err(LlmError::Unknown(text)),
        }
    }

    async fn stream_complete_raw(&self, request: &LlmRequest) -> Result<tokio::sync::mpsc::Receiver<Result<LlmResponse, LlmError>>, LlmError> {
        let url = format!("{}/v1/messages", self.base_url);
        let api_key = self.api_key.clone();

        let model_name = request.model.clone();
        let max_tokens = request.max_tokens;
        let thinking_budget = request.thinking_budget;
        let temperature = request.temperature_clean();
        let provider_params = request.provider_params.clone();

        let system_msg = request.messages.iter().find(|m| m.role == Role::System);
        let system = system_msg.map(|m| m.content.clone());
        let cache_prefix = request.cacheable_prefix_tokens;

        let non_system: Vec<&Message> = request
            .messages
            .iter()
            .filter(|m| m.role != Role::System)
            .collect();
        // P0-4 prefix caching: 在稳定前缀边界消息上打 cache_control 断点。
        let boundary = cache_prefix.and_then(|n| prefix_boundary_index(&non_system, n));
        let messages: Vec<serde_json::Value> = non_system
            .iter()
            .enumerate()
            .map(|(i, m)| serialize_message(m, Some(i) == boundary))
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

        if let Some(s) = system { body["system"] = serialize_system(&s, cache_prefix.is_some()); }

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
                        let text = response.text().await.unwrap_or_else(|_| status.canonical_reason().unwrap_or("unknown error").to_string());
                        let _ = tx.send(Err(match status.as_u16() {
                            401 => LlmError::Authentication(text),
                            429 => LlmError::RateLimit(text),
                            _ => LlmError::Server(text),
                        })).await;
                        return;
                    }
                    let full = match response.text().await {
                        Ok(t) => t,
                        Err(e) => {
                            let _ = tx.send(Err(LlmError::Network(format!("Failed to read streaming response: {}", e)))).await;
                            return;
                        }
                    };

                    // Accumulator state for streaming tool_use blocks
                    let mut tool_calls: Vec<super::types::ToolCallInfo> = Vec::new();
                    let mut current_tool_input = String::new();
                    let mut in_tool_block = false;

                    for line in full.lines() {
                        let line = line.trim();
                        if line.is_empty() { continue; }
                        if let Some(data) = line.strip_prefix("data: ") {
                            match serde_json::from_str::<serde_json::Value>(data) {
                                Ok(v) => {
                                    match v["type"].as_str() {
                                        Some("content_block_start") => {
                                            if v["content_block"]["type"].as_str() == Some("tool_use") {
                                                in_tool_block = true;
                                                current_tool_input.clear();
                                                let id = v["content_block"]["id"].as_str().unwrap_or("").to_string();
                                                let name = v["content_block"]["name"].as_str().unwrap_or("").to_string();
                                                // Store id and name for later; arguments come via deltas
                                                // Use a placeholder that will be updated
                                                tool_calls.push(super::types::ToolCallInfo {
                                                    id,
                                                    call_type: "function".to_string(),
                                                    function: super::types::ToolCallFunction {
                                                        name,
                                                        arguments: String::new(),
                                                    },
                                                });
                                            }
                                        }
                                        Some("content_block_delta") => {
                                            if in_tool_block && v["delta"]["type"].as_str() == Some("input_json_delta") {
                                                if let Some(partial) = v["delta"]["partial_json"].as_str() {
                                                    current_tool_input.push_str(partial);
                                                }
                                            } else if let Some(text) = v["delta"]["text"].as_str() {
                                                let _ = tx.send(Ok(LlmResponse {
                                                    content: text.to_string(),
                                                    model: model_name.clone(),
                                                    usage: Usage::default(),
                                                    finish_reason: FinishReason::Unknown,
                                                    tool_calls: None,
                                                    reasoning: None,
                                                })).await;
                                            }
                                        }
                                        Some("content_block_stop") => {
                                            if in_tool_block {
                                                // Finalize the current tool call with accumulated input
                                                if let Some(last) = tool_calls.last_mut() {
                                                    last.function.arguments = current_tool_input.clone();
                                                }
                                                in_tool_block = false;
                                                current_tool_input.clear();
                                            }
                                        }
                                        Some("message_delta") => {
                                            // Final message — emit tool_calls if any
                                            if !tool_calls.is_empty() {
                                                let _ = tx.send(Ok(LlmResponse {
                                                    content: String::new(),
                                                    model: model_name.clone(),
                                                    usage: Usage::default(),
                                                    finish_reason: FinishReason::Tool,
                                                    tool_calls: Some(tool_calls.clone()),
                                                    reasoning: None,
                                                })).await;
                                                tool_calls.clear();
                                            }
                                        }
                                        Some("message_stop") => {
                                            // If there were tool calls not yet emitted (no message_delta), emit now
                                            if !tool_calls.is_empty() {
                                                let _ = tx.send(Ok(LlmResponse {
                                                    content: String::new(),
                                                    model: model_name.clone(),
                                                    usage: Usage::default(),
                                                    finish_reason: FinishReason::Tool,
                                                    tool_calls: Some(tool_calls.clone()),
                                                    reasoning: None,
                                                })).await;
                                            } else {
                                                let _ = tx.send(Ok(LlmResponse {
                                                    content: String::new(),
                                                    model: model_name.clone(),
                                                    usage: Usage::default(),
                                                    finish_reason: FinishReason::Stop,
                                                    tool_calls: None,
                                                    reasoning: None,
                                                })).await;
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                                Err(e) => {
                                    log::warn!("[anthropic] failed to parse streaming SSE line: {} (data: {:.80})", e, data);
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
