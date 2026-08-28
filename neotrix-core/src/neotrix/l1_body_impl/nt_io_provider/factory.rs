//! Provider 工厂和配置
//!
//! 2026-07-04: ProviderCatalog 自动注册 — 自我/客体分离
//! - Local (主体): Ollama/LM Studio/llama.cpp 自动探测注册
//! - Proxy (客体): 自定义 OpenAI 兼容代理通过 NEOTRIX_PROXY_* 注册
//! - Cloud (客体): 主流 API 通过各自 env var 自动注册
//!
//! 2026-08: 网络隔离 (默认阻断) — 非白名单/非本地端点一律返回 DeniedProvider。
//! 逃生门: NEOTRIX_NETWORK_UNBLOCK=1 显式放行, 或切换 shield profile 为 general。

use super::types::{LlmProvider, LlmRequest, LlmResponse, LlmError};
use super::openai::OpenAiProvider;
use super::anthropic::AnthropicProvider;
use super::ollama::OllamaProvider;
use super::gemini::GeminiProvider;
use super::free_catalog::FreeModelCatalog;
use super::free_providers::{GroqProvider, OpenRouterProvider, PollinationsProvider, CerebrasProvider};
use super::gateway::GatewayV2;
use super::provider_catalog::{ProviderCategory, CommunicationProfile};
use crate::core::nt_core_span::CostTracker;
use std::sync::Arc;

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
    Empero,
    Vllm,
    Sglang,
    Aihub,
    Xai,
    Moonshot,
    Qwen,
    Doubao,
    MiniMax,
    Perplexity,
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
            "empero" | "free-empero" | "free_empero" => Some(Self::Empero),
            "vllm" => Some(Self::Vllm),
            "sglang" => Some(Self::Sglang),
            "aihub" | "aihub.humorously.cn" => Some(Self::Aihub),
            "xai" | "grok" => Some(Self::Xai),
            "moonshot" | "kimi" => Some(Self::Moonshot),
            "qwen" | "dashscope" => Some(Self::Qwen),
            "doubao" | "ark" => Some(Self::Doubao),
            "minimax" => Some(Self::MiniMax),
            "perplexity" | "pplx" => Some(Self::Perplexity),
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
            Self::ApiAirforce | Self::Empero
        )
    }

    pub fn needs_api_key(self) -> bool {
        matches!(self,
            Self::OpenAI | Self::Anthropic | Self::Gemini | Self::Groq |
            Self::OpenRouter | Self::Cerebras | Self::SambaNova | Self::BazaarLink |
            Self::ZeroLimit | Self::CustomProxy |
            Self::Cloudflare | Self::Nvidia | Self::GitHubModels | Self::HuggingFace |
            Self::Cohere | Self::TogetherFree | Self::SiliconFlow | Self::ZAI |
            Self::DeepSeekFree | Self::OpenCodeZen | Self::FreeTheAi | Self::Aihub
        )
    }

    pub fn category(self) -> ProviderCategory {
        match self {
            Self::Ollama | Self::Vllm | Self::Sglang => ProviderCategory::Local,
            Self::CustomProxy => ProviderCategory::Proxy,
            Self::Aihub => ProviderCategory::Cloud,
            _ => ProviderCategory::Cloud,
        }
    }

    /// 是否本地推理 (数据不出设备)。`Ollama`/`Vllm`/`Sglang` 为 localhost 自托管。
    pub fn is_local(self) -> bool {
        matches!(self, Self::Ollama | Self::Vllm | Self::Sglang)
    }

    /// 规范名 (用于日志 / 隐私守卫错误信息)。反向映射 `from_name`。
    pub fn as_name(self) -> &'static str {
        match self {
            Self::OpenAI => "openai",
            Self::Anthropic => "anthropic",
            Self::Gemini => "gemini",
            Self::Ollama => "ollama",
            Self::Groq => "groq",
            Self::OpenRouter => "openrouter",
            Self::Cerebras => "cerebras",
            Self::SambaNova => "sambanova",
            Self::Pollinations => "pollinations",
            Self::BazaarLink => "bazaarlink",
            Self::FreeTheAi => "freetheai",
            Self::ZeroLimit => "zerolimit",
            Self::FreeApi => "freeapi",
            Self::CustomProxy => "custom-proxy",
            Self::Cloudflare => "cloudflare",
            Self::Nvidia => "nvidia",
            Self::GitHubModels => "github-models",
            Self::HuggingFace => "huggingface",
            Self::Cohere => "cohere",
            Self::TogetherFree => "together-free",
            Self::Llm7 => "llm7",
            Self::Kilo => "kilo",
            Self::SiliconFlow => "siliconflow",
            Self::ZAI => "zai",
            Self::OpenCodeZen => "opencode-zen",
            Self::Ovh => "ovh",
            Self::DeepSeekFree => "deepseek-free",
            Self::ModelScope => "modelscope",
            Self::ApiAirforce => "api-airforce",
            Self::Empero => "empero",
            Self::Vllm => "vllm",
            Self::Sglang => "sglang",
            Self::Aihub => "aihub",
            Self::Xai => "xai",
            Self::Moonshot => "moonshot",
            Self::Qwen => "qwen",
            Self::Doubao => "doubao",
            Self::MiniMax => "minimax",
            Self::Perplexity => "perplexity",
        }
    }

    /// 数据信任分级 — 隐私出网门控的核心依据 (R-P42 强化现有节点, 不建平行适配器)。
    ///
    /// - `Trusted`  : 本地推理, 数据不出设备, 无需脱敏。
    /// - `Contracted`: 付费云端 (OpenAI/Anthropic 等), 有"不拿 API 数据训练"商业条款,
    ///                 作为belt-and-suspenders 仍脱敏 NeoTrix 内部指纹。
    /// - `Untrusted`: 免费/代理端点 (xiaohuxing/llm7/pollinations/opencode-zen 等),
    ///                靠日志/数据回灌维持免费, 是"拿去喂模型训练"的真实载体 —
    ///                检测到 NeoTrix 内部指纹时必须阻断 (fail-closed) 或脱敏。
    pub fn data_trust(self) -> DataTrust {
        if self.is_local() {
            DataTrust::Trusted
        } else if self.is_free() || self.category() == ProviderCategory::Proxy {
            DataTrust::Untrusted
        } else {
            DataTrust::Contracted
        }
    }
}

/// 数据信任分级 — 复用 core 层定义 (`crate::core::nt_core_llm::DataTrust`),
/// 避免 core/neotrix 双定义 (core 不得依赖 neotrix, 故单一事实源在 core)。
pub use crate::core::nt_core_llm::DataTrust;

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
            "empero" | "free-empero" | "free_empero" => LlmProviderType::Empero,
            "vllm" => LlmProviderType::Vllm,
            "sglang" => LlmProviderType::Sglang,
            "aihub" | "aihub.humorously.cn" => LlmProviderType::Aihub,
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

/// 网络隔离拒绝型 provider — 默认策略下非白名单端点返回此类, 所有调用立即失败。
#[derive(Debug, Clone)]
pub struct DeniedProvider {
    pub host: String,
}

#[async_trait::async_trait]
impl LlmProvider for DeniedProvider {
    fn data_trust(&self) -> DataTrust {
        DataTrust::Untrusted
    }

    async fn complete_raw(&self, _request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        Err(LlmError::InvalidRequest(format!(
            "network access to '{}' is blocked by default isolation policy; \
             allowlist it or set NEOTRIX_NETWORK_UNBLOCK=1 to opt out",
            self.host
        )))
    }

    async fn stream_complete_raw(&self, _request: &LlmRequest) -> Result<tokio::sync::mpsc::Receiver<Result<LlmResponse, LlmError>>, LlmError> {
        Err(LlmError::InvalidRequest(format!(
            "network access to '{}' is blocked by default isolation policy",
            self.host
        )))
    }
}

/// 从 base_url 提取主机名 (剥离 scheme/path/port, 兼容 IPv6 括号)。
pub fn host_of(base_url: &str) -> String {
    let s = base_url.trim();
    let after_scheme = s.split_once("://").map(|x| x.1).unwrap_or(s);
    let authority = after_scheme.split(['/', '?', '#']).next().unwrap_or(after_scheme);
    let authority = authority.trim();
    if authority.starts_with('[') {
        if let Some(close) = authority.find(']') {
            return authority[..=close].to_string();
        }
        return authority.to_string();
    }
    authority.split(':').next().unwrap_or(authority).to_string()
}

/// 是否为本地/内网回环端点 (Local 主体 provider 直连)。
pub fn is_local_host(host: &str) -> bool {
    let h = host.trim().trim_start_matches('[').trim_end_matches(']').to_lowercase();
    matches!(h.as_str(), "localhost" | "127.0.0.1" | "::1" | "0.0.0.0")
        || h.starts_with("127.")
        || h.starts_with("10.")
        || h.starts_with("192.168.")
        || h.starts_with("172.1") || h.starts_with("172.2") || h.starts_with("172.3")
        || h.ends_with(".local")
}

/// provider 类型无显式 base_url 时的默认端点主机 (用于隔离判定)。
fn default_host(provider_type: LlmProviderType) -> Option<&'static str> {
    match provider_type {
        LlmProviderType::OpenAI => Some("api.openai.com"),
        LlmProviderType::Anthropic => Some("api.anthropic.com"),
        LlmProviderType::Gemini => Some("generativelanguage.googleapis.com"),
        LlmProviderType::Groq => Some("api.groq.com"),
        LlmProviderType::OpenRouter => Some("api.openrouter.ai"),
        LlmProviderType::Cerebras => Some("api.cerebras.ai"),
        LlmProviderType::SambaNova => Some("api.sambanova.ai"),
        LlmProviderType::Pollinations | LlmProviderType::FreeApi => Some("pollinations.ai"),
        LlmProviderType::BazaarLink => Some("api.bazaarlink.ai"),
        LlmProviderType::FreeTheAi => Some("api.freetheai.com"),
        LlmProviderType::ZeroLimit => Some("api.zerolimit.ai"),
        LlmProviderType::Cloudflare => Some("api.cloudflare.com"),
        LlmProviderType::Nvidia => Some("integrate.api.nvidia.com"),
        LlmProviderType::GitHubModels => Some("models.inference.ai.azure.com"),
        LlmProviderType::HuggingFace => Some("api-inference.huggingface.co"),
        LlmProviderType::Cohere => Some("api.cohere.ai"),
        LlmProviderType::TogetherFree => Some("api.together.xyz"),
        LlmProviderType::Llm7 => Some("api.llm7.io"),
        LlmProviderType::Kilo => Some("api.kilocode.ai"),
        LlmProviderType::SiliconFlow => Some("api.siliconflow.cn"),
        LlmProviderType::Xai => Some("api.x.ai"),
        LlmProviderType::Moonshot => Some("api.moonshot.cn"),
        LlmProviderType::Qwen => Some("dashscope.aliyuncs.com"),
        LlmProviderType::Doubao => Some("ark.cn-beijing.volces.com"),
        LlmProviderType::MiniMax => Some("api.minimax.chat"),
        LlmProviderType::Perplexity => Some("api.perplexity.ai"),
        LlmProviderType::ZAI => Some("open.bigmodel.cn"),
        LlmProviderType::OpenCodeZen => Some("opencode.ai"),
        LlmProviderType::Ovh => Some("ai-endpoints.ovh.net"),
        LlmProviderType::DeepSeekFree => Some("api.deepseek.com"),
        LlmProviderType::ModelScope => Some("api.modelscope.cn"),
        LlmProviderType::ApiAirforce => Some("api.airforce"),
        LlmProviderType::Empero => Some("free.empero.org"),
        LlmProviderType::Aihub => Some("aihub.humorously.cn"),
        // 本地主体: Ollama / vLLM / SGLang / 自定义代理默认走 localhost
        LlmProviderType::Ollama
        | LlmProviderType::Vllm
        | LlmProviderType::Sglang
        | LlmProviderType::CustomProxy => Some("localhost"),
    }
}

/// 网络隔离判定 — 默认阻断非白名单云端端点。
///
/// 放行条件 (任一):
/// 1. 端点为主机回环/内网/本地域 → 放行
/// 2. 域名在 shield 网络白名单 → 放行
/// 3. shield profile 判定 Allow / RequireConfirmation → 放行
/// 4. 显式逃生门 NEOTRIX_NETWORK_UNBLOCK=1 → 放行 (告警)
///
/// 阻断: 其余一律 DeniedProvider。shield 不可用时不静默放行 (安全默认)。
pub fn network_access_allowed(provider_type: LlmProviderType, base_url: Option<&str>) -> bool {
    let host = match base_url {
        Some(url) if !url.trim().is_empty() => host_of(url),
        _ => match default_host(provider_type) {
            Some(h) => h.to_string(),
            None => return true,
        },
    };
    if is_local_host(&host) {
        return true;
    }
    if let Ok(v) = std::env::var("NEOTRIX_NETWORK_UNBLOCK") {
        let v = v.trim().to_lowercase();
        if !v.is_empty() && v != "0" && v != "false" && v != "off" {
            log::warn!(
                "[network-isolation] NEOTRIX_NETWORK_UNBLOCK set — allowing unrestricted network to '{}'",
                host
            );
            return true;
        }
    }
    match crate::cli::shield_enforcer::global_shield().lock() {
        Ok(shield) => match shield.policy.evaluate_network(&host) {
            crate::neotrix::l1_body_impl::nt_shield::policy::PolicyDecision::Allow => true,
            crate::neotrix::l1_body_impl::nt_shield::policy::PolicyDecision::RequireConfirmation => {
                log::info!("[network-isolation] provider domain '{}' requires confirmation — allowing", host);
                true
            }
            crate::neotrix::l1_body_impl::nt_shield::policy::PolicyDecision::Deny => {
                log::warn!("[network-isolation] BLOCKED provider domain '{}' (not in allowlist, default deny)", host);
                false
            }
        },
        Err(_) => {
            log::warn!("[network-isolation] shield unavailable — default-deny for '{}'", host);
            false
        }
    }
}

pub fn create_provider(config: ProviderConfig) -> Arc<dyn LlmProvider> {
    // 网络隔离 (默认阻断): 非白名单/非本地端点 → DeniedProvider (显式逃生门见 network_access_allowed)
    if !network_access_allowed(config.provider_type, config.base_url.as_deref()) {
        let host = config
            .base_url
            .as_deref()
            .map(host_of)
            .or_else(|| default_host(config.provider_type).map(String::from))
            .unwrap_or_else(|| "unknown".to_string());
        log::warn!(
            "[factory] BLOCKED provider {:?}: host '{}' not in network allowlist (default isolation). \
             Add to allowlist or set NEOTRIX_NETWORK_UNBLOCK=1 to allow.",
            config.provider_type, host
        );
        return Arc::new(DeniedProvider { host });
    }
    let mut provider: Arc<dyn LlmProvider> = match config.provider_type {
        LlmProviderType::OpenAI => {
            let api_key = config.api_key.unwrap_or_else(|| {
                std::env::var("OPENAI_API_KEY").unwrap_or_default()
            });
            let mut provider = OpenAiProvider::new(api_key);
            if let Some(url) = config.base_url {
                provider = provider.with_base_url(&url);
            }
            Arc::new(provider)
        }
        LlmProviderType::Anthropic => {
            let api_key = config.api_key.unwrap_or_else(|| {
                std::env::var("ANTHROPIC_API_KEY").unwrap_or_default()
            });
            Arc::new(AnthropicProvider::new(api_key))
        }
        LlmProviderType::Ollama => {
            let mut provider = OllamaProvider::new();
            if let Some(url) = config.base_url {
                provider = provider.with_base_url(&url);
            }
            Arc::new(provider)
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
            Arc::new(provider)
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
            Arc::new(provider)
        }
        LlmProviderType::Gemini => {
            let api_key = config.api_key.unwrap_or_else(|| {
                std::env::var("GOOGLE_API_KEY").unwrap_or_default()
            });
            Arc::new(GeminiProvider::new(api_key))
        }
        LlmProviderType::Groq => {
            let api_key = config.api_key.unwrap_or_else(|| {
                std::env::var("GROQ_API_KEY").unwrap_or_default()
            });
            let mut provider = GroqProvider::new(api_key);
            if let Some(url) = config.base_url {
                provider = provider.with_base_url(&url);
            }
            Arc::new(provider)
        }
        LlmProviderType::OpenRouter => {
            let api_key = config.api_key.unwrap_or_else(|| {
                std::env::var("OPENROUTER_API_KEY").unwrap_or_default()
            });
            let mut provider = OpenRouterProvider::new(api_key);
            if let Some(url) = config.base_url {
                provider = provider.with_base_url(&url);
            }
            Arc::new(provider)
        }
        LlmProviderType::Cerebras => {
            let api_key = config.api_key.unwrap_or_else(|| {
                std::env::var("CEREBRAS_API_KEY").unwrap_or_default()
            });
            Arc::new(CerebrasProvider::new(api_key))
        }
        LlmProviderType::Pollinations | LlmProviderType::FreeApi => {
            Arc::new(PollinationsProvider::new())
        }
        LlmProviderType::BazaarLink => {
            let api_key = config.api_key.unwrap_or_else(|| {
                std::env::var("BAZAARLINK_API_KEY").unwrap_or_default()
            });
            let mut provider = OpenAiProvider::new(api_key);
            provider = provider.with_base_url("https://api.bazaarlink.ai/v1");
            Arc::new(provider)
        }
        LlmProviderType::FreeTheAi => {
            // Keyless — uses community API, OpenAI-compatible
            let mut provider = OpenAiProvider::new(String::new());
            provider = provider.with_base_url("https://api.freetheai.com/v1");
            Arc::new(provider)
        }
        LlmProviderType::ZeroLimit => {
            let api_key = config.api_key.unwrap_or_else(|| {
                std::env::var("ZEROLIMIT_API_KEY").unwrap_or_default()
            });
            let mut provider = OpenAiProvider::new(api_key);
            provider = provider.with_base_url("https://api.zerolimit.ai/v1");
            Arc::new(provider)
        }
        LlmProviderType::SambaNova => {
            let api_key = config.api_key.unwrap_or_else(|| {
                std::env::var("SAMBANOVA_API_KEY").unwrap_or_default()
            });
            let base_url = config.base_url.unwrap_or_else(|| "https://api.sambanova.ai/v1".to_string());
            let mut provider = GroqProvider::new(api_key);
            provider = provider.with_base_url(&base_url);
            Arc::new(provider)
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
            Arc::new(provider)
        }

        // ── Free cloud providers ──
        LlmProviderType::Cloudflare => {
            let api_key = config.api_key.unwrap_or_else(|| {
                std::env::var("CLOUDFLARE_API_TOKEN").unwrap_or_default()
            });
            let base_url = config.base_url.unwrap_or_else(|| "https://api.cloudflare.com/client/v4/ai".to_string());
            let mut provider = OpenAiProvider::new(api_key);
            provider = provider.with_base_url(&base_url);
            Arc::new(provider)
        }
        LlmProviderType::Nvidia => {
            let api_key = config.api_key.unwrap_or_else(|| {
                std::env::var("NVIDIA_API_KEY").unwrap_or_default()
            });
            let base_url = config.base_url.unwrap_or_else(|| "https://integrate.api.nvidia.com/v1".to_string());
            let mut provider = OpenAiProvider::new(api_key);
            provider = provider.with_base_url(&base_url);
            Arc::new(provider)
        }
        LlmProviderType::GitHubModels => {
            let api_key = config.api_key.unwrap_or_else(|| {
                std::env::var("GITHUB_TOKEN").unwrap_or_default()
            });
            let base_url = config.base_url.unwrap_or_else(|| "https://models.inference.ai.azure.com/v1".to_string());
            let mut provider = OpenAiProvider::new(api_key);
            provider = provider.with_base_url(&base_url);
            Arc::new(provider)
        }
        LlmProviderType::HuggingFace => {
            let api_key = config.api_key.unwrap_or_else(|| {
                std::env::var("HF_API_KEY").unwrap_or_default()
            });
            let base_url = config.base_url.unwrap_or_else(|| "https://api-inference.huggingface.co/v1".to_string());
            let mut provider = OpenAiProvider::new(api_key);
            provider = provider.with_base_url(&base_url);
            Arc::new(provider)
        }
        LlmProviderType::Cohere => {
            let api_key = config.api_key.unwrap_or_else(|| {
                std::env::var("COHERE_API_KEY").unwrap_or_default()
            });
            let base_url = config.base_url.unwrap_or_else(|| "https://api.cohere.ai/v1".to_string());
            let mut provider = OpenAiProvider::new(api_key);
            provider = provider.with_base_url(&base_url);
            Arc::new(provider)
        }
        LlmProviderType::TogetherFree => {
            let api_key = config.api_key.unwrap_or_else(|| {
                std::env::var("TOGETHER_API_KEY").unwrap_or_default()
            });
            let base_url = config.base_url.unwrap_or_else(|| "https://api.together.xyz/v1".to_string());
            let mut provider = OpenAiProvider::new(api_key);
            provider = provider.with_base_url(&base_url);
            Arc::new(provider)
        }
        LlmProviderType::Llm7 => {
            // .ai 域名已死（HTTP 000）；.io 是当前匿名可用端点（2026-08 实测 200）
            // 2026-08-22 实测可用模型: codestral-latest（gpt-oss-20b 已下线 400）。
            // catalog default_model 已更新，此处兜底同步，防 catalog 未命中时裸 "llm7" 上游。
            let base_url = config.base_url.unwrap_or_else(|| "https://api.llm7.io/v1".to_string());
            let mut provider = OpenAiProvider::new(String::new());
            provider = provider.with_base_url(&base_url);
            Arc::new(provider)
        }
        LlmProviderType::Kilo => {
            let base_url = config.base_url.unwrap_or_else(|| "https://api.kilocode.ai/v1".to_string());
            let mut provider = OpenAiProvider::new(String::new());
            provider = provider.with_base_url(&base_url);
            Arc::new(provider)
        }
        LlmProviderType::SiliconFlow => {
            let api_key = config.api_key.unwrap_or_else(|| {
                std::env::var("SILICONFLOW_API_KEY").unwrap_or_default()
            });
            let base_url = config.base_url.unwrap_or_else(|| "https://api.siliconflow.cn/v1".to_string());
            let mut provider = OpenAiProvider::new(api_key);
            provider = provider.with_base_url(&base_url);
            Arc::new(provider)
        }
        LlmProviderType::Xai => {
            let api_key = config.api_key.unwrap_or_else(|| std::env::var("XAI_API_KEY").unwrap_or_default());
            let base_url = config.base_url.unwrap_or_else(|| "https://api.x.ai/v1".to_string());
            let mut provider = OpenAiProvider::new(api_key);
            provider = provider.with_base_url(&base_url);
            Arc::new(provider)
        }
        LlmProviderType::Moonshot => {
            let api_key = config.api_key.unwrap_or_else(|| std::env::var("MOONSHOT_API_KEY").unwrap_or_default());
            let base_url = config.base_url.unwrap_or_else(|| "https://api.moonshot.cn/v1".to_string());
            let mut provider = OpenAiProvider::new(api_key);
            provider = provider.with_base_url(&base_url);
            Arc::new(provider)
        }
        LlmProviderType::Qwen => {
            let api_key = config.api_key.unwrap_or_else(|| std::env::var("QWEN_API_KEY").unwrap_or_default());
            let base_url = config.base_url.unwrap_or_else(|| "https://dashscope.aliyuncs.com/compatible-mode/v1".to_string());
            let mut provider = OpenAiProvider::new(api_key);
            provider = provider.with_base_url(&base_url);
            Arc::new(provider)
        }
        LlmProviderType::Doubao => {
            let api_key = config.api_key.unwrap_or_else(|| std::env::var("DOUBAO_API_KEY").unwrap_or_default());
            let base_url = config.base_url.unwrap_or_else(|| "https://ark.cn-beijing.volces.com/api/v3".to_string());
            let mut provider = OpenAiProvider::new(api_key);
            provider = provider.with_base_url(&base_url);
            Arc::new(provider)
        }
        LlmProviderType::MiniMax => {
            let api_key = config.api_key.unwrap_or_else(|| std::env::var("MINIMAX_API_KEY").unwrap_or_default());
            let base_url = config.base_url.unwrap_or_else(|| "https://api.minimax.chat/v1".to_string());
            let mut provider = OpenAiProvider::new(api_key);
            provider = provider.with_base_url(&base_url);
            Arc::new(provider)
        }
        LlmProviderType::Perplexity => {
            let api_key = config.api_key.unwrap_or_else(|| std::env::var("PERPLEXITY_API_KEY").unwrap_or_default());
            let base_url = config.base_url.unwrap_or_else(|| "https://api.perplexity.ai".to_string());
            let mut provider = OpenAiProvider::new(api_key);
            provider = provider.with_base_url(&base_url);
            Arc::new(provider)
        }
        LlmProviderType::ZAI => {
            let api_key = config.api_key.unwrap_or_else(|| {
                std::env::var("ZAI_API_KEY").unwrap_or_default()
            });
            let base_url = config.base_url.unwrap_or_else(|| "https://open.bigmodel.cn/api/paas/v4".to_string());
            let mut provider = OpenAiProvider::new(api_key);
            provider = provider.with_base_url(&base_url);
            Arc::new(provider)
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
            Arc::new(provider)
        }
        LlmProviderType::Ovh => {
            let base_url = config.base_url.unwrap_or_else(|| "https://ai-endpoints.ovh.net/v1".to_string());
            let mut provider = OpenAiProvider::new(String::new());
            provider = provider.with_base_url(&base_url);
            Arc::new(provider)
        }
        LlmProviderType::DeepSeekFree => {
            let api_key = config.api_key.unwrap_or_else(|| {
                std::env::var("DEEPSEEK_API_KEY").unwrap_or_default()
            });
            let base_url = config.base_url.unwrap_or_else(|| "https://api.deepseek.com/v1".to_string());
            let mut provider = OpenAiProvider::new(api_key);
            provider = provider.with_base_url(&base_url);
            Arc::new(provider)
        }
        LlmProviderType::ModelScope => {
            let base_url = config.base_url.unwrap_or_else(|| "https://api.modelscope.cn/v1".to_string());
            let mut provider = OpenAiProvider::new(String::new());
            provider = provider.with_base_url(&base_url);
            Arc::new(provider)
        }
        LlmProviderType::ApiAirforce => {
            // Truly keyless — accepts any Bearer token, even empty/not-needed
            // Verified working 2026-07-22: 209+ free models with `:free` suffix
            let base_url = config.base_url.unwrap_or_else(|| "https://api.airforce/v1".to_string());
            let api_key = config.api_key.unwrap_or_default();
            let mut provider = OpenAiProvider::new(api_key);
            provider = provider.with_base_url(&base_url);
            Arc::new(provider)
        }
        LlmProviderType::Aihub => {
            // Aihub (aihub.humorously.cn) — OpenAI-compatible, requires API key
            // Verified working 2026-08-21: models include glm-5.2, Qwen/Qwen3.6-35B-A3B-FP8
            let api_key = config.api_key.unwrap_or_else(|| {
                std::env::var("NEOTRIX_AIHUB_API_KEY").unwrap_or_default()
            });
            let base_url = config.base_url.unwrap_or_else(|| "https://aihub.humorously.cn/v1".to_string());
            let mut provider = OpenAiProvider::new(api_key);
            provider = provider.with_base_url(&base_url);
            Arc::new(provider)
        }
        LlmProviderType::Empero => {
            // free.empero.org — keyless OpenAI 兼容免费端点, key 随便填 "free" 即可。
            // 收编进 failover mesh: 503/维护窗由 CircuitBreaker + ProviderSwapManager
            // 透明切换备用源, 无需手动等其维护 (R-P42 复用 OpenAiProvider 节点)。
            let base_url = config.base_url.unwrap_or_else(|| "https://free.empero.org/v1".to_string());
            let mut provider = OpenAiProvider::new("free".to_string());
            provider = provider.with_base_url(&base_url);
            Arc::new(provider)
        }
    };

    // 代理注入: 若配置了代理 (子母阵 Proxied/Tor 画像), 将 provider 客户端切换到代理路由
    if let Some(proxy_url) = &config.proxy {
        if let Some(inner) = Arc::get_mut(&mut provider) {
            inner.set_proxy(proxy_url);
        }
        log::info!("[factory] provider {:?} routed through proxy {}", config.provider_type, proxy_url);
    }
    Arc::from(provider)
}

pub fn create_provider_from_type(provider_type: LlmProviderType, api_key: Option<String>) -> Arc<dyn LlmProvider> {
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
    // R-P79 接线 (response_cache): 主工厂启用 LRU 响应缓存 (读 complete_with_selection
    // :1595 写 heal_and_cache_response :924 已就位仅缺 flag)。默认开启, 可经环境变量关闭。
    let cache_enabled = std::env::var("NEOTRIX_RESPONSE_CACHE")
        .map(|v| v != "0" && !v.eq_ignore_ascii_case("off"))
        .unwrap_or(true);
    if cache_enabled {
        gateway.enable_response_cache(true);
        log::info!("[gateway] response cache enabled (LRU, fingerprint-keyed)");
    }

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
    register_if!("NEOTRIX_AIHUB_API_KEY", "aihub", LlmProviderType::Aihub, false);
    register_if!("XAI_API_KEY", "xai", LlmProviderType::Xai, false);
    register_if!("MOONSHOT_API_KEY", "moonshot", LlmProviderType::Moonshot, false);
    register_if!("QWEN_API_KEY", "qwen", LlmProviderType::Qwen, false);
    register_if!("DOUBAO_API_KEY", "doubao", LlmProviderType::Doubao, false);
    register_if!("MINIMAX_API_KEY", "minimax", LlmProviderType::MiniMax, false);
    register_if!("PERPLEXITY_API_KEY", "perplexity", LlmProviderType::Perplexity, false);

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
    let keyless_provider = |ptype: LlmProviderType| {
        create_provider(ProviderConfig {
            provider_type: ptype,
            api_key: None,
            proxy: proxy.clone(),
            ..Default::default()
        })
    };

    let mut pollinations: Box<dyn LlmProvider> = Box::new(PollinationsProvider::new());
    if let Some(proxy_url) = &proxy {
        pollinations.set_proxy(proxy_url);
    }
    gateway.register_provider_with_category("pollinations", Arc::from(pollinations), true, ProviderCategory::Cloud);
    log::info!("[gateway] Registered keyless: pollinations");

    // LLM7 — 匿名 keyless（Bearer unused 即可），turbo 层模型（gpt-oss:20b 等），~30 RPM。
    // 2026-08 实测 .io 端点 200 可用；.ai 旧域名已死。
    gateway.register_provider_with_category("llm7", keyless_provider(LlmProviderType::Llm7), true, ProviderCategory::Cloud);
    log::info!("[gateway] Registered keyless: llm7 (api.llm7.io, anonymous turbo models)");

    let api_airforce = keyless_provider(LlmProviderType::ApiAirforce);
    gateway.register_provider_with_category("api-airforce", api_airforce, true, ProviderCategory::Cloud);
    log::info!("[gateway] Registered keyless: api-airforce (api.airforce, 254+ models; 实测 POST 需真 key 时返回 401)");

    // Empero (free.empero.org) — keyless OpenAI 兼容免费端点 (glm-5.3-flash / qwen3.8-flash)。
    // 收编进 failover mesh: 503/维护窗由 CircuitBreaker 透明切换 Groq/OpenRouter/Pollinations 等,
    // 出站的密钥经 gateway egress redaction 脱敏 (防其记录 prompt 训模型)。
    let empero = keyless_provider(LlmProviderType::Empero);
    gateway.register_provider_with_category("empero", empero, true, ProviderCategory::Cloud);
    log::info!("[gateway] Registered keyless: empero (free.empero.org, glm-5.3-flash / qwen3.8-flash)");

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

    // ── 6. LLM 代理池: 注册持久化第三方 key 条目 (provider_pool.toml) ──
    // 每个条目按 label 注册进 gateway (统一路由/健康/配额) + AccountPool
    // (并发租约/检疫/自动恢复)。池为空时零开销跳过。
    let pool = super::provider_pool::global_provider_pool();
    if let Ok(guard) = pool.lock() {
        if !guard.entries.is_empty() {
            let n = guard.register_into_gateway(&mut gateway);
            log::info!("[gateway] LLM provider pool: {} entries registered", n);
        }
    }

    // Read prefer_free from config/env (priority: env > config > default false)
    let prefer_free = std::env::var("NEOTRIX_PREFER_FREE")
        .map(|v| v != "0" && !v.eq_ignore_ascii_case("off"))
        .unwrap_or_else(|_| {
            crate::config::NeoTrixConfig::load()
                .prefer_free
                .unwrap_or(false)
        });
    gateway.set_prefer_free(prefer_free);
    log::info!("[gateway] prefer_free = {}", prefer_free);

    gateway
}

/// 同步版本 — 保留向后兼容 (内部调用 block_on)
/// 如果已通过 Handle::try_current 或 enter() 存在 runtime 上下文，使用它；
/// 否则创建新 runtime 避免嵌套 runtime 冲突。
///
/// 启动期安全: 整个初始化带总超时 (15s), 防止任一 provider 探测的网络调用
/// 无超时保护时导致 app 启动永久卡死; 超时后返回空 gateway (可后续懒加载)。
///
/// 注意: 新建的 tokio Runtime 通过 Box::leak 长期存活, 避免 drop 时等待
/// blocking 线程池 (若 spawn_blocking 的 reqwest 阻塞请求被 timeout 中断,
/// 其后台线程仍可能存活, Runtime::drop 的 BlockingPool::shutdown 会死等)。
pub fn create_gateway() -> GatewayV2 {
    let fut = async {
        tokio::time::timeout(
            std::time::Duration::from_secs(15),
            create_gateway_async(),
        )
        .await
        .unwrap_or_else(|_elapsed| {
            log::warn!("[gateway] init timed out after 15s; returning empty gateway (lazy-load providers later)");
            GatewayV2::new()
        })
    };
    // 在 tokio runtime 上下文内 (如 reason/exec 在 rt.block_on 里经
    // init_reasoning_engine → create_gateway) 时, 直接 Runtime::new() 或
    // Handle::block_on 都会 panic ("Cannot start a runtime from within a runtime",
    // tokio 1.52+ 严格检查)。方案: 独立线程执行 — 新线程无 runtime 上下文,
    // Runtime::new() + block_on 安全, 主线程 rx.recv() 等待结果。
    if tokio::runtime::Handle::try_current().is_ok() {
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let rt = match tokio::runtime::Runtime::new() {
                Ok(rt) => rt,
                Err(e) => {
                    log::error!("[gateway] failed to create tokio runtime: {e}");
                    std::process::exit(1);
                }
            };
            let gateway = rt.block_on(fut);
            // 泄漏 runtime: 进程生命周期内保持存活, 避免 drop 卡死在 BlockingPool::shutdown。
            std::mem::forget(rt);
            let _ = tx.send(gateway);
        });
        return rx.recv().unwrap_or_else(|_| GatewayV2::new());
    }
    // 非 runtime 上下文: 直接新建 runtime。
    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            log::error!("[gateway] failed to create tokio runtime: {e}");
            std::process::exit(1);
        }
    };
    let gateway = rt.block_on(fut);
    // 泄漏 runtime: 进程生命周期内保持存活, 避免 drop 卡死在 BlockingPool::shutdown。
    std::mem::forget(rt);
    gateway
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_host_of_strips_scheme_path_port() {
        assert_eq!(host_of("https://api.openai.com/v1/chat"), "api.openai.com");
        assert_eq!(host_of("http://localhost:11434/api/tags"), "localhost");
        assert_eq!(host_of("https://[::1]:8080/v1"), "[::1]");
        assert_eq!(host_of("https://opencode.ai"), "opencode.ai");
        assert_eq!(host_of("api.deepseek.com"), "api.deepseek.com");
    }

    #[test]
    fn test_is_local_host_detects_loopback_and_private() {
        assert!(is_local_host("localhost"));
        assert!(is_local_host("127.0.0.1"));
        assert!(is_local_host("[::1]"));
        assert!(is_local_host("192.168.1.5"));
        assert!(is_local_host("10.0.0.2"));
        assert!(is_local_host("172.16.0.1"));
        assert!(is_local_host("dev.local"));
        assert!(!is_local_host("api.openai.com"));
        assert!(!is_local_host("pollinations.ai"));
        assert!(!is_local_host("evil.example.com"));
    }

    #[test]
    fn test_default_deny_for_non_allowlisted_cloud_host() {
        // llm7 已入白名单 (RouterConfig 默认 tier 端点, 2026-08)
        assert!(network_access_allowed(LlmProviderType::Llm7, Some("https://api.llm7.io/v1")));
        assert!(!network_access_allowed(LlmProviderType::CustomProxy, Some("https://evil.example.com/v1")));
        assert!(!network_access_allowed(LlmProviderType::ApiAirforce, Some("https://api.airforce/v1")));
    }

    #[test]
    fn test_allowlist_and_local_allowed() {
        assert!(network_access_allowed(LlmProviderType::Anthropic, None));
        assert!(network_access_allowed(LlmProviderType::Anthropic, Some("https://api.anthropic.com/v1")));
        assert!(network_access_allowed(LlmProviderType::OpenAI, Some("https://api.openai.com/v1")));
        assert!(network_access_allowed(LlmProviderType::Ollama, None));
        assert!(network_access_allowed(LlmProviderType::Vllm, Some("http://localhost:8000/v1")));
        assert!(network_access_allowed(LlmProviderType::CustomProxy, Some("http://localhost:3000/v1")));
    }

    #[test]
    fn test_denied_provider_errors() {
        let p = DeniedProvider { host: "evil.example.com".to_string() };
        let rt = tokio::runtime::Runtime::new().expect("tokio");
        let err = rt.block_on(p.complete(&LlmRequest::new("gpt-4o", "hi"))).expect_err("must error");
        assert!(err.to_string().contains("blocked"), "got: {err}");
    }

    #[test]
    fn test_create_provider_blocks_unknown_domain() {
        let provider = create_provider(ProviderConfig {
            provider_type: LlmProviderType::CustomProxy,
            api_key: None,
            base_url: Some("https://evil.example.com/v1".to_string()),
            model: None,
            timeout_secs: 10,
            proxy: None,
        });
        let rt = tokio::runtime::Runtime::new().expect("tokio");
        let err = rt.block_on(provider.complete(&LlmRequest::new("m", "hi"))).expect_err("must error");
        assert!(err.to_string().contains("blocked"), "got: {err}");
    }

    #[test]
    fn test_pool_entry_registers_into_gateway() {
        // 直接验证 ProviderPool::register_into_gateway 接线:
        // 池条目按 label 注册为 gateway provider (可被 providers() 发现)。
        let mut pool = crate::neotrix::nt_io_provider::provider_pool::ProviderPool::default();
        pool.entries.push(crate::neotrix::nt_io_provider::provider_pool::PoolEntry {
            label: "t-pool-gw".to_string(),
            provider: "openai".to_string(),
            api_key: "sk-test-pool".to_string(),
            model: "gpt-4o-mini".to_string(),
            tags: vec!["test".to_string()],
            base_url: None,
            created_ts: 0,
        });
    let mut gateway = GatewayV2::new();
    // 注: 需求驱动自愈 (T3) 接线点 enable_pool_self_heal 尚未实现; 此处仅验证 register_into_gateway 契约。
        let n = pool.register_into_gateway(&mut gateway);
        assert_eq!(n, 1);
        // 契约 (provider_pool.rs register_into_gateway): 以 `{provider}/{model}` 为 gateway 名注册
        // 使 provider_model() '/' 拆分与候选链前缀路由直接可用; label 仅作 AccountPool 键
        let names = gateway.providers();
        assert!(
            names.iter().any(|p| p == "openai/gpt-4o-mini"),
            "pool 条目应以 provider/model 注册为 gateway provider, got {names:?}"
        );
        // AccountPool 以 provider/label 登记 label
        let acc_pool = gateway.account_pool.lock().expect("lock");
        assert!(acc_pool.contains("openai/t-pool-gw") || acc_pool.contains("t-pool-gw"));
    }

    #[test]
    fn test_empero_provider_type_wiring() {
        assert_eq!(LlmProviderType::from_name("empero"), Some(LlmProviderType::Empero));
        assert!(LlmProviderType::Empero.is_free());
        assert!(!LlmProviderType::Empero.needs_api_key());
        assert_eq!(LlmProviderType::Empero.category(), ProviderCategory::Cloud);
        assert_eq!(default_host(LlmProviderType::Empero), Some("free.empero.org"));
    }

    #[test]
    fn test_empero_network_allowed_by_allowlist() {
        // free.empero.org 已加入 shield 网络白名单 (policy.rs default_llm_domains),
        // 否则网络隔离默认 Deny 会直接拒绝连接。
        assert!(network_access_allowed(LlmProviderType::Empero, None));
        assert!(network_access_allowed(
            LlmProviderType::Empero,
            Some("https://free.empero.org/v1")
        ));
    }

    #[test]
    fn test_create_provider_empero_is_openai_compatible() {
        // 收编进 failover mesh: empero 复用 OpenAiProvider 节点 (R-P42), keyless。
        let p = create_provider(ProviderConfig {
            provider_type: LlmProviderType::Empero,
            api_key: None,
            base_url: None,
            model: Some("glm-5.3-flash".to_string()),
            timeout_secs: 10,
            proxy: None,
        });
        // 网络放行 → 不是 DeniedProvider
        let rt = tokio::runtime::Runtime::new().expect("tokio");
        let err = rt
            .block_on(p.complete(&LlmRequest::new("glm-5.3-flash", "hi")))
            .expect_err("expected network/upstream error, not a block");
        assert!(
            !err.to_string().contains("blocked"),
            "empero must not be network-blocked, got: {err}"
        );
    }
}
