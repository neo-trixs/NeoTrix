//! Intent Router — translates natural language to (domain, action, args).
//!
//! Two-tier routing:
//! - Tier 1: Pattern matching (fast, zero-cost) for common commands
//! - Tier 2: LLM classification (fallback) for complex requests

use serde_json::{json, Value};
use crate::domain::{DomainRegistry, DomainError};

/// Intent parsed from user message.
#[derive(Debug, Clone)]
pub struct Intent {
    pub domain: String,
    pub action: String,
    pub args: Value,
    pub response_hint: Option<String>, // NL response template
}

pub struct IntentRouter;

impl IntentRouter {
    /// Parse user message into an intent.
    pub async fn parse(message: &str, registry: &DomainRegistry) -> Result<Intent, DomainError> {
        // Tier 1: Pattern matching
        if let Some(intent) = Self::pattern_match(message) {
            return Ok(intent);
        }
        
        // Tier 2: LLM classification
        Self::llm_classify(message, registry).await
    }

    /// Fast pattern matching for common commands.
    fn pattern_match(message: &str) -> Option<Intent> {
        let msg = message.trim().to_lowercase();
        
        // Proxy pool commands
        if msg.contains("代理") || msg.contains("proxy") {
            if msg.contains("添加") || msg.contains("add") || msg.contains("新增") {
                let url = Self::extract_url(message)?;
                return Some(Intent {
                    domain: "proxy_pool".into(),
                    action: "add".into(),
                    args: json!({"url": url}),
                    response_hint: Some(format!("已添加代理: {url}")),
                });
            }
            if msg.contains("状态") || msg.contains("status") || msg.contains("查看") {
                return Some(Intent {
                    domain: "proxy_pool".into(),
                    action: "status".into(),
                    args: json!({}),
                    response_hint: Some("代理池状态:".into()),
                });
            }
            if msg.contains("策略") || msg.contains("strategy") {
                if let Some(strategy) = Self::extract_strategy(message) {
                    return Some(Intent {
                        domain: "proxy_pool".into(),
                        action: "set_strategy".into(),
                        args: json!({"strategy": strategy}),
                        response_hint: Some(format!("已切换策略: {strategy}")),
                    });
                }
            }
        }

        // Model commands
        if msg.contains("模型") || msg.contains("model") {
            if msg.contains("列表") || msg.contains("list") || msg.contains("查看") {
                return Some(Intent {
                    domain: "model_pool".into(),
                    action: "status".into(),
                    args: json!({}),
                    response_hint: Some("模型池状态:".into()),
                });
            }
            if msg.contains("添加") || msg.contains("add") {
                return Some(Intent {
                    domain: "model_pool".into(),
                    action: "add".into(),
                    args: Self::extract_model_args(message),
                    response_hint: Some("已添加模型配置".into()),
                });
            }
        }

        // IM commands
        if msg.contains("频道") || msg.contains("channel") || msg.contains("机器人") || msg.contains("bot") {
            if msg.contains("状态") || msg.contains("status") {
                return Some(Intent {
                    domain: "im".into(),
                    action: "status".into(),
                    args: json!({}),
                    response_hint: Some("IM状态:".into()),
                });
            }
            if msg.contains("切换") || msg.contains("toggle") {
                let channel = Self::extract_channel(message)?;
                let enabled = !msg.contains("关闭") && !msg.contains("禁用") && !msg.contains("disable");
                return Some(Intent {
                    domain: "im".into(),
                    action: "toggle_channel".into(),
                    args: json!({"channel": channel, "enabled": enabled}),
                    response_hint: Some(format!("已{}频道: {channel}", if enabled { "启用" } else { "禁用" })),
                });
            }
        }

        // Provider commands → llamacpp/provider_status (no standalone provider plugin exists)
        if msg.contains("provider") || msg.contains("供应商") || msg.contains("提供商") {
            if msg.contains("列表") || msg.contains("list") || msg.contains("查看") || msg.contains("状态") || msg.contains("status") {
                return Some(Intent {
                    domain: "llamacpp".into(),
                    action: "provider_status".into(),
                    args: json!({}),
                    response_hint: Some("Provider状态:".into()),
                });
            }
        }

        // System commands → system/system_info (SystemPlugin has no `state` action)
        if msg.contains("状态") || msg.contains("status") || msg.contains("健康") || msg.contains("health") {
            return Some(Intent {
                domain: "system".into(),
                action: "system_info".into(),
                args: json!({}),
                response_hint: Some("系统状态:".into()),
            });
        }

        // Help
        if msg.contains("帮助") || msg.contains("help") || msg.contains("你能做什么") || msg.contains("功能") {
            return Some(Intent {
                domain: "chat".into(),
                action: "help".into(),
                args: json!({}),
                response_hint: Some(Self::help_text()),
            });
        }

        None
    }

    /// Fallback classification for messages that miss Tier-1 patterns.
    ///
    /// Scores every registered `(domain, action)` by keyword overlap with the
    /// message (domain/action names weighted highest, then description tokens).
    /// A real LLM classifier can replace this when wired; the scoring contract
    /// (`Intent { domain, action, args }`) stays the same.
    async fn llm_classify(message: &str, registry: &DomainRegistry) -> Result<Intent, DomainError> {
        let msg = message.to_lowercase();
        // (score, domain, action, description)
        let mut best: Option<(usize, String, String, String)> = None;

        for info in registry.list() {
            let domain_name = info.name.to_lowercase();
            let domain_hit = if msg.contains(&domain_name.as_str()) {
                domain_name.len() * 2
            } else {
                0
            };
            for action in &info.actions {
                let mut score = domain_hit;
                let action_name = action.name.to_lowercase();
                if msg.contains(&action_name.as_str()) {
                    score += action_name.len() * 3;
                }
                // Description tokens: byte-len >= 6 skips single CJK chars
                // (3 bytes each) and tiny English words; action-name
                // matching above already covers short tokens.
                for token in action.description.split(|c: char| {
                    c.is_whitespace() || "，。、；：！？（）()[]{}<>\"'".contains(c)
                }) {
                    let token = token.trim().to_lowercase();
                    if token.len() >= 6 && msg.contains(&token) {
                        score += token.len();
                    }
                }
                let is_better = match &best {
                    Some((prev, _, _, _)) => score > *prev,
                    None => score > 0,
                };
                if is_better {
                    best = Some((
                        score,
                        info.name.clone(),
                        action.name.clone(),
                        action.description.clone(),
                    ));
                }
            }
        }

        match best {
            Some((_, domain, action, description)) => Ok(Intent {
                domain,
                action,
                args: json!({}),
                response_hint: Some(description),
            }),
            None => Err(DomainError {
                code: "INTENT_NOT_FOUND".into(),
                message: format!(
                    "无法理解指令: {message}. 请尝试更具体的描述，或输入'帮助'查看支持的操作。"
                ),
                recoverable: true,
            }),
        }
    }

    fn extract_url(message: &str) -> Option<String> {
        // URL extraction: strip wrapping quotes/brackets and trailing punctuation
        // (e.g. "http://1.2.3.4:8080," or "http://h:8080。" from CJK input).
        let words: Vec<&str> = message.split_whitespace().collect();
        for word in words {
            let trimmed = word.trim_matches(|c: char| {
                "\"'()[]<>,;!?，。！？；：、）".contains(c)
            });
            let lower = trimmed.to_lowercase();
            if lower.starts_with("http://")
                || lower.starts_with("https://")
                || lower.starts_with("socks5://")
            {
                let clean = trimmed.trim_end_matches(|c: char| c == '.' || c == ',' || c == '。' || c == '，');
                if !clean.is_empty() {
                    return Some(clean.to_string());
                }
            }
        }
        None
    }

    /// Strategy names must be members of `domain::proxy_pool::VALID_STRATEGIES`.
    fn extract_strategy(message: &str) -> Option<String> {
        // Valid tokens first (exact backend vocabulary).
        for s in crate::domain::proxy_pool::VALID_STRATEGIES {
            if message.contains(s) {
                return Some(s.to_string());
            }
        }
        // Aliases → valid members.
        if message.contains("最快") || message.contains("fastest") {
            return Some("fastest".into());
        }
        if message.contains("延迟") || message.contains("低延迟") || message.contains("latency") {
            return Some("least_latency".into());
        }
        if message.contains("错误率") || message.contains("失败率") || message.contains("least_error_rate") || message.contains("least_failure") {
            return Some("least_failure".into());
        }
        if message.contains("权重") || message.contains("weighted") {
            return Some("weighted_random".into());
        }
        if message.contains("地理") || message.contains("geo") {
            return Some("geo_preferred".into());
        }
        if message.contains("轮询") || message.contains("round") {
            return Some("round_robin".into());
        }
        if message.contains("自适应") || message.contains("adaptive") {
            return Some("adaptive".into());
        }
        if message.contains("随机") || message.contains("random") {
            return Some("round_robin".into());
        }
        if message.contains("自动") || message.contains("auto") {
            return Some("auto".into());
        }
        None
    }

    fn extract_channel(message: &str) -> Option<String> {
        let channels = ["telegram", "discord", "slack", "wechat", "feishu", "dingtalk"];
        for c in channels {
            if message.to_lowercase().contains(c) {
                return Some(c.to_string());
            }
        }
        if message.contains("电报") || message.contains("TG") { return Some("telegram".into()); }
        if message.contains("飞书") { return Some("feishu".into()); }
        if message.contains("钉钉") { return Some("dingtalk".into()); }
        if message.contains("微信") { return Some("wechat".into()); }
        None
    }

    fn extract_model_args(_message: &str) -> Value {
        // Simple extraction — will be enhanced with LLM
        json!({})
    }

    pub(crate) fn help_text() -> String {
        r#"NeoTrix 自然语言助手 — 支持的操作:

🔹 代理管理
  "添加代理 http://1.2.3.4:8080" — 添加代理节点
  "查看代理状态" — 查看代理池状态
  "切换策略 random" — 切换负载均衡策略

🔹 模型管理
  "查看模型列表" — 查看所有模型
  "添加模型" — 添加模型配置

🔹 IM频道
  "查看IM状态" — 查看频道状态
  "启用telegram频道" — 启用指定频道
  "禁用discord频道" — 禁用指定频道

🔹 Provider
  "查看provider列表" — 查看所有供应商

🔹 系统
  "查看状态" — 查看系统健康状态
  "帮助" — 显示此帮助信息

💡 提示: 直接用自然语言描述你想要做的事情即可。"#.to_string()
    }
}
