//! Provider 工厂和配置

use super::anthropic::AnthropicProvider;
use super::gemini::GeminiProvider;
use super::identity_council::global_council;
use super::ollama::OllamaProvider;
use super::openai::OpenAiProvider;
use super::types::LlmProvider;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LlmProviderType {
    OpenAI,
    Anthropic,
    Gemini,
    Ollama,
    FreeApi,      // 自动发现的免费API
    OpencodeFree, // OpenCode 免费层模型
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
    // ShieldEnforcer governance check for provider access
    if let Ok(shield) = crate::cli::shield_enforcer::global_shield().lock() {
        let domain = config.base_url.as_deref().unwrap_or("api.anthropic.com");
        match shield.policy.decide("network_request") {
            crate::neotrix::nt_shield::policy::PolicyDecision::Allow => {}
            crate::neotrix::nt_shield::policy::PolicyDecision::RequireConfirmation => {
                log::info!(
                    "[shield] Provider access requires confirmation for {}",
                    domain
                );
            }
            crate::neotrix::nt_shield::policy::PolicyDecision::Deny => {
                if !shield.policy.is_domain_allowed(domain) {
                    log::warn!("[shield] Provider domain '{}' not in allowlist, but allowing (R6 non-blocking)", domain);
                }
            }
        }
    }
    match config.provider_type {
        LlmProviderType::OpenAI => {
            let api_key = config
                .api_key
                .unwrap_or_else(|| std::env::var("OPENAI_API_KEY").unwrap_or_default());
            let mut provider = OpenAiProvider::new(api_key);
            if let Some(url) = config.base_url {
                provider = provider.with_base_url(&url);
            }
            Box::new(provider)
        }
        LlmProviderType::Anthropic => {
            let api_key = config
                .api_key
                .unwrap_or_else(|| std::env::var("ANTHROPIC_API_KEY").unwrap_or_default());
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
            let base_url = config
                .base_url
                .unwrap_or_else(|| "https://api.groq.com/openai/v1".to_string());
            let model = config
                .model
                .unwrap_or_else(|| "llama-3.3-70b-versatile".to_string());
            let council = global_council();
            let provider = super::AnonymousLlmProvider::new(
                council,
                &base_url,
                &model,
                "groq",
                config.api_key.or_else(|| std::env::var("GROQ_API_KEY").ok()),
            );
            Box::new(provider)
        }
        LlmProviderType::Gemini => {
            let api_key = config
                .api_key
                .unwrap_or_else(|| std::env::var("GOOGLE_API_KEY").unwrap_or_default());
            Box::new(GeminiProvider::new(api_key))
        }
        LlmProviderType::OpencodeFree => {
            let base_url = config
                .base_url
                .unwrap_or_else(|| "https://api.groq.com/openai/v1".to_string());
            let model = config
                .model
                .unwrap_or_else(|| "llama-3.3-70b-versatile".to_string());
            let council = global_council();
            let provider = super::AnonymousLlmProvider::new(
                council,
                &base_url,
                &model,
                "groq",
                config.api_key.or_else(|| std::env::var("GROQ_API_KEY").ok()),
            );
            Box::new(provider)
        }
    }
}

pub fn create_provider_from_type(
    provider_type: LlmProviderType,
    api_key: Option<String>,
) -> Box<dyn LlmProvider> {
    create_provider(ProviderConfig {
        provider_type,
        api_key,
        ..Default::default()
    })
}
