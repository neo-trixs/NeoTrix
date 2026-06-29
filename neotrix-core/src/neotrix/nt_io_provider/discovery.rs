use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::factory::{LlmProviderType, ProviderConfig};

/// 模型来源
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelSource {
    EnvVar,
    BuiltinFree,
    LocalEndpoint,
    DynamicDiscovery,
    ConfigFile,
}

/// 发现的模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredModel {
    pub provider_id: String,
    pub model_id: String,
    pub display_name: String,
    pub provider_type: LlmProviderType,
    pub base_url: String,
    pub requires_api_key: bool,
    pub api_key_env: Option<String>,
    pub source: ModelSource,
    pub is_free: bool,
    pub tier: String,
}

impl DiscoveredModel {
    pub fn qualified_id(&self) -> String {
        format!("{}/{}", self.provider_id, self.model_id)
    }
}

/// 环境变量到 Provider 映射
const ENV_VAR_MAP: &[(&str, &str, LlmProviderType, &str, bool)] = &[
    (
        "OPENAI_API_KEY",
        "openai",
        LlmProviderType::OpenAI,
        "https://api.openai.com/v1",
        true,
    ),
    (
        "ANTHROPIC_API_KEY",
        "anthropic",
        LlmProviderType::Anthropic,
        "https://api.anthropic.com/v1",
        true,
    ),
    (
        "GOOGLE_API_KEY",
        "gemini",
        LlmProviderType::Gemini,
        "https://generativelanguage.googleapis.com/v1beta",
        true,
    ),
    (
        "GROQ_API_KEY",
        "groq",
        LlmProviderType::OpenAI,
        "https://api.groq.com/openai/v1",
        true,
    ),
    (
        "DEEPSEEK_API_KEY",
        "deepseek",
        LlmProviderType::OpenAI,
        "https://api.deepseek.com/v1",
        true,
    ),
    (
        "NEOTRIX_API_KEY",
        "neotrix",
        LlmProviderType::OpenAI,
        "",
        false,
    ),
];

/// 模型发现器
pub struct ModelDiscovery;

impl ModelDiscovery {
    pub fn discover_all() -> Vec<DiscoveredModel> {
        let mut models = Vec::new();
        models.extend(Self::discover_from_env());
        models.extend(Self::discover_builtin_free());
        models.extend(Self::discover_local_endpoints());
        models.extend(Self::discover_local_free_models());
        models
    }

    /// 从 FreeModelCatalog 集成免费模型到发现结果
    pub fn discover_local_free_models() -> Vec<DiscoveredModel> {
        let mut catalog = super::free_catalog::FreeModelCatalog::new();
        let entries = catalog.refresh();
        entries
            .into_iter()
            .map(|e| {
                let tier = if e.tier.is_empty() {
                    "t1-standard".into()
                } else {
                    e.tier.clone()
                };
                DiscoveredModel {
                    provider_id: e.provider,
                    model_id: e.model_id,
                    display_name: e.display_name,
                    provider_type: e.provider_type,
                    base_url: e.base_url,
                    requires_api_key: e.requires_api_key,
                    api_key_env: e.api_key_env,
                    source: ModelSource::DynamicDiscovery,
                    is_free: e.is_free,
                    tier,
                }
            })
            .collect()
    }

    /// 从环境变量发现 API Key → 注册对应 Provider
    pub fn discover_from_env() -> Vec<DiscoveredModel> {
        let mut models = Vec::new();
        for (env_var, provider_id, provider_type, base_url, requires_key) in ENV_VAR_MAP {
            if std::env::var(env_var).is_ok() {
                let models_for_provider = match *provider_id {
                    "openai" => vec![
                        Self::make_model(
                            provider_id,
                            "gpt-4o-mini",
                            "GPT-4o Mini",
                            *provider_type,
                            base_url,
                            true,
                            Some(env_var),
                            ModelSource::EnvVar,
                            "t1-standard",
                        ),
                        Self::make_model(
                            provider_id,
                            "gpt-4o",
                            "GPT-4o",
                            *provider_type,
                            base_url,
                            true,
                            Some(env_var),
                            ModelSource::EnvVar,
                            "t3-powerful",
                        ),
                    ],
                    "anthropic" => vec![
                        Self::make_model(
                            provider_id,
                            "claude-sonnet-4",
                            "Claude Sonnet 4",
                            *provider_type,
                            base_url,
                            true,
                            Some(env_var),
                            ModelSource::EnvVar,
                            "t4-frontier",
                        ),
                        Self::make_model(
                            provider_id,
                            "claude-haiku-3.5",
                            "Claude Haiku 3.5",
                            *provider_type,
                            base_url,
                            true,
                            Some(env_var),
                            ModelSource::EnvVar,
                            "t1-standard",
                        ),
                    ],
                    "gemini" => vec![
                        Self::make_model(
                            provider_id,
                            "gemini-2.0-flash",
                            "Gemini 2.0 Flash",
                            *provider_type,
                            base_url,
                            true,
                            Some(env_var),
                            ModelSource::EnvVar,
                            "t2-balanced",
                        ),
                        Self::make_model(
                            provider_id,
                            "gemini-2.0-flash-lite",
                            "Gemini 2.0 Flash Lite",
                            *provider_type,
                            base_url,
                            true,
                            Some(env_var),
                            ModelSource::EnvVar,
                            "t0-cheap",
                        ),
                    ],
                    "groq" => vec![
                        Self::make_model(
                            provider_id,
                            "mixtral-8x7b-32768",
                            "Mixtral 8x7B",
                            *provider_type,
                            base_url,
                            true,
                            Some(env_var),
                            ModelSource::EnvVar,
                            "t2-balanced",
                        ),
                        Self::make_model(
                            provider_id,
                            "llama-3.3-70b-versatile",
                            "Llama 3.3 70B",
                            *provider_type,
                            base_url,
                            true,
                            Some(env_var),
                            ModelSource::EnvVar,
                            "t3-powerful",
                        ),
                        Self::make_model(
                            provider_id,
                            "deepseek-r1-distill-llama-70b",
                            "DeepSeek R1 Distill",
                            *provider_type,
                            base_url,
                            true,
                            Some(env_var),
                            ModelSource::EnvVar,
                            "t3-powerful",
                        ),
                    ],
                    "deepseek" => vec![Self::make_model(
                        provider_id,
                        "deepseek-chat",
                        "DeepSeek V3",
                        *provider_type,
                        base_url,
                        true,
                        Some(env_var),
                        ModelSource::EnvVar,
                        "t3-powerful",
                    )],
                    _ => vec![Self::make_model(
                        provider_id,
                        "default",
                        provider_id,
                        *provider_type,
                        base_url,
                        *requires_key,
                        Some(env_var),
                        ModelSource::EnvVar,
                        "t1-standard",
                    )],
                };
                models.extend(models_for_provider);
            }
        }

        // NEOTRIX_PROVIDER/API_KEY/MODEL — custom provider
        if std::env::var("NEOTRIX_API_KEY").is_ok() {
            let provider_name =
                std::env::var("NEOTRIX_PROVIDER").unwrap_or_else(|_| "custom".to_string());
            let base_url = std::env::var("NEOTRIX_BASE_URL")
                .unwrap_or_else(|_| "https://api.openai.com/v1".to_string());
            let model_name =
                std::env::var("NEOTRIX_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_string());
            models.push(Self::make_model(
                &provider_name,
                &model_name,
                &format!("{} ({})", model_name, provider_name),
                LlmProviderType::OpenAI,
                &base_url,
                true,
                Some("NEOTRIX_API_KEY"),
                ModelSource::EnvVar,
                "t2-balanced",
            ));
        }
        models
    }

    /// 内置免费模型（无需 API Key）
    pub fn discover_builtin_free() -> Vec<DiscoveredModel> {
        let mut models = Vec::new();
        let neotrix_base = std::env::var("NEOTRIX_ZEN_URL")
            .unwrap_or_else(|_| "https://api.opencode.ai/zen/v1".to_string());

        // 如果用户明确配置了 NEOTRIX_ZEN_URL 或者我们检测到可用
        if std::env::var("NEOTRIX_ZEN_URL").is_ok() {
            models.push(DiscoveredModel {
                provider_id: "neotrix-zen".into(),
                model_id: "deepseek-v4-flash-free".into(),
                display_name: "DeepSeek V4 Flash Free".into(),
                provider_type: LlmProviderType::OpenAI,
                base_url: neotrix_base.clone(),
                requires_api_key: false,
                api_key_env: None,
                source: ModelSource::BuiltinFree,
                is_free: true,
                tier: "t2-balanced".into(),
            });
        }

        // Ollama 是本地免费，无需 API Key。如果 localhost:11434 可达则标记
        models
    }

    /// 探测本地端点是否可达
    fn probe_endpoint(url: &str) -> bool {
        if cfg!(test) {
            return false; // 测试中不发起真实 HTTP 请求
        }
        reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_millis(1500))
            .build()
            .ok()
            .and_then(|c| c.head(url).send().ok())
            .map(|r| r.status().is_success() || r.status().as_u16() == 405)
            .unwrap_or(false)
    }

    /// 发现本地端点（Ollama, LM Studio, llama.cpp）— 自动探测可达性
    pub fn discover_local_endpoints() -> Vec<DiscoveredModel> {
        let mut models = Vec::new();
        let force = std::env::var("NEOTRIX_DISCOVER_LOCAL").is_ok();

        let local_endpoints = [
            ("ollama", "http://localhost:11434/v1", "Ollama"),
            ("lm-studio", "http://localhost:1234/v1", "LM Studio"),
            ("llama-cpp", "http://localhost:8080/v1", "llama.cpp"),
            ("vllm", "http://localhost:8000/v1", "vLLM"),
        ];

        for (id, url, name) in &local_endpoints {
            if !force && !Self::probe_endpoint(url) {
                continue;
            }
            let models_for_endpoint = match *id {
                "ollama" => vec![
                    Self::make_model(
                        id,
                        "llama3.2",
                        &format!("Llama 3.2 ({name})"),
                        LlmProviderType::Ollama,
                        url,
                        false,
                        None,
                        ModelSource::LocalEndpoint,
                        "t1-standard",
                    ),
                    Self::make_model(
                        id,
                        "qwen2.5",
                        &format!("Qwen 2.5 ({name})"),
                        LlmProviderType::Ollama,
                        url,
                        false,
                        None,
                        ModelSource::LocalEndpoint,
                        "t1-standard",
                    ),
                ],
                _ => vec![Self::make_model(
                    id,
                    "default",
                    &format!("{name} (Local)"),
                    LlmProviderType::OpenAI,
                    url,
                    false,
                    None,
                    ModelSource::LocalEndpoint,
                    "t1-standard",
                )],
            };
            models.extend(models_for_endpoint);
        }

        // 同时检查 NEOTRIX_LOCAL_ENDPOINT 自定义本地端点
        if let Ok(endpoint) = std::env::var("NEOTRIX_LOCAL_ENDPOINT") {
            models.push(Self::make_model(
                "local",
                "default",
                "Local Endpoint",
                LlmProviderType::OpenAI,
                &endpoint,
                false,
                None,
                ModelSource::LocalEndpoint,
                "t1-standard",
            ));
        }

        models
    }

    fn make_model(
        provider_id: &str,
        model_id: &str,
        display_name: &str,
        provider_type: LlmProviderType,
        base_url: &str,
        requires_api_key: bool,
        api_key_env: Option<&str>,
        source: ModelSource,
        tier: &str,
    ) -> DiscoveredModel {
        DiscoveredModel {
            provider_id: provider_id.to_string(),
            model_id: model_id.to_string(),
            display_name: display_name.to_string(),
            provider_type,
            base_url: base_url.to_string(),
            requires_api_key,
            api_key_env: api_key_env.map(|s| s.to_string()),
            source,
            is_free: !requires_api_key,
            tier: tier.to_string(),
        }
    }

    /// 从 OpenAI 兼容的 /v1/models 端点动态查询可用模型
    pub async fn discover_dynamic(base_url: &str, api_key: Option<&str>) -> Vec<DiscoveredModel> {
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .ok();

        let client = match client {
            Some(c) => c,
            None => return vec![],
        };

        let url = format!("{}/models", base_url.trim_end_matches('/'));
        let mut req = client.get(&url);
        if let Some(key) = api_key {
            req = req.header("Authorization", format!("Bearer {}", key));
        }

        let resp = req.send().await;
        let resp = match resp {
            Ok(r) if r.status().is_success() => r,
            _ => return vec![],
        };

        let body: serde_json::Value = match resp.json().await {
            Ok(v) => v,
            Err(_) => return vec![],
        };

        let models = body.get("data").and_then(|d| d.as_array());
        let models = match models {
            Some(m) => m,
            None => return vec![],
        };

        let provider_id = Self::extract_provider_id(base_url);
        models
            .iter()
            .filter_map(|m| {
                let id = m.get("id").and_then(|v| v.as_str())?;
                Some(DiscoveredModel {
                    provider_id: provider_id.clone(),
                    model_id: id.to_string(),
                    display_name: m
                        .get("display_name")
                        .and_then(|v| v.as_str())
                        .unwrap_or(id)
                        .to_string(),
                    provider_type: LlmProviderType::OpenAI,
                    base_url: base_url.to_string(),
                    requires_api_key: api_key.is_some(),
                    api_key_env: None,
                    source: ModelSource::DynamicDiscovery,
                    is_free: false,
                    tier: "t2-balanced".to_string(),
                })
            })
            .collect()
    }

    fn extract_provider_id(base_url: &str) -> String {
        let host = base_url
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            .split('/')
            .next()
            .unwrap_or("unknown");
        host.split('.').next().unwrap_or("unknown").to_string()
    }
}

/// 模型注册表 — 管理所有发现的模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRegistry {
    pub models: Vec<DiscoveredModel>,
    /// provider_id → selected model_id
    pub active_selections: HashMap<String, String>,
    pub default_provider: String,
    pub default_model: String,
}

impl ModelRegistry {
    pub fn new() -> Self {
        Self {
            models: Vec::new(),
            active_selections: HashMap::new(),
            default_provider: "ollama".into(),
            default_model: "llama3.2".into(),
        }
    }

    /// 发现并注册所有模型
    pub fn refresh(&mut self) {
        self.models = ModelDiscovery::discover_all();
    }

    /// 获取所有可用模型
    pub fn available_models(&self) -> Vec<&DiscoveredModel> {
        self.models
            .iter()
            .filter(|m| {
                if m.requires_api_key {
                    if let Some(ref env) = m.api_key_env {
                        return std::env::var(env).is_ok();
                    }
                }
                true
            })
            .collect()
    }

    /// 获取免费模型
    pub fn free_models(&self) -> Vec<&DiscoveredModel> {
        self.models
            .iter()
            .filter(|m| {
                if m.requires_api_key {
                    if let Some(ref env) = m.api_key_env {
                        if std::env::var(env).is_ok() {
                            return true; // 有 API key 的也算可用
                        }
                        return false;
                    }
                    false
                } else {
                    true
                }
            })
            .collect()
    }

    /// 按 tier 获取模型
    pub fn models_by_tier(&self, tier: &str) -> Vec<&DiscoveredModel> {
        self.models.iter().filter(|m| m.tier == tier).collect()
    }

    /// 按 provider 获取模型
    pub fn models_by_provider(&self, provider_id: &str) -> Vec<&DiscoveredModel> {
        self.models
            .iter()
            .filter(|m| m.provider_id == provider_id)
            .collect()
    }

    /// 设置活跃模型
    pub fn set_active(&mut self, provider_id: &str, model_id: &str) {
        self.active_selections
            .insert(provider_id.to_string(), model_id.to_string());
        self.default_provider = provider_id.to_string();
        self.default_model = model_id.to_string();
    }

    /// 获取活跃模型的 ProviderConfig
    pub fn active_config(&self) -> Option<ProviderConfig> {
        let model = self
            .models
            .iter()
            .find(|m| m.provider_id == self.default_provider && m.model_id == self.default_model)?;
        Some(ProviderConfig {
            provider_type: model.provider_type,
            api_key: model
                .api_key_env
                .as_ref()
                .and_then(|env| std::env::var(env).ok()),
            base_url: Some(model.base_url.clone()),
            model: Some(model.model_id.clone()),
            timeout_secs: 120,
        })
    }

    /// 通过 qualified ID 查找模型
    pub fn find_by_qualified_id(&self, qid: &str) -> Option<&DiscoveredModel> {
        self.models.iter().find(|m| m.qualified_id() == qid)
    }

    /// 导出统计信息
    pub fn stats(&self) -> HashMap<&'static str, usize> {
        let mut s = HashMap::new();
        s.insert("total", self.models.len());
        s.insert("free", self.models.iter().filter(|m| m.is_free).count());
        s.insert(
            "requires_key",
            self.models.iter().filter(|m| m.requires_api_key).count(),
        );
        s.insert(
            "from_env",
            self.models
                .iter()
                .filter(|m| m.source == ModelSource::EnvVar)
                .count(),
        );
        s.insert(
            "from_builtin",
            self.models
                .iter()
                .filter(|m| m.source == ModelSource::BuiltinFree)
                .count(),
        );
        s.insert(
            "from_local",
            self.models
                .iter()
                .filter(|m| m.source == ModelSource::LocalEndpoint)
                .count(),
        );
        s
    }
}

impl Default for ModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 格式化显示模型列表
pub fn format_model_list(models: &[&DiscoveredModel]) -> String {
    let mut output = String::new();
    output.push_str(&format!(
        "╭─ Models ({}) ─────────────────────────────╮\n",
        models.len()
    ));

    let mut by_tier: HashMap<String, Vec<&DiscoveredModel>> = HashMap::new();
    for m in models {
        by_tier.entry(m.tier.clone()).or_default().push(m);
    }

    let mut tiers: Vec<&String> = by_tier.keys().collect();
    tiers.sort();
    for &tier in &tiers {
        let label = match tier.as_str() {
            "t0-cheap" => "T0 💰 Cheap/Fast",
            "t1-standard" => "T1 📋 Standard",
            "t2-balanced" => "T2 ⚖️ Balanced",
            "t3-powerful" => "T3 🚀 Powerful",
            "t4-frontier" => "T4 🧠 Frontier",
            _ => tier,
        };
        output.push_str(&format!("  {}:\n", label));
        if let Some(models) = by_tier.get(tier.as_str()) {
            for m in models {
                let free = if m.is_free { " 🆓" } else { "" };
                let _active = ""; // Could be marked if active
                let source = match m.source {
                    ModelSource::EnvVar => "🔑",
                    ModelSource::BuiltinFree => "📦",
                    ModelSource::LocalEndpoint => "💻",
                    ModelSource::DynamicDiscovery => "🌐",
                    ModelSource::ConfigFile => "📄",
                };
                output.push_str(&format!(
                    "    {}{}  {}/{}  ({})\n",
                    source, free, m.provider_id, m.model_id, m.display_name
                ));
            }
        }
    }
    output.push_str("╰──────────────────────────────────────────────╯");
    output
}
