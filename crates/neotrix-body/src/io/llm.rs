//! # LLM Device Driver
//!
//! Unified LLM provider interface. Supports OpenAI, Anthropic, and local models.
//! All responses are VSA-encoded for the consciousness loop.

/// LLM provider types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LlmProvider {
    OpenAI,
    Anthropic,
    Local,
    Custom(&'static str),
}

impl LlmProvider {
    pub fn name(&self) -> &str {
        match self {
            Self::OpenAI => "openai",
            Self::Anthropic => "anthropic",
            Self::Local => "local",
            Self::Custom(s) => s,
        }
    }
}

/// LLM completion request
#[derive(Debug, Clone)]
pub struct LlmRequest {
    pub provider: LlmProvider,
    pub model: String,
    pub system_prompt: String,
    pub user_prompt: String,
    pub temperature: f64,
    pub max_tokens: u32,
}

impl Default for LlmRequest {
    fn default() -> Self {
        Self {
            provider: LlmProvider::OpenAI,
            model: "gpt-4".into(),
            system_prompt: String::new(),
            user_prompt: String::new(),
            temperature: 0.7,
            max_tokens: 2048,
        }
    }
}

/// LLM completion response
#[derive(Debug, Clone)]
pub struct LlmResponse {
    pub text: String,
    pub model: String,
    pub usage_tokens: u32,
    pub latency_ms: u64,
    pub finish_reason: String,
}

/// LLM driver trait — implement per provider
pub trait LlmDriver: std::fmt::Debug + Send + Sync {
    fn complete(&self, request: LlmRequest) -> Result<LlmResponse, String>;
    fn name(&self) -> &str;
}

/// Mock LLM driver for testing
#[derive(Debug, Clone)]
pub struct MockLlmDriver {
    pub name: String,
}

impl MockLlmDriver {
    pub fn new(name: &str) -> Self {
        Self { name: name.into() }
    }
}

impl LlmDriver for MockLlmDriver {
    fn complete(&self, request: LlmRequest) -> Result<LlmResponse, String> {
        Ok(LlmResponse {
            text: format!("Mock response to: {}", request.user_prompt),
            model: request.model,
            usage_tokens: 42,
            latency_ms: 100,
            finish_reason: "stop".into(),
        })
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// LLM device state
#[derive(Debug, Clone)]
pub struct LlmState {
    pub driver_name: String,
    pub total_requests: u64,
    pub total_tokens: u64,
    pub enable_tracking: bool,
}

impl LlmState {
    pub fn new(driver_name: &str) -> Self {
        Self {
            driver_name: driver_name.into(),
            total_requests: 0,
            total_tokens: 0,
            enable_tracking: true,
        }
    }

    pub fn record_request(&mut self, tokens: u32) {
        if self.enable_tracking {
            self.total_requests += 1;
            self.total_tokens += tokens as u64;
        }
    }

    pub fn report(&self) -> String {
        format!("llm:driver_{}_req_{}_tokens_{}", self.driver_name, self.total_requests, self.total_tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_llm_driver() {
        let driver = MockLlmDriver::new("test");
        let request = LlmRequest {
            user_prompt: "Hello".into(),
            ..Default::default()
        };
        let response = driver.complete(request).unwrap();
        assert!(response.text.contains("Mock response"));
    }

    #[test]
    fn test_llm_state_tracking() {
        let mut state = LlmState::new("test");
        state.record_request(100);
        state.record_request(200);
        assert_eq!(state.total_requests, 2);
        assert_eq!(state.total_tokens, 300);
    }

    #[test]
    fn test_llm_provider_names() {
        assert_eq!(LlmProvider::OpenAI.name(), "openai");
        assert_eq!(LlmProvider::Anthropic.name(), "anthropic");
    }
}
