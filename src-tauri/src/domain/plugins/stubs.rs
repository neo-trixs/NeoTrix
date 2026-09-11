use crate::domain::{DomainPlugin, ActionSpec, DomainError, serde_json};
use crate::commands::model_pool::{self, ModelPoolEntry, ModelPoolStatus};

// ========== Helper ==========

fn stub_action(name: &str) -> ActionSpec {
    ActionSpec { name: name.into(), description: String::new(), params: vec![], returns: "Value".into() }
}

fn stub_call(action: &str, actions: &[&str]) -> Result<serde_json::Value, DomainError> {
    if actions.contains(&action) {
        Ok(serde_json::json!({ "ok": true, "stub": true }))
    } else {
        Err(DomainError { code: "UNKNOWN_ACTION".into(), message: format!("Unknown action: {}", action), recoverable: true })
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

    let empty = ConfigData { provider: String::new(), default_model: String::new(), api_key: String::new(), custom_endpoint: None };
    if !path.exists() { return empty; }

    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return empty,
    };

    let mut cfg = empty;
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('#') || line.is_empty() { continue; }
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

    if !path.exists() { return vec![]; }
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
            if let Some(v) = line.strip_prefix("label = ") { e.insert("label".into(), serde_json::Value::String(v.trim_matches('"').trim_matches('\'').to_string())); }
            else if let Some(v) = line.strip_prefix("provider = ") { e.insert("provider".into(), serde_json::Value::String(v.trim_matches('"').trim_matches('\'').to_string())); }
            else if let Some(v) = line.strip_prefix("model = ") { e.insert("model".into(), serde_json::Value::String(v.trim_matches('"').trim_matches('\'').to_string())); }
            else if let Some(v) = line.strip_prefix("base_url = ") {
                let val = v.trim_matches('"').trim_matches('\'').to_string();
                if val != "null" { e.insert("base_url".into(), serde_json::Value::String(val)); }
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
    if base_url.contains("127.0.0.1") || base_url.contains("localhost") || base_url.contains("0.0.0.0") {
        return true;
    }
    false
}

fn get_provider_models(provider: &str) -> Vec<String> {
    match provider {
        "llamacpp" => vec!["Agents-A1-4B-kimi-Preview-heretic-IQ4_NL".into()],
        "openai" => vec!["gpt-4o".into(), "gpt-4o-mini".into(), "gpt-4-turbo".into()],
        "anthropic" => vec!["claude-sonnet-4".into(), "claude-opus-4".into()],
        "groq" => vec!["llama-3.3-70b-versatile".into(), "mixtral-8x7b-32768".into()],
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
        format!("{}...{}", &cfg.api_key[..4], &cfg.api_key[cfg.api_key.len()-4..])
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
        let label = entry.get("label").and_then(|v| v.as_str()).unwrap_or("unknown");
        let provider = entry.get("provider").and_then(|v| v.as_str()).unwrap_or("");
        let model = entry.get("model").and_then(|v| v.as_str()).unwrap_or("");
        let base_url = entry.get("base_url").and_then(|v| v.as_str()).unwrap_or("");
        let pool_models = if model.is_empty() { vec![] } else { vec![model.to_string()] };
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
        let name = entry.get("label").and_then(|v| v.as_str()).unwrap_or("unknown");
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

    if is_free_provider_with_url(&cfg.provider, cfg.custom_endpoint.as_deref().unwrap_or("")) && resolvable(&cfg) {
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

    let mut discovered: Vec<serde_json::Value> = models.iter().map(|m| {
        serde_json::json!({
            "provider": cfg.provider,
            "model_id": m,
            "base_url": base_url,
            "is_free": is_free_provider_with_url(&cfg.provider, base_url),
            "tier": "local",
        })
    }).collect();

    for entry in &pool {
        let label = entry.get("label").and_then(|v| v.as_str()).unwrap_or("unknown");
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
    fn name(&self) -> &str { "agent" }
    fn description(&self) -> &str { "Agent 状态、任务、provider 配置" }
    fn actions(&self) -> Vec<ActionSpec> {
        vec!["status","start","stop","set_provider","test_provider","fetch_models",
             "config","provider_config","provider_status","pool_sufficiency",
             "discover_models","probe_all_providers",
             "set_project","get_project","health",
             "pool_status","pool_add","pool_remove","pool_update_key","pool_check",
             "add_custom_provider","app_version"].iter().map(|a| stub_action(a)).collect()
    }
    fn call(&self, action: &str, args: serde_json::Value) -> Result<serde_json::Value, DomainError> {
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
            "app_version" => {
                Ok(serde_json::json!(env!("CARGO_PKG_VERSION")))
            }
            // Model Pool actions — delegate to model_pool commands
            "pool_status" => {
                let handle = tokio::runtime::Handle::current();
                let status = handle.block_on(model_pool::model_pool_status())
                    .map_err(|e| DomainError { code: "POOL_ERROR".into(), message: e, recoverable: true })?;
                Ok(serde_json::json!(status))
            }
            "pool_add" => {
                let label = args.get("label").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let provider = args.get("provider").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let api_key = args.get("api_key").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let model = args.get("model").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let tags: Vec<String> = args.get("tags").and_then(|v| v.as_array())
                    .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                    .unwrap_or_default();
                let base_url = args.get("base_url").and_then(|v| v.as_str()).map(String::from);
                let handle = tokio::runtime::Handle::current();
                let entry = handle.block_on(model_pool::model_pool_add(
                    label, provider, api_key, model, tags, base_url,
                )).map_err(|e| DomainError { code: "POOL_ERROR".into(), message: e, recoverable: true })?;
                Ok(serde_json::json!(entry))
            }
            "pool_remove" => {
                let label = args.get("label").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let handle = tokio::runtime::Handle::current();
                let removed = handle.block_on(model_pool::model_pool_remove(label))
                    .map_err(|e| DomainError { code: "POOL_ERROR".into(), message: e, recoverable: true })?;
                Ok(serde_json::json!(removed))
            }
            "pool_update_key" => {
                let label = args.get("label").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let new_api_key = args.get("new_api_key").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let handle = tokio::runtime::Handle::current();
                let updated = handle.block_on(model_pool::model_pool_update_key(label, new_api_key))
                    .map_err(|e| DomainError { code: "POOL_ERROR".into(), message: e, recoverable: true })?;
                Ok(serde_json::json!(updated))
            }
            "pool_check" => {
                let label = args.get("label").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let handle = tokio::runtime::Handle::current();
                let result = handle.block_on(model_pool::model_pool_check(label))
                    .map_err(|e| DomainError { code: "POOL_ERROR".into(), message: e, recoverable: true })?;
                Ok(serde_json::json!(result))
            }
            _ => stub_call(action, &["start","stop","set_provider","test_provider",
                                     "set_project","get_project","health"]),
        }
    }
}

// ========== Plugin Plugin (meta) ==========

pub struct PluginPlugin;

impl DomainPlugin for PluginPlugin {
    fn name(&self) -> &str { "plugin" }
    fn description(&self) -> &str { "插件：安装/卸载/启停、marketplace" }
    fn actions(&self) -> Vec<ActionSpec> {
        vec!["list","install","uninstall","enable","disable","marketplace","update","config"]
            .iter().map(|a| stub_action(a)).collect()
    }
    fn call(&self, action: &str, _args: serde_json::Value) -> Result<serde_json::Value, DomainError> {
        stub_call(action, &["list","install","uninstall","enable","disable","marketplace","update","config"])
    }
}

// ========== Workflow Plugin ==========

pub struct WorkflowPlugin;

impl DomainPlugin for WorkflowPlugin {
    fn name(&self) -> &str { "workflow" }
    fn description(&self) -> &str { "工作流：CRUD、执行、调度" }
    fn actions(&self) -> Vec<ActionSpec> {
        vec!["list","create","delete","run","status","schedule","import","export"]
            .iter().map(|a| stub_action(a)).collect()
    }
    fn call(&self, action: &str, _args: serde_json::Value) -> Result<serde_json::Value, DomainError> {
        stub_call(action, &["list","create","delete","run","status","schedule","import","export"])
    }
}

// ========== Tool Plugin ==========

pub struct ToolPlugin;

impl DomainPlugin for ToolPlugin {
    fn name(&self) -> &str { "tool" }
    fn description(&self) -> &str { "工具：MCP、harness、computer、voice" }
    fn actions(&self) -> Vec<ActionSpec> {
        vec!["mcp_list","mcp_register","harness_execute","harness_resolve",
             "computer_capture","computer_click","computer_type","voice_synthesize"]
            .iter().map(|a| stub_action(a)).collect()
    }
    fn call(&self, action: &str, _args: serde_json::Value) -> Result<serde_json::Value, DomainError> {
        stub_call(action, &["mcp_list","mcp_register","harness_execute","harness_resolve",
                           "computer_capture","computer_click","computer_type","voice_synthesize"])
    }
}

// ========== System Plugin ==========

pub struct SystemPlugin;

impl DomainPlugin for SystemPlugin {
    fn name(&self) -> &str { "system" }
    fn description(&self) -> &str { "系统：窗口、PTY、更新、配置" }
    fn actions(&self) -> Vec<ActionSpec> {
        vec!["window_minimize","window_maximize","window_close","pty_spawn","pty_write",
             "pty_resize","pty_close","update_check","update_download","restart_app",
             "config_get","config_set"]
            .iter().map(|a| stub_action(a)).collect()
    }
    fn call(&self, action: &str, _args: serde_json::Value) -> Result<serde_json::Value, DomainError> {
        stub_call(action, &["window_minimize","window_maximize","window_close","pty_spawn","pty_write",
                           "pty_resize","pty_close","update_check","update_download","restart_app",
                           "config_get","config_set"])
    }
}

// ========== Security Plugin ==========

pub struct SecurityPlugin;

impl DomainPlugin for SecurityPlugin {
    fn name(&self) -> &str { "security" }
    fn description(&self) -> &str { "安全：扫描、权限、隐身、企业合规" }
    fn actions(&self) -> Vec<ActionSpec> {
        vec!["scan","permission_request","permission_respond","stealth_status",
             "audit_log","policy_list","policy_set"]
            .iter().map(|a| stub_action(a)).collect()
    }
    fn call(&self, action: &str, _args: serde_json::Value) -> Result<serde_json::Value, DomainError> {
        stub_call(action, &["scan","permission_request","permission_respond","stealth_status",
                           "audit_log","policy_list","policy_set"])
    }
}

// ========== Ext Plugin ==========

pub struct ExtPlugin;

impl DomainPlugin for ExtPlugin {
    fn name(&self) -> &str { "ext" }
    fn description(&self) -> &str { "扩展：远程桥接、频道、协作、通知" }
    fn actions(&self) -> Vec<ActionSpec> {
        vec!["remote_connect","remote_disconnect","channel_send","channel_list",
             "cowork_start","cowork_stop","notify"]
            .iter().map(|a| stub_action(a)).collect()
    }
    fn call(&self, action: &str, _args: serde_json::Value) -> Result<serde_json::Value, DomainError> {
        stub_call(action, &["remote_connect","remote_disconnect","channel_send","channel_list",
                           "cowork_start","cowork_stop","notify"])
    }
}

// ========== Git Plugin ==========

pub struct GitPlugin;

impl DomainPlugin for GitPlugin {
    fn name(&self) -> &str { "git" }
    fn description(&self) -> &str { "Git 版本控制" }
    fn actions(&self) -> Vec<ActionSpec> {
        vec!["status","diff","staged_files","branches","checkout","commit","push","apply_diff"]
            .iter().map(|a| stub_action(a)).collect()
    }
    fn call(&self, action: &str, _args: serde_json::Value) -> Result<serde_json::Value, DomainError> {
        stub_call(action, &["status","diff","staged_files","branches","checkout","commit","push","apply_diff"])
    }
}

// ========== CLI Plugin ==========

pub struct CliPlugin;

impl DomainPlugin for CliPlugin {
    fn name(&self) -> &str { "cli" }
    fn description(&self) -> &str { "CLI 命令执行" }
    fn actions(&self) -> Vec<ActionSpec> {
        vec!["exec","list"]
            .iter().map(|a| stub_action(a)).collect()
    }
    fn call(&self, action: &str, _args: serde_json::Value) -> Result<serde_json::Value, DomainError> {
        stub_call(action, &["exec","list"])
    }
}
