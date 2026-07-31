use super::types::*;
use crate::neotrix::nt_io_http_factory::global_client;

const GROQ_BASE: &str = "https://api.groq.com/openai/v1";
const OPENROUTER_BASE: &str = "https://openrouter.ai/api/v1";
const POLLINATIONS_BASE: &str = "https://text.pollinations.ai/openai";


pub struct GroqProvider {
    api_key: String,
    base_url: String,
    client: reqwest::Client,
}

impl GroqProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            base_url: GROQ_BASE.to_string(),
            client: global_client().clone(),
        }
    }

    pub fn with_base_url(mut self, url: &str) -> Self {
        self.base_url = url.trim_end_matches('/').to_string();
        self
    }

    fn build_request_body(&self, request: &LlmRequest) -> serde_json::Value {
        let mut body = serde_json::json!({
            "model": request.model,
            "messages": request.messages.iter().map(|m| {
                serde_json::json!({
                    "role": match m.role {
                        Role::System => "system",
                        Role::User => "user",
                        Role::Assistant => "assistant",
                        Role::Tool => "tool",
                    },
                    "content": m.content,
                })
            }).collect::<Vec<_>>(),
            "max_tokens": request.max_tokens,
            "stream": false,
        });

        if let Some(temp) = request.temperature {
            body["temperature"] = serde_json::json!(temp);
        }
        body
    }

    fn parse_response(&self, text: &str) -> Result<LlmResponse, LlmError> {
        let v: serde_json::Value = serde_json::from_str(text)
            .map_err(|e| LlmError::Unknown(format!("JSON parse: {}", e)))?;

        let content = v["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let model = v["model"].as_str().unwrap_or("unknown").to_string();
        let usage = v["usage"].as_object().map(|u| Usage {
            prompt_tokens: u["prompt_tokens"].as_u64().unwrap_or(0) as u32,
            completion_tokens: u["completion_tokens"].as_u64().unwrap_or(0) as u32,
            total_tokens: u["total_tokens"].as_u64().unwrap_or(0) as u32,
        }).unwrap_or_default();

        let finish = match v["choices"][0]["finish_reason"].as_str() {
            Some("stop") => FinishReason::Stop,
            Some("length") => FinishReason::Length,
            Some("tool_calls") => FinishReason::Tool,
            Some("content_filter") => FinishReason::ContentFilter,
            _ => FinishReason::Unknown,
        };

        Ok(LlmResponse { content, model, usage, finish_reason: finish })
    }
}

#[async_trait::async_trait]
impl LlmProvider for GroqProvider {
    fn set_proxy(&mut self, proxy_url: &str) {
        self.client = crate::neotrix::nt_io_http_factory::build_async_client_with_proxy(Some(proxy_url));
    }

    async fn complete(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        let body = self.build_request_body(request);
        let url = format!("{}/chat/completions", self.base_url);

        let resp = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await
            .map_err(|e| LlmError::Network(format!("{}", e)))?;

        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();

        match status.as_u16() {
            200 => self.parse_response(&text),
            401 => Err(LlmError::Authentication(text)),
            429 => Err(LlmError::RateLimit(text)),
            400 => Err(LlmError::InvalidRequest(text)),
            500..=599 => Err(LlmError::Server(text)),
            _ => Err(LlmError::Unknown(text)),
        }
    }

    async fn stream_complete(&self, request: &LlmRequest) -> Result<tokio::sync::mpsc::Receiver<Result<LlmResponse, LlmError>>, LlmError> {
        let mut body = serde_json::json!({
            "model": request.model,
            "messages": request.messages.iter().map(|m| {
                serde_json::json!({
                    "role": match m.role {
                        Role::System => "system",
                        Role::User => "user",
                        Role::Assistant => "assistant",
                        Role::Tool => "tool",
                    },
                    "content": m.content,
                })
            }).collect::<Vec<_>>(),
            "max_tokens": request.max_tokens,
            "stream": true,
        });

        if let Some(temp) = request.temperature {
            body["temperature"] = serde_json::json!(temp);
        }

        let url = format!("{}/chat/completions", self.base_url);
        let api_key = self.api_key.clone();

        let (tx, rx) = tokio::sync::mpsc::channel(64);

        tokio::spawn(async move {
            let client = global_client().clone();
            match client
                .post(&url)
                .header("Authorization", format!("Bearer {}", api_key))
                .json(&body)
                .send()
                .await
            {
                Ok(response) => {
                    let status = response.status();
                    if !status.is_success() {
                        let text = response.text().await.unwrap_or_default();
                        let err = match status.as_u16() {
                            401 => LlmError::Authentication(text),
                            429 => LlmError::RateLimit(text),
                            400 => LlmError::InvalidRequest(text),
                            _ => LlmError::Server(text),
                        };
                        let _ = tx.send(Err(err)).await;
                        return;
                    }

                    let full_text = response.text().await.unwrap_or_default();
                    for line in full_text.lines() {
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
                Err(e) => {
                    let _ = tx.send(Err(LlmError::Network(format!("{}", e)))).await;
                }
            }
        });

        Ok(rx)
    }
}

pub struct OpenRouterProvider {
    api_key: String,
    base_url: String,
    client: reqwest::Client,
}

impl OpenRouterProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            base_url: OPENROUTER_BASE.to_string(),
            client: global_client().clone(),
        }
    }

    pub fn with_base_url(mut self, url: &str) -> Self {
        self.base_url = url.trim_end_matches('/').to_string();
        self
    }
}

#[async_trait::async_trait]
impl LlmProvider for OpenRouterProvider {
    fn set_proxy(&mut self, proxy_url: &str) {
        self.client = crate::neotrix::nt_io_http_factory::build_async_client_with_proxy(Some(proxy_url));
    }

    async fn complete(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        let mut body = serde_json::json!({
            "model": request.model,
            "messages": request.messages.iter().map(|m| {
                serde_json::json!({
                    "role": match m.role {
                        Role::System => "system",
                        Role::User => "user",
                        Role::Assistant => "assistant",
                        Role::Tool => "tool",
                    },
                    "content": m.content,
                })
            }).collect::<Vec<_>>(),
            "max_tokens": request.max_tokens,
        });

        if let Some(temp) = request.temperature {
            body["temperature"] = serde_json::json!(temp);
        }

        let url = format!("{}/chat/completions", self.base_url);
        let resp = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("HTTP-Referer", "https://neotrix.ai")
            .header("X-Title", "NeoTrix")
            .json(&body)
            .send()
            .await
            .map_err(|e| LlmError::Network(format!("{}", e)))?;

        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();

        match status.as_u16() {
            200 => {
                let v: serde_json::Value = serde_json::from_str(&text)
                    .map_err(|e| LlmError::Unknown(format!("JSON: {}", e)))?;
                let content = v["choices"][0]["message"]["content"].as_str().unwrap_or("").to_string();
                let model = v["model"].as_str().unwrap_or("unknown").to_string();
                let usage = v["usage"].as_object().map(|u| Usage {
                    prompt_tokens: u["prompt_tokens"].as_u64().unwrap_or(0) as u32,
                    completion_tokens: u["completion_tokens"].as_u64().unwrap_or(0) as u32,
                    total_tokens: u["total_tokens"].as_u64().unwrap_or(0) as u32,
                }).unwrap_or_default();
                let finish = match v["choices"][0]["finish_reason"].as_str() {
                    Some("stop") => FinishReason::Stop,
                    Some("length") => FinishReason::Length,
                    _ => FinishReason::Unknown,
                };
                Ok(LlmResponse { content, model, usage, finish_reason: finish })
            }
            401 => Err(LlmError::Authentication(text)),
            429 => Err(LlmError::RateLimit(text)),
            400 => Err(LlmError::InvalidRequest(text)),
            500..=599 => Err(LlmError::Server(text)),
            _ => Err(LlmError::Unknown(text)),
        }
    }

    async fn stream_complete(&self, request: &LlmRequest) -> Result<tokio::sync::mpsc::Receiver<Result<LlmResponse, LlmError>>, LlmError> {
        let mut body = serde_json::json!({
            "model": request.model,
            "messages": request.messages.iter().map(|m| {
                serde_json::json!({
                    "role": match m.role {
                        Role::System => "system",
                        Role::User => "user",
                        Role::Assistant => "assistant",
                        Role::Tool => "tool",
                    },
                    "content": m.content,
                })
            }).collect::<Vec<_>>(),
            "max_tokens": request.max_tokens,
            "stream": true,
        });

        if let Some(temp) = request.temperature {
            body["temperature"] = serde_json::json!(temp);
        }

        let url = format!("{}/chat/completions", self.base_url);
        let api_key = self.api_key.clone();
        let (tx, rx) = tokio::sync::mpsc::channel(64);

        tokio::spawn(async move {
            let client = global_client().clone();
            match client
                .post(&url)
                .header("Authorization", format!("Bearer {}", api_key))
                .json(&body)
                .send()
                .await
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
                    let full_text = response.text().await.unwrap_or_default();
                    for line in full_text.lines() {
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
                Err(e) => {
                    let _ = tx.send(Err(LlmError::Network(format!("{}", e)))).await;
                }
            }
        });

        Ok(rx)
    }
}

pub struct PollinationsProvider {
    base_url: String,
    client: reqwest::Client,
}

impl Default for PollinationsProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl PollinationsProvider {
    pub fn new() -> Self {
        Self {
            base_url: POLLINATIONS_BASE.to_string(),
            client: global_client().clone(),
        }
    }
}

#[async_trait::async_trait]
impl LlmProvider for PollinationsProvider {
    fn set_proxy(&mut self, proxy_url: &str) {
        self.client = crate::neotrix::nt_io_http_factory::build_async_client_with_proxy(Some(proxy_url));
    }

    async fn complete(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        let mut body = serde_json::json!({
            "model": request.model,
            "messages": request.messages.iter().map(|m| serde_json::json!({
                "role": match m.role {
                    Role::System => "system",
                    Role::User => "user",
                    Role::Assistant => "assistant",
                    Role::Tool => "tool",
                },
                "content": m.content,
            })).collect::<Vec<_>>(),
            "max_tokens": request.max_tokens,
        });

        if let Some(temp) = request.temperature {
            body["temperature"] = serde_json::json!(temp);
        }

        let resp = self.client
            .post(&self.base_url)
            .json(&body)
            .send()
            .await
            .map_err(|e| LlmError::Network(format!("{}", e)))?;

        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();

        match status.as_u16() {
            200 => {
                let content = text.trim().to_string();
                Ok(LlmResponse {
                    content,
                    model: request.model.clone(),
                    usage: Usage::default(),
                    finish_reason: FinishReason::Stop,
                })
            }
            429 => Err(LlmError::RateLimit(text)),
            500..=599 => Err(LlmError::Server(text)),
            _ => Err(LlmError::Unknown(text)),
        }
    }

    async fn stream_complete(&self, request: &LlmRequest) -> Result<tokio::sync::mpsc::Receiver<Result<LlmResponse, LlmError>>, LlmError> {
        let mut body = serde_json::json!({
            "model": request.model,
            "messages": request.messages.iter().map(|m| serde_json::json!({
                "role": match m.role {
                    Role::System => "system",
                    Role::User => "user",
                    Role::Assistant => "assistant",
                    Role::Tool => "tool",
                },
                "content": m.content,
            })).collect::<Vec<_>>(),
            "max_tokens": request.max_tokens,
            "stream": true,
        });

        if let Some(temp) = request.temperature {
            body["temperature"] = serde_json::json!(temp);
        }

        let base_url = self.base_url.clone();
        let (tx, rx) = tokio::sync::mpsc::channel(64);

        tokio::spawn(async move {
            let client = global_client().clone();
            if let Ok(response) = client.post(&base_url).json(&body).send().await {
                if !response.status().is_success() { return; }
                let full_text = response.text().await.unwrap_or_default();
                if !full_text.is_empty() {
                    let _ = tx.send(Ok(LlmResponse {
                        content: full_text,
                        model: String::new(),
                        usage: Usage::default(),
                        finish_reason: FinishReason::Unknown,
                    })).await;
                }
            }
        });

        Ok(rx)
    }
}

pub struct CerebrasProvider {
    api_key: String,
    client: reqwest::Client,
}

impl CerebrasProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: global_client().clone(),
        }
    }
}

#[async_trait::async_trait]
impl LlmProvider for CerebrasProvider {
    fn set_proxy(&mut self, proxy_url: &str) {
        self.client = crate::neotrix::nt_io_http_factory::build_async_client_with_proxy(Some(proxy_url));
    }

    async fn complete(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        let mut body = serde_json::json!({
            "model": request.model,
            "messages": request.messages.iter().map(|m| serde_json::json!({
                "role": match m.role {
                    Role::System => "system",
                    Role::User => "user",
                    Role::Assistant => "assistant",
                    Role::Tool => "tool",
                },
                "content": m.content,
            })).collect::<Vec<_>>(),
            "max_tokens": request.max_tokens,
        });

        if let Some(temp) = request.temperature {
            body["temperature"] = serde_json::json!(temp);
        }

        let resp = self.client
            .post("https://api.cerebras.ai/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await
            .map_err(|e| LlmError::Network(format!("{}", e)))?;

        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();

        match status.as_u16() {
            200 => {
                let v: serde_json::Value = serde_json::from_str(&text).map_err(|e| LlmError::Unknown(format!("JSON: {}", e)))?;
                Ok(LlmResponse {
                    content: v["choices"][0]["message"]["content"].as_str().unwrap_or("").to_string(),
                    model: v["model"].as_str().unwrap_or("").to_string(),
                    usage: v["usage"].as_object().map(|u| Usage {
                        prompt_tokens: u["prompt_tokens"].as_u64().unwrap_or(0) as u32,
                        completion_tokens: u["completion_tokens"].as_u64().unwrap_or(0) as u32,
                        total_tokens: u["total_tokens"].as_u64().unwrap_or(0) as u32,
                    }).unwrap_or_default(),
                    finish_reason: FinishReason::Stop,
                })
            }
            401 => Err(LlmError::Authentication(text)),
            429 => Err(LlmError::RateLimit(text)),
            _ => Err(LlmError::Server(text)),
        }
    }

    async fn stream_complete(&self, request: &LlmRequest) -> Result<tokio::sync::mpsc::Receiver<Result<LlmResponse, LlmError>>, LlmError> {
        let mut body = serde_json::json!({
            "model": request.model,
            "messages": request.messages.iter().map(|m| serde_json::json!({
                "role": match m.role {
                    Role::System => "system",
                    Role::User => "user",
                    Role::Assistant => "assistant",
                    Role::Tool => "tool",
                },
                "content": m.content,
            })).collect::<Vec<_>>(),
            "max_tokens": request.max_tokens,
            "stream": true,
        });

        if let Some(temp) = request.temperature {
            body["temperature"] = serde_json::json!(temp);
        }

        let api_key = self.api_key.clone();
        let (tx, rx) = tokio::sync::mpsc::channel(64);

        tokio::spawn(async move {
            let client = global_client().clone();
            match client
                .post("https://api.cerebras.ai/v1/chat/completions")
                .header("Authorization", format!("Bearer {}", api_key))
                .json(&body)
                .send()
                .await
            {
                Ok(response) => {
                    let status = response.status();
                    if !status.is_success() {
                        let text = response.text().await.unwrap_or_default();
                        let err = match status.as_u16() {
                            401 => LlmError::Authentication(text),
                            429 => LlmError::RateLimit(text),
                            400 => LlmError::InvalidRequest(text),
                            _ => LlmError::Server(text),
                        };
                        let _ = tx.send(Err(err)).await;
                        return;
                    }

                    let full_text = response.text().await.unwrap_or_default();
                    for line in full_text.lines() {
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
                Err(e) => {
                    let _ = tx.send(Err(LlmError::Network(format!("{}", e)))).await;
                }
            }
        });

        Ok(rx)
    }
}


#[cfg(test)]
mod tests {

    #[test]
    fn test_basic() {
        assert!(true);
    }
}
