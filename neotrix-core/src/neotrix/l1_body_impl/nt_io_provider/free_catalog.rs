use std::collections::{HashMap, HashSet};
use std::time::Duration;

use serde::{Deserialize, Serialize};

use super::factory::LlmProviderType;

/// 免费模型条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreeModelEntry {
    pub provider: String,
    pub model_id: String,
    pub display_name: String,
    pub base_url: String,
    pub tier: String,
    pub is_free: bool,
    pub requires_api_key: bool,
    pub api_key_env: Option<String>,
    pub provider_type: LlmProviderType,
}

/// 免费模型目录 — 聚合 OpenRouter 免费层 + Groq 等社区免费模型
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FreeModelCatalog {
    pub entries: Vec<FreeModelEntry>,
}

#[derive(Debug, Deserialize)]
struct OpenRouterResponse {
    data: Vec<OpenRouterModel>,
}

#[derive(Debug, Deserialize)]
struct OpenRouterModel {
    id: String,
    name: Option<String>,
    pricing: OpenRouterPricing,
}

#[derive(Debug, Deserialize)]
struct OpenRouterPricing {
    #[serde(default)]
    prompt: String,
}

impl FreeModelCatalog {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    /// 从 OpenRouter API 获取免费模型
    pub fn discover_openrouter_free() -> Vec<FreeModelEntry> {
        let resp = match reqwest::blocking::get("https://openrouter.ai/api/v1/models") {
            Ok(r) if r.status().is_success() => r,
            _ => return Vec::new(),
        };
        let body: OpenRouterResponse = match resp.json() {
            Ok(b) => b,
            Err(_) => return Vec::new(),
        };
        body.data
            .into_iter()
            .filter(|m| m.pricing.prompt.trim() == "0")
            .map(|m| {
                let display_name = m.name.unwrap_or_else(|| m.id.clone());
                let tier = Self::classify_tier(&display_name);
                FreeModelEntry {
                    provider: "openrouter".into(),
                    model_id: m.id,
                    display_name,
                    base_url: "https://openrouter.ai/api/v1".into(),
                    tier,
                    is_free: true,
                    requires_api_key: true,
                    api_key_env: Some("OPENROUTER_API_KEY".into()),
                    provider_type: LlmProviderType::FreeApi,
                }
            })
            .collect()
    }

    /// 硬编码 Groq 免费模型列表
    pub fn discover_groq_models() -> Vec<FreeModelEntry> {
        let base = "https://api.groq.com/openai/v1";
        vec![
            FreeModelEntry {
                provider: "groq".into(),
                model_id: "llama-4-scout-17b-16e-instruct".into(),
                display_name: "Llama 4 Scout 17B (Groq)".into(),
                base_url: base.into(), tier: "t0-cheap".into(),
                is_free: true, requires_api_key: true,
                api_key_env: Some("GROQ_API_KEY".into()),
                provider_type: LlmProviderType::FreeApi,
            },
            FreeModelEntry {
                provider: "groq".into(),
                model_id: "gemma2-9b-it".into(),
                display_name: "Gemma 2 9B IT (Groq)".into(),
                base_url: base.into(), tier: "t0-cheap".into(),
                is_free: true, requires_api_key: true,
                api_key_env: Some("GROQ_API_KEY".into()),
                provider_type: LlmProviderType::FreeApi,
            },
            FreeModelEntry {
                provider: "groq".into(),
                model_id: "llama-3.3-70b-versatile".into(),
                display_name: "Llama 3.3 70B (Groq)".into(),
                base_url: base.into(), tier: "t3-powerful".into(),
                is_free: true, requires_api_key: true,
                api_key_env: Some("GROQ_API_KEY".into()),
                provider_type: LlmProviderType::FreeApi,
            },
            FreeModelEntry {
                provider: "groq".into(),
                model_id: "mixtral-8x7b-32768".into(),
                display_name: "Mixtral 8x7B 32K (Groq)".into(),
                base_url: base.into(), tier: "t2-balanced".into(),
                is_free: true, requires_api_key: true,
                api_key_env: Some("GROQ_API_KEY".into()),
                provider_type: LlmProviderType::FreeApi,
            },
            FreeModelEntry {
                provider: "groq".into(),
                model_id: "deepseek-r1-distill-llama-70b".into(),
                display_name: "DeepSeek R1 Distill 70B (Groq)".into(),
                base_url: base.into(), tier: "t4-frontier".into(),
                is_free: true, requires_api_key: true,
                api_key_env: Some("GROQ_API_KEY".into()),
                provider_type: LlmProviderType::FreeApi,
            },
        ]
    }

    /// 刷新模型列表（从所有来源重新发现，去重）
    pub fn refresh(&mut self) -> Vec<FreeModelEntry> {
        let mut merged = Vec::new();
        merged.extend(Self::discover_openrouter_free());
        merged.extend(Self::discover_groq_models());
        merged.extend(Self::discover_cloudflare_models());
        merged.extend(Self::discover_nvidia_models());
        merged.extend(Self::discover_github_models());
        merged.extend(Self::discover_huggingface_models());
        merged.extend(Self::discover_keyless_providers());
        merged.extend(Self::discover_together_free());
        merged.extend(Self::discover_siliconflow_models());
        merged.extend(Self::discover_zai_models());
        merged.extend(Self::discover_api_airforce_models());
        let mut seen = HashSet::new();
        merged.retain(|e| seen.insert(format!("{}/{}", e.provider, e.model_id)));
        self.entries = merged.clone();
        merged
    }

    /// 发现 Cloudflare Workers AI 免费模型
    pub fn discover_cloudflare_models() -> Vec<FreeModelEntry> {
        let base = "https://api.cloudflare.com/client/v4/ai";
        vec![
            FreeModelEntry {
                provider: "cloudflare".into(),
                model_id: "@cf/meta/llama-3.3-70b-instruct-fp8-fast".into(),
                display_name: "Llama 3.3 70B (Cloudflare)".into(),
                base_url: base.into(), tier: "t3-powerful".into(),
                is_free: true, requires_api_key: true,
                api_key_env: Some("CLOUDFLARE_API_TOKEN".into()),
                provider_type: LlmProviderType::Cloudflare,
            },
            FreeModelEntry {
                provider: "cloudflare".into(),
                model_id: "@cf/meta/llama-3.1-8b-instruct".into(),
                display_name: "Llama 3.1 8B (Cloudflare)".into(),
                base_url: base.into(), tier: "t1-standard".into(),
                is_free: true, requires_api_key: true,
                api_key_env: Some("CLOUDFLARE_API_TOKEN".into()),
                provider_type: LlmProviderType::Cloudflare,
            },
            FreeModelEntry {
                provider: "cloudflare".into(),
                model_id: "@hf/deepseek-r1-distill-qwen-32b".into(),
                display_name: "DeepSeek R1 Distill 32B (Cloudflare)".into(),
                base_url: base.into(), tier: "t3-powerful".into(),
                is_free: true, requires_api_key: true,
                api_key_env: Some("CLOUDFLARE_API_TOKEN".into()),
                provider_type: LlmProviderType::Cloudflare,
            },
        ]
    }

    /// 发现 NVIDIA NIM 免费模型
    pub fn discover_nvidia_models() -> Vec<FreeModelEntry> {
        let base = "https://integrate.api.nvidia.com/v1";
        vec![
            FreeModelEntry {
                provider: "nvidia".into(),
                model_id: "meta/llama-3.3-70b-instruct".into(),
                display_name: "Llama 3.3 70B (NVIDIA)".into(),
                base_url: base.into(), tier: "t3-powerful".into(),
                is_free: true, requires_api_key: true,
                api_key_env: Some("NVIDIA_API_KEY".into()),
                provider_type: LlmProviderType::Nvidia,
            },
            FreeModelEntry {
                provider: "nvidia".into(),
                model_id: "nvidia/llama-3.1-nemotron-70b-instruct".into(),
                display_name: "Nemotron 70B (NVIDIA)".into(),
                base_url: base.into(), tier: "t3-powerful".into(),
                is_free: true, requires_api_key: true,
                api_key_env: Some("NVIDIA_API_KEY".into()),
                provider_type: LlmProviderType::Nvidia,
            },
            FreeModelEntry {
                provider: "nvidia".into(),
                model_id: "deepseek-ai/deepseek-r1".into(),
                display_name: "DeepSeek R1 (NVIDIA)".into(),
                base_url: base.into(), tier: "t4-frontier".into(),
                is_free: true, requires_api_key: true,
                api_key_env: Some("NVIDIA_API_KEY".into()),
                provider_type: LlmProviderType::Nvidia,
            },
        ]
    }

    /// 发现 GitHub Models 免费模型
    pub fn discover_github_models() -> Vec<FreeModelEntry> {
        let base = "https://models.inference.ai.azure.com/v1";
        vec![
            FreeModelEntry {
                provider: "github-models".into(),
                model_id: "gpt-4o-mini".into(),
                display_name: "GPT-4o Mini (GitHub)".into(),
                base_url: base.into(), tier: "t2-balanced".into(),
                is_free: true, requires_api_key: true,
                api_key_env: Some("GITHUB_TOKEN".into()),
                provider_type: LlmProviderType::GitHubModels,
            },
            FreeModelEntry {
                provider: "github-models".into(),
                model_id: "gpt-4.1-mini".into(),
                display_name: "GPT-4.1 Mini (GitHub)".into(),
                base_url: base.into(), tier: "t3-powerful".into(),
                is_free: true, requires_api_key: true,
                api_key_env: Some("GITHUB_TOKEN".into()),
                provider_type: LlmProviderType::GitHubModels,
            },
            FreeModelEntry {
                provider: "github-models".into(),
                model_id: "meta-llama-3.3-70b-instruct".into(),
                display_name: "Llama 3.3 70B (GitHub)".into(),
                base_url: base.into(), tier: "t3-powerful".into(),
                is_free: true, requires_api_key: true,
                api_key_env: Some("GITHUB_TOKEN".into()),
                provider_type: LlmProviderType::GitHubModels,
            },
            FreeModelEntry {
                provider: "github-models".into(),
                model_id: "Phi-4".into(),
                display_name: "Phi-4 (GitHub)".into(),
                base_url: base.into(), tier: "t1-standard".into(),
                is_free: true, requires_api_key: true,
                api_key_env: Some("GITHUB_TOKEN".into()),
                provider_type: LlmProviderType::GitHubModels,
            },
        ]
    }

    /// 发现 HuggingFace 免费模型
    pub fn discover_huggingface_models() -> Vec<FreeModelEntry> {
        let base = "https://api-inference.huggingface.co/v1";
        vec![
            FreeModelEntry {
                provider: "huggingface".into(),
                model_id: "meta-llama/Llama-3.3-70B-Instruct".into(),
                display_name: "Llama 3.3 70B (HuggingFace)".into(),
                base_url: base.into(), tier: "t3-powerful".into(),
                is_free: true, requires_api_key: true,
                api_key_env: Some("HF_API_KEY".into()),
                provider_type: LlmProviderType::HuggingFace,
            },
            FreeModelEntry {
                provider: "huggingface".into(),
                model_id: "mistralai/Mistral-7B-Instruct-v0.3".into(),
                display_name: "Mistral 7B (HuggingFace)".into(),
                base_url: base.into(), tier: "t1-standard".into(),
                is_free: true, requires_api_key: true,
                api_key_env: Some("HF_API_KEY".into()),
                provider_type: LlmProviderType::HuggingFace,
            },
        ]
    }

    /// 发现 Keyless/Key-required 免费提供者 (OpenCode Zen free models with free-tier API key)
    pub fn discover_keyless_providers() -> Vec<FreeModelEntry> {
        let mut entries = Vec::new();
        // Only OpenCode Zen — verified working with free tier API key (opencode.ai)
        // LLM7/Kilo/OVH/ModelScope endpoints all dead as of 2026-07-22 (404/timeout)
        let zen_base = "https://opencode.ai/zen/v1";
        entries.push(FreeModelEntry {
            provider: "opencode-zen".into(),
            model_id: "deepseek-v4-flash-free".into(),
            display_name: "DeepSeek V4 Flash Free (OpenCode)".into(),
            base_url: zen_base.into(), tier: "t4-frontier".into(),
            is_free: true, requires_api_key: true,
            api_key_env: Some("OPENCODE_API_KEY".into()),
            provider_type: LlmProviderType::OpenCodeZen,
        });
        entries.push(FreeModelEntry {
            provider: "opencode-zen".into(),
            model_id: "mimo-v2.5-free".into(),
            display_name: "MiMo V2.5 Free (OpenCode)".into(),
            base_url: zen_base.into(), tier: "t4-frontier".into(),
            is_free: true, requires_api_key: true,
            api_key_env: Some("OPENCODE_API_KEY".into()),
            provider_type: LlmProviderType::OpenCodeZen,
        });
        entries.push(FreeModelEntry {
            provider: "opencode-zen".into(),
            model_id: "qwen3.6-plus-free".into(),
            display_name: "Qwen 3.6 Plus Free (OpenCode)".into(),
            base_url: zen_base.into(), tier: "t4-frontier".into(),
            is_free: true, requires_api_key: true,
            api_key_env: Some("OPENCODE_API_KEY".into()),
            provider_type: LlmProviderType::OpenCodeZen,
        });
        entries.push(FreeModelEntry {
            provider: "opencode-zen".into(),
            model_id: "minimax-m3-free".into(),
            display_name: "MiniMax M3 Free (OpenCode)".into(),
            base_url: zen_base.into(), tier: "t3-powerful".into(),
            is_free: true, requires_api_key: true,
            api_key_env: Some("OPENCODE_API_KEY".into()),
            provider_type: LlmProviderType::OpenCodeZen,
        });
        entries.push(FreeModelEntry {
            provider: "opencode-zen".into(),
            model_id: "nemotron-3-ultra-free".into(),
            display_name: "Nemotron 3 Ultra Free (OpenCode)".into(),
            base_url: zen_base.into(), tier: "t3-powerful".into(),
            is_free: true, requires_api_key: true,
            api_key_env: Some("OPENCODE_API_KEY".into()),
            provider_type: LlmProviderType::OpenCodeZen,
        });
        entries.push(FreeModelEntry {
            provider: "opencode-zen".into(),
            model_id: "north-mini-code-free".into(),
            display_name: "North Mini Code Free (OpenCode)".into(),
            base_url: zen_base.into(), tier: "t1-standard".into(),
            is_free: true, requires_api_key: true,
            api_key_env: Some("OPENCODE_API_KEY".into()),
            provider_type: LlmProviderType::OpenCodeZen,
        });
        entries.push(FreeModelEntry {
            provider: "opencode-zen".into(),
            model_id: "big-pickle".into(),
            display_name: "Big Pickle (OpenCode)".into(),
            base_url: zen_base.into(), tier: "t4-frontier".into(),
            is_free: true, requires_api_key: true,
            api_key_env: Some("OPENCODE_API_KEY".into()),
            provider_type: LlmProviderType::OpenCodeZen,
        });
        entries
    }

    /// 发现 Together AI 免费模型 (with -Free suffix)
    pub fn discover_together_free() -> Vec<FreeModelEntry> {
        // Try to discover from Together API
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(5))
            .build().ok();
        if let Some(client) = client {
            if let Ok(resp) = client.get("https://api.together.xyz/v1/models").send() {
                if let Ok(body) = resp.json::<serde_json::Value>() {
                    if let Some(data) = body.get("data").and_then(|d| d.as_array()) {
                        let free_models: Vec<FreeModelEntry> = data.iter()
                            .filter(|m| {
                                let id = m.get("id").and_then(|v| v.as_str()).unwrap_or("");
                                id.ends_with("-Free")
                            })
                            .map(|m| {
                                let model_id = m.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                FreeModelEntry {
                                    provider: "together-free".into(),
                                    model_id: model_id.clone(),
                                    display_name: format!("{} (Together Free)", model_id),
                                    base_url: "https://api.together.xyz/v1".into(),
                                    tier: Self::classify_tier(&model_id),
                                    is_free: true,
                                    requires_api_key: true,
                                    api_key_env: Some("TOGETHER_API_KEY".into()),
                                    provider_type: LlmProviderType::TogetherFree,
                                }
                            })
                            .collect();
                        if !free_models.is_empty() {
                            return free_models;
                        }
                    }
                }
            }
        }
        // fallback hardcoded
        let base = "https://api.together.xyz/v1";
        vec![
            FreeModelEntry {
                provider: "together-free".into(),
                model_id: "meta-llama/Llama-3.3-70B-Instruct-Turbo-Free".into(),
                display_name: "Llama 3.3 70B (Together Free)".into(),
                base_url: base.into(), tier: "t3-powerful".into(),
                is_free: true, requires_api_key: true,
                api_key_env: Some("TOGETHER_API_KEY".into()),
                provider_type: LlmProviderType::TogetherFree,
            },
            FreeModelEntry {
                provider: "together-free".into(),
                model_id: "mistralai/Mixtral-8x7B-Instruct-v0.1-Free".into(),
                display_name: "Mixtral 8x7B (Together Free)".into(),
                base_url: base.into(), tier: "t2-balanced".into(),
                is_free: true, requires_api_key: true,
                api_key_env: Some("TOGETHER_API_KEY".into()),
                provider_type: LlmProviderType::TogetherFree,
            },
        ]
    }

    /// 发现 SiliconFlow 免费模型
    pub fn discover_siliconflow_models() -> Vec<FreeModelEntry> {
        let base = "https://api.siliconflow.cn/v1";
        vec![
            FreeModelEntry {
                provider: "siliconflow".into(),
                model_id: "deepseek-ai/DeepSeek-V3".into(),
                display_name: "DeepSeek V3 (SiliconFlow)".into(),
                base_url: base.into(), tier: "t4-frontier".into(),
                is_free: true, requires_api_key: true,
                api_key_env: Some("SILICONFLOW_API_KEY".into()),
                provider_type: LlmProviderType::SiliconFlow,
            },
            FreeModelEntry {
                provider: "siliconflow".into(),
                model_id: "Qwen/Qwen3-235B-A22B".into(),
                display_name: "Qwen3 235B (SiliconFlow)".into(),
                base_url: base.into(), tier: "t4-frontier".into(),
                is_free: true, requires_api_key: true,
                api_key_env: Some("SILICONFLOW_API_KEY".into()),
                provider_type: LlmProviderType::SiliconFlow,
            },
        ]
    }

    /// 发现 Z.AI (GLM) 免费模型
    pub fn discover_zai_models() -> Vec<FreeModelEntry> {
        let base = "https://open.bigmodel.cn/api/paas/v4";
        vec![
            FreeModelEntry {
                provider: "zai".into(),
                model_id: "glm-4-flash".into(),
                display_name: "GLM-4 Flash (Z.AI)".into(),
                base_url: base.into(), tier: "t2-balanced".into(),
                is_free: true, requires_api_key: true,
                api_key_env: Some("ZAI_API_KEY".into()),
                provider_type: LlmProviderType::ZAI,
            },
            FreeModelEntry {
                provider: "zai".into(),
                model_id: "glm-4".into(),
                display_name: "GLM-4 (Z.AI)".into(),
                base_url: base.into(), tier: "t3-powerful".into(),
                is_free: true, requires_api_key: true,
                api_key_env: Some("ZAI_API_KEY".into()),
                provider_type: LlmProviderType::ZAI,
            },
        ]
    }

    /// 发现 ApiAirforce 免费模型（truly keyless）
    /// Verified 2026-07-22: 209+ models with `:free` suffix at api.airforce
    /// Accepts any Bearer token (even empty/not-needed), rate-limited to ~1 req/s
    pub fn discover_api_airforce_models() -> Vec<FreeModelEntry> {
        let base = "https://api.airforce/v1";
        vec![
            FreeModelEntry {
                provider: "api-airforce".into(),
                model_id: "grok-4.1-mini:free".into(),
                display_name: "Grok 4.1 Mini (ApiAirforce)".into(),
                base_url: base.into(), tier: "t4-frontier".into(),
                is_free: true, requires_api_key: false,
                api_key_env: None,
                provider_type: LlmProviderType::ApiAirforce,
            },
            FreeModelEntry {
                provider: "api-airforce".into(),
                model_id: "deepseek-v3.2:free".into(),
                display_name: "DeepSeek V3.2 (ApiAirforce)".into(),
                base_url: base.into(), tier: "t4-frontier".into(),
                is_free: true, requires_api_key: false,
                api_key_env: None,
                provider_type: LlmProviderType::ApiAirforce,
            },
            FreeModelEntry {
                provider: "api-airforce".into(),
                model_id: "gemma3-270m:free".into(),
                display_name: "Gemma 3 270M (ApiAirforce)".into(),
                base_url: base.into(), tier: "t0-cheap".into(),
                is_free: true, requires_api_key: false,
                api_key_env: None,
                provider_type: LlmProviderType::ApiAirforce,
            },
            FreeModelEntry {
                provider: "api-airforce".into(),
                model_id: "step-3.5-flash:free".into(),
                display_name: "Step 3.5 Flash (ApiAirforce)".into(),
                base_url: base.into(), tier: "t2-balanced".into(),
                is_free: true, requires_api_key: false,
                api_key_env: None,
                provider_type: LlmProviderType::ApiAirforce,
            },
        ]
    }

    /// 格式化显示
    pub fn format_list(entries: &[FreeModelEntry]) -> String {
        let mut output = format!("╭─ Free Models ({}) ─────────────────────────╮\n", entries.len());
        let mut by_tier: HashMap<String, Vec<&FreeModelEntry>> = HashMap::new();
        for e in entries {
            by_tier.entry(e.tier.clone()).or_default().push(e);
        }
        let mut tiers: Vec<&String> = by_tier.keys().collect();
        tiers.sort();
        for tier in &tiers {
            let label = match tier.as_str() {
                "t0-cheap" => "T0 Cheap/Fast",
                "t1-standard" => "T1 Standard",
                "t2-balanced" => "T2 Balanced",
                "t3-powerful" => "T3 Powerful",
                "t4-frontier" => "T4 Frontier",
                _ => tier.as_str(),
            };
            output.push_str(&format!("  {label}:\n"));
            if let Some(models) = by_tier.get(tier.as_str()) {
                for m in models {
                    let tag = match m.provider.as_str() {
                        "openrouter" => "\u{1f310}",
                        "groq" => "\u{26a1}",
                        "cloudflare" => "\u{2601}\u{fe0f}",
                        "nvidia" => "\u{1f4f9}",
                        "github-models" => "\u{1f5a5}\u{fe0f}",
                        "huggingface" => "\u{1f917}",
                        "llm7" => "\u{1f916}",
                        "kilo" => "\u{2696}\u{fe0f}",
                        "together-free" => "\u{1f91d}",
                        "siliconflow" => "\u{1f4a1}",
                        "zai" => "\u{1f3f4}",
                        "opencode-zen" => "\u{2728}",
                        "ovh" => "\u{2601}\u{fe0f}",
                        "deepseek-free" => "\u{1f9d0}",
                        "modelscope" => "\u{1f30d}",
                        "api-airforce" => "\u{2708}\u{fe0f}",
                        _ => "\u{1f4e6}",
                    };
                    let key = if m.requires_api_key { " \u{1f511}" } else { " \u{1f512}" };
                    output.push_str(&format!("    {tag}{key}  {}/{}\n", m.provider, m.model_id));
                }
            }
        }
        output.push_str("\u{2570}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}");
        output
    }

    fn classify_tier(name: &str) -> String {
        let c = name.to_lowercase();
        // t4-frontier: May 2026 frontier models
        if c.contains("claude") || c.contains("deepseek") || c.contains("gpt-5")
            || c.contains("gemini-2.5") || c.contains("kimi") || c.contains("glm-5")
            || c.contains("qwen3") || c.contains("mimo-v2")
        {
            return "t4-frontier".into();
        }
        if (c.contains("llama") && (c.contains("70b") || c.contains("405b")))
            || c.contains("gpt-4o")
            || c.contains("command-a")
            || (c.contains("gemma") && c.contains("4"))
            || c.contains("minimax-m2")
        {
            return "t3-powerful".into();
        }
        if c.contains("mixtral") || c.contains("qwen") || c.contains("gemma") {
            return "t2-balanced".into();
        }
        if c.contains("8b") || c.contains("7b") || c.contains("9b") || c.contains("13b") || c.contains("scout") {
            return "t1-standard".into();
        }
        if c.contains("3b") || c.contains("2b") || c.contains("1b") || c.contains("nano") {
            return "t0-cheap".into();
        }
        "t1-standard".into()
    }
}
