use async_trait::async_trait;

use super::types::{FinishReason, LlmError, LlmProvider, LlmRequest, LlmResponse, Usage};

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

    fn build_prompt(&self, request: &LlmRequest) -> String {
        request.messages.iter()
            .map(|m| {
                let role = match m.role {
                    super::types::Role::System => "system",
                    super::types::Role::User => "user",
                    super::types::Role::Assistant => "assistant",
                    super::types::Role::Tool => "tool",
                };
                format!("<|{}|>\n{}\n<|end|>", role, m.content)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl Default for OllamaProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LlmProvider for OllamaProvider {
    fn set_proxy(&mut self, proxy_url: &str) {
        self.client = crate::neotrix::nt_io_http_factory::build_async_client_with_proxy(Some(proxy_url));
    }

    async fn complete(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        let prompt = self.build_prompt(request);
        let mut body = serde_json::json!({
            "model": request.model,
            "prompt": prompt,
            "stream": false,
            "options": {
                "num_predict": request.max_tokens,
            }
        });

        if let Some(temp) = request.temperature {
            body["options"]["temperature"] = serde_json::json!(temp);
        }

        let response = self.client
            .post(format!("{}/api/generate", self.base_url))
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
                let content = resp["response"].as_str().unwrap_or("").to_string();
                let usage = Usage {
                    prompt_tokens: resp.get("prompt_eval_count").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                    completion_tokens: resp.get("eval_count").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                    total_tokens: 0,
                };
                Ok(LlmResponse { content, model: request.model.clone(), usage, finish_reason: FinishReason::Stop })
            }
            400 => Err(LlmError::InvalidRequest(text)),
            500..=599 => Err(LlmError::Server(text)),
            _ => Err(LlmError::Unknown(text)),
        }
    }

    async fn stream_complete(&self, request: &LlmRequest) -> Result<tokio::sync::mpsc::Receiver<Result<LlmResponse, LlmError>>, LlmError> {
        let prompt = self.build_prompt(request);
        let mut body = serde_json::json!({
            "model": request.model,
            "prompt": prompt,
            "stream": true,
            "options": {
                "num_predict": request.max_tokens,
            }
        });

        if let Some(temp) = request.temperature {
            body["options"]["temperature"] = serde_json::json!(temp);
        }
        let base_url = self.base_url.clone();
        let model = request.model.clone();

        let (tx, rx) = tokio::sync::mpsc::channel(64);

        tokio::spawn(async move {
            let client = crate::neotrix::nt_io_http_factory::global_client().clone();
            if let Ok(response) = client.post(format!("{}/api/generate", base_url))
                .json(&body)
                .send().await {
                if !response.status().is_success() { return; }
                let full = response.text().await.unwrap_or_default();
                for line in full.lines() {
                    let line = line.trim();
                    if line.is_empty() { continue; }
                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(line) {
                        if let Some(text) = v["response"].as_str() {
                            if text.is_empty() { continue; }
                            let _ = tx.send(Ok(LlmResponse {
                                content: text.to_string(),
                                model: model.clone(),
                                usage: Usage::default(),
                                finish_reason: FinishReason::Unknown,
                            })).await;
                        }
                        if v.get("done").and_then(|d| d.as_bool()).unwrap_or(false) {
                            let _ = tx.send(Ok(LlmResponse {
                                content: String::new(),
                                model: model.clone(),
                                usage: Usage::default(),
                                finish_reason: FinishReason::Stop,
                            })).await;
                        }
                    }
                }
            }
        });

        Ok(rx)
    }
}
