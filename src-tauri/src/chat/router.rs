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

    /// Clarification reply — answered locally by chat_send, no domain call.
    fn clarify(question: &str) -> Intent {
        Intent {
            domain: "chat".into(),
            action: "clarify".into(),
            args: json!({}),
            response_hint: Some(question.into()),
        }
    }

    /// Fast pattern matching for common commands.
    ///
    /// Arms are ordered most-specific-first within each domain block so that
    /// e.g. subscription handling wins over the generic add arm, and snapshot
    /// wins over the generic status arm. Missing-argument cases fall through
    /// to a clarification reply instead of aborting the whole match.
    fn pattern_match(message: &str) -> Option<Intent> {
        let msg = message.trim().to_lowercase();

        // Proxy pool commands — most-specific arms first.
        if msg.contains("代理") || msg.contains("proxy") {
            // Subscription management (must precede generic add/remove).
            if msg.contains("订阅") || msg.contains("subscription") {
                if msg.contains("删除")
                    || msg.contains("移除")
                    || msg.contains("remove")
                    || msg.contains("取消")
                {
                    match Self::extract_url(message) {
                        Some(url) => {
                            return Some(Intent {
                                domain: "proxy_pool".into(),
                                action: "remove_subscription".into(),
                                args: json!({"url": url}),
                                response_hint: Some(format!("已删除订阅: {url}")),
                            });
                        }
                        None => {
                            return Some(Self::clarify("请提供要删除的订阅 URL。"));
                        }
                    }
                }
                match Self::extract_url(message) {
                    Some(url) => {
                        return Some(Intent {
                            domain: "proxy_pool".into(),
                            action: "add_subscription".into(),
                            args: json!({"url": url}),
                            response_hint: Some(format!("已添加订阅: {url}")),
                        });
                    }
                    None => {
                        return Some(Self::clarify("请提供要添加的订阅 URL。"));
                    }
                }
            }
            // Remove node.
            if msg.contains("删除") || msg.contains("移除") || msg.contains("remove") {
                match Self::extract_url(message) {
                    Some(url) => {
                        return Some(Intent {
                            domain: "proxy_pool".into(),
                            action: "remove".into(),
                            args: json!({"url": url}),
                            response_hint: Some(format!("已删除代理: {url}")),
                        });
                    }
                    None => {
                        return Some(Self::clarify("请提供要删除的代理 URL。"));
                    }
                }
            }
            // Snapshot (must precede the generic 查看/status arm).
            if msg.contains("快照") || msg.contains("snapshot") {
                return Some(Intent {
                    domain: "proxy_pool".into(),
                    action: "snapshot".into(),
                    args: json!({}),
                    response_hint: Some("代理池快照:".into()),
                });
            }
            // Strategy list (must precede generic 查看/status; only when no
            // concrete strategy token is present).
            if (msg.contains("策略") || msg.contains("strategy"))
                && (msg.contains("列表")
                    || msg.contains("list")
                    || msg.contains("可用")
                    || msg.contains("查看"))
                && Self::extract_strategy(message).is_none()
            {
                return Some(Intent {
                    domain: "proxy_pool".into(),
                    action: "list_strategies".into(),
                    args: json!({}),
                    response_hint: Some("可用策略:".into()),
                });
            }
            // Add node.
            if msg.contains("添加") || msg.contains("add") || msg.contains("新增") {
                match Self::extract_url(message) {
                    Some(url) => {
                        return Some(Intent {
                            domain: "proxy_pool".into(),
                            action: "add".into(),
                            args: json!({"url": url}),
                            response_hint: Some(format!("已添加代理: {url}")),
                        });
                    }
                    None => {
                        return Some(Self::clarify(
                            "请提供要添加的代理 URL，例如：添加代理 http://1.2.3.4:8080",
                        ));
                    }
                }
            }
            // Strategy set.
            if msg.contains("策略") || msg.contains("strategy") {
                match Self::extract_strategy(message) {
                    Some(strategy) => {
                        return Some(Intent {
                            domain: "proxy_pool".into(),
                            action: "set_strategy".into(),
                            args: json!({"strategy": strategy}),
                            response_hint: Some(format!("已切换策略: {strategy}")),
                        });
                    }
                    None => {
                        return Some(Self::clarify(
                            "请指定策略名称，可用策略：fastest, least_latency, least_failure, weighted_random, geo_preferred, round_robin, adaptive, auto",
                        ));
                    }
                }
            }
            // Status (generic catch-all, last).
            if msg.contains("状态") || msg.contains("status") || msg.contains("查看") {
                return Some(Intent {
                    domain: "proxy_pool".into(),
                    action: "status".into(),
                    args: json!({}),
                    response_hint: Some("代理池状态:".into()),
                });
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
            if msg.contains("添加") || msg.contains("add") || msg.contains("新增") {
                match Self::extract_model_args(message) {
                    Some((label, provider, model, api_key)) => {
                        return Some(Intent {
                            domain: "model_pool".into(),
                            action: "add".into(),
                            args: json!({
                                "label": label,
                                "provider": provider,
                                "model": model,
                                "api_key": api_key,
                            }),
                            response_hint: Some(format!("已添加模型配置: {label}")),
                        });
                    }
                    None => {
                        return Some(Self::clarify(
                            "添加模型需要提供：标签、供应商、模型名和 API Key。例如：添加模型 我的模型 openai gpt-4 sk-xxx",
                        ));
                    }
                }
            }
        }

        // IM commands — checked before the generic system arm.
        if msg.contains("频道")
            || msg.contains("渠道")
            || msg.contains("channel")
            || msg.contains("机器人")
            || msg.contains("bot")
            || msg.contains("im")
            || msg.contains("dsh")
        {
            if msg.contains("状态")
                || msg.contains("status")
                || msg.contains("查看")
                || msg.contains("列表")
                || msg.contains("list")
            {
                return Some(Intent {
                    domain: "im".into(),
                    action: "status".into(),
                    args: json!({}),
                    response_hint: Some("IM状态:".into()),
                });
            }
            if msg.contains("切换")
                || msg.contains("toggle")
                || msg.contains("启用")
                || msg.contains("禁用")
                || msg.contains("开启")
                || msg.contains("关闭")
                || msg.contains("enable")
                || msg.contains("disable")
            {
                match Self::extract_channel(message) {
                    Some(channel) => {
                        let enabled = !(msg.contains("关闭")
                            || msg.contains("禁用")
                            || msg.contains("停用")
                            || msg.contains("disable")
                            || msg.contains("off")
                            || msg.contains("close"));
                        return Some(Intent {
                            domain: "im".into(),
                            action: "toggle_channel".into(),
                            args: json!({"channel": channel, "enabled": enabled}),
                            response_hint: Some(format!(
                                "已{}频道: {channel}",
                                if enabled { "启用" } else { "禁用" }
                            )),
                        });
                    }
                    None => {
                        return Some(Self::clarify(
                            "请指定频道，例如：启用 telegram 频道",
                        ));
                    }
                }
            }
        }

        // Market/plugin commands → plugin/list (no standalone market domain exists)
        if msg.contains("插件") || msg.contains("市场") || msg.contains("安装") || msg.contains("market") || msg.contains("plugin") {
            if msg.contains("列表") || msg.contains("list") || msg.contains("查看") || msg.contains("有哪些") || msg.contains("状态") || msg.contains("status") {
                return Some(Intent {
                    domain: "plugin".into(),
                    action: "list".into(),
                    args: json!({}),
                    response_hint: Some("插件列表:".into()),
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
    /// Destructive actions the fuzzy matcher must never route to.
    ///
    /// Fuzzy intents always carry empty `args`, so any action that mutates
    /// state without required arguments (or whose required arguments cannot
    /// be extracted) is a data-loss risk — e.g. `memory/clear` wipes all
    /// memories, `cli/exec` spawns arbitrary processes. Explicit Tier-1 arms
    /// (which extract real arguments) are unaffected by this list.
    const FUZZY_DENY: &'static [(&'static str, &'static str)] = &[
        ("memory", "clear"),
        ("autostart", "toggle"),
        ("autostart", "enable"),
        ("autostart", "disable"),
        ("system", "restart_app"),
        ("system", "update_download"),
        ("system", "window_close"),
        ("chat", "clear"),
        ("chat", "delete_message"),
        ("session", "delete"),
        ("session_sync", "delete_session"),
        ("workflow", "cancel"),
        ("workflow", "delete"),
        ("plugin", "uninstall"),
        ("model_pool", "remove"),
        ("proxy_pool", "remove"),
        ("proxy_pool", "remove_subscription"),
        ("ext", "remote_disconnect"),
        ("security", "quarantine"),
        ("git", "push"),
        ("git", "commit"),
        ("git", "checkout"),
        ("file", "write"),
        ("kb", "doc_delete"),
        ("cli", "exec"),
        ("cli", "run"),
        ("tool", "computer_click"),
        ("tool", "computer_type"),
        ("mcp_extension", "uninstall"),
    ];

    /// Minimum fuzzy score to accept a match.
    ///
    /// Domain-only hits score `len * 2` and action hits `len * 3`, so the
    /// threshold keeps meaningful matches (e.g. `status` = 18, `session` =
    /// 14, `list` = 12, `add` = 9) while rejecting short-name noise such as
    /// `im` (4) or `cli` (6) matching "him"/"time"/"click".
    const FUZZY_MIN_SCORE: usize = 8;

    fn fuzzy_denied(domain: &str, action: &str) -> bool {
        Self::FUZZY_DENY
            .iter()
            .any(|(d, a)| *d == domain && *a == action)
    }

    async fn llm_classify(message: &str, registry: &DomainRegistry) -> Result<Intent, DomainError> {
        let msg = message.to_lowercase();
        // (score, domain, action, description)
        let mut best: Option<(usize, String, String, String)> = None;

        for info in registry.list() {
            let domain_name = info.name.to_lowercase();
            let domain_hit = if msg.contains(domain_name.as_str()) {
                domain_name.len() * 2
            } else {
                0
            };
            for action in &info.actions {
                if Self::fuzzy_denied(&info.name, &action.name) {
                    continue;
                }
                let mut score = domain_hit;
                let action_name = action.name.to_lowercase();
                if msg.contains(action_name.as_str()) {
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
                    None => score >= Self::FUZZY_MIN_SCORE,
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
            // No transport error — answer locally as a clarification so the
            // UI renders helpful text instead of an unhandled rejection.
            None => Ok(Self::clarify(&format!(
                "无法理解指令: {message}。请尝试更具体的描述，或输入“帮助”查看支持的操作。"
            ))),
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

    /// Channel detection over all 9 registered channel types
    /// (`ALL_CHANNEL_TYPES` is the single source of truth).
    fn extract_channel(message: &str) -> Option<String> {
        let msg = message.to_lowercase();
        for channel in crate::domain::plugins::im::ALL_CHANNEL_TYPES {
            if msg.contains(&channel.to_string()) || message.contains(channel.display_name()) {
                return Some(channel.to_string());
            }
        }
        // Aliases not covered by canonical/display names.
        if message.contains("电报") || message.contains("TG") {
            return Some("telegram".into());
        }
        None
    }

    /// Best-effort extraction of `(label, provider, model, api_key)`.
    ///
    /// Drops command keywords/stopwords, treats an `sk-`-prefixed or long
    /// alphanumeric token as the API key, and assigns the remaining tokens
    /// positionally. Returns `None` when the message is under-specified so
    /// the caller can ask a clarification question instead of dispatching
    /// a call that is guaranteed to fail with `INVALID_ARGS`.
    fn extract_model_args(message: &str) -> Option<(String, String, String, String)> {
        const STOPWORDS: &[&str] = &[
            "添加", "模型", "提供者", "供应商", "提供商", "新增", "创建", "配置",
            "add", "model", "provider", "label", "api_key", "apikey", "key",
            "密钥", "标签", "名称", "名字", "的", "和", "与", "为", "个",
            "把", "将", "用", "一个", "请", "帮", "我", "给",
        ];
        let mut candidates: Vec<String> = Vec::new();
        let mut key_candidate: Option<String> = None;
        for word in message.split_whitespace() {
            let cleaned = word
                .trim_matches(|c: char| "\"'()[]<>,;!?，。！？；：、）".contains(c));
            if cleaned.is_empty() {
                continue;
            }
            let lower = cleaned.to_lowercase();
            if STOPWORDS.iter().any(|s| lower == *s) {
                continue;
            }
            let alnum = cleaned.chars().filter(|c| c.is_ascii_alphanumeric()).count();
            if lower.starts_with("sk-")
                || lower.starts_with("sk_")
                || (cleaned.len() >= 20 && alnum * 2 >= cleaned.len())
            {
                key_candidate = Some(cleaned.to_string());
                continue;
            }
            candidates.push(cleaned.to_string());
        }
        let api_key = key_candidate?;
        if candidates.len() < 3 {
            return None;
        }
        Some((
            candidates[0].clone(),
            candidates[1].clone(),
            candidates[2].clone(),
            api_key,
        ))
    }

    pub(crate) fn help_text() -> String {
        r#"NeoTrix 自然语言助手 — 支持的操作:

🔹 代理管理
  "添加代理 http://1.2.3.4:8080" — 添加代理节点
  "查看代理状态" — 查看代理池状态
  "切换策略 round_robin" — 切换负载均衡策略

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
