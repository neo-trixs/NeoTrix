use std::collections::HashSet;
use std::time::{Duration, Instant};

use futures::future::join_all;

use super::discovery::{DiscoveredModel, ModelSource};
use super::factory::LlmProviderType;
use super::free_catalog; // for classify_tier

/// 互联网免费模型发现器
///
/// 多源并行探测 → 连通性验证 → 去重合并
/// 所有发现结果自动注入 ModelRegistry
pub struct InternetModelDiscovery {
    cache: Vec<DiscoveredModel>,
    last_refresh: Option<Instant>,
    cache_ttl: Duration,
}

/// 动态探测源配置
#[derive(Debug, Clone)]
pub struct ProbeSource {
    pub name: String,
    pub base_url: String,
    pub provider_type: LlmProviderType,
    pub requires_api_key: bool,
    pub api_key_env: Option<String>,
    pub api_key_value: Option<String>,
    pub is_free: bool,
}

/// 已知免费提供商（需 API Key 但免费额度，不可动态探测）
#[derive(Debug, Clone)]
pub struct KnownFreeProvider {
    pub provider_id: String,
    pub models: Vec<KnownFreeModel>,
    pub base_url: String,
    pub api_key_env: &'static str,
    pub provider_type: LlmProviderType,
    pub signup_url: &'static str,
}

#[derive(Debug, Clone)]
pub struct KnownFreeModel {
    pub model_id: &'static str,
    pub display_name: &'static str,
    pub tier: &'static str,
}

impl InternetModelDiscovery {
    pub fn new() -> Self {
        Self {
            cache: Vec::new(),
            last_refresh: None,
            cache_ttl: Duration::from_secs(300),
        }
    }

    pub fn with_cache_ttl(mut self, ttl: Duration) -> Self {
        self.cache_ttl = ttl;
        self
    }

    pub fn should_refresh(&self) -> bool {
        match self.last_refresh {
            Some(t) => t.elapsed() >= self.cache_ttl,
            None => true,
        }
    }

    /// 动态探测源（不需要 API Key 即可列出模型）
    pub fn dynamic_sources() -> Vec<ProbeSource> {
        vec![
            ProbeSource {
                name: "opencode".into(),
                base_url: "https://opencode.ai/zen/v1".into(),
                provider_type: LlmProviderType::OpencodeFree,
                requires_api_key: false,
                api_key_env: None,
                api_key_value: Some("public".into()),
                is_free: true,
            },
            ProbeSource {
                name: "openrouter".into(),
                base_url: "https://openrouter.ai/api/v1".into(),
                provider_type: LlmProviderType::FreeApi,
                requires_api_key: false,
                api_key_env: None,
                api_key_value: None,
                is_free: true,
            },
        ]
    }

    /// 已知免费提供商（需 API Key，但免费额度）
    pub fn known_free_providers() -> Vec<KnownFreeProvider> {
        vec![
            KnownFreeProvider {
                provider_id: "groq".into(),
                base_url: "https://api.groq.com/openai/v1".into(),
                api_key_env: "GROQ_API_KEY",
                provider_type: LlmProviderType::OpenAI,
                signup_url: "https://console.groq.com/keys",
                models: vec![
                    KnownFreeModel {
                        model_id: "llama-4-scout-17b-16e-instruct",
                        display_name: "Llama 4 Scout 17B",
                        tier: "t1-standard",
                    },
                    KnownFreeModel {
                        model_id: "llama-3.3-70b-versatile",
                        display_name: "Llama 3.3 70B",
                        tier: "t3-powerful",
                    },
                    KnownFreeModel {
                        model_id: "llama-3.1-8b-instant",
                        display_name: "Llama 3.1 8B Instant",
                        tier: "t1-standard",
                    },
                    KnownFreeModel {
                        model_id: "mixtral-8x7b-32768",
                        display_name: "Mixtral 8x7B",
                        tier: "t2-balanced",
                    },
                    KnownFreeModel {
                        model_id: "gemma2-9b-it",
                        display_name: "Gemma 2 9B",
                        tier: "t1-standard",
                    },
                    KnownFreeModel {
                        model_id: "deepseek-r1-distill-llama-70b",
                        display_name: "DeepSeek R1 Distill 70B",
                        tier: "t4-frontier",
                    },
                ],
            },
            KnownFreeProvider {
                provider_id: "cerebras".into(),
                base_url: "https://api.cerebras.ai/v1".into(),
                api_key_env: "CEREBRAS_API_KEY",
                provider_type: LlmProviderType::OpenAI,
                signup_url: "https://cloud.cerebras.ai/",
                models: vec![
                    KnownFreeModel {
                        model_id: "gpt-oss-120b",
                        display_name: "GPT-OSS 120B",
                        tier: "t3-powerful",
                    },
                    KnownFreeModel {
                        model_id: "llama-3.3-70b",
                        display_name: "Llama 3.3 70B",
                        tier: "t3-powerful",
                    },
                ],
            },
            KnownFreeProvider {
                provider_id: "sambanova".into(),
                base_url: "https://api.sambanova.ai/v1".into(),
                api_key_env: "SAMBANOVA_API_KEY",
                provider_type: LlmProviderType::OpenAI,
                signup_url: "https://cloud.sambanova.ai/",
                models: vec![KnownFreeModel {
                    model_id: "llama-3.3-70b",
                    display_name: "Llama 3.3 70B",
                    tier: "t3-powerful",
                }],
            },
            KnownFreeProvider {
                provider_id: "deepseek".into(),
                base_url: "https://api.deepseek.com/v1".into(),
                api_key_env: "DEEPSEEK_API_KEY",
                provider_type: LlmProviderType::OpenAI,
                signup_url: "https://platform.deepseek.com/",
                models: vec![
                    KnownFreeModel {
                        model_id: "deepseek-chat",
                        display_name: "DeepSeek V3",
                        tier: "t4-frontier",
                    },
                    KnownFreeModel {
                        model_id: "deepseek-reasoner",
                        display_name: "DeepSeek R1",
                        tier: "t4-frontier",
                    },
                ],
            },
            KnownFreeProvider {
                provider_id: "mistral".into(),
                base_url: "https://api.mistral.ai/v1".into(),
                api_key_env: "MISTRAL_API_KEY",
                provider_type: LlmProviderType::OpenAI,
                signup_url: "https://console.mistral.ai/",
                models: vec![
                    KnownFreeModel {
                        model_id: "mistral-small-latest",
                        display_name: "Mistral Small",
                        tier: "t1-standard",
                    },
                    KnownFreeModel {
                        model_id: "codestral-latest",
                        display_name: "Codestral",
                        tier: "t3-powerful",
                    },
                ],
            },
            KnownFreeProvider {
                provider_id: "cloudflare".into(),
                base_url: "https://api.cloudflare.com/client/v4/accounts/{account_id}/ai/v1".into(),
                api_key_env: "CLOUDFLARE_API_KEY",
                provider_type: LlmProviderType::OpenAI,
                signup_url: "https://dash.cloudflare.com/",
                models: vec![KnownFreeModel {
                    model_id: "@cf/meta/llama-3.3-70b-instruct",
                    display_name: "Llama 3.3 70B (Cloudflare)",
                    tier: "t3-powerful",
                }],
            },
            KnownFreeProvider {
                provider_id: "github-models".into(),
                base_url: "https://models.inference.ai.azure.com".into(),
                api_key_env: "GITHUB_TOKEN",
                provider_type: LlmProviderType::OpenAI,
                signup_url: "https://github.com/settings/tokens",
                models: vec![
                    KnownFreeModel {
                        model_id: "gpt-4o-mini",
                        display_name: "GPT-4o Mini",
                        tier: "t1-standard",
                    },
                    KnownFreeModel {
                        model_id: "gpt-4o",
                        display_name: "GPT-4o",
                        tier: "t3-powerful",
                    },
                ],
            },
        ]
    }

    /// 从所有源发现免费模型（动态探测 + 已知提供商）
    pub async fn discover_all() -> Vec<DiscoveredModel> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build();

        let client = match client {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };

        // 1. 并行探测动态源
        let sources = Self::dynamic_sources();
        let probe_futs: Vec<_> = sources
            .iter()
            .map(|s| Self::probe_source(&client, s))
            .collect();
        let probe_results = join_all(probe_futs).await;

        // 2. 已知免费提供商（无需网络探测）
        let known_providers = Self::known_free_providers();
        let known_models: Vec<DiscoveredModel> = known_providers
            .into_iter()
            .flat_map(|p| {
                p.models
                    .into_iter()
                    .map(|m| {
                        let tier = if m.tier.is_empty() {
                            "t1-standard"
                        } else {
                            m.tier
                        };
                        let env = p.api_key_env.to_string();
                        DiscoveredModel {
                            provider_id: p.provider_id.clone(),
                            model_id: m.model_id.to_string(),
                            display_name: format!("{} ({})", m.display_name, p.provider_id),
                            provider_type: p.provider_type,
                            base_url: p.base_url.clone(),
                            requires_api_key: true,
                            api_key_env: Some(env),
                            source: ModelSource::DynamicDiscovery,
                            is_free: true,
                            tier: tier.to_string(),
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        // 3. 合并 + 去重
        let mut all = Vec::new();
        for models in probe_results {
            all.extend(models);
        }
        all.extend(known_models);

        let mut seen = HashSet::new();
        all.retain(|m| seen.insert((m.provider_id.clone(), m.model_id.clone())));
        all
    }

    /// 探测单个源（共享 client）
    pub async fn probe_source(
        client: &reqwest::Client,
        source: &ProbeSource,
    ) -> Vec<DiscoveredModel> {
        match source.name.as_str() {
            "opencode" => Self::probe_opencode_zen(client, source).await,
            "openrouter" => Self::probe_openrouter_free(client, source).await,
            _ => Self::probe_openai_compatible(client, source).await,
        }
    }

    /// 探测 opencode.ai Zen API
    async fn probe_opencode_zen(
        client: &reqwest::Client,
        source: &ProbeSource,
    ) -> Vec<DiscoveredModel> {
        let url = format!("{}/models", source.base_url.trim_end_matches('/'));
        let mut req = client.get(&url);
        if let Some(key) = &source.api_key_value {
            req = req.header("Authorization", format!("Bearer {}", key));
        }

        let resp = match req.send().await {
            Ok(r) if r.status().is_success() || r.status().as_u16() == 403 => r,
            _ => return vec![],
        };

        // 解析模型列表
        let body: serde_json::Value = match resp.json().await {
            Ok(v) => v,
            Err(_) => return fallback_opencode_models(source),
        };

        let models = match body.get("data") {
            Some(data) => data.as_array(),
            None if body.is_array() => body.as_array(),
            _ => None,
        };

        let models = match models {
            Some(m) => m,
            None => return fallback_opencode_models(source),
        };

        let discovered: Vec<DiscoveredModel> = models
            .iter()
            .filter_map(|m| {
                let id = m.get("id").and_then(|v| v.as_str())?;
                if id.is_empty() {
                    return None;
                }
                let display = m
                    .get("display_name")
                    .or_else(|| m.get("name"))
                    .and_then(|v| v.as_str())
                    .unwrap_or(id);
                let tier = free_catalog::classify_tier(id);
                Some(DiscoveredModel {
                    provider_id: "opencode".into(),
                    model_id: id.to_string(),
                    display_name: format!("{} (OpenCode)", display),
                    provider_type: LlmProviderType::OpencodeFree,
                    base_url: source.base_url.clone(),
                    requires_api_key: false,
                    api_key_env: None,
                    source: ModelSource::DynamicDiscovery,
                    is_free: true,
                    tier,
                })
            })
            .collect();

        if discovered.is_empty() {
            return fallback_opencode_models(source);
        }
        discovered
    }

    /// 探测 OpenRouter 免费模型
    async fn probe_openrouter_free(
        client: &reqwest::Client,
        source: &ProbeSource,
    ) -> Vec<DiscoveredModel> {
        let url = format!("{}/models", source.base_url.trim_end_matches('/'));
        let resp = match client.get(&url).send().await {
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

        models
            .iter()
            .filter_map(|m| {
                let id = m.get("id").and_then(|v| v.as_str())?;
                let is_free = id.ends_with(":free");
                let pricing = m.get("pricing")?;
                let prompt_price = pricing
                    .get("prompt")
                    .and_then(|v| v.as_str())
                    .unwrap_or("inf");
                let completion_price = pricing
                    .get("completion")
                    .and_then(|v| v.as_str())
                    .unwrap_or("inf");
                if prompt_price != "0" && completion_price != "0" && !is_free {
                    return None;
                }
                let display = m.get("name").and_then(|v| v.as_str()).unwrap_or(id);
                let tier = free_catalog::classify_tier(id);
                Some(DiscoveredModel {
                    provider_id: "openrouter".into(),
                    model_id: id.to_string(),
                    display_name: format!("{} (OpenRouter)", display),
                    provider_type: LlmProviderType::FreeApi,
                    base_url: source.base_url.clone(),
                    requires_api_key: true,
                    api_key_env: Some("OPENROUTER_API_KEY".into()),
                    source: ModelSource::DynamicDiscovery,
                    is_free: true,
                    tier,
                })
            })
            .collect()
    }

    /// 探测通用 OpenAI 兼容端点
    async fn probe_openai_compatible(
        client: &reqwest::Client,
        source: &ProbeSource,
    ) -> Vec<DiscoveredModel> {
        let url = format!("{}/models", source.base_url.trim_end_matches('/'));
        let mut req = client.get(&url);
        if let Some(key) = &source.api_key_value {
            req = req.header("Authorization", format!("Bearer {}", key));
        }

        let resp = match req.send().await {
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

        let provider_id = source.name.clone();
        models
            .iter()
            .take(20)
            .filter_map(|m| {
                let id = m.get("id").and_then(|v| v.as_str())?;
                if id.is_empty() {
                    return None;
                }
                let tier = free_catalog::classify_tier(id);
                Some(DiscoveredModel {
                    provider_id: provider_id.clone(),
                    model_id: id.to_string(),
                    display_name: format!("{} ({})", id, source.name),
                    provider_type: source.provider_type,
                    base_url: source.base_url.clone(),
                    requires_api_key: source.requires_api_key,
                    api_key_env: source.api_key_env.clone(),
                    source: ModelSource::DynamicDiscovery,
                    is_free: source.is_free,
                    tier,
                })
            })
            .collect()
    }

    /// 运行完整发现周期（带缓存）
    pub async fn refresh(&mut self) -> &[DiscoveredModel] {
        if !self.should_refresh() {
            return &self.cache;
        }
        let discovered = Self::discover_all().await;
        self.cache = discovered;
        self.last_refresh = Some(Instant::now());
        &self.cache
    }

    pub async fn force_refresh(&mut self) -> &[DiscoveredModel] {
        self.last_refresh = None;
        self.refresh().await
    }
}

/// opencode Zen API 回退硬编码列表
fn fallback_opencode_models(source: &ProbeSource) -> Vec<DiscoveredModel> {
    let known: &[(&str, &str, &str)] = &[
        (
            "deepseek-v4-flash-free",
            "DeepSeek V4 Flash Free",
            "t4-frontier",
        ),
        ("kimi-k2.5-free", "Kimi K2.5 Free", "t4-frontier"),
        (
            "nemotron-3-ultra-free",
            "Nemotron 3 Ultra Free",
            "t4-frontier",
        ),
        ("qwen3.6-plus-free", "Qwen3.6 Plus Free", "t3-powerful"),
        ("big-pickle", "Big Pickle", "t2-balanced"),
        (
            "north-mini-code-free",
            "North Mini Code Free",
            "t1-standard",
        ),
        ("ling-2.6-flash-free", "Ling 2.6 Flash Free", "t2-balanced"),
        ("glm-5-free", "GLM-5 Free", "t3-powerful"),
        ("grok-code", "Grok Code Fast 1", "t2-balanced"),
        ("ring-2.6-1t-free", "Ring 2.6 1T Free", "t3-powerful"),
        (
            "trinity-large-preview-free",
            "Trinity Large Preview",
            "t3-powerful",
        ),
    ];
    known
        .iter()
        .map(|(id, display, tier)| DiscoveredModel {
            provider_id: "opencode".into(),
            model_id: id.to_string(),
            display_name: display.to_string(),
            provider_type: LlmProviderType::OpencodeFree,
            base_url: source.base_url.clone(),
            requires_api_key: false,
            api_key_env: None,
            source: ModelSource::DynamicDiscovery,
            is_free: true,
            tier: tier.to_string(),
        })
        .collect()
}

impl Default for InternetModelDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_tier() {
        assert_eq!(
            free_catalog::classify_tier("deepseek-v4-flash-free"),
            "t4-frontier"
        );
        assert_eq!(free_catalog::classify_tier("gpt-4o-mini"), "t3-powerful");
        assert_eq!(free_catalog::classify_tier("mixtral-8x7b"), "t2-balanced");
        assert_eq!(free_catalog::classify_tier("llama-3.2-3b"), "t0-cheap");
        assert_eq!(free_catalog::classify_tier("llama-3.1-8b"), "t1-standard");
        assert_eq!(free_catalog::classify_tier("gemma-4-9b"), "t3-powerful");
    }

    #[test]
    fn test_dynamic_sources() {
        let sources = InternetModelDiscovery::dynamic_sources();
        assert_eq!(sources.len(), 2);
        assert!(sources.iter().any(|s| s.name == "opencode"));
        assert!(sources.iter().any(|s| s.name == "openrouter"));
    }

    #[test]
    fn test_known_free_providers() {
        let providers = InternetModelDiscovery::known_free_providers();
        assert!(
            providers.len() >= 6,
            "expected >=6 known providers, got {}",
            providers.len()
        );
        let names: HashSet<&str> = providers.iter().map(|p| p.provider_id.as_str()).collect();
        assert!(names.contains("groq"), "groq should be in known providers");
        assert!(
            names.contains("deepseek"),
            "deepseek should be in known providers"
        );
        assert!(
            names.contains("cerebras"),
            "cerebras should be in known providers"
        );
    }

    #[test]
    fn test_fallback_opencode_has_models() {
        let source = ProbeSource {
            name: "opencode".into(),
            base_url: "https://opencode.ai/zen/v1".into(),
            provider_type: LlmProviderType::OpencodeFree,
            requires_api_key: false,
            api_key_env: None,
            api_key_value: Some("public".into()),
            is_free: true,
        };
        let models = fallback_opencode_models(&source);
        assert!(!models.is_empty());
        assert!(models.iter().all(|m| m.provider_id == "opencode"));
        assert!(models.iter().all(|m| !m.requires_api_key));
    }

    #[test]
    fn test_cache_ttl() {
        let mut d = InternetModelDiscovery::new();
        assert!(d.should_refresh());
        d.last_refresh = Some(Instant::now());
        assert!(!d.should_refresh());
        d.last_refresh = Some(Instant::now() - Duration::from_secs(600));
        assert!(d.should_refresh());
    }

    #[test]
    fn test_no_duplicate_ids_in_fallback() {
        let source = ProbeSource {
            name: "opencode".into(),
            base_url: "https://opencode.ai/zen/v1".into(),
            provider_type: LlmProviderType::OpencodeFree,
            requires_api_key: false,
            api_key_env: None,
            api_key_value: Some("public".into()),
            is_free: true,
        };
        let models = fallback_opencode_models(&source);
        let mut ids = HashSet::new();
        for m in &models {
            assert!(ids.insert(m.model_id.clone()), "duplicate: {}", m.model_id);
        }
    }

    #[test]
    fn test_known_provider_env_vars_are_standards() {
        let providers = InternetModelDiscovery::known_free_providers();
        for p in &providers {
            assert!(
                !p.api_key_env.is_empty(),
                "provider {} has empty api_key_env",
                p.provider_id
            );
            assert!(
                p.signup_url.starts_with("https://"),
                "provider {} signup_url not https",
                p.provider_id
            );
        }
    }
}
