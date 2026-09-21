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

        // Provider commands
        if msg.contains("provider") || msg.contains("供应商") || msg.contains("提供商") {
            if msg.contains("列表") || msg.contains("list") || msg.contains("查看") {
                return Some(Intent {
                    domain: "provider".into(),
                    action: "list".into(),
                    args: json!({}),
                    response_hint: Some("Provider列表:".into()),
                });
            }
        }

        // System commands
        if msg.contains("状态") || msg.contains("status") || msg.contains("健康") || msg.contains("health") {
            return Some(Intent {
                domain: "system".into(),
                action: "state".into(),
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

    /// LLM classification for complex requests.
    async fn llm_classify(message: &str, registry: &DomainRegistry) -> Result<Intent, DomainError> {
        // Get available domains and actions
        let domains: Vec<String> = registry.list()
            .iter()
            .map(|info| format!("{}: {}", info.name, info.actions.iter().map(|a| &a.name).collect::<Vec<_>>().join(", ")))
            .collect();
        
        let _prompt = format!(
            "You are an intent classifier. Given a user message and available domains, \
             return a JSON object with: domain, action, args.\n\n\
             Available domains:\n{}\n\n\
             User message: {}\n\n\
             Return ONLY a JSON object like: {{\"domain\": \"...\", \"action\": \"...\", \"args\": {{}}}}",
            domains.join("\n"),
            message
        );

        // For now, return a default intent — LLM integration will be added later
        Err(DomainError {
            code: "INTENT_NOT_FOUND".into(),
            message: format!("无法理解指令: {message}. 请尝试更具体的描述，或输入'帮助'查看支持的操作。"),
            recoverable: true,
        })
    }

    fn extract_url(message: &str) -> Option<String> {
        // Simple URL extraction
        let words: Vec<&str> = message.split_whitespace().collect();
        for word in words {
            if word.starts_with("http://") || word.starts_with("https://") {
                return Some(word.to_string());
            }
        }
        None
    }

    fn extract_strategy(message: &str) -> Option<String> {
        let strategies = ["random", "round_robin", "least_latency", "least_error_rate", "weighted"];
        for s in strategies {
            if message.contains(s) {
                return Some(s.to_string());
            }
        }
        // Chinese strategy names
        if message.contains("随机") { return Some("random".into()); }
        if message.contains("轮询") || message.contains("round") { return Some("round_robin".into()); }
        if message.contains("延迟") || message.contains("latency") { return Some("least_latency".into()); }
        if message.contains("错误率") || message.contains("error") { return Some("least_error_rate".into()); }
        if message.contains("权重") || message.contains("weighted") { return Some("weighted".into()); }
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

    fn help_text() -> String {
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
