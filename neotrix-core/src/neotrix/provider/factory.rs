//! Provider 工厂和配置

use super::types::LlmProvider;
use super::openai::OpenAiProvider;
use super::anthropic::AnthropicProvider;
use super::ollama::OllamaProvider;
use super::gemini::GeminiProvider;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LlmProviderType {
    OpenAI,
    Anthropic,
    Gemini,
    Ollama,
    FreeApi,  // 自动发现的免费API
}

#[derive(Debug, Clone)]
pub struct ProviderConfig {
    pub provider_type: LlmProviderType,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub model: Option<String>,
    pub timeout_secs: u64,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            provider_type: LlmProviderType::Anthropic,
            api_key: None,
            base_url: None,
            model: None,
            timeout_secs: 120,
        }
    }
}

impl ProviderConfig {
    pub fn from_env() -> Self {
        let provider = std::env::var("NEOTRIX_PROVIDER")
            .unwrap_or_else(|_| "anthropic".to_string())
            .to_lowercase();

        let provider_type = match provider.as_str() {
            "openai" => LlmProviderType::OpenAI,
            "anthropic" => LlmProviderType::Anthropic,
            "gemini" => LlmProviderType::Gemini,
            "ollama" => LlmProviderType::Ollama,
            _ => LlmProviderType::Anthropic,
        };

        Self {
            provider_type,
            api_key: std::env::var("NEOTRIX_API_KEY").ok(),
            base_url: std::env::var("NEOTRIX_BASE_URL").ok(),
            model: std::env::var("NEOTRIX_MODEL").ok(),
            timeout_secs: std::env::var("NEOTRIX_TIMEOUT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(120),
        }
    }
}

pub fn create_provider(config: ProviderConfig) -> Box<dyn LlmProvider> {
    match config.provider_type {
        LlmProviderType::OpenAI => {
            let api_key = config.api_key.unwrap_or_else(|| {
                std::env::var("OPENAI_API_KEY").unwrap_or_default()
            });
            let mut provider = OpenAiProvider::new(api_key);
            if let Some(url) = config.base_url {
                provider = provider.with_base_url(&url);
            }
            Box::new(provider)
        }
        LlmProviderType::Anthropic => {
            let api_key = config.api_key.unwrap_or_else(|| {
                std::env::var("ANTHROPIC_API_KEY").unwrap_or_default()
            });
            Box::new(AnthropicProvider::new(api_key))
        }
        LlmProviderType::Ollama => {
            let mut provider = OllamaProvider::new();
            if let Some(url) = config.base_url {
                provider = provider.with_base_url(&url);
            }
            Box::new(provider)
        }
        LlmProviderType::FreeApi => {
            // FreeApi 使用 Ollama 作为回退
            let provider = OllamaProvider::new();
            Box::new(provider)
        }
        LlmProviderType::Gemini => {
            let api_key = config.api_key.unwrap_or_else(|| {
                std::env::var("GOOGLE_API_KEY").unwrap_or_default()
            });
            Box::new(GeminiProvider::new(api_key))
        }
    }
}

pub fn create_provider_from_type(provider_type: LlmProviderType, api_key: Option<String>) -> Box<dyn LlmProvider> {
    create_provider(ProviderConfig {
        provider_type,
        api_key,
        ..Default::default()
    })
}
