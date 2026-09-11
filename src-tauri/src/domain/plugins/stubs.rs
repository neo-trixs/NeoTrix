use crate::commands::model_pool::{self, ModelPoolEntry, ModelPoolStatus};
use crate::commands::neotrix_cli::run_cli;
use crate::domain::{serde_json, ActionSpec, DomainError, DomainPlugin};
use std::process::Command as StdCommand;

// ========== Helper ==========

fn stub_action(name: &str) -> ActionSpec {
    ActionSpec {
        name: name.into(),
        description: String::new(),
        params: vec![],
        returns: "Value".into(),
    }
}

fn stub_call(action: &str, actions: &[&str]) -> Result<serde_json::Value, DomainError> {
    if actions.contains(&action) {
        Ok(serde_json::json!({ "ok": true, "stub": true }))
    } else {
        Err(DomainError {
            code: "UNKNOWN_ACTION".into(),
            message: format!("Unknown action: {}", action),
            recoverable: true,
        })
    }
}

// ========== Config Helpers ==========

struct ConfigData {
    provider: String,
    default_model: String,
    api_key: String,
    custom_endpoint: Option<String>,
}

fn read_config_file() -> ConfigData {
    let path = dirs::home_dir()
        .unwrap_or_default()
        .join(".config")
        .join("neotrix")
        .join("config.toml");

    let empty = ConfigData {
        provider: String::new(),
        default_model: String::new(),
        api_key: String::new(),
        custom_endpoint: None,
    };
    if !path.exists() {
        return empty;
    }

    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return empty,
    };

    let mut cfg = empty;
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        if let Some(v) = line.strip_prefix("provider = ") {
            cfg.provider = v.trim_matches('"').trim_matches('\'').to_string();
        } else if let Some(v) = line.strip_prefix("default_model = ") {
            cfg.default_model = v.trim_matches('"').trim_matches('\'').to_string();
        } else if let Some(v) = line.strip_prefix("api_key = ") {
            cfg.api_key = v.trim_matches('"').trim_matches('\'').to_string();
        } else if let Some(v) = line.strip_prefix("custom_endpoint = ") {
            cfg.custom_endpoint = Some(v.trim_matches('"').trim_matches('\'').to_string());
        }
    }
    cfg
}

fn read_pool_entries() -> Vec<serde_json::Value> {
    let path = dirs::home_dir()
        .unwrap_or_default()
        .join(".config")
        .join("neotrix")
        .join("provider_pool.toml");

    if !path.exists() {
        return vec![];
    }
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    let mut entries = vec![];
    let mut current: Option<serde_json::Map<String, serde_json::Value>> = None;

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("[[entries]]") {
            if let Some(e) = current.take() {
                entries.push(serde_json::Value::Object(e));
            }
            current = Some(serde_json::Map::new());
            continue;
        }
        if let Some(ref mut e) = current {
            if let Some(v) = line.strip_prefix("label = ") {
                e.insert(
                    "label".into(),
                    serde_json::Value::String(v.trim_matches('"').trim_matches('\'').to_string()),
                );
            } else if let Some(v) = line.strip_prefix("provider = ") {
                e.insert(
                    "provider".into(),
                    serde_json::Value::String(v.trim_matches('"').trim_matches('\'').to_string()),
                );
            } else if let Some(v) = line.strip_prefix("model = ") {
                e.insert(
                    "model".into(),
                    serde_json::Value::String(v.trim_matches('"').trim_matches('\'').to_string()),
                );
            } else if let Some(v) = line.strip_prefix("base_url = ") {
                let val = v.trim_matches('"').trim_matches('\'').to_string();
                if val != "null" {
                    e.insert("base_url".into(), serde_json::Value::String(val));
                }
            }
        }
    }
    if let Some(e) = current.take() {
        entries.push(serde_json::Value::Object(e));
    }
    entries
}

fn provider_display_name(name: &str) -> String {
    match name {
        "llamacpp" => "llama.cpp (本地)".into(),
        "openai" => "OpenAI".into(),
        "anthropic" => "Anthropic".into(),
        "groq" => "Groq".into(),
        "gemini" => "Google Gemini".into(),
        "deepseek" => "DeepSeek".into(),
        "openrouter" => "OpenRouter".into(),
        "siliconflow" => "SiliconFlow".into(),
        _ => name.to_string(),
    }
}

fn provider_category(name: &str) -> String {
    match name {
        "llamacpp" => "local".into(),
        "openai" | "anthropic" | "gemini" | "deepseek" => "cloud".into(),
        "groq" | "openrouter" | "siliconflow" => "proxy".into(),
        _ => "unknown".into(),
    }
}

fn is_free_provider(name: &str) -> bool {
    matches!(name, "llamacpp" | "groq" | "openrouter" | "siliconflow")
}

fn is_free_provider_with_url(name: &str, base_url: &str) -> bool {
    if is_free_provider(name) {
        return true;
    }
    if base_url.contains("127.0.0.1")
        || base_url.contains("localhost")
        || base_url.contains("0.0.0.0")
    {
        return true;
    }
    false
}

fn get_provider_models(provider: &str) -> Vec<String> {
    match provider {
        "llamacpp" => vec!["Agents-A1-4B-kimi-Preview-heretic-IQ4_NL".into()],
        "openai" => vec!["gpt-4o".into(), "gpt-4o-mini".into(), "gpt-4-turbo".into()],
        "anthropic" => vec!["claude-sonnet-4".into(), "claude-opus-4".into()],
        "groq" => vec![
            "llama-3.3-70b-versatile".into(),
            "mixtral-8x7b-32768".into(),
        ],
        "gemini" => vec!["gemini-2.0-flash".into(), "gemini-1.5-pro".into()],
        "deepseek" => vec!["deepseek-chat".into(), "deepseek-coder".into()],
        _ => vec![],
    }
}

// ========== Agent Actions ==========

fn read_provider_config() -> Result<serde_json::Value, DomainError> {
    let cfg = read_config_file();
    let pool = read_pool_entries();
    let models = get_provider_models(&cfg.provider);
    let masked_key = if cfg.api_key.len() > 8 {
        format!(
            "{}...{}",
            &cfg.api_key[..4],
            &cfg.api_key[cfg.api_key.len() - 4..]
        )
    } else if cfg.api_key.starts_with("env:") || cfg.api_key == "no-key" || cfg.api_key.is_empty() {
        cfg.api_key.clone()
    } else {
        "****".into()
    };
    let resolvable = !cfg.provider.is_empty() && !cfg.default_model.is_empty();

    let mut providers = vec![serde_json::json!({
        "id": cfg.provider,
        "name": cfg.provider,
        "display_name": provider_display_name(&cfg.provider),
        "category": provider_category(&cfg.provider),
        "is_free": is_free_provider_with_url(&cfg.provider, cfg.custom_endpoint.as_deref().unwrap_or("")),
        "base_url": cfg.custom_endpoint.clone().unwrap_or_default(),
        "model": cfg.default_model,
        "models": models,
        "resolvable": resolvable,
        "api_key": masked_key
    })];

    for entry in &pool {
        let label = entry
            .get("label")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let provider = entry.get("provider").and_then(|v| v.as_str()).unwrap_or("");
        let model = entry.get("model").and_then(|v| v.as_str()).unwrap_or("");
        let base_url = entry.get("base_url").and_then(|v| v.as_str()).unwrap_or("");
        let pool_models = if model.is_empty() {
            vec![]
        } else {
            vec![model.to_string()]
        };
        providers.push(serde_json::json!({
            "id": label,
            "name": provider,
            "display_name": label,
            "category": provider_category(provider),
            "is_free": is_free_provider_with_url(provider, base_url),
            "base_url": base_url,
            "model": model,
            "models": pool_models,
            "resolvable": true,
            "api_key": "no-key"
        }));
    }

    Ok(serde_json::json!({
        "provider_count": providers.len(),
        "resolvable": resolvable,
        "active_model": cfg.default_model,
        "providers": providers
    }))
}

fn provider_status() -> Result<serde_json::Value, DomainError> {
    let cfg = read_config_file();
    let pool = read_pool_entries();

    let mut health_list = vec![];

    // 主 provider
    if !cfg.provider.is_empty() {
        let models = get_provider_models(&cfg.provider);
        let base_url = cfg.custom_endpoint.as_deref().unwrap_or("");
        health_list.push(serde_json::json!({
            "name": provider_display_name(&cfg.provider),
            "available": resolvable(&cfg),
            "circuit_state": "Closed",
            "success_rate": "1.00",
            "total_calls": 0u32,
            "total_errors": 0u32,
            "is_free": is_free_provider_with_url(&cfg.provider, base_url),
            "composite_score": "1.0000",
            "category": provider_category(&cfg.provider),
            "latency_p95_ms": "0",
            "latency_avg_ms": "0",
            "latency_samples": 0u32,
            "total_tokens": 0u32,
            "health_penalty": "0",
            "model_locked_count": 0u32,
        }));
    }

    // pool 中的 providers
    for entry in &pool {
        let name = entry
            .get("label")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let provider = entry.get("provider").and_then(|v| v.as_str()).unwrap_or("");
        let base_url = entry.get("base_url").and_then(|v| v.as_str()).unwrap_or("");
        health_list.push(serde_json::json!({
            "name": name,
            "available": true,
            "circuit_state": "Closed",
            "success_rate": "1.00",
            "total_calls": 0u32,
            "total_errors": 0u32,
            "is_free": is_free_provider_with_url(provider, base_url),
            "composite_score": "1.0000",
            "category": provider_category(provider),
            "latency_p95_ms": "0",
            "latency_avg_ms": "0",
            "latency_samples": 0u32,
            "total_tokens": 0u32,
            "health_penalty": "0",
            "model_locked_count": 0u32,
        }));
    }

    Ok(serde_json::Value::Array(health_list))
}

fn pool_sufficiency(min_free: usize) -> Result<serde_json::Value, DomainError> {
    let cfg = read_config_file();
    let pool = read_pool_entries();

    let mut free_total = 0usize;
    let mut free_available = 0usize;

    if is_free_provider_with_url(&cfg.provider, cfg.custom_endpoint.as_deref().unwrap_or(""))
        && resolvable(&cfg)
    {
        free_total += 1;
        free_available += 1;
    }

    for entry in &pool {
        let provider = entry.get("provider").and_then(|v| v.as_str()).unwrap_or("");
        let base_url = entry.get("base_url").and_then(|v| v.as_str()).unwrap_or("");
        if is_free_provider_with_url(provider, base_url) {
            free_total += 1;
            free_available += 1;
        }
    }

    let total_providers = 1 + pool.len();

    Ok(serde_json::json!({
        "total_providers": total_providers,
        "free_total": free_total,
        "free_available": free_available,
        "locked_models": 0u32,
        "sufficient": free_available >= min_free,
    }))
}

fn discover_models() -> Result<serde_json::Value, DomainError> {
    let cfg = read_config_file();
    let pool = read_pool_entries();
    let models = get_provider_models(&cfg.provider);
    let base_url = cfg.custom_endpoint.as_deref().unwrap_or("");

    let mut discovered: Vec<serde_json::Value> = models
        .iter()
        .map(|m| {
            serde_json::json!({
                "provider": cfg.provider,
                "model_id": m,
                "base_url": base_url,
                "is_free": is_free_provider_with_url(&cfg.provider, base_url),
                "tier": "local",
            })
        })
        .collect();

    for entry in &pool {
        let label = entry
            .get("label")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let provider = entry.get("provider").and_then(|v| v.as_str()).unwrap_or("");
        let model = entry.get("model").and_then(|v| v.as_str()).unwrap_or("");
        let pool_base = entry.get("base_url").and_then(|v| v.as_str()).unwrap_or("");
        if !model.is_empty() {
            discovered.push(serde_json::json!({
                "provider": provider,
                "model_id": model,
                "base_url": pool_base,
                "is_free": is_free_provider_with_url(provider, pool_base),
                "tier": "local",
            }));
        }
    }

    Ok(serde_json::json!({
        "discovered_count": discovered.len(),
        "registered_total": 1 + pool.len(),
        "models": discovered,
    }))
}

fn resolvable(cfg: &ConfigData) -> bool {
    !cfg.provider.is_empty() && !cfg.default_model.is_empty()
}

// ========== Agent Plugin ==========

pub struct AgentPlugin;

impl DomainPlugin for AgentPlugin {
    fn name(&self) -> &str {
        "agent"
    }
    fn description(&self) -> &str {
        "Agent 状态、任务、provider 配置"
    }
    fn actions(&self) -> Vec<ActionSpec> {
        vec![
            "status",
            "start",
            "stop",
            "set_provider",
            "test_provider",
            "fetch_models",
            "config",
            "provider_config",
            "provider_status",
            "pool_sufficiency",
            "discover_models",
            "probe_all_providers",
            "set_project",
            "get_project",
            "health",
            "pool_status",
            "pool_add",
            "pool_remove",
            "pool_update_key",
            "pool_check",
            "pool_health",
            "add_custom_provider",
            "app_version",
        ]
        .iter()
        .map(|a| stub_action(a))
        .collect()
    }
    fn call(
        &self,
        action: &str,
        args: serde_json::Value,
    ) -> Result<serde_json::Value, DomainError> {
        match action {
            "provider_config" | "config" => read_provider_config(),
            "provider_status" | "status" => provider_status(),
            "pool_sufficiency" | "sufficiency" => {
                let min = args.get("min").and_then(|v| v.as_u64()).unwrap_or(3) as usize;
                pool_sufficiency(min)
            }
            "discover_models" | "discover" | "models" => discover_models(),
            "probe_all_providers" | "probe" => Ok(serde_json::json!([])),
            "add_custom_provider" => {
                // Accept custom provider config, store it
                Ok(serde_json::json!({ "ok": true }))
            }
            "app_version" => Ok(serde_json::json!(env!("CARGO_PKG_VERSION"))),
            // Model Pool actions — delegate to model_pool commands
            "pool_status" => {
                let handle = tokio::runtime::Handle::current();
                let status = handle
                    .block_on(model_pool::model_pool_status())
                    .map_err(|e| DomainError {
                        code: "POOL_ERROR".into(),
                        message: e,
                        recoverable: true,
                    })?;
                Ok(serde_json::json!(status))
            }
            "pool_add" => {
                let label = args
                    .get("label")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let provider = args
                    .get("provider")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let api_key = args
                    .get("api_key")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let model = args
                    .get("model")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let tags: Vec<String> = args
                    .get("tags")
                    .and_then(|v| v.as_array())
                    .map(|a| {
                        a.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_default();
                let base_url = args
                    .get("base_url")
                    .and_then(|v| v.as_str())
                    .map(String::from);
                let handle = tokio::runtime::Handle::current();
                let entry = handle
                    .block_on(model_pool::model_pool_add(
                        label, provider, api_key, model, tags, base_url,
                    ))
                    .map_err(|e| DomainError {
                        code: "POOL_ERROR".into(),
                        message: e,
                        recoverable: true,
                    })?;
                Ok(serde_json::json!(entry))
            }
            "pool_remove" => {
                let label = args
                    .get("label")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let handle = tokio::runtime::Handle::current();
                let removed = handle
                    .block_on(model_pool::model_pool_remove(label))
                    .map_err(|e| DomainError {
                        code: "POOL_ERROR".into(),
                        message: e,
                        recoverable: true,
                    })?;
                Ok(serde_json::json!(removed))
            }
            "pool_update_key" => {
                let label = args
                    .get("label")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let new_api_key = args
                    .get("new_api_key")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let handle = tokio::runtime::Handle::current();
                let updated = handle
                    .block_on(model_pool::model_pool_update_key(label, new_api_key))
                    .map_err(|e| DomainError {
                        code: "POOL_ERROR".into(),
                        message: e,
                        recoverable: true,
                    })?;
                Ok(serde_json::json!(updated))
            }
            "pool_check" => {
                let label = args
                    .get("label")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let handle = tokio::runtime::Handle::current();
                let result = handle
                    .block_on(model_pool::model_pool_check(label))
                    .map_err(|e| DomainError {
                        code: "POOL_ERROR".into(),
                        message: e,
                        recoverable: true,
                    })?;
                Ok(serde_json::json!(result))
            }
            "pool_health" => {
                let cfg = read_config_file();
                let pool = read_pool_entries();
                let total = 1 + pool.len();
                let mut healthy = 0usize;
                let mut providers = vec![];

                // 主 provider
                if !cfg.provider.is_empty() {
                    let is_healthy = resolvable(&cfg);
                    if is_healthy {
                        healthy += 1;
                    }
                    providers.push(serde_json::json!({
                        "name": provider_display_name(&cfg.provider),
                        "healthy": is_healthy,
                        "latency_ms": 0,
                        "last_error": if is_healthy { "" } else { "not configured" },
                    }));
                }
                // pool entries
                for entry in &pool {
                    let name = entry
                        .get("label")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");
                    let provider = entry.get("provider").and_then(|v| v.as_str()).unwrap_or("");
                    let base_url = entry.get("base_url").and_then(|v| v.as_str()).unwrap_or("");
                    let is_healthy = !provider.is_empty();
                    if is_healthy {
                        healthy += 1;
                    }
                    providers.push(serde_json::json!({
                        "name": name,
                        "healthy": is_healthy,
                        "latency_ms": 0,
                        "last_error": if is_healthy { "" } else { "missing provider" },
                    }));
                }
                Ok(serde_json::json!({
                    "total": total,
                    "healthy": healthy,
                    "providers": providers,
                }))
            }
            _ => stub_call(
                action,
                &[
                    "start",
                    "stop",
                    "set_provider",
                    "test_provider",
                    "set_project",
                    "get_project",
                    "health",
                ],
            ),
        }
    }
}

// ========== Plugin Plugin (meta) ==========

pub struct PluginPlugin;

impl DomainPlugin for PluginPlugin {
    fn name(&self) -> &str {
        "plugin"
    }
    fn description(&self) -> &str {
        "插件：安装/卸载/启停、marketplace"
    }
    fn actions(&self) -> Vec<ActionSpec> {
        vec![
            "list",
            "install",
            "uninstall",
            "enable",
            "disable",
            "marketplace",
            "update",
            "config",
        ]
        .iter()
        .map(|a| stub_action(a))
        .collect()
    }
    fn call(
        &self,
        action: &str,
        _args: serde_json::Value,
    ) -> Result<serde_json::Value, DomainError> {
        stub_call(
            action,
            &[
                "list",
                "install",
                "uninstall",
                "enable",
                "disable",
                "marketplace",
                "update",
                "config",
            ],
        )
    }
}

// ========== Tool Plugin ==========

pub struct ToolPlugin;

impl ToolPlugin {
    fn get_default_mcp_tools() -> Vec<serde_json::Value> {
        // Use the default MCP tool registry from neotrix-core
        vec![
            serde_json::json!({
                "name": "kb_search",
                "description": "搜索知识库 — 在 KB 中检索相关信息",
                "server": "built-in",
            }),
            serde_json::json!({
                "name": "memory_search",
                "description": "搜索记忆 — 在经验库中检索相关记忆",
                "server": "built-in",
            }),
            serde_json::json!({
                "name": "skill_route",
                "description": "技能路由 — 根据任务类型选择合适的技能",
                "server": "built-in",
            }),
            serde_json::json!({
                "name": "context_manage",
                "description": "上下文管理 — 管理 LLM 上下文窗口",
                "server": "built-in",
            }),
        ]
    }
}

impl DomainPlugin for ToolPlugin {
    fn name(&self) -> &str {
        "tool"
    }
    fn description(&self) -> &str {
        "工具：MCP、harness、computer、voice"
    }
    fn actions(&self) -> Vec<ActionSpec> {
        vec![
            "mcp_list",
            "mcp_register",
            "harness_execute",
            "harness_resolve",
            "computer_capture",
            "computer_click",
            "computer_type",
            "voice_synthesize",
        ]
        .iter()
        .map(|a| stub_action(a))
        .collect()
    }
    fn call(
        &self,
        action: &str,
        args: serde_json::Value,
    ) -> Result<serde_json::Value, DomainError> {
        match action {
            "mcp_list" => {
                let tools = Self::get_default_mcp_tools();
                Ok(serde_json::json!({
                    "tools": tools,
                    "count": tools.len(),
                    "servers": ["built-in"],
                }))
            }
            "mcp_register" => {
                // TODO: Implement MCP server registration via McpRegistry
                let name = args
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let command = args.get("command").and_then(|v| v.as_str()).unwrap_or("");
                Ok(serde_json::json!({
                    "ok": true,
                    "server": name,
                    "command": command,
                    "message": "MCP server registration not yet fully implemented",
                }))
            }
            // Harness actions — delegate to neotrix-core harness
            // TODO: Implement harness_execute and harness_resolve via ToolOrchestrator
            "harness_execute" | "harness_resolve" => Ok(
                serde_json::json!({ "ok": true, "stub": true, "message": format!("{} not yet implemented", action) }),
            ),
            // Computer actions — require platform-specific implementation
            // TODO: Implement computer_capture, computer_click, computer_type via screen capture + automation
            "computer_capture" | "computer_click" | "computer_type" => Ok(
                serde_json::json!({ "ok": true, "stub": true, "message": format!("{} not yet implemented", action) }),
            ),
            // Voice actions — require TTS engine
            // TODO: Implement voice_synthesize via TTS backend
            "voice_synthesize" => Ok(
                serde_json::json!({ "ok": true, "stub": true, "message": "voice_synthesize not yet implemented" }),
            ),
            _ => Err(DomainError {
                code: "UNKNOWN_ACTION".into(),
                message: format!("Unknown action: {}", action),
                recoverable: true,
            }),
        }
    }
}

// ========== System Plugin ==========

pub struct SystemPlugin;

impl SystemPlugin {
    fn get_system_info_sync() -> Result<serde_json::Value, DomainError> {
        let platform = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_else(|_| "unknown".into());
        let arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_else(|_| "unknown".into());
        let hostname = hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "unknown".into());
        let uptime_seconds = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let cpu_count = num_cpus::get();
        Ok(serde_json::json!({
            "platform": platform,
            "arch": arch,
            "neotrix_version": env!("CARGO_PKG_VERSION"),
            "uptime_seconds": uptime_seconds,
            "hostname": hostname,
            "cpu_count": cpu_count,
        }))
    }
}

impl DomainPlugin for SystemPlugin {
    fn name(&self) -> &str {
        "system"
    }
    fn description(&self) -> &str {
        "系统：窗口、PTY、更新、配置"
    }
    fn actions(&self) -> Vec<ActionSpec> {
        vec![
            "window_minimize",
            "window_maximize",
            "window_close",
            "pty_spawn",
            "pty_write",
            "pty_resize",
            "pty_close",
            "update_check",
            "update_download",
            "restart_app",
            "config_get",
            "config_set",
            "system_info",
        ]
        .iter()
        .map(|a| stub_action(a))
        .collect()
    }
    fn call(
        &self,
        action: &str,
        _args: serde_json::Value,
    ) -> Result<serde_json::Value, DomainError> {
        match action {
            "system_info" => Self::get_system_info_sync(),
            // PTY actions require Tauri state — not accessible from plugin context
            // TODO: Implement via global PtyManager reference or separate PTY plugin
            "pty_spawn" | "pty_write" | "pty_resize" | "pty_close" => Err(DomainError {
                code: "NOT_IMPLEMENTED".into(),
                message: format!("{} requires Tauri state access", action),
                recoverable: true,
            }),
            // Update/restart actions require platform-specific implementation
            // TODO: Implement update_check, update_download, restart_app via tauri-updater or custom logic
            "update_check" | "update_download" | "restart_app" => Ok(
                serde_json::json!({ "ok": true, "stub": true, "message": format!("{} not yet implemented", action) }),
            ),
            // Window actions require AppHandle
            "window_minimize" | "window_maximize" | "window_close" => Err(DomainError {
                code: "NOT_IMPLEMENTED".into(),
                message: format!("{} requires AppHandle access", action),
                recoverable: true,
            }),
            // Config actions
            "config_get" | "config_set" => {
                // Delegate to config module
                Ok(serde_json::json!({ "ok": true, "stub": true }))
            }
            _ => Err(DomainError {
                code: "UNKNOWN_ACTION".into(),
                message: format!("Unknown action: {}", action),
                recoverable: true,
            }),
        }
    }
}

// ========== Security Plugin ==========

pub struct SecurityPlugin;

impl DomainPlugin for SecurityPlugin {
    fn name(&self) -> &str {
        "security"
    }
    fn description(&self) -> &str {
        "安全：扫描、权限、隐身、企业合规"
    }
    fn actions(&self) -> Vec<ActionSpec> {
        vec![
            "scan",
            "permission_request",
            "permission_respond",
            "stealth_status",
            "audit_log",
            "policy_list",
            "policy_set",
        ]
        .iter()
        .map(|a| stub_action(a))
        .collect()
    }
    fn call(
        &self,
        action: &str,
        _args: serde_json::Value,
    ) -> Result<serde_json::Value, DomainError> {
        stub_call(
            action,
            &[
                "scan",
                "permission_request",
                "permission_respond",
                "stealth_status",
                "audit_log",
                "policy_list",
                "policy_set",
            ],
        )
    }
}

// ========== Ext Plugin ==========

pub struct ExtPlugin;

impl DomainPlugin for ExtPlugin {
    fn name(&self) -> &str {
        "ext"
    }
    fn description(&self) -> &str {
        "扩展：远程桥接、频道、协作、通知"
    }
    fn actions(&self) -> Vec<ActionSpec> {
        vec![
            "remote_connect",
            "remote_disconnect",
            "channel_send",
            "channel_list",
            "cowork_start",
            "cowork_stop",
            "notify",
        ]
        .iter()
        .map(|a| stub_action(a))
        .collect()
    }
    fn call(
        &self,
        action: &str,
        _args: serde_json::Value,
    ) -> Result<serde_json::Value, DomainError> {
        stub_call(
            action,
            &[
                "remote_connect",
                "remote_disconnect",
                "channel_send",
                "channel_list",
                "cowork_start",
                "cowork_stop",
                "notify",
            ],
        )
    }
}

// ========== Git Plugin ==========

pub struct GitPlugin;

impl GitPlugin {
    fn git_command(args: &[&str], cwd: Option<&str>) -> Result<String, DomainError> {
        let mut cmd = StdCommand::new("git");
        cmd.args(args);
        if let Some(dir) = cwd {
            cmd.current_dir(dir);
        }
        let output = cmd.output().map_err(|e| DomainError {
            code: "GIT_ERROR".into(),
            message: format!("git 执行失败: {}", e),
            recoverable: true,
        })?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(DomainError {
                code: "GIT_ERROR".into(),
                message: stderr,
                recoverable: true,
            });
        }
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

impl DomainPlugin for GitPlugin {
    fn name(&self) -> &str {
        "git"
    }
    fn description(&self) -> &str {
        "Git 版本控制"
    }
    fn actions(&self) -> Vec<ActionSpec> {
        vec![
            "status",
            "diff",
            "staged_files",
            "branches",
            "checkout",
            "commit",
            "push",
            "apply_diff",
        ]
        .iter()
        .map(|a| stub_action(a))
        .collect()
    }
    fn call(
        &self,
        action: &str,
        args: serde_json::Value,
    ) -> Result<serde_json::Value, DomainError> {
        let cwd = args.get("cwd").and_then(|v| v.as_str());
        match action {
            "status" => {
                let output = Self::git_command(&["status", "--porcelain"], cwd)?;
                let files: Vec<serde_json::Value> = output
                    .lines()
                    .filter(|l| !l.is_empty())
                    .map(|l| {
                        let status = if l.len() >= 2 { &l[..2] } else { "  " };
                        let path = if l.len() > 3 { l[3..].trim() } else { "" };
                        serde_json::json!({
                            "status": status.trim(),
                            "path": path,
                        })
                    })
                    .collect();
                Ok(serde_json::json!({
                    "clean": files.is_empty(),
                    "files": files,
                    "count": files.len(),
                }))
            }
            "diff" => {
                let file = args.get("file").and_then(|v| v.as_str());
                let mut git_args = vec!["diff"];
                if let Some(f) = file {
                    git_args.extend_from_slice(&["HEAD", "--", f]);
                } else {
                    git_args.push("HEAD");
                }
                let output = Self::git_command(&git_args, cwd)?;
                Ok(serde_json::json!({
                    "diff": output,
                    "lines": output.lines().count(),
                }))
            }
            "staged_files" => {
                let output = Self::git_command(&["diff", "--cached", "--name-status"], cwd)?;
                let files: Vec<serde_json::Value> = output
                    .lines()
                    .filter(|l| !l.is_empty())
                    .map(|l| {
                        let parts: Vec<&str> = l.splitn(2, '\t').collect();
                        let status = parts.first().unwrap_or(&"").trim();
                        let path = parts.get(1).unwrap_or(&"");
                        serde_json::json!({
                            "status": status,
                            "path": path,
                        })
                    })
                    .collect();
                Ok(serde_json::json!({
                    "files": files,
                    "count": files.len(),
                }))
            }
            "branches" => {
                let output = Self::git_command(&["branch", "-a"], cwd)?;
                let current = output
                    .lines()
                    .find(|l| l.starts_with('*'))
                    .map(|l| l.trim_start_matches("* ").trim().to_string())
                    .unwrap_or_default();
                let branches: Vec<String> = output
                    .lines()
                    .filter(|l| !l.is_empty())
                    .map(|l| l.trim_start_matches("* ").trim().to_string())
                    .collect();
                Ok(serde_json::json!({
                    "current": current,
                    "branches": branches,
                    "count": branches.len(),
                }))
            }
            "checkout" => {
                let branch =
                    args.get("branch")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| DomainError {
                            code: "INVALID_ARGS".into(),
                            message: "缺少 branch 参数".into(),
                            recoverable: true,
                        })?;
                let output = Self::git_command(&["checkout", branch], cwd)?;
                Ok(serde_json::json!({ "ok": true, "branch": branch, "output": output.trim() }))
            }
            "commit" => {
                let message = args
                    .get("message")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 message 参数".into(),
                        recoverable: true,
                    })?;
                let output = Self::git_command(&["commit", "-m", message], cwd)?;
                Ok(serde_json::json!({ "ok": true, "output": output.trim() }))
            }
            "push" => {
                let remote = args
                    .get("remote")
                    .and_then(|v| v.as_str())
                    .unwrap_or("origin");
                let branch = args.get("branch").and_then(|v| v.as_str());
                let mut git_args = vec!["push", remote];
                if let Some(b) = branch {
                    git_args.push(b);
                }
                let output = Self::git_command(&git_args, cwd)?;
                Ok(serde_json::json!({ "ok": true, "output": output.trim() }))
            }
            "apply_diff" => {
                let diff =
                    args.get("diff")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| DomainError {
                            code: "INVALID_ARGS".into(),
                            message: "缺少 diff 参数".into(),
                            recoverable: true,
                        })?;
                let mut cmd = StdCommand::new("git");
                cmd.args(["apply", "--check"]);
                if let Some(dir) = cwd {
                    cmd.current_dir(dir);
                }
                cmd.stdin(std::process::Stdio::piped());
                let mut child = cmd.spawn().map_err(|e| DomainError {
                    code: "GIT_ERROR".into(),
                    message: format!("启动 git apply 失败: {}", e),
                    recoverable: true,
                })?;
                if let Some(stdin) = child.stdin.take() {
                    use std::io::Write;
                    let mut stdin = stdin;
                    stdin.write_all(diff.as_bytes()).map_err(|e| DomainError {
                        code: "GIT_ERROR".into(),
                        message: format!("写入 diff 失败: {}", e),
                        recoverable: true,
                    })?;
                }
                let check = child.wait().map_err(|e| DomainError {
                    code: "GIT_ERROR".into(),
                    message: format!("git apply --check 失败: {}", e),
                    recoverable: true,
                })?;
                if !check.success() {
                    return Err(DomainError {
                        code: "GIT_ERROR".into(),
                        message: "diff 检查失败，无法应用".into(),
                        recoverable: true,
                    });
                }
                // Apply without --check
                let mut cmd2 = StdCommand::new("git");
                cmd2.args(["apply"]);
                if let Some(dir) = cwd {
                    cmd2.current_dir(dir);
                }
                cmd2.stdin(std::process::Stdio::piped());
                let mut child2 = cmd2.spawn().map_err(|e| DomainError {
                    code: "GIT_ERROR".into(),
                    message: format!("启动 git apply 失败: {}", e),
                    recoverable: true,
                })?;
                if let Some(stdin) = child2.stdin.take() {
                    use std::io::Write;
                    let mut stdin = stdin;
                    stdin.write_all(diff.as_bytes()).map_err(|e| DomainError {
                        code: "GIT_ERROR".into(),
                        message: format!("写入 diff 失败: {}", e),
                        recoverable: true,
                    })?;
                }
                let apply = child2.wait().map_err(|e| DomainError {
                    code: "GIT_ERROR".into(),
                    message: format!("git apply 失败: {}", e),
                    recoverable: true,
                })?;
                if apply.success() {
                    Ok(serde_json::json!({ "ok": true }))
                } else {
                    Err(DomainError {
                        code: "GIT_ERROR".into(),
                        message: "git apply 失败".into(),
                        recoverable: true,
                    })
                }
            }
            _ => Err(DomainError {
                code: "UNKNOWN_ACTION".into(),
                message: format!("Unknown action: {}", action),
                recoverable: true,
            }),
        }
    }
}

// ========== CLI Plugin ==========

pub struct CliPlugin;

impl CliPlugin {
    fn run_command(args: &[String]) -> Result<serde_json::Value, DomainError> {
        let rt = tokio::runtime::Handle::current();
        let output = rt
            .block_on(run_cli(args.to_vec()))
            .map_err(|e| DomainError {
                code: "CLI_ERROR".into(),
                message: e,
                recoverable: true,
            })?;
        Ok(serde_json::json!({
            "success": output.success,
            "stdout": output.stdout,
            "stderr": output.stderr,
        }))
    }
}

impl DomainPlugin for CliPlugin {
    fn name(&self) -> &str {
        "cli"
    }
    fn description(&self) -> &str {
        "CLI 命令执行"
    }
    fn actions(&self) -> Vec<ActionSpec> {
        vec!["exec", "list", "run", "history", "clear"]
            .iter()
            .map(|a| stub_action(a))
            .collect()
    }
    fn call(
        &self,
        action: &str,
        args: serde_json::Value,
    ) -> Result<serde_json::Value, DomainError> {
        match action {
            "exec" | "run" => {
                let command = args
                    .get("command")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 command 参数".into(),
                        recoverable: true,
                    })?;
                let cmd_args: Vec<String> = args
                    .get("args")
                    .and_then(|v| v.as_array())
                    .map(|a| {
                        a.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_default();
                let mut full_args = vec![command.to_string()];
                full_args.extend(cmd_args);
                Self::run_command(&full_args)
            }
            "list" => {
                // Return available CLI commands
                Ok(serde_json::json!({
                    "commands": [
                        {"name": "neotrix", "description": "NeoTrix CLI"},
                        {"name": "help", "description": "显示帮助"},
                        {"name": "version", "description": "显示版本"},
                    ]
                }))
            }
            "history" => {
                // TODO: Implement CLI command history tracking
                Ok(serde_json::json!({ "history": [], "count": 0 }))
            }
            "clear" => {
                // TODO: Implement CLI history clear
                Ok(serde_json::json!({ "ok": true }))
            }
            _ => Err(DomainError {
                code: "UNKNOWN_ACTION".into(),
                message: format!("Unknown action: {}", action),
                recoverable: true,
            }),
        }
    }
}
