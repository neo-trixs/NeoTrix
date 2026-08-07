//! Provider 工厂和配置
//!
//! 2026-07-04: ProviderCatalog 自动注册 — 自我/客体分离
//! - Local (主体): Ollama/LM Studio/llama.cpp 自动探测注册
//! - Proxy (客体): 自定义 OpenAI 兼容代理通过 NEOTRIX_PROXY_* 注册
//! - Cloud (客体): 主流 API 通过各自 env var 自动注册

use super::types::LlmProvider;
use super::openai::OpenAiProvider;
use super::anthropic::AnthropicProvider;
use super::ollama::OllamaProvider;
use super::gemini::GeminiProvider;
use super::free_catalog::FreeModelCatalog;
use super::free_providers::{GroqProvider, OpenRouterProvider, PollinationsProvider, CerebrasProvider};
use super::gateway::GatewayV2;
use super::provider_catalog::{ProviderCategory, CommunicationProfile};
use crate::core::nt_io_telemetry::CostTracker;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub enum LlmProviderType {
    OpenAI,
    Anthropic,
    Gemini,
    Ollama,
    Groq,
    OpenRouter,
    Cerebras,
    SambaNova,
    Pollinations,
    BazaarLink,
    FreeTheAi,
    ZeroLimit,
    FreeApi,
    CustomProxy,
    // New free providers
    Cloudflare,
    Nvidia,
    GitHubModels,
    HuggingFace,
    Cohere,
    TogetherFree,
    Llm7,
    Kilo,
    SiliconFlow,
    ZAI,
    OpenCodeZen,
    Ovh,
    DeepSeekFree,
    ModelScope,
    ApiAirforce,
    Vllm,
    Sglang,
}

impl LlmProviderType {
    /// Map a provider name string to its LlmProviderType variant
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "openai" => Some(Self::OpenAI),
            "anthropic" => Some(Self::Anthropic),
            "gemini" => Some(Self::Gemini),
            "ollama" => Some(Self::Ollama),
            "groq" => Some(Self::Groq),
            "openrouter" => Some(Self::OpenRouter),
            "cerebras" => Some(Self::Cerebras),
            "sambanova" => Some(Self::SambaNova),
            "pollinations" => Some(Self::Pollinations),
            "bazaarlink" => Some(Self::BazaarLink),
            "freetheai" => Some(Self::FreeTheAi),
            "zerolimit" => Some(Self::ZeroLimit),
            "cloudflare" => Some(Self::Cloudflare),
            "nvidia" => Some(Self::Nvidia),
            "github-models" | "github_models" => Some(Self::GitHubModels),
            "huggingface" | "hf" => Some(Self::HuggingFace),
            "cohere" => Some(Self::Cohere),
            "together-free" | "together_free" => Some(Self::TogetherFree),
            "llm7" => Some(Self::Llm7),
            "kilo" => Some(Self::Kilo),
            "siliconflow" => Some(Self::SiliconFlow),
            "zai" | "z.ai" => Some(Self::ZAI),
            "opencode-zen" | "opencode_zen" => Some(Self::OpenCodeZen),
            "ovh" => Some(Self::Ovh),
            "deepseek-free" | "deepseek_free" => Some(Self::DeepSeekFree),
            "modelscope" => Some(Self::ModelScope),
            "api-airforce" | "api_airforce" => Some(Self::ApiAirforce),
            "vllm" => Some(Self::Vllm),
            "sglang" => Some(Self::Sglang),
            _ => None,
        }
    }

    pub fn is_free(self) -> bool {
        matches!(self,
            Self::Gemini | Self::Groq | Self::OpenRouter | Self::Cerebras |
            Self::SambaNova | Self::Pollinations | Self::BazaarLink | Self::FreeTheAi |
            Self::ZeroLimit | Self::FreeApi | Self::Ollama |
            Self::Cloudflare | Self::Nvidia | Self::GitHubModels | Self::HuggingFace |
            Self::TogetherFree | Self::Llm7 | Self::Kilo | Self::SiliconFlow |
            Self::ZAI | Self::OpenCodeZen | Self::Ovh | Self::DeepSeekFree | Self::ModelScope |
            Self::ApiAirforce
        )
    }

    pub fn needs_api_key(self) -> bool {
        matches!(self,
            Self::OpenAI | Self::Anthropic | Self::Gemini | Self::Groq |
            Self::OpenRouter | Self::Cerebras | Self::SambaNova | Self::BazaarLink |
            Self::ZeroLimit | Self::CustomProxy |
            Self::Cloudflare | Self::Nvidia | Self::GitHubModels | Self::HuggingFace |
            Self::Cohere | Self::TogetherFree | Self::SiliconFlow | Self::ZAI |
            Self::DeepSeekFree | Self::OpenCodeZen | Self::FreeTheAi
        )
    }

    pub fn category(self) -> ProviderCategory {
        match self {
            Self::Ollama | Self::Vllm | Self::Sglang => ProviderCategory::Local,
            Self::CustomProxy => ProviderCategory::Proxy,
            _ => ProviderCategory::Cloud,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProviderConfig {
    pub provider_type: LlmProviderType,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub model: Option<String>,
    pub timeout_secs: u64,
    /// 代理注入: 设置后将 provider 的 HTTP 客户端切换到代理路由 (子母阵 Proxied/Tor 画像)
    pub proxy: Option<String>,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            provider_type: LlmProviderType::Anthropic,
            api_key: None,
            base_url: None,
            model: None,
            timeout_secs: 120,
            proxy: None,
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
            "groq" => LlmProviderType::Groq,
            "openrouter" => LlmProviderType::OpenRouter,
            "cerebras" => LlmProviderType::Cerebras,
            "pollinations" => LlmProviderType::Pollinations,
            "bazaarlink" => LlmProviderType::BazaarLink,
            "freetheai" => LlmProviderType::FreeTheAi,
            "zerolimit" => LlmProviderType::ZeroLimit,
            "proxy" | "custom-proxy" => LlmProviderType::CustomProxy,
            "free" | "freeapi" => LlmProviderType::FreeApi,
            "cloudflare" => LlmProviderType::Cloudflare,
            "nvidia" => LlmProviderType::Nvidia,
            "github-models" | "github_models" => LlmProviderType::GitHubModels,
            "huggingface" | "hf" => LlmProviderType::HuggingFace,
            "cohere" => LlmProviderType::Cohere,
            "together-free" | "together_free" => LlmProviderType::TogetherFree,
            "llm7" => LlmProviderType::Llm7,
            "kilo" => LlmProviderType::Kilo,
            "siliconflow" => LlmProviderType::SiliconFlow,
            "zai" | "z.ai" => LlmProviderType::ZAI,
            "opencode-zen" | "opencode_zen" => LlmProviderType::OpenCodeZen,
            "ovh" => LlmProviderType::Ovh,
            "deepseek-free" | "deepseek_free" => LlmProviderType::DeepSeekFree,
            "modelscope" => LlmProviderType::ModelScope,
            "api-airforce" | "api_airforce" => LlmProviderType::ApiAirforce,
            "vllm" => LlmProviderType::Vllm,
            "sglang" => LlmProviderType::Sglang,
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
            proxy: super::super::nt_io_http_factory::proxy_from_env(),
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
                log::info!("[shield] Provider access requires confirmation for {}", domain);
            }
            crate::neotrix::nt_shield::policy::PolicyDecision::Deny => {
                if !shield.policy.is_domain_allowed(domain) {
                    log::warn!("[shield] Provider domain '{}' not in allowlist, but allowing (R6 non-blocking)", domain);
                }
            }
        }
    }
    let mut provider: Box<dyn LlmProvider> = match config.provider_type {
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
        LlmProviderType::Vllm => {
            // vLLM serves an OpenAI-compatible API. Self-hosted: base URL defaults
            // to the standard vLLM endpoint (override with NEOTRIX_VLLM_BASE_URL).
            let api_key = config.api_key.unwrap_or_else(|| {
                std::env::var("NEOTRIX_VLLM_API_KEY").unwrap_or_else(|_| "local".to_string())
            });
            let base_url = config.base_url.unwrap_or_else(|| {
                std::env::var("NEOTRIX_VLLM_BASE_URL").unwrap_or_else(|_| "http://localhost:8000/v1".to_string())
            });
            let mut provider = OpenAiProvider::new(api_key);
            provider = provider.with_base_url(&base_url);
            Box::new(provider)
        }
        LlmProviderType::Sglang => {
            // SGLang serves an OpenAI-compatible API. Self-hosted: base URL defaults
            // to the standard SGLang endpoint (override with NEOTRIX_SGLANG_BASE_URL).
            let api_key = config.api_key.unwrap_or_else(|| {
                std::env::var("NEOTRIX_SGLANG_API_KEY").unwrap_or_else(|_| "local".to_string())
            });
            let base_url = config.base_url.unwrap_or_else(|| {
                std::env::var("NEOTRIX_SGLANG_BASE_URL").unwrap_or_else(|_| "http://localhost:30000/v1".to_string())
            });
            let mut provider = OpenAiProvider::new(api_key);
            provider = provider.with_base_url(&base_url);
            Box::new(provider)
        }
        LlmProviderType::Gemini => {
            let api_key = config.api_key.unwrap_or_else(|| {
                std::env::var("GOOGLE_API_KEY").unwrap_or_default()
            });
            Box::new(GeminiProvider::new(api_key))
        }
        LlmProviderType::Groq => {
            let api_key = config.api_key.unwrap_or_else(|| {
                std::env::var("GROQ_API_KEY").unwrap_or_default()
            });
            let mut provider = GroqProvider::new(api_key);
            if let Some(url) = config.base_url {
                provider = provider.with_base_url(&url);
            }
            Box::new(provider)
        }
        LlmProviderType::OpenRouter => {
            let api_key = config.api_key.unwrap_or_else(|| {
                std::env::var("OPENROUTER_API_KEY").unwrap_or_default()
            });
            let mut provider = OpenRouterProvider::new(api_key);
            if let Some(url) = config.base_url {
                provider = provider.with_base_url(&url);
            }
            Box::new(provider)
        }
        LlmProviderType::Cerebras => {
            let api_key = config.api_key.unwrap_or_else(|| {
                std::env::var("CEREBRAS_API_KEY").unwrap_or_default()
            });
            Box::new(CerebrasProvider::new(api_key))
        }
        LlmProviderType::Pollinations | LlmProviderType::FreeApi => {
            Box::new(PollinationsProvider::new())
        }
        LlmProviderType::BazaarLink => {
            let api_key = config.api_key.unwrap_or_else(|| {
                std::env::var("BAZAARLINK_API_KEY").unwrap_or_default()
            });
            let mut provider = OpenAiProvider::new(api_key);
            provider = provider.with_base_url("https://api.bazaarlink.ai/v1");
            Box::new(provider)
        }
        LlmProviderType::FreeTheAi => {
            // Keyless — uses community API, OpenAI-compatible
            let mut provider = OpenAiProvider::new(String::new());
            provider = provider.with_base_url("https://api.freetheai.com/v1");
            Box::new(provider)
        }
        LlmProviderType::ZeroLimit => {
            let api_key = config.api_key.unwrap_or_else(|| {
                std::env::var("ZEROLIMIT_API_KEY").unwrap_or_default()
            });
            let mut provider = OpenAiProvider::new(api_key);
            provider = provider.with_base_url("https://api.zerolimit.ai/v1");
            Box::new(provider)
        }
        LlmProviderType::SambaNova => {
            let api_key = config.api_key.unwrap_or_else(|| {
                std::env::var("SAMBANOVA_API_KEY").unwrap_or_default()
            });
            let base_url = config.base_url.unwrap_or_else(|| "https://api.sambanova.ai/v1".to_string());
            let mut provider = GroqProvider::new(api_key);
            provider = provider.with_base_url(&base_url);
            Box::new(provider)
        }
        LlmProviderType::CustomProxy => {
            let api_key = config.api_key.unwrap_or_else(|| {
                std::env::var("NEOTRIX_PROXY_API_KEY").unwrap_or_default()
            });
            let mut provider = OpenAiProvider::new(api_key);
            let base_url = config.base_url.unwrap_or_else(|| {
                std::env::var("NEOTRIX_PROXY_BASE_URL").unwrap_or_else(|_| "http://localhost:3000/v1".to_string())
            });
            provider = provider.with_base_url(&base_url);
            Box::new(provider)
        }

        // ── Free cloud providers ──
        LlmProviderType::Cloudflare => {
            let api_key = config.api_key.unwrap_or_else(|| {
                std::env::var("CLOUDFLARE_API_TOKEN").unwrap_or_default()
            });
            let base_url = config.base_url.unwrap_or_else(|| "https://api.cloudflare.com/client/v4/ai".to_string());
            let mut provider = OpenAiProvider::new(api_key);
            provider = provider.with_base_url(&base_url);
            Box::new(provider)
        }
        LlmProviderType::Nvidia => {
            let api_key = config.api_key.unwrap_or_else(|| {
                std::env::var("NVIDIA_API_KEY").unwrap_or_default()
            });
            let base_url = config.base_url.unwrap_or_else(|| "https://integrate.api.nvidia.com/v1".to_string());
            let mut provider = OpenAiProvider::new(api_key);
            provider = provider.with_base_url(&base_url);
            Box::new(provider)
        }
        LlmProviderType::GitHubModels => {
            let api_key = config.api_key.unwrap_or_else(|| {
                std::env::var("GITHUB_TOKEN").unwrap_or_default()
            });
            let base_url = config.base_url.unwrap_or_else(|| "https://models.inference.ai.azure.com/v1".to_string());
            let mut provider = OpenAiProvider::new(api_key);
            provider = provider.with_base_url(&base_url);
            Box::new(provider)
        }
        LlmProviderType::HuggingFace => {
            let api_key = config.api_key.unwrap_or_else(|| {
                std::env::var("HF_API_KEY").unwrap_or_default()
            });
            let base_url = config.base_url.unwrap_or_else(|| "https://api-inference.huggingface.co/v1".to_string());
            let mut provider = OpenAiProvider::new(api_key);
            provider = provider.with_base_url(&base_url);
            Box::new(provider)
        }
        LlmProviderType::Cohere => {
            let api_key = config.api_key.unwrap_or_else(|| {
                std::env::var("COHERE_API_KEY").unwrap_or_default()
            });
            let base_url = config.base_url.unwrap_or_else(|| "https://api.cohere.ai/v1".to_string());
            let mut provider = OpenAiProvider::new(api_key);
            provider = provider.with_base_url(&base_url);
            Box::new(provider)
        }
        LlmProviderType::TogetherFree => {
            let api_key = config.api_key.unwrap_or_else(|| {
                std::env::var("TOGETHER_API_KEY").unwrap_or_default()
            });
            let base_url = config.base_url.unwrap_or_else(|| "https://api.together.xyz/v1".to_string());
            let mut provider = OpenAiProvider::new(api_key);
            provider = provider.with_base_url(&base_url);
            Box::new(provider)
        }
        LlmProviderType::Llm7 => {
            // .ai 域名已死（HTTP 000）；.io 是当前匿名可用端点（2026-08 实测 200）
            let base_url = config.base_url.unwrap_or_else(|| "https://api.llm7.io/v1".to_string());
            let mut provider = OpenAiProvider::new(String::new());
            provider = provider.with_base_url(&base_url);
            Box::new(provider)
        }
        LlmProviderType::Kilo => {
            let base_url = config.base_url.unwrap_or_else(|| "https://api.kilocode.ai/v1".to_string());
            let mut provider = OpenAiProvider::new(String::new());
            provider = provider.with_base_url(&base_url);
            Box::new(provider)
        }
        LlmProviderType::SiliconFlow => {
            let api_key = config.api_key.unwrap_or_else(|| {
                std::env::var("SILICONFLOW_API_KEY").unwrap_or_default()
            });
            let base_url = config.base_url.unwrap_or_else(|| "https://api.siliconflow.cn/v1".to_string());
            let mut provider = OpenAiProvider::new(api_key);
            provider = provider.with_base_url(&base_url);
            Box::new(provider)
        }
        LlmProviderType::ZAI => {
            let api_key = config.api_key.unwrap_or_else(|| {
                std::env::var("ZAI_API_KEY").unwrap_or_default()
            });
            let base_url = config.base_url.unwrap_or_else(|| "https://open.bigmodel.cn/api/paas/v4".to_string());
            let mut provider = OpenAiProvider::new(api_key);
            provider = provider.with_base_url(&base_url);
            Box::new(provider)
        }
        LlmProviderType::OpenCodeZen => {
            let api_key = config.api_key.unwrap_or_else(|| {
                std::env::var("OPENCODE_API_KEY").unwrap_or_default()
            });
            let base_url = config.base_url.unwrap_or_else(|| {
                std::env::var("NEOTRIX_ZEN_URL").unwrap_or_else(|_| "https://opencode.ai/zen/v1".to_string())
            });
            let mut provider = OpenAiProvider::new(api_key);
            provider = provider.with_base_url(&base_url);
            Box::new(provider)
        }
        LlmProviderType::Ovh => {
            let base_url = config.base_url.unwrap_or_else(|| "https://ai-endpoints.ovh.net/v1".to_string());
            let mut provider = OpenAiProvider::new(String::new());
            provider = provider.with_base_url(&base_url);
            Box::new(provider)
        }
        LlmProviderType::DeepSeekFree => {
            let api_key = config.api_key.unwrap_or_else(|| {
                std::env::var("DEEPSEEK_API_KEY").unwrap_or_default()
            });
            let base_url = config.base_url.unwrap_or_else(|| "https://api.deepseek.com/v1".to_string());
            let mut provider = OpenAiProvider::new(api_key);
            provider = provider.with_base_url(&base_url);
            Box::new(provider)
        }
        LlmProviderType::ModelScope => {
            let base_url = config.base_url.unwrap_or_else(|| "https://api.modelscope.cn/v1".to_string());
            let mut provider = OpenAiProvider::new(String::new());
            provider = provider.with_base_url(&base_url);
            Box::new(provider)
        }
        LlmProviderType::ApiAirforce => {
            // Truly keyless — accepts any Bearer token, even empty/not-needed
            // Verified working 2026-07-22: 209+ free models with `:free` suffix
            let base_url = config.base_url.unwrap_or_else(|| "https://api.airforce/v1".to_string());
            let api_key = config.api_key.unwrap_or_default();
            let mut provider = OpenAiProvider::new(api_key);
            provider = provider.with_base_url(&base_url);
            Box::new(provider)
        }
    };

    // 代理注入: 若配置了代理 (子母阵 Proxied/Tor 画像), 将 provider 客户端切换到代理路由
    if let Some(proxy_url) = &config.proxy {
        provider.set_proxy(proxy_url);
        log::info!("[factory] provider {:?} routed through proxy {}", config.provider_type, proxy_url);
    }
    provider
}

pub fn create_provider_from_type(provider_type: LlmProviderType, api_key: Option<String>) -> Box<dyn LlmProvider> {
    create_provider(ProviderConfig {
        provider_type,
        api_key,
        ..Default::default()
    })
}

/// 探测本地 Ollama 端点是否可达
async fn probe_ollama() -> bool {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
        .ok();
    let client = match client {
        Some(c) => c,
        None => return false,
    };
    match client.head("http://localhost:11434/api/tags").send().await {
        Ok(resp) => resp.status().is_success(),
        Err(_) => false,
    }
}

/// 构建统一网关 — 自动注册所有可用提供者
///
/// 注册策略:
/// 1. Local (主体): 自动探测 Ollama/LM Studio/llama.cpp/vLLM 本地端点
/// 2. Proxy (客体): 如果配置了 NEOTRIX_PROXY_BASE_URL, 注册自定义代理
/// 3. Cloud (客体): 根据环境变量自动注册所有可用云端 API
pub async fn create_gateway_async() -> GatewayV2 {
    let mut gateway = GatewayV2::new();

    // ── 1. Local (主体): 自动探测本地推理端点 ──
    if probe_ollama().await {
        let ollama = create_provider_from_type(LlmProviderType::Ollama, None);
        gateway.register_provider_with_category("ollama", ollama, true, ProviderCategory::Local);
        log::info!("[gateway] Auto-registered: ollama (local)");
    }

    // ── 2. Proxy (客体): 自定义 OpenAI 兼容代理 ──
    let proxy_base_url = std::env::var("NEOTRIX_PROXY_BASE_URL").ok();
    let proxy_api_key = std::env::var("NEOTRIX_PROXY_API_KEY").ok();
    if let Some(url) = proxy_base_url {
        if !url.is_empty() {
            let provider = create_provider(ProviderConfig {
                provider_type: LlmProviderType::CustomProxy,
                api_key: proxy_api_key,
                base_url: Some(url.clone()),
                ..Default::default()
            });
            gateway.register_provider_with_category("custom-proxy", provider, false, ProviderCategory::Proxy);
            log::info!("[gateway] Auto-registered: custom-proxy ({})", url);
        }
    }

    // ── 3. Cloud (客体): 根据环境变量自动注册 ──
    macro_rules! register_if {
        ($var:expr, $name:expr, $provider_type:expr, $is_free:expr) => {
            if let Ok(key) = std::env::var($var) {
                if !key.is_empty() {
                    let provider = create_provider_from_type($provider_type, Some(key));
                    gateway.register_provider_with_category($name, provider, $is_free, ProviderCategory::Cloud);
                    log::info!("[gateway] Auto-registered: {} (cloud)", $name);
                }
            }
        };
    }

    register_if!("OPENAI_API_KEY", "openai", LlmProviderType::OpenAI, false);
    register_if!("ANTHROPIC_API_KEY", "anthropic", LlmProviderType::Anthropic, false);
    register_if!("GOOGLE_API_KEY", "gemini", LlmProviderType::Gemini, true);
    register_if!("GROQ_API_KEY", "groq", LlmProviderType::Groq, true);
    register_if!("OPENROUTER_API_KEY", "openrouter", LlmProviderType::OpenRouter, true);
    register_if!("CEREBRAS_API_KEY", "cerebras", LlmProviderType::Cerebras, true);
    register_if!("SAMBANOVA_API_KEY", "sambanova", LlmProviderType::SambaNova, true);
    register_if!("BAZAARLINK_API_KEY", "bazaarlink", LlmProviderType::BazaarLink, true);
    register_if!("ZEROLIMIT_API_KEY", "zerolimit", LlmProviderType::ZeroLimit, true);
    register_if!("CLOUDFLARE_API_TOKEN", "cloudflare", LlmProviderType::Cloudflare, true);
    register_if!("NVIDIA_API_KEY", "nvidia", LlmProviderType::Nvidia, true);
    register_if!("GITHUB_TOKEN", "github-models", LlmProviderType::GitHubModels, true);
    register_if!("HF_API_KEY", "huggingface", LlmProviderType::HuggingFace, true);
    register_if!("COHERE_API_KEY", "cohere", LlmProviderType::Cohere, false);
    register_if!("TOGETHER_API_KEY", "together-free", LlmProviderType::TogetherFree, true);
    register_if!("SILICONFLOW_API_KEY", "siliconflow", LlmProviderType::SiliconFlow, true);
    register_if!("ZAI_API_KEY", "zai", LlmProviderType::ZAI, true);
    register_if!("DEEPSEEK_API_KEY", "deepseek-free", LlmProviderType::DeepSeekFree, true);
    register_if!("OPENCODE_API_KEY", "opencode-zen", LlmProviderType::OpenCodeZen, true);
    register_if!("FREETHEAI_API_KEY", "freetheai", LlmProviderType::FreeTheAi, true);

    // ── 4. FreeModelCatalog: 从目录中发现并注册所有可用免费模型 ──
    // Use spawn_blocking to avoid tokio 1.52+ panic when reqwest::blocking drops
    // its internal Runtime while already inside a block_on context.
    let mut catalog = FreeModelCatalog::new();
    let discovered = tokio::task::spawn_blocking(move || catalog.refresh()).await.unwrap_or_default();
    let registered_count = discovered.len();
    gateway.register_from_catalog(&discovered);
    if !discovered.is_empty() {
        log::info!("[gateway] FreeModelCatalog: {} entries discovered, registered those with keys", registered_count);
    }

    // 始终注册 keyless 免费提供者
    // 代理注入: 本机常为 fake-ip 分流网络 (如 198.18.0.x + 系统代理), 直连会全部超时,
    // 因此统一把 NEOTRIX_PROXY_URL / NEOTRIX_TOR_PROXY 注入每个 keyless provider 客户端。
    let proxy = super::super::nt_io_http_factory::proxy_from_env();
    let mut keyless_provider = |ptype: LlmProviderType| {
        let mut p = create_provider_from_type(ptype, None);
        if let Some(proxy_url) = &proxy {
            p.set_proxy(proxy_url);
            log::debug!("[gateway] keyless provider {:?} routed through proxy {}", ptype, proxy_url);
        }
        p
    };

    let pollinations = PollinationsProvider::new();
    let mut pollinations: Box<dyn LlmProvider> = Box::new(pollinations);
    if let Some(proxy_url) = &proxy {
        pollinations.set_proxy(proxy_url);
    }
    gateway.register_provider_with_category("pollinations", pollinations, true, ProviderCategory::Cloud);
    log::info!("[gateway] Registered keyless: pollinations");

    // LLM7 — 匿名 keyless（Bearer unused 即可），turbo 层模型（gpt-oss:20b 等），~30 RPM。
    // 2026-08 实测 .io 端点 200 可用；.ai 旧域名已死。
    gateway.register_provider_with_category("llm7", keyless_provider(LlmProviderType::Llm7), true, ProviderCategory::Cloud);
    log::info!("[gateway] Registered keyless: llm7 (api.llm7.io, anonymous turbo models)");

    let api_airforce = keyless_provider(LlmProviderType::ApiAirforce);
    gateway.register_provider_with_category("api-airforce", api_airforce, true, ProviderCategory::Cloud);
    log::info!("[gateway] Registered keyless: api-airforce (api.airforce, 254+ models; 实测 POST 需真 key 时返回 401)");

    // ── free_pool 已断言 budget 的 keyless 提供者 (类型实现齐全, 此处补接线) ──
    // 2026-08-06 走代理实测:
    //   llm7(api.llm7.io)           ✅ 匿名可用 (已在上面注册)
    //   kilo(api.kilocode.ai)       ❌ HTML 404 端点已死 → 不注册
    //   opencode-zen(opencode.ai)   ❌ POST 需 API key (AuthError) → 不注册
    //   ovh(modelscope/freetheai)   ❌ DNS 不可达 (fake-ip 未命中) → 不注册
    // 结论: 当前真 keyless 仅 llm7 + pollinations(匿名层已关, 探测项)。

    // Install CostTracker for per-query budget enforcement
    let tracker = CostTracker::new();
    gateway.set_cost_tracker(tracker);

    // ── 5. SubGrid Auto-Composition (子母阵自动组合) ──
    // 基于已注册 provider 的通信安全画像，自动组合三个默认子网格:
    //   - anonymous-local: 最高隐匿 (本地主体, 数据不出设备)
    //   - proxied: 元数据隐匿 (自定义代理)
    //   - open: 标准 HTTPS (云端 API)
    // 调用方可通过 select_best_for_profile() 按需路由到对应子网格
    gateway.compose_sub_grid("anonymous-local", CommunicationProfile::Anonymous, true);
    gateway.compose_sub_grid("proxied", CommunicationProfile::Proxied, true);
    gateway.compose_sub_grid("open", CommunicationProfile::Open, false);
    log::info!("[gateway] SubGrid auto-composed: anonymous-local / proxied / open");

    gateway
}

/// 同步版本 — 保留向后兼容 (内部调用 block_on)
/// 如果已通过 Handle::try_current 或 enter() 存在 runtime 上下文，使用它；
/// 否则创建新 runtime 避免嵌套 runtime 冲突。
pub fn create_gateway() -> GatewayV2 {
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        handle.block_on(create_gateway_async())
    } else {
        let rt = match tokio::runtime::Runtime::new() {
            Ok(rt) => rt,
            Err(e) => {
                log::error!("[gateway] failed to create tokio runtime: {e}");
                std::process::exit(1);
            }
        };
        rt.block_on(create_gateway_async())
    }
}
