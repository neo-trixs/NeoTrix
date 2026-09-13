use crate::commands::model_pool::{self, ModelPoolEntry, ModelPoolStatus};
use crate::domain::{serde_json, ActionSpec, DomainError, DomainPlugin};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant};

pub struct LlamacppPlugin {
    state: Mutex<LlamacppState>,
}

struct LlamacppState {
    child: Option<Child>,
    port: u16,
    started_at: Option<Instant>,
    current_model: Option<String>,
    models_dir: PathBuf,
}

impl LlamacppPlugin {
    pub fn new() -> Self {
        let models_dir = dirs::home_dir()
            .unwrap_or_default()
            .join("Downloads")
            .join("neotrix")
            .join("models");
        Self {
            state: Mutex::new(LlamacppState {
                child: None,
                port: 8080,
                started_at: None,
                current_model: None,
                models_dir,
            }),
        }
    }

    fn find_binary() -> Option<PathBuf> {
        let candidates = [
            PathBuf::from("/opt/homebrew/bin/llama-server"),
            PathBuf::from("/usr/local/bin/llama-server"),
        ];
        for c in &candidates {
            if c.exists() {
                return Some(c.clone());
            }
        }
        std::process::Command::new("which")
            .arg("llama-server")
            .output()
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    let p = String::from_utf8_lossy(&o.stdout).trim().to_string();
                    if !p.is_empty() {
                        Some(PathBuf::from(p))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
    }
}

// ========== Config Helpers (provider/pool) ==========

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

fn resolvable(cfg: &ConfigData) -> bool {
    !cfg.provider.is_empty() && !cfg.default_model.is_empty()
}

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

    if !cfg.provider.is_empty() {
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

// ========== Llamacpp Plugin ==========

impl DomainPlugin for LlamacppPlugin {
    fn name(&self) -> &str {
        "llamacpp"
    }
    fn description(&self) -> &str {
        "本地推理：llama.cpp 进程管理、模型扫描、provider/pool 管理、健康检查"
    }

    fn actions(&self) -> Vec<ActionSpec> {
        vec![
            ActionSpec {
                name: "health".into(),
                description: "获取 llama.cpp 健康状态".into(),
                params: vec![],
                returns: "Value".into(),
            },
            ActionSpec {
                name: "models".into(),
                description: "扫描本地模型列表".into(),
                params: vec![],
                returns: "Value".into(),
            },
            ActionSpec {
                name: "start".into(),
                description: "启动 llama-server".into(),
                params: vec![],
                returns: "Value".into(),
            },
            ActionSpec {
                name: "stop".into(),
                description: "停止 llama-server".into(),
                params: vec![],
                returns: "Value".into(),
            },
            ActionSpec {
                name: "swap".into(),
                description: "切换模型".into(),
                params: vec![],
                returns: "Value".into(),
            },
            ActionSpec {
                name: "send".into(),
                description: "发送推理请求".into(),
                params: vec![],
                returns: "Value".into(),
            },
            ActionSpec {
                name: "provider_config".into(),
                description: "获取 provider 配置".into(),
                params: vec![],
                returns: "Value".into(),
            },
            ActionSpec {
                name: "provider_status".into(),
                description: "获取 provider 健康状态".into(),
                params: vec![],
                returns: "Value".into(),
            },
            ActionSpec {
                name: "pool_sufficiency".into(),
                description: "检查 pool 资源充足性".into(),
                params: vec![],
                returns: "Value".into(),
            },
            ActionSpec {
                name: "discover_models".into(),
                description: "发现所有可用模型".into(),
                params: vec![],
                returns: "Value".into(),
            },
            ActionSpec {
                name: "pool_status".into(),
                description: "pool 状态查询".into(),
                params: vec![],
                returns: "Value".into(),
            },
            ActionSpec {
                name: "pool_add".into(),
                description: "添加 provider 到 pool".into(),
                params: vec![],
                returns: "Value".into(),
            },
            ActionSpec {
                name: "pool_remove".into(),
                description: "从 pool 移除 provider".into(),
                params: vec![],
                returns: "Value".into(),
            },
            ActionSpec {
                name: "pool_update_key".into(),
                description: "更新 pool provider API key".into(),
                params: vec![],
                returns: "Value".into(),
            },
            ActionSpec {
                name: "pool_check".into(),
                description: "检查 pool provider 可用性".into(),
                params: vec![],
                returns: "Value".into(),
            },
            ActionSpec {
                name: "pool_health".into(),
                description: "pool 健康概览".into(),
                params: vec![],
                returns: "Value".into(),
            },
        ]
    }

    fn call(
        &self,
        action: &str,
        args: serde_json::Value,
    ) -> Result<serde_json::Value, DomainError> {
        match action {
            // ---- Process management (requires lock) ----
            "start" | "stop" | "swap" | "send" => {
                let mut state = self
                    .state
                    .lock()
                    .map_err(|e| DomainError::from(e.to_string()))?;
                match action {
                    "start" => {
                        let model_path = args.get("model_path").and_then(|v| v.as_str());
                        if let Some(path) = model_path {
                            start_server(&mut state, path)?;
                        } else if let Some(model) = find_model(&state.models_dir) {
                            start_server(&mut state, &model.to_string_lossy())?;
                        } else {
                            return Err(DomainError::from("No model found in models directory"));
                        }
                        Ok(serde_json::json!({ "started": true }))
                    }
                    "stop" => {
                        stop_server(&mut state);
                        Ok(serde_json::json!({ "stopped": true }))
                    }
                    "swap" => {
                        let model_path = args
                            .get("model_path")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| DomainError::from("model_path required"))?;
                        stop_server(&mut state);
                        start_server(&mut state, model_path)?;
                        Ok(serde_json::json!({ "swapped": true }))
                    }
                    "send" => {
                        let messages = args
                            .get("messages")
                            .cloned()
                            .ok_or_else(|| DomainError::from("messages required"))?;
                        let temperature = args
                            .get("temperature")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.7);
                        let max_tokens = args
                            .get("max_tokens")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(2048);

                        let port = state.port;
                        drop(state);

                        let body = serde_json::json!({
                            "messages": messages,
                            "temperature": temperature,
                            "max_tokens": max_tokens,
                            "stream": false,
                        });

                        let url = format!("http://127.0.0.1:{port}/v1/chat/completions");
                        let client = reqwest::blocking::Client::new();
                        let resp = client
                            .post(&url)
                            .json(&body)
                            .timeout(Duration::from_secs(120))
                            .send()
                            .map_err(|e| DomainError::from(format!("Request failed: {e}")))?;

                        let json: serde_json::Value = resp
                            .json()
                            .map_err(|e| DomainError::from(format!("Parse response: {e}")))?;

                        Ok(json)
                    }
                    _ => unreachable!(),
                }
            }
            // ---- Health / models (read-only, needs state) ----
            "health" => {
                let state = self
                    .state
                    .lock()
                    .map_err(|e| DomainError::from(e.to_string()))?;
                let running = state.child.is_some();
                Ok(serde_json::json!({
                    "running": running,
                    "port": state.port,
                    "pid": state.child.as_ref().map(|c| c.id()),
                    "uptime_secs": state.started_at.map(|t| t.elapsed().as_secs()).unwrap_or(0),
                    "model_loaded": state.current_model,
                    "binary_found": Self::find_binary().is_some(),
                }))
            }
            "models" => {
                let state = self
                    .state
                    .lock()
                    .map_err(|e| DomainError::from(e.to_string()))?;
                let models = scan_models(&state.models_dir);
                Ok(serde_json::json!({ "models": models }))
            }
            // ---- Provider config / pool management (file-based, no lock needed) ----
            "provider_config" | "config" => read_provider_config(),
            "provider_status" | "status" => provider_status(),
            "pool_sufficiency" | "sufficiency" => {
                let min = args.get("min").and_then(|v| v.as_u64()).unwrap_or(3) as usize;
                pool_sufficiency(min)
            }
            "discover_models" | "discover" => discover_models(),
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
                for entry in &pool {
                    let name = entry
                        .get("label")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");
                    let provider = entry.get("provider").and_then(|v| v.as_str()).unwrap_or("");
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
            _ => Err(DomainError {
                code: "UNKNOWN_ACTION".into(),
                message: format!("Unknown action: {}", action),
                recoverable: true,
            }),
        }
    }
}

fn start_server(state: &mut LlamacppState, model_path: &str) -> Result<(), DomainError> {
    let binary =
        LlamacppPlugin::find_binary().ok_or_else(|| DomainError::from("llama-server not found"))?;

    let mut child = Command::new(&binary)
        .args([
            "--host",
            "127.0.0.1",
            "--port",
            &state.port.to_string(),
            "--model",
            model_path,
            "--ctx-size",
            "4096",
            "--parallel",
            "2",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| DomainError::from(format!("Failed to start: {e}")))?;

    std::thread::sleep(Duration::from_secs(2));

    match child.try_wait() {
        Ok(Some(status)) => Err(DomainError::from(format!("Exited: {status}"))),
        Ok(None) => {
            state.child = Some(child);
            state.started_at = Some(Instant::now());
            state.current_model = Some(
                PathBuf::from(model_path)
                    .file_name()
                    .map(|f| f.to_string_lossy().to_string())
                    .unwrap_or_default(),
            );
            Ok(())
        }
        Err(e) => Err(DomainError::from(format!("Status check failed: {e}"))),
    }
}

fn stop_server(state: &mut LlamacppState) {
    if let Some(mut child) = state.child.take() {
        let _ = child.kill();
        let _ = child.wait();
    }
    state.started_at = None;
    state.current_model = None;
}

fn scan_models(dir: &PathBuf) -> Vec<serde_json::Value> {
    if !dir.exists() {
        return vec![];
    }
    std::fs::read_dir(dir)
        .ok()
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.path()
                        .extension()
                        .map(|ext| ext == "gguf" || ext == "onnx")
                        .unwrap_or(false)
                })
                .filter_map(|e| {
                    let path = e.path();
                    let meta = e.metadata().ok()?;
                    let name = path.file_stem()?.to_string_lossy().to_string();
                    let format = path.extension()
                        .map(|ext| ext.to_string_lossy().to_uppercase())
                        .unwrap_or_default();
                    let backend = if format == "ONNX" { "onnxruntime" } else { "llamacpp" };
                    Some(serde_json::json!({
                        "name": name,
                        "path": path.to_string_lossy(),
                        "size_bytes": meta.len(),
                        "format": format,
                        "backend": backend,
                        "quantization": extract_quant(&name),
                    }))
                })
                .collect()
        })
        .unwrap_or_default()
}

fn find_model(dir: &PathBuf) -> Option<PathBuf> {
    if !dir.exists() {
        return None;
    }
    const MIN_MAIN_MODEL_BYTES: u64 = 1024 * 1024 * 1024; // 1 GB
    std::fs::read_dir(dir)
        .ok()?
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name().to_string_lossy().to_lowercase();
            e.path()
                .extension()
                .map(|ext| ext == "gguf" || ext == "onnx")
                .unwrap_or(false)
                && !name.contains("mmproj")
        })
        .filter(|e| e.metadata().map(|m| m.len()).unwrap_or(0) >= MIN_MAIN_MODEL_BYTES)
        .max_by_key(|e| e.metadata().map(|m| m.len()).unwrap_or(0))
        .map(|e| e.path())
}

fn extract_quant(name: &str) -> String {
    let lower = name.to_lowercase();
    for q in &[
        "iq4_nl", "q8_0", "q6_k", "q5_k_m", "q4_k_m", "q4_0", "f16", "f32", "bf16",
    ] {
        if lower.contains(q) {
            return q.to_uppercase();
        }
    }
    "unknown".into()
}
