use async_trait::async_trait;

use crate::l1_action::nt_io::nt_io_provider::common::types::{FinishReason, LlmError, LlmProvider, LlmRequest, LlmResponse, Usage};

pub struct OllamaProvider {
    base_url: String,
    client: reqwest::Client,
}

impl OllamaProvider {
    pub fn new() -> Self {
        Self {
            base_url: "http://localhost:11434".to_string(),
            client: crate::neotrix::nt_io_http_factory::global_client().clone(),
        }
    }

    pub fn with_base_url(mut self, url: &str) -> Self {
        self.base_url = url.to_string();
        self
    }

    fn build_messages(&self, request: &LlmRequest) -> Vec<serde_json::Value> {
        request.messages.iter().map(|m| {
            let role = match m.role {
                super::types::Role::System => "system",
                super::types::Role::User => "user",
                super::types::Role::Assistant => "assistant",
                super::types::Role::Tool => "tool",
            };
            let mut msg = serde_json::json!({
                "role": role,
                "content": m.content,
            });
            if let Some(ref tool_calls) = m.tool_calls {
                msg["tool_calls"] = serde_json::json!(tool_calls);
            }
            if let Some(ref tool_call_id) = m.tool_call_id {
                msg["tool_call_id"] = serde_json::json!(tool_call_id);
            }
            msg
        }).collect()
    }
}

impl Default for OllamaProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LlmProvider for OllamaProvider {
    fn data_trust(&self) -> crate::core::nt_core_llm::DataTrust {
        crate::core::nt_core_llm::DataTrust::Trusted
    }

    fn set_proxy(&mut self, proxy_url: &str) {
        self.client = crate::neotrix::nt_io_http_factory::build_async_client_with_proxy(Some(proxy_url));
    }

    async fn complete_raw(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        let messages = self.build_messages(request);
        let mut body = serde_json::json!({
            "model": request.model,
            "messages": messages,
            "stream": false,
        });

        if let Some(temp) = request.temperature_clean() {
            body["options"] = serde_json::json!({ "temperature": temp });
        }

        if !request.tools.is_empty() {
            body["tools"] = serde_json::json!(request.tools);
        }

        let response = self.client
            .post(format!("{}/api/chat", self.base_url))
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

                let content = resp["message"]["content"].as_str().unwrap_or("").to_string();

                let tool_calls = resp.get("message")
                    .and_then(|m| m.get("tool_calls"))
                    .and_then(|tc| serde_json::from_value::<Vec<super::types::ToolCallInfo>>(tc.clone()).ok())
                    .filter(|v| !v.is_empty());

                let prompt_tokens = resp.get("prompt_eval_count").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                let completion_tokens = resp.get("eval_count").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                let usage = Usage {
                    prompt_tokens,
                    completion_tokens,
                    total_tokens: prompt_tokens + completion_tokens,
                };

                let finish_reason = if tool_calls.is_some() {
                    FinishReason::Tool
                } else {
                    FinishReason::Stop
                };

                Ok(LlmResponse { content, model: request.model.clone(), usage, finish_reason, tool_calls, reasoning: None })
            }
            400 => Err(LlmError::InvalidRequest(text)),
            500..=599 => Err(LlmError::Server(text)),
            _ => Err(LlmError::Unknown(text)),
        }
    }

    async fn stream_complete_raw(&self, request: &LlmRequest) -> Result<tokio::sync::mpsc::Receiver<Result<LlmResponse, LlmError>>, LlmError> {
        let messages = self.build_messages(request);
        let mut body = serde_json::json!({
            "model": request.model,
            "messages": messages,
            "stream": true,
        });

        if let Some(temp) = request.temperature_clean() {
            body["options"] = serde_json::json!({ "temperature": temp });
        }

        if !request.tools.is_empty() {
            body["tools"] = serde_json::json!(request.tools);
        }

        let base_url = self.base_url.clone();
        let model = request.model.clone();

        let (tx, rx) = tokio::sync::mpsc::channel(64);

        tokio::spawn(async move {
            let client = crate::neotrix::nt_io_http_factory::global_client().clone();
            if let Ok(response) = client.post(format!("{}/api/chat", base_url))
                .json(&body)
                .send().await {
                if !response.status().is_success() { return; }
                let full = response.text().await.unwrap_or_default();
                for line in full.lines() {
                    let line = line.trim();
                    if line.is_empty() { continue; }
                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(line) {
                        let content = v["message"]["content"].as_str().unwrap_or("").to_string();
                        let tool_calls = v.get("message")
                            .and_then(|m| m.get("tool_calls"))
                            .and_then(|tc| serde_json::from_value::<Vec<super::types::ToolCallInfo>>(tc.clone()).ok())
                            .filter(|v| !v.is_empty());

                        if !content.is_empty() || tool_calls.is_some() {
                            let finish_reason = if tool_calls.is_some() {
                                FinishReason::Tool
                            } else {
                                FinishReason::Unknown
                            };
                            let _ = tx.send(Ok(LlmResponse {
                                content,
                                model: model.clone(),
                                usage: Usage::default(),
                                finish_reason,
                                tool_calls,
                                reasoning: None,
                            })).await;
                        }

                        if v.get("done").and_then(|d| d.as_bool()).unwrap_or(false) {
                            let _ = tx.send(Ok(LlmResponse {
                                content: String::new(),
                                model: model.clone(),
                                usage: Usage::default(),
                                finish_reason: FinishReason::Stop,
                                tool_calls: None,
                                reasoning: None,
                            })).await;
                        }
                    }
                }
            }
        });

        Ok(rx)
    }
}
