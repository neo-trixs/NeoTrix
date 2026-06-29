use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LlmProvider {
    Anthropic,
    OpenAI,
    DeepSeek,
    OpenRouter,
}

impl LlmProvider {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "anthropic" => Some(Self::Anthropic),
            "openai" => Some(Self::OpenAI),
            "deepseek" => Some(Self::DeepSeek),
            "openrouter" => Some(Self::OpenRouter),
            _ => None,
        }
    }

    pub fn env_key_name(&self) -> &'static str {
        match self {
            Self::Anthropic => "ANTHROPIC_API_KEY",
            Self::OpenAI => "OPENAI_API_KEY",
            Self::DeepSeek => "DEEPSEEK_API_KEY",
            Self::OpenRouter => "OPENROUTER_API_KEY",
        }
    }

    pub fn api_key(&self) -> Option<String> {
        std::env::var(self.env_key_name()).ok()
    }

    pub fn defaults(&self) -> (&'static str, &'static str) {
        match self {
            Self::Anthropic => ("https://api.anthropic.com/v1/messages", "claude-sonnet-4-20250514"),
            Self::OpenAI => ("https://api.openai.com/v1/chat/completions", "gpt-4o"),
            Self::DeepSeek => ("https://api.deepseek.com/chat/completions", "deepseek-chat"),
            Self::OpenRouter => ("https://openrouter.ai/api/v1/chat/completions", "openai/gpt-4o"),
        }
    }

    fn model_override(&self) -> Option<String> {
        let env_name = match self {
            Self::Anthropic => "ANTHROPIC_MODEL",
            Self::OpenAI => "OPENAI_MODEL",
            Self::DeepSeek => "DEEPSEEK_MODEL",
            Self::OpenRouter => "OPENROUTER_MODEL",
        };
        std::env::var(env_name).ok()
    }

    pub fn all() -> Vec<Self> {
        vec![Self::Anthropic, Self::OpenAI, Self::DeepSeek, Self::OpenRouter]
    }
}

impl fmt::Display for LlmProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Anthropic => write!(f, "Anthropic"),
            Self::OpenAI => write!(f, "OpenAI"),
            Self::DeepSeek => write!(f, "DeepSeek"),
            Self::OpenRouter => write!(f, "OpenRouter"),
        }
    }
}

pub struct LlmClient {
    pub provider: LlmProvider,
    api_key: String,
    base_url: String,
    pub model: String,
}

impl fmt::Debug for LlmClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LlmClient")
            .field("provider", &self.provider)
            .field("model", &self.model)
            .finish()
    }
}

impl LlmClient {
    pub fn new(provider: LlmProvider) -> Option<Self> {
        let api_key = provider.api_key()?;
        let (base_url, default_model) = provider.defaults();
        let model = provider.model_override().unwrap_or_else(|| default_model.to_string());
        Some(Self { provider, api_key, base_url: base_url.to_string(), model })
    }

    pub fn any_available() -> Option<Self> {
        for p in LlmProvider::all() {
            if let Some(client) = Self::new(p) {
                return Some(client);
            }
        }
        None
    }

    fn headers(&self) -> Result<reqwest::header::HeaderMap, String> {
        let mut h = reqwest::header::HeaderMap::new();
        match self.provider {
            LlmProvider::Anthropic => {
                h.insert("x-api-key", self.api_key.parse().map_err(|e| format!("Invalid header value: {}", e))?);
                h.insert("anthropic-version", "2023-06-01".parse().map_err(|e| format!("Invalid header value: {}", e))?);
            }
            _ => {
                h.insert("authorization", format!("Bearer {}", self.api_key).parse().map_err(|e| format!("Invalid header value: {}", e))?);
            }
        }
        h.insert("content-type", "application/json".parse().map_err(|e| format!("Invalid header value: {}", e))?);
        Ok(h)
    }

    fn body_json(&self, prompt: &str, system: &str, stream: bool) -> serde_json::Value {
        match self.provider {
            LlmProvider::Anthropic => serde_json::json!({
                "model": self.model,
                "max_tokens": 4096,
                "system": system,
                "messages": [{"role": "user", "content": prompt}],
                "stream": stream,
            }),
            _ => serde_json::json!({
                "model": self.model,
                "messages": [
                    {"role": "system", "content": system},
                    {"role": "user", "content": prompt}
                ],
                "stream": stream,
            }),
        }
    }

    pub async fn send_prompt(&self, prompt: &str, system: &str) -> Result<String, String> {
        let client = reqwest::Client::new();
        let body = self.body_json(prompt, system, false);
        let resp = client.post(&self.base_url)
            .headers(self.headers()?)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("HTTP {}: {}", status, text));
        }

        let json: serde_json::Value = resp.json().await.map_err(|e| format!("JSON parse: {}", e))?;

        match self.provider {
            LlmProvider::Anthropic => {
                json["content"]
                    .as_array()
                    .and_then(|arr| arr.first())
                    .and_then(|first| first["text"].as_str())
                    .map(|s| s.to_string())
                    .ok_or_else(|| "No response content in Anthropic response".to_string())
            }
            _ => {
                json["choices"]
                    .as_array()
                    .and_then(|arr| arr.first())
                    .and_then(|first| first["message"]["content"].as_str())
                    .map(|s| s.to_string())
                    .ok_or_else(|| "No response content".to_string())
            }
        }
    }

    pub async fn send_prompt_stream(
        &self,
        prompt: &str,
        system: &str,
        on_token: &mut dyn FnMut(&str),
    ) -> Result<String, String> {
        let client = reqwest::Client::new();
        let body = self.body_json(prompt, system, true);
        let mut resp = client
            .post(&self.base_url)
            .headers(self.headers()?)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("HTTP {}: {}", status, text));
        }

        let mut full = String::new();
        let mut buf = String::new();

        loop {
            let chunk = resp.chunk().await.map_err(|e| format!("Stream error: {}", e))?;
            let bytes = match chunk {
                Some(b) => b,
                None => break,
            };
            let s = String::from_utf8_lossy(&bytes);
            buf.push_str(&s);

            while let Some(pos) = buf.find('\n') {
                let line = buf[..pos].trim().to_string();
                buf = buf[pos + 1..].to_string();

                if line.is_empty() || line.starts_with(':') {
                    continue;
                }

                if let Some(data) = line.strip_prefix("data: ") {
                    if data == "[DONE]" {
                        return Ok(full);
                    }

                    match self.provider {
                        LlmProvider::Anthropic => {
                            if let Ok(val) = serde_json::from_str::<serde_json::Value>(data) {
                                let t = val["type"].as_str();
                                if t == Some("content_block_start") {
                                    if let Some(text) = val.pointer("/content_block/text").and_then(|t| t.as_str()) {
                                        full.push_str(text);
                                        on_token(text);
                                    }
                                } else if t == Some("content_block_delta") {
                                    if let Some(text) = val.pointer("/delta/text").and_then(|t| t.as_str()) {
                                        full.push_str(text);
                                        on_token(text);
                                    }
                                } else if t == Some("message_stop") {
                                    return Ok(full);
                                }
                            }
                        }
                        _ => {
                            if let Ok(chunk_val) = serde_json::from_str::<serde_json::Value>(data) {
                                if let Some(choices) = chunk_val["choices"].as_array() {
                                    for choice in choices {
                                        if let Some(text) = choice["delta"]["content"].as_str() {
                                            full.push_str(text);
                                            on_token(text);
                                        }
                                        if choice["finish_reason"].as_str() == Some("stop") {
                                            return Ok(full);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(full)
    }

    pub async fn send_prompt_streaming(
        &self,
        prompt: &str,
        system: &str,
        tx: tokio::sync::mpsc::Sender<String>,
    ) -> Result<(), String> {
        let client = reqwest::Client::new();
        let body = self.body_json(prompt, system, true);
        let mut resp = client
            .post(&self.base_url)
            .headers(self.headers()?)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("HTTP {}: {}", status, text));
        }

        let mut buf = String::new();

        while let Some(chunk) = resp.chunk().await.map_err(|e| format!("Stream error: {}", e))? {
            let s = String::from_utf8_lossy(&chunk);
            buf.push_str(&s);

            while let Some(pos) = buf.find('\n') {
                let line = buf[..pos].trim().to_string();
                buf = buf[pos + 1..].to_string();

                if line.is_empty() || line.starts_with(':') { continue; }

                if let Some(data) = line.strip_prefix("data: ") {
                    if data == "[DONE]" { return Ok(()); }

                    match self.provider {
                        LlmProvider::Anthropic => {
                            if let Ok(val) = serde_json::from_str::<serde_json::Value>(data) {
                                let t = val["type"].as_str();
                                if t == Some("content_block_delta") {
                                    if let Some(text) = val.pointer("/delta/text").and_then(|t| t.as_str()) {
                                        if tx.send(text.to_string()).await.is_err() {
                                            log::warn!("llm Anthropic delta send failed: channel closed");
                                        }
                                    }
                                } else if t == Some("message_stop") {
                                    return Ok(());
                                } else if t == Some("content_block_start") {
                                    if let Some(text) = val.pointer("/content_block/text").and_then(|t| t.as_str()) {
                                        if tx.send(text.to_string()).await.is_err() {
                                            log::warn!("llm Anthropic block_start send failed: channel closed");
                                        }
                                    }
                                }
                            }
                        }
                        _ => {
                            if let Ok(chunk_val) = serde_json::from_str::<serde_json::Value>(data) {
                                if let Some(choices) = chunk_val["choices"].as_array() {
                                    for choice in choices {
                                        if let Some(text) = choice["delta"]["content"].as_str() {
                                            if tx.send(text.to_string()).await.is_err() {
                                                log::warn!("llm default choice send failed: channel closed");
                                            }
                                        }
                                        if choice["finish_reason"].as_str() == Some("stop") {
                                            return Ok(());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub fn send_prompt_blocking(&self, prompt: &str, system: &str) -> Result<String, String> {
        let client = reqwest::blocking::Client::new();
        let body = self.body_json(prompt, system, false);
        let resp = client
            .post(&self.base_url)
            .headers(self.headers()?)
            .json(&body)
            .send()
            .map_err(|e| format!("Request failed: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().unwrap_or_default();
            return Err(format!("HTTP {}: {}", status, text));
        }

        let json: serde_json::Value = resp.json().map_err(|e| format!("JSON parse: {}", e))?;

        match self.provider {
            LlmProvider::Anthropic => {
                json["content"]
                    .as_array()
                    .and_then(|arr| arr.first())
                    .and_then(|first| first["text"].as_str())
                    .map(|s| s.to_string())
                    .ok_or_else(|| "No response content".to_string())
            }
            _ => {
                json["choices"]
                    .as_array()
                    .and_then(|arr| arr.first())
                    .and_then(|first| first["message"]["content"].as_str())
                    .map(|s| s.to_string())
                    .ok_or_else(|| "No response content".to_string())
            }
        }
    }
}

/// Accumulates streaming text chunks and optional tool calls.
pub struct StreamingResponse {
    full: String,
    tool_calls: Vec<(String, String)>,
}

impl StreamingResponse {
    pub fn new() -> Self {
        Self { full: String::new(), tool_calls: Vec::new() }
    }

    pub fn push_text(&mut self, text: &str) {
        self.full.push_str(text);
    }

    pub fn push_tool_call(&mut self, name: &str, args: &str) {
        self.tool_calls.push((name.to_string(), args.to_string()));
    }

    pub fn full_text(&self) -> &str {
        &self.full
    }

    pub fn has_tool_calls(&self) -> bool {
        !self.tool_calls.is_empty()
    }
}

/// Rough token estimate (~4 chars per token, ~1.3 tokens per word, averaged).
pub fn estimate_tokens(text: &str) -> usize {
    if text.is_empty() {
        return 0;
    }
    let char_est = text.len() / 4;
    let word_est = (text.split_whitespace().count() as f64 * 1.3) as usize;
    (char_est + word_est).max(1) / 2 + 1
}

/// Extract content from an OpenAI-style chat completion JSON response.
pub fn parse_openai_completion(json: &str) -> Result<String, String> {
    let val: serde_json::Value =
        serde_json::from_str(json).map_err(|e| format!("JSON parse error: {}", e))?;
    val["choices"]
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|first| first["message"]["content"].as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "No response content in OpenAI response".to_string())
}

/// Extract content from an Anthropic-style messages API JSON response.
pub fn parse_anthropic_completion(json: &str) -> Result<String, String> {
    let val: serde_json::Value =
        serde_json::from_str(json).map_err(|e| format!("JSON parse error: {}", e))?;
    val["content"]
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|first| first["text"].as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "No response content in Anthropic response".to_string())
}

/// Extract content from a Gemini-style response JSON.
pub fn parse_gemini_completion(json: &str) -> Result<String, String> {
    let val: serde_json::Value =
        serde_json::from_str(json).map_err(|e| format!("JSON parse error: {}", e))?;
    val["candidates"]
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|first| first["content"]["parts"].as_array())
        .and_then(|parts| parts.first())
        .and_then(|part| part["text"].as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "No response content in Gemini response".to_string())
}

#[cfg(test)]
mod tests {
    // NOTE: Tests in this module use std::env::set_var for API keys.
    // Each #[test] runs on its own OS thread (Rust test harness), so
    // concurrent set_var races across tests are structurally impossible.
    use super::*;

    #[test]
    fn test_lm_provider_all() {
        let all = LlmProvider::all();
        assert_eq!(all.len(), 4);
        assert!(all.contains(&LlmProvider::Anthropic));
        assert!(all.contains(&LlmProvider::OpenAI));
        assert!(all.contains(&LlmProvider::DeepSeek));
        assert!(all.contains(&LlmProvider::OpenRouter));
    }

    #[test]
    fn test_provider_from_str() {
        assert_eq!(LlmProvider::from_str("Anthropic"), Some(LlmProvider::Anthropic));
        assert_eq!(LlmProvider::from_str("openai"), Some(LlmProvider::OpenAI));
        assert_eq!(LlmProvider::from_str("DEEPSEEK"), Some(LlmProvider::DeepSeek));
        assert_eq!(LlmProvider::from_str("OpenRouter"), Some(LlmProvider::OpenRouter));
        assert_eq!(LlmProvider::from_str("unknown"), None);
    }

    #[test]
    fn test_provider_env_key() {
        assert_eq!(LlmProvider::Anthropic.env_key_name(), "ANTHROPIC_API_KEY");
        assert_eq!(LlmProvider::OpenAI.env_key_name(), "OPENAI_API_KEY");
        assert_eq!(LlmProvider::DeepSeek.env_key_name(), "DEEPSEEK_API_KEY");
        assert_eq!(LlmProvider::OpenRouter.env_key_name(), "OPENROUTER_API_KEY");
    }

    #[test]
    fn test_provider_defaults() {
        let (url, _) = LlmProvider::Anthropic.defaults();
        assert!(url.contains("anthropic.com"));
        let (_, model) = LlmProvider::OpenAI.defaults();
        assert!(model.contains("gpt"));
    }

    #[test]
    fn test_headers_contains_auth() {
        if std::env::var("OPENAI_API_KEY").is_err() { return; }
        let client = LlmClient::new(LlmProvider::OpenAI).unwrap();
        let headers = client.headers().unwrap();
        assert!(headers.contains_key("authorization"));
        assert!(headers.contains_key("content-type"));
    }

    #[test]
    fn test_any_available_when_no_keys() {
        let keys = ["ANTHROPIC_API_KEY", "OPENAI_API_KEY", "DEEPSEEK_API_KEY", "OPENROUTER_API_KEY"];
        let saved: Vec<_> = keys.iter().map(|k| (k, std::env::var(k).ok())).collect();
        for k in &keys { std::env::remove_var(k); }
        assert!(LlmClient::any_available().is_none());
        for (k, v) in saved { if let Some(v) = v { std::env::set_var(k, v); } }
    }

    #[test]
    fn test_provider_display() {
        assert_eq!(LlmProvider::Anthropic.to_string(), "Anthropic");
        assert_eq!(LlmProvider::OpenAI.to_string(), "OpenAI");
        assert_eq!(LlmProvider::DeepSeek.to_string(), "DeepSeek");
        assert_eq!(LlmProvider::OpenRouter.to_string(), "OpenRouter");
    }

    #[test]
    fn test_body_json_openai() {
        let saved = std::env::var("OPENAI_API_KEY").ok();
        std::env::set_var("OPENAI_API_KEY", "test-key");
        let client = LlmClient::new(LlmProvider::OpenAI).unwrap();
        let body = client.body_json("Hello", "System prompt", false);
        assert_eq!(body["model"], "gpt-4o");
        assert_eq!(body["messages"][0]["role"], "system");
        assert_eq!(body["messages"][0]["content"], "System prompt");
        assert_eq!(body["messages"][1]["role"], "user");
        assert_eq!(body["messages"][1]["content"], "Hello");
        assert!(!body["stream"].as_bool().unwrap());
        restore_env("OPENAI_API_KEY", saved);
    }

    #[test]
    fn test_body_json_anthropic() {
        let saved = std::env::var("ANTHROPIC_API_KEY").ok();
        std::env::set_var("ANTHROPIC_API_KEY", "test-key");
        let client = LlmClient::new(LlmProvider::Anthropic).unwrap();
        let body = client.body_json("Hello", "System prompt", true);
        assert!(body["model"].as_str().unwrap_or("").contains("claude"));
        assert_eq!(body["system"], "System prompt");
        assert_eq!(body["messages"][0]["role"], "user");
        assert_eq!(body["messages"][0]["content"], "Hello");
        assert!(body["max_tokens"].as_u64().unwrap_or(0) > 0);
        assert!(body["stream"].as_bool().unwrap());
        restore_env("ANTHROPIC_API_KEY", saved);
    }

    #[test]
    fn test_body_json_deepseek() {
        let saved = std::env::var("DEEPSEEK_API_KEY").ok();
        std::env::set_var("DEEPSEEK_API_KEY", "test-key");
        let client = LlmClient::new(LlmProvider::DeepSeek).unwrap();
        let body = client.body_json("Translate this", "You are a translator", false);
        assert_eq!(body["model"], "deepseek-chat");
        assert_eq!(body["messages"][0]["role"], "system");
        assert_eq!(body["messages"][1]["content"], "Translate this");
        assert!(!body["stream"].as_bool().unwrap());
        restore_env("DEEPSEEK_API_KEY", saved);
    }

    #[test]
    fn test_body_json_openrouter() {
        let saved = std::env::var("OPENROUTER_API_KEY").ok();
        std::env::set_var("OPENROUTER_API_KEY", "test-key");
        let client = LlmClient::new(LlmProvider::OpenRouter).unwrap();
        let body = client.body_json("Hi", "Assistant", true);
        assert!(body["stream"].as_bool().unwrap());
        assert_eq!(body["messages"][0]["role"], "system");
        assert_eq!(body["messages"][1]["content"], "Hi");
        restore_env("OPENROUTER_API_KEY", saved);
    }

    #[test]
    fn test_streaming_chunks_accumulate() {
        let mut collector = StreamingResponse::new();
        collector.push_text("Hello");
        collector.push_text(" World");
        assert_eq!(collector.full_text(), "Hello World");
    }

    #[test]
    fn test_streaming_chunks_with_tool_calls() {
        let mut collector = StreamingResponse::new();
        collector.push_text("");
        collector.push_tool_call("bash", "ls -la");
        collector.push_text("Done");
        assert!(collector.has_tool_calls());
    }

    #[test]
    fn test_token_counting() {
        let count = estimate_tokens("Hello World");
        assert!(count > 0);
        assert!(count >= 1);
    }

    #[test]
    fn test_token_counting_empty() {
        assert_eq!(estimate_tokens(""), 0);
    }

    #[test]
    fn test_token_counting_long_text() {
        let text = "word ".repeat(1000);
        let count = estimate_tokens(&text);
        assert!(count > 100);
    }

    #[test]
    fn test_parse_openai_response() {
        let json = r#"{"choices":[{"message":{"content":"Hello","role":"assistant"}}]}"#;
        let result = parse_openai_completion(json);
        assert_eq!(result.unwrap(), "Hello");
    }

    #[test]
    fn test_parse_anthropic_response() {
        let json = r#"{"content":[{"type":"text","text":"Hello"}],"role":"assistant"}"#;
        let result = parse_anthropic_completion(json);
        assert_eq!(result.unwrap(), "Hello");
    }

    #[test]
    fn test_parse_gemini_response() {
        let json = r#"{"candidates":[{"content":{"parts":[{"text":"Hello"}]}}]}"#;
        let result = parse_gemini_completion(json);
        assert_eq!(result.unwrap(), "Hello");
    }

    #[test]
    fn test_parse_empty_response() {
        assert!(parse_openai_completion("{}").is_err());
    }

    #[test]
    fn test_parse_malformed_json() {
        assert!(parse_openai_completion("not json").is_err());
    }

    fn restore_env(key: &str, saved: Option<String>) {
        match saved {
            Some(v) => std::env::set_var(key, v),
            None => std::env::remove_var(key),
        }
    }
}
