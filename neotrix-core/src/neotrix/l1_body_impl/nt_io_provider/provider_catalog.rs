//! Provider Catalog — 自我(主体) vs 代理(客体) vs 云(客体) 分类管理
//!
//! 设计原则:
//! - **Self (主体)**: 本地推理 (Ollama, LM Studio, llama.cpp) — 数据不出设备, 默认零成本
//! - **Proxy (代理)**: OpenAI 兼容中转站 (One API / New API / LiteLLM) — 由用户自定义, 无隐私泄露
//! - **Cloud (云)**: 主流第三方 API (OpenAI, Anthropic, Google, Groq 等) — 需 API key, 数据发送至第三方
//!
//! 所有 provider 的基本信息集中于此, 无需分散在各处硬编码。

/// Provider 分类: 自我主体 vs 外部客体
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ProviderCategory {
    /// 本地推理 (Ollama, LM Studio, llama.cpp, vLLM local) — 数据不出设备
    Local,
    /// OpenAI 兼容代理 (One API, New API, LiteLLM, 自定义中转) — 用户自定义端点
    Proxy,
    /// 主流云端 API (OpenAI, Anthropic, Google, Groq 等) — 需 API key, 数据送第三方
    Cloud,
}

impl ProviderCategory {
    pub fn label(&self) -> &str {
        match self {
            ProviderCategory::Local => "本地推理 (主体)",
            ProviderCategory::Proxy => "自定义代理 (客体)",
            ProviderCategory::Cloud => "云端 API (客体)",
        }
    }

    /// 路由优先级: Local > Proxy > Cloud
    pub fn route_priority(&self) -> u8 {
        match self {
            ProviderCategory::Local => 0,
            ProviderCategory::Proxy => 1,
            ProviderCategory::Cloud => 2,
        }
    }
}

/// Communication security profile for sub-grid composition (子母阵通信安全画像)
/// 定义子网格的安全姿态，用于动态组合已有节点能力实现隐匿通信
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum CommunicationProfile {
    /// 标准 HTTPS，直连，数据对提供方可见
    Open,
    /// 经用户定义代理路由，元数据隐匿
    Proxied,
    /// 经 Tor SOCKS 代理路由，身份匿名
    Tor,
    /// 多跳混淆 + 随机化 UA + 指纹伪装，最大隐匿
    Anonymous,
}

impl CommunicationProfile {
    /// 返回是否满足或超过目标安全级别
    pub fn meets(&self, required: CommunicationProfile) -> bool {
        // 安全级别排序: Open < Proxied < Tor < Anonymous
        let self_level = match self {
            CommunicationProfile::Open => 0,
            CommunicationProfile::Proxied => 1,
            CommunicationProfile::Tor => 2,
            CommunicationProfile::Anonymous => 3,
        };
        let req_level = match required {
            CommunicationProfile::Open => 0,
            CommunicationProfile::Proxied => 1,
            CommunicationProfile::Tor => 2,
            CommunicationProfile::Anonymous => 3,
        };
        self_level >= req_level
    }
}

impl ProviderCategory {
    /// 基于分类推断的默认通信安全画像
    pub fn default_security_profile(&self) -> CommunicationProfile {
        match self {
            // Local 主体：数据不出设备，天然 Anonymous
            ProviderCategory::Local => CommunicationProfile::Anonymous,
            // Proxy 客体：用户自定义代理，天然 Proxied
            ProviderCategory::Proxy => CommunicationProfile::Proxied,
            // Cloud 客体：标准 HTTPS，可升级到 Proxied/Tor
            ProviderCategory::Cloud => CommunicationProfile::Open,
        }
    }
}

/// Provider 基本信息
#[derive(Debug, Clone)]
pub struct ProviderInfo {
    /// 内部标识名 (如 "openai", "ollama")
    pub name: &'static str,
    /// 显示名称 (如 "OpenAI", "Anthropic Claude")
    pub display_name: &'static str,
    /// 分类
    pub category: ProviderCategory,
    /// 默认 Base URL
    pub base_url: &'static str,
    /// 默认模型 (fallback)
    pub default_model: &'static str,
    /// API key 环境变量 (None = keyless)
    pub api_key_env: Option<&'static str>,
    /// 是否免费 (keyless 或 free tier)
    pub is_free: bool,
    /// 支持的模型列表
    pub models: &'static [&'static str],
    /// 通信安全画像：决定该 provider 可组合进哪些安全级别的子网格
    pub security_profile: CommunicationProfile,
}

/// 所有主流 Provider 目录
pub static PROVIDER_CATALOG: &[ProviderInfo] = &[
    // ── Local (主体): 本地推理 — 数据不出设备 ──
    ProviderInfo {
        name: "ollama",
        display_name: "Ollama (本地)",
        category: ProviderCategory::Local,
        base_url: "http://localhost:11434/v1",
        default_model: "llama3.2",
        api_key_env: None,
        is_free: true,
        models: &["llama3.2", "llama3.1", "qwen2.5", "mistral", "codellama", "phi-4", "deepseek-coder"],
        security_profile: CommunicationProfile::Anonymous,
    },
    ProviderInfo {
        name: "lm-studio",
        display_name: "LM Studio (本地)",
        category: ProviderCategory::Local,
        base_url: "http://localhost:1234/v1",
        default_model: "local-model",
        api_key_env: None,
        is_free: true,
        models: &["local-model"],
        security_profile: CommunicationProfile::Anonymous,
    },
    ProviderInfo {
        name: "llamacpp",
        display_name: "llama.cpp (本地)",
        category: ProviderCategory::Local,
        base_url: "http://localhost:8080/v1",
        default_model: "local-model",
        api_key_env: None,
        is_free: true,
        models: &["local-model"],
        security_profile: CommunicationProfile::Anonymous,
    },
    ProviderInfo {
        name: "vllm-local",
        display_name: "vLLM (本地)",
        category: ProviderCategory::Local,
        base_url: "http://localhost:8000/v1",
        default_model: "local-model",
        api_key_env: None,
        is_free: true,
        models: &["local-model"],
        security_profile: CommunicationProfile::Anonymous,
    },
    ProviderInfo {
        name: "sglang-local",
        display_name: "SGLang (本地)",
        category: ProviderCategory::Local,
        base_url: "http://localhost:30000/v1",
        default_model: "local-model",
        api_key_env: None,
        is_free: true,
        models: &["local-model"],
        security_profile: CommunicationProfile::Anonymous,
    },

    // ── Proxy (客体): 自定义 OpenAI 兼容代理 ──
    ProviderInfo {
        name: "custom-proxy",
        display_name: "自定义代理 (OpenAI兼容)",
        category: ProviderCategory::Proxy,
        base_url: "",
        default_model: "gpt-4o-mini",
        api_key_env: Some("NEOTRIX_PROXY_API_KEY"),
        is_free: false,
        models: &["gpt-4o-mini", "gpt-4o", "claude-sonnet-4", "custom"],
        security_profile: CommunicationProfile::Proxied,
    },

    // ── Cloud (客体): 主流云端 API ──
    ProviderInfo {
        name: "openai",
        display_name: "OpenAI",
        category: ProviderCategory::Cloud,
        base_url: "https://api.openai.com/v1",
        default_model: "gpt-4o-mini",
        api_key_env: Some("OPENAI_API_KEY"),
        is_free: false,
        models: &["gpt-4o", "gpt-4o-mini", "gpt-4-turbo", "gpt-3.5-turbo", "o1", "o1-mini", "o3-mini"],
        security_profile: CommunicationProfile::Open,
    },
    ProviderInfo {
        name: "anthropic",
        display_name: "Anthropic Claude",
        category: ProviderCategory::Cloud,
        base_url: "https://api.anthropic.com/v1",
        default_model: "claude-sonnet-4",
        api_key_env: Some("ANTHROPIC_API_KEY"),
        is_free: false,
        models: &["claude-sonnet-4", "claude-haiku-3.5", "claude-3-opus", "claude-3.5-sonnet"],
        security_profile: CommunicationProfile::Open,
    },
    ProviderInfo {
        name: "gemini",
        display_name: "Google Gemini",
        category: ProviderCategory::Cloud,
        base_url: "https://generativelanguage.googleapis.com/v1beta",
        default_model: "gemini-2.0-flash",
        api_key_env: Some("GOOGLE_API_KEY"),
        is_free: true,
        models: &["gemini-2.0-flash", "gemini-2.0-flash-lite", "gemini-2.0-pro", "gemini-1.5-pro"],
        security_profile: CommunicationProfile::Open,
    },
    ProviderInfo {
        name: "groq",
        display_name: "Groq",
        category: ProviderCategory::Cloud,
        base_url: "https://api.groq.com/openai/v1",
        default_model: "llama-3.3-70b-versatile",
        api_key_env: Some("GROQ_API_KEY"),
        is_free: true,
        models: &["llama-3.3-70b-versatile", "llama-3.1-8b-instant", "mixtral-8x7b-32768"],
        security_profile: CommunicationProfile::Open,
    },
    ProviderInfo {
        name: "openrouter",
        display_name: "OpenRouter",
        category: ProviderCategory::Cloud,
        base_url: "https://openrouter.ai/api/v1",
        default_model: "auto",
        api_key_env: Some("OPENROUTER_API_KEY"),
        is_free: false,
        models: &["auto"],
        security_profile: CommunicationProfile::Open,
    },
    ProviderInfo {
        name: "cerebras",
        display_name: "Cerebras",
        category: ProviderCategory::Cloud,
        base_url: "https://api.cerebras.ai/v1",
        default_model: "llama-3.3-70b",
        api_key_env: Some("CEREBRAS_API_KEY"),
        is_free: true,
        models: &["llama-3.3-70b", "llama-3.1-8b"],
        security_profile: CommunicationProfile::Open,
    },
    ProviderInfo {
        name: "sambanova",
        display_name: "SambaNova",
        category: ProviderCategory::Cloud,
        base_url: "https://api.sambanova.ai/v1",
        default_model: "Meta-Llama-3.1-70B-Instruct",
        api_key_env: Some("SAMBANOVA_API_KEY"),
        is_free: true,
        models: &["Meta-Llama-3.1-70B-Instruct", "Meta-Llama-3.1-8B-Instruct"],
        security_profile: CommunicationProfile::Open,
    },
    ProviderInfo {
        name: "bazaarlink",
        display_name: "BazaarLink",
        category: ProviderCategory::Cloud,
        base_url: "https://api.bazaarlink.ai/v1",
        default_model: "auto:free",
        api_key_env: Some("BAZAARLINK_API_KEY"),
        is_free: true,
        models: &["auto:free", "gpt-4o-mini", "claude-sonnet-4"],
        security_profile: CommunicationProfile::Open,
    },
    ProviderInfo {
        name: "freetheai",
        display_name: "FreeTheAi",
        category: ProviderCategory::Cloud,
        base_url: "https://api.freetheai.com/v1",
        default_model: "auto",
        api_key_env: None,
        is_free: true,
        models: &["auto", "gpt-4o", "gpt-4o-mini", "claude-3.5-sonnet"],
        security_profile: CommunicationProfile::Open,
    },
    ProviderInfo {
        name: "zerolimit",
        display_name: "ZeroLimitAI",
        category: ProviderCategory::Cloud,
        base_url: "https://api.zerolimit.ai/v1",
        default_model: "auto",
        api_key_env: Some("ZEROLIMIT_API_KEY"),
        is_free: true,
        models: &["auto"],
        security_profile: CommunicationProfile::Open,
    },
    ProviderInfo {
        name: "pollinations",
        display_name: "Pollinations.ai",
        category: ProviderCategory::Cloud,
        base_url: "https://text.pollinations.ai/openai",
        default_model: "openai",
        api_key_env: None,
        is_free: true,
        models: &["openai", "mistral", "llama"],
        security_profile: CommunicationProfile::Open,
    },
    ProviderInfo {
        name: "deepseek",
        display_name: "DeepSeek",
        category: ProviderCategory::Cloud,
        base_url: "https://api.deepseek.com/v1",
        default_model: "deepseek-chat",
        api_key_env: Some("DEEPSEEK_API_KEY"),
        is_free: false,
        models: &["deepseek-chat", "deepseek-reasoner"],
        security_profile: CommunicationProfile::Open,
    },
    ProviderInfo {
        name: "together",
        display_name: "Together AI",
        category: ProviderCategory::Cloud,
        base_url: "https://api.together.xyz/v1",
        default_model: "meta-llama/Llama-3.3-70B-Instruct-Turbo",
        api_key_env: Some("TOGETHER_API_KEY"),
        is_free: false,
        models: &["meta-llama/Llama-3.3-70B-Instruct-Turbo"],
        security_profile: CommunicationProfile::Open,
    },
    ProviderInfo {
        name: "mistral",
        display_name: "Mistral AI",
        category: ProviderCategory::Cloud,
        base_url: "https://api.mistral.ai/v1",
        default_model: "mistral-large-latest",
        api_key_env: Some("MISTRAL_API_KEY"),
        is_free: false,
        models: &["mistral-large-latest", "mistral-small-latest", "codestral-latest"],
        security_profile: CommunicationProfile::Open,
    },

    // ── Free Cloud (客体): 免费/免费层云端 API ──
    ProviderInfo {
        name: "cloudflare",
        display_name: "Cloudflare Workers AI",
        category: ProviderCategory::Cloud,
        base_url: "https://api.cloudflare.com/client/v4/ai",
        default_model: "@cf/meta/llama-3.3-70b-instruct-fp8-fast",
        api_key_env: Some("CLOUDFLARE_API_TOKEN"),
        is_free: true,
        models: &["@cf/meta/llama-3.3-70b-instruct-fp8-fast", "@cf/meta/llama-3.1-8b-instruct", "@hf/deepseek-r1-distill-qwen-32b"],
        security_profile: CommunicationProfile::Open,
    },
    ProviderInfo {
        name: "nvidia",
        display_name: "NVIDIA NIM",
        category: ProviderCategory::Cloud,
        base_url: "https://integrate.api.nvidia.com/v1",
        default_model: "meta/llama-3.3-70b-instruct",
        api_key_env: Some("NVIDIA_API_KEY"),
        is_free: true,
        models: &["meta/llama-3.3-70b-instruct", "meta/llama-3.1-405b-instruct", "nvidia/llama-3.1-nemotron-70b-instruct", "deepseek-ai/deepseek-r1"],
        security_profile: CommunicationProfile::Open,
    },
    ProviderInfo {
        name: "github-models",
        display_name: "GitHub Models",
        category: ProviderCategory::Cloud,
        base_url: "https://models.inference.ai.azure.com/v1",
        default_model: "gpt-4o-mini",
        api_key_env: Some("GITHUB_TOKEN"),
        is_free: true,
        models: &["gpt-4o-mini", "gpt-4o", "gpt-4.1-mini", "meta-llama-3.3-70b-instruct", "mistral-large-2407", "Phi-4", "cohere-command-r+"],
        security_profile: CommunicationProfile::Open,
    },
    ProviderInfo {
        name: "huggingface",
        display_name: "HuggingFace Inference",
        category: ProviderCategory::Cloud,
        base_url: "https://api-inference.huggingface.co/v1",
        default_model: "meta-llama/Llama-3.3-70B-Instruct",
        api_key_env: Some("HF_API_KEY"),
        is_free: true,
        models: &["meta-llama/Llama-3.3-70B-Instruct", "mistralai/Mistral-7B-Instruct-v0.3", "HuggingFaceH4/zephyr-7b-beta"],
        security_profile: CommunicationProfile::Open,
    },
    ProviderInfo {
        name: "cohere",
        display_name: "Cohere",
        category: ProviderCategory::Cloud,
        base_url: "https://api.cohere.ai/v1",
        default_model: "command-a",
        api_key_env: Some("COHERE_API_KEY"),
        is_free: false,
        models: &["command-a", "command-r", "command-r-plus"],
        security_profile: CommunicationProfile::Open,
    },
    ProviderInfo {
        name: "together-free",
        display_name: "Together AI (Free)",
        category: ProviderCategory::Cloud,
        base_url: "https://api.together.xyz/v1",
        default_model: "meta-llama/Llama-3.3-70B-Instruct-Turbo-Free",
        api_key_env: Some("TOGETHER_API_KEY"),
        is_free: true,
        models: &["meta-llama/Llama-3.3-70B-Instruct-Turbo-Free", "mistralai/Mixtral-8x7B-Instruct-v0.1-Free", "Qwen/Qwen2.5-72B-Instruct-Free"],
        security_profile: CommunicationProfile::Open,
    },
    ProviderInfo {
        name: "llm7",
        display_name: "LLM7 (Keyless)",
        category: ProviderCategory::Cloud,
        base_url: "https://api.llm7.ai/v1",
        default_model: "gpt-oss-20b",
        api_key_env: None,
        is_free: true,
        models: &["gpt-oss-20b", "llama-3.1-8b", "glm-4-flash"],
        security_profile: CommunicationProfile::Open,
    },
    ProviderInfo {
        name: "kilo",
        display_name: "KiloCode (Keyless)",
        category: ProviderCategory::Cloud,
        base_url: "https://api.kilocode.ai/v1",
        default_model: "nemotron-70b",
        api_key_env: None,
        is_free: true,
        models: &["nemotron-70b", "stepfun-32k", "deepseek-v4-flash"],
        security_profile: CommunicationProfile::Open,
    },
    ProviderInfo {
        name: "siliconflow",
        display_name: "SiliconFlow",
        category: ProviderCategory::Cloud,
        base_url: "https://api.siliconflow.cn/v1",
        default_model: "deepseek-ai/DeepSeek-V3",
        api_key_env: Some("SILICONFLOW_API_KEY"),
        is_free: true,
        models: &["deepseek-ai/DeepSeek-V3", "Qwen/Qwen3-235B-A22B", "meta-llama/Llama-3.3-70B-Instruct"],
        security_profile: CommunicationProfile::Open,
    },
    ProviderInfo {
        name: "zai",
        display_name: "Z.AI (GLM)",
        category: ProviderCategory::Cloud,
        base_url: "https://open.bigmodel.cn/api/paas/v4",
        default_model: "glm-4-flash",
        api_key_env: Some("ZAI_API_KEY"),
        is_free: true,
        models: &["glm-4-flash", "glm-4", "glm-4v"],
        security_profile: CommunicationProfile::Open,
    },
    ProviderInfo {
        name: "opencode-zen",
        display_name: "OpenCode Zen",
        category: ProviderCategory::Cloud,
        base_url: "https://api.opencode.ai/zen/v1",
        default_model: "deepseek-v4-flash-free",
        api_key_env: None,
        is_free: true,
        models: &["deepseek-v4-flash-free", "deepseek-v4-flash"],
        security_profile: CommunicationProfile::Open,
    },
    ProviderInfo {
        name: "ovh",
        display_name: "OVH AI Endpoints",
        category: ProviderCategory::Cloud,
        base_url: "https://ai-endpoints.ovh.net/v1",
        default_model: "Qwen3.5-397B-A22B",
        api_key_env: None,
        is_free: true,
        models: &["Qwen3.5-397B-A22B", "meta-llama/Llama-3.3-70B-Instruct", "gpt-oss-120b"],
        security_profile: CommunicationProfile::Open,
    },
    ProviderInfo {
        name: "deepseek-free",
        display_name: "DeepSeek (Free Tier)",
        category: ProviderCategory::Cloud,
        base_url: "https://api.deepseek.com/v1",
        default_model: "deepseek-chat",
        api_key_env: Some("DEEPSEEK_API_KEY"),
        is_free: true,
        models: &["deepseek-chat", "deepseek-reasoner"],
        security_profile: CommunicationProfile::Open,
    },
    ProviderInfo {
        name: "modelscope",
        display_name: "ModelScope",
        category: ProviderCategory::Cloud,
        base_url: "https://api.modelscope.cn/v1",
        default_model: "Qwen/Qwen3-235B-A22B",
        api_key_env: None,
        is_free: true,
        models: &["Qwen/Qwen3-235B-A22B", "iic/LLM"],
        security_profile: CommunicationProfile::Open,
    },
];

/// 按名称查找 ProviderInfo
pub fn lookup_provider(name: &str) -> Option<&'static ProviderInfo> {
    PROVIDER_CATALOG.iter().find(|p| p.name == name)
}

/// 按分类列出 providers
pub fn providers_by_category(category: ProviderCategory) -> Vec<&'static ProviderInfo> {
    PROVIDER_CATALOG.iter().filter(|p| p.category == category).collect()
}

/// 列出所有有可用 API key 的 provider
pub fn providers_with_key() -> Vec<&'static ProviderInfo> {
    PROVIDER_CATALOG.iter().filter(|p| {
        p.api_key_env.is_some_and(|env| {
            std::env::var(env).is_ok_and(|k| !k.is_empty())
        })
    }).collect()
}

/// 列出所有 keyless provider (本地端点 + 免费 API)
pub fn keyless_providers() -> Vec<&'static ProviderInfo> {
    PROVIDER_CATALOG.iter().filter(|p| p.api_key_env.is_none()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_catalog_has_all_categories() {
        assert!(!providers_by_category(ProviderCategory::Local).is_empty());
        assert!(!providers_by_category(ProviderCategory::Cloud).is_empty());
        // Proxy may be empty if not configured — that's fine
    }

    #[test]
    fn test_lookup_existing() {
        let p = lookup_provider("openai").expect("openai should be in catalog");
        assert_eq!(p.category, ProviderCategory::Cloud);
        assert!(p.models.contains(&"gpt-4o"));
    }

    #[test]
    fn test_lookup_missing() {
        assert!(lookup_provider("nonexistent").is_none());
    }

    #[test]
    fn test_route_priority_order() {
        assert!(ProviderCategory::Local.route_priority() < ProviderCategory::Proxy.route_priority());
        assert!(ProviderCategory::Proxy.route_priority() < ProviderCategory::Cloud.route_priority());
    }

    #[test]
    fn test_all_providers_have_unique_names() {
        let mut names = std::collections::HashSet::new();
        for p in PROVIDER_CATALOG {
            assert!(names.insert(p.name), "duplicate provider name: {}", p.name);
        }
    }

    #[test]
    fn test_keyless_providers_include_local() {
        let kl = keyless_providers();
        assert!(kl.iter().any(|p| p.name == "ollama"));
        assert!(kl.iter().any(|p| p.name == "pollinations"));
    }
}
