//! Plugin Plugin (meta) — 插件：安装/卸载/启停、marketplace

use super::common::{bump_version, stub_action};
use crate::domain::{serde_json, ActionSpec, DomainError, DomainPlugin};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

static PLUGIN_STATE: LazyLock<Mutex<HashMap<String, serde_json::Value>>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert(
            "plugins".into(),
            serde_json::json!({
                "session": { "enabled": true, "description": "会话管理", "version": "0.1.0" },
                "chat": { "enabled": true, "description": "对话/LLM", "version": "0.1.0" },
                "kb": { "enabled": true, "description": "知识库", "version": "0.1.0" },
                "file": { "enabled": true, "description": "文件操作", "version": "0.1.0" },
                "memory": { "enabled": true, "description": "记忆管理", "version": "0.1.0" },
                "world": { "enabled": true, "description": "世界感知", "version": "0.1.0" },
                "workflow": { "enabled": true, "description": "工作流", "version": "0.1.0" },
                "agent": { "enabled": true, "description": "Agent 状态/任务/provider", "version": "0.1.0" },
                "tool": { "enabled": true, "description": "工具管理", "version": "0.1.0" },
                "system": { "enabled": true, "description": "系统管理", "version": "0.1.0" },
                "security": { "enabled": true, "description": "安全扫描/审计", "version": "0.1.0" },
                "ext": { "enabled": true, "description": "扩展/协作", "version": "0.1.0" },
                "git": { "enabled": true, "description": "Git 版本控制", "version": "0.1.0" },
                "cli": { "enabled": true, "description": "CLI 命令执行", "version": "0.1.0" },
                "llamacpp": { "enabled": true, "description": "llama.cpp 本地推理", "version": "0.1.0" },
            }),
        );
    m.insert("config".into(), serde_json::json!({}));
    Mutex::new(m)
});

pub struct PluginPlugin;

#[async_trait]
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
    async fn call(
        &self,
        action: &str,
        args: serde_json::Value,
    ) -> Result<serde_json::Value, DomainError> {
        match action {
            "list" => {
                let state = PLUGIN_STATE.lock().map_err(|e| DomainError {
                    code: "LOCK_ERROR".into(),
                    message: e.to_string(),
                    recoverable: true,
                })?;
                let plugins = state
                    .get("plugins")
                    .cloned()
                    .unwrap_or(serde_json::json!({}));
                let count = plugins.as_object().map(|m| m.len()).unwrap_or(0);
                Ok(serde_json::json!({
                    "plugins": plugins,
                    "count": count,
                }))
            }
            "install" => {
                let name = args
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                if name.is_empty() {
                    return Err(DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 name 参数".into(),
                        recoverable: true,
                    });
                }
                let description = args
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let mut state = PLUGIN_STATE.lock().map_err(|e| DomainError {
                    code: "LOCK_ERROR".into(),
                    message: e.to_string(),
                    recoverable: true,
                })?;
                let plugins = state
                    .get_mut("plugins")
                    .and_then(|v| v.as_object_mut())
                    .ok_or_else(|| DomainError {
                        code: "STATE_ERROR".into(),
                        message: "Plugin state corrupted".into(),
                        recoverable: true,
                    })?;
                if plugins.contains_key(&name) {
                    return Ok(serde_json::json!({
                        "ok": false,
                        "message": format!("Plugin '{}' already installed", name),
                    }));
                }
                plugins.insert(
                    name.clone(),
                    serde_json::json!({ "enabled": true, "description": description }),
                );
                Ok(serde_json::json!({ "ok": true, "name": name }))
            }
            "uninstall" => {
                let name = args
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                if name.is_empty() {
                    return Err(DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 name 参数".into(),
                        recoverable: true,
                    });
                }
                let mut state = PLUGIN_STATE.lock().map_err(|e| DomainError {
                    code: "LOCK_ERROR".into(),
                    message: e.to_string(),
                    recoverable: true,
                })?;
                let plugins = state
                    .get_mut("plugins")
                    .and_then(|v| v.as_object_mut())
                    .ok_or_else(|| DomainError {
                        code: "STATE_ERROR".into(),
                        message: "Plugin state corrupted".into(),
                        recoverable: true,
                    })?;
                if plugins.remove(&name).is_none() {
                    return Ok(serde_json::json!({
                        "ok": false,
                        "message": format!("Plugin '{}' not found", name),
                    }));
                }
                Ok(serde_json::json!({ "ok": true, "name": name }))
            }
            "enable" => {
                let name = args
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                if name.is_empty() {
                    return Err(DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 name 参数".into(),
                        recoverable: true,
                    });
                }
                let mut state = PLUGIN_STATE.lock().map_err(|e| DomainError {
                    code: "LOCK_ERROR".into(),
                    message: e.to_string(),
                    recoverable: true,
                })?;
                let plugins = state
                    .get_mut("plugins")
                    .and_then(|v| v.as_object_mut())
                    .ok_or_else(|| DomainError {
                        code: "STATE_ERROR".into(),
                        message: "Plugin state corrupted".into(),
                        recoverable: true,
                    })?;
                match plugins.get_mut(&name) {
                    Some(p) => {
                        if let Some(obj) = p.as_object_mut() {
                            obj.insert("enabled".into(), serde_json::json!(true));
                        }
                        Ok(serde_json::json!({ "ok": true, "name": name, "enabled": true }))
                    }
                    None => Err(DomainError {
                        code: "NOT_FOUND".into(),
                        message: format!("Plugin '{}' not found", name),
                        recoverable: true,
                    }),
                }
            }
            "disable" => {
                let name = args
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                if name.is_empty() {
                    return Err(DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 name 参数".into(),
                        recoverable: true,
                    });
                }
                let mut state = PLUGIN_STATE.lock().map_err(|e| DomainError {
                    code: "LOCK_ERROR".into(),
                    message: e.to_string(),
                    recoverable: true,
                })?;
                let plugins = state
                    .get_mut("plugins")
                    .and_then(|v| v.as_object_mut())
                    .ok_or_else(|| DomainError {
                        code: "STATE_ERROR".into(),
                        message: "Plugin state corrupted".into(),
                        recoverable: true,
                    })?;
                match plugins.get_mut(&name) {
                    Some(p) => {
                        if let Some(obj) = p.as_object_mut() {
                            obj.insert("enabled".into(), serde_json::json!(false));
                        }
                        Ok(serde_json::json!({ "ok": true, "name": name, "enabled": false }))
                    }
                    None => Err(DomainError {
                        code: "NOT_FOUND".into(),
                        message: format!("Plugin '{}' not found", name),
                        recoverable: true,
                    }),
                }
            }
            "marketplace" => {
                let installed = {
                    let state = PLUGIN_STATE.lock().map_err(|e| DomainError {
                        code: "LOCK_ERROR".into(),
                        message: e.to_string(),
                        recoverable: true,
                    })?;
                    state
                        .get("plugins")
                        .cloned()
                        .unwrap_or(serde_json::json!({}))
                };
                let available = vec![
                    serde_json::json!({
                        "name": "session",
                        "description": "会话管理",
                        "version": "0.1.0",
                        "installed": installed.get("session").is_some(),
                    }),
                    serde_json::json!({
                        "name": "chat",
                        "description": "对话/LLM",
                        "version": "0.1.0",
                        "installed": installed.get("chat").is_some(),
                    }),
                    serde_json::json!({
                        "name": "kb",
                        "description": "知识库",
                        "version": "0.1.0",
                        "installed": installed.get("kb").is_some(),
                    }),
                    serde_json::json!({
                        "name": "file",
                        "description": "文件操作",
                        "version": "0.1.0",
                        "installed": installed.get("file").is_some(),
                    }),
                    serde_json::json!({
                        "name": "memory",
                        "description": "记忆管理",
                        "version": "0.1.0",
                        "installed": installed.get("memory").is_some(),
                    }),
                    serde_json::json!({
                        "name": "world",
                        "description": "世界感知",
                        "version": "0.1.0",
                        "installed": installed.get("world").is_some(),
                    }),
                    serde_json::json!({
                        "name": "workflow",
                        "description": "工作流",
                        "version": "0.1.0",
                        "installed": installed.get("workflow").is_some(),
                    }),
                    serde_json::json!({
                        "name": "agent",
                        "description": "Agent 状态/任务/provider",
                        "version": "0.1.0",
                        "installed": installed.get("agent").is_some(),
                    }),
                    serde_json::json!({
                        "name": "tool",
                        "description": "工具管理",
                        "version": "0.1.0",
                        "installed": installed.get("tool").is_some(),
                    }),
                    serde_json::json!({
                        "name": "system",
                        "description": "系统管理",
                        "version": "0.1.0",
                        "installed": installed.get("system").is_some(),
                    }),
                    serde_json::json!({
                        "name": "security",
                        "description": "安全扫描/审计",
                        "version": "0.1.0",
                        "installed": installed.get("security").is_some(),
                    }),
                    serde_json::json!({
                        "name": "ext",
                        "description": "扩展/协作",
                        "version": "0.1.0",
                        "installed": installed.get("ext").is_some(),
                    }),
                    serde_json::json!({
                        "name": "git",
                        "description": "Git 版本控制",
                        "version": "0.1.0",
                        "installed": installed.get("git").is_some(),
                    }),
                    serde_json::json!({
                        "name": "cli",
                        "description": "CLI 命令执行",
                        "version": "0.1.0",
                        "installed": installed.get("cli").is_some(),
                    }),
                    serde_json::json!({
                        "name": "llamacpp",
                        "description": "llama.cpp 本地推理",
                        "version": "0.1.0",
                        "installed": installed.get("llamacpp").is_some(),
                    }),
                ];
                Ok(serde_json::json!({
                    "marketplace": available,
                    "count": available.len(),
                }))
            }
            "update" => {
                let name = args
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                if name.is_empty() {
                    return Err(DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 name 参数".into(),
                        recoverable: true,
                    });
                }
                let mut state = PLUGIN_STATE.lock().map_err(|e| DomainError {
                    code: "LOCK_ERROR".into(),
                    message: e.to_string(),
                    recoverable: true,
                })?;
                let plugins = state
                    .get_mut("plugins")
                    .and_then(|v| v.as_object_mut())
                    .ok_or_else(|| DomainError {
                        code: "STATE_ERROR".into(),
                        message: "Plugin state corrupted".into(),
                        recoverable: true,
                    })?;
                match plugins.get_mut(&name) {
                    Some(p) => {
                        if let Some(obj) = p.as_object_mut() {
                            let current_version = obj
                                .get("version")
                                .and_then(|v| v.as_str())
                                .unwrap_or("0.0.0")
                                .to_string();
                            let bumped = bump_version(&current_version);
                            obj.insert("version".into(), serde_json::json!(bumped));
                            Ok(serde_json::json!({
                                "ok": true,
                                "name": name,
                                "from_version": current_version,
                                "to_version": bumped,
                            }))
                        } else {
                            Err(DomainError {
                                code: "STATE_ERROR".into(),
                                message: format!("Plugin '{}' state corrupted", name),
                                recoverable: true,
                            })
                        }
                    }
                    None => Err(DomainError {
                        code: "NOT_FOUND".into(),
                        message: format!("Plugin '{}' not found", name),
                        recoverable: true,
                    }),
                }
            }
            "config" => {
                let plugin_name = args
                    .get("plugin")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let set_key = args.get("key").and_then(|v| v.as_str());
                let set_value = args.get("value").cloned();
                let mut state = PLUGIN_STATE.lock().map_err(|e| DomainError {
                    code: "LOCK_ERROR".into(),
                    message: e.to_string(),
                    recoverable: true,
                })?;
                let config = state
                    .get_mut("config")
                    .and_then(|v| v.as_object_mut())
                    .ok_or_else(|| DomainError {
                        code: "STATE_ERROR".into(),
                        message: "Config state corrupted".into(),
                        recoverable: true,
                    })?;
                if let Some(key) = set_key {
                    // Set config value
                    let target = if plugin_name.is_empty() {
                        "_global"
                    } else {
                        &plugin_name
                    };
                    let plugin_cfg = config
                        .entry(target.to_string())
                        .or_insert_with(|| serde_json::json!({}))
                        .as_object_mut()
                        .ok_or_else(|| DomainError {
                            code: "STATE_ERROR".into(),
                            message: "Plugin config corrupted".into(),
                            recoverable: true,
                        })?;
                    plugin_cfg.insert(
                        key.to_string(),
                        set_value.unwrap_or(serde_json::json!(null)),
                    );
                    Ok(serde_json::json!({ "ok": true, "plugin": target, "key": key }))
                } else {
                    // Get config
                    let target = if plugin_name.is_empty() {
                        "_global"
                    } else {
                        &plugin_name
                    };
                    let plugin_cfg = config.get(target).cloned().unwrap_or(serde_json::json!({}));
                    Ok(serde_json::json!({
                        "plugin": target,
                        "config": plugin_cfg,
                    }))
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
