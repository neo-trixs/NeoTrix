use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use super::factory::LlmProviderType;

/// Classify a model name into a tier string (t4-frontier, t3-powerful, t2-balanced, t1-standard, t0-cheap).
pub fn classify_tier(name: &str) -> String {
    FreeModelCatalog::classify_tier(name)
}

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
        Self {
            entries: Vec::new(),
        }
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
                    requires_api_key: false,
                    api_key_env: None,
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
                base_url: base.into(),
                tier: "t0-cheap".into(),
                is_free: true,
                requires_api_key: true,
                api_key_env: Some("GROQ_API_KEY".into()),
                provider_type: LlmProviderType::FreeApi,
            },
            FreeModelEntry {
                provider: "groq".into(),
                model_id: "gemma2-9b-it".into(),
                display_name: "Gemma 2 9B IT (Groq)".into(),
                base_url: base.into(),
                tier: "t0-cheap".into(),
                is_free: true,
                requires_api_key: true,
                api_key_env: Some("GROQ_API_KEY".into()),
                provider_type: LlmProviderType::FreeApi,
            },
            FreeModelEntry {
                provider: "groq".into(),
                model_id: "llama-3.3-70b-versatile".into(),
                display_name: "Llama 3.3 70B (Groq)".into(),
                base_url: base.into(),
                tier: "t3-powerful".into(),
                is_free: true,
                requires_api_key: true,
                api_key_env: Some("GROQ_API_KEY".into()),
                provider_type: LlmProviderType::FreeApi,
            },
            FreeModelEntry {
                provider: "groq".into(),
                model_id: "mixtral-8x7b-32768".into(),
                display_name: "Mixtral 8x7B 32K (Groq)".into(),
                base_url: base.into(),
                tier: "t2-balanced".into(),
                is_free: true,
                requires_api_key: true,
                api_key_env: Some("GROQ_API_KEY".into()),
                provider_type: LlmProviderType::FreeApi,
            },
            FreeModelEntry {
                provider: "groq".into(),
                model_id: "deepseek-r1-distill-llama-70b".into(),
                display_name: "DeepSeek R1 Distill 70B (Groq)".into(),
                base_url: base.into(),
                tier: "t4-frontier".into(),
                is_free: true,
                requires_api_key: true,
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
        let mut seen = HashSet::new();
        merged.retain(|e| seen.insert(e.model_id.clone()));
        self.entries = merged.clone();
        merged
    }

    /// 格式化显示
    pub fn format_list(entries: &[FreeModelEntry]) -> String {
        let mut output = format!(
            "╭─ Free Models ({}) ─────────────────────────╮\n",
            entries.len()
        );
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
                        _ => "\u{1f4e6}",
                    };
                    let key = if m.requires_api_key {
                        " \u{1f511}"
                    } else {
                        " \u{1f193}"
                    };
                    output.push_str(&format!("    {tag}{key}  {}/{}\n", m.provider, m.model_id));
                }
            }
        }
        output.push_str("\u{2570}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}");
        output
    }

    pub fn classify_tier(name: &str) -> String {
        let c = name.to_lowercase();
        // t4-frontier: May 2026 frontier models
        if c.contains("claude")
            || c.contains("deepseek")
            || c.contains("gpt-5")
            || c.contains("gemini-2.5")
            || c.contains("kimi")
            || c.contains("glm-5")
            || c.contains("qwen3")
            || c.contains("mimo-v2")
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
        if c.contains("8b")
            || c.contains("7b")
            || c.contains("9b")
            || c.contains("13b")
            || c.contains("scout")
        {
            return "t1-standard".into();
        }
        if c.contains("3b") || c.contains("2b") || c.contains("1b") || c.contains("nano") {
            return "t0-cheap".into();
        }
        "t1-standard".into()
    }
}
