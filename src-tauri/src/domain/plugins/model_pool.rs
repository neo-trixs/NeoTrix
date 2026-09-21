//! ModelPool domain plugin — thin adapter over `crate::domain::model_pool`.
//!
//! Exposes the model-provider pool (TOML-backed) through the
//! `DomainRegistry` so `domain_call` and `chat_send` can reach it.

use crate::domain::{serde_json, ActionSpec, DomainError, DomainPlugin, ParamSpec};
use async_trait::async_trait;

/// 模型池域插件
pub struct ModelPoolPlugin;

impl ModelPoolPlugin {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ModelPoolPlugin {
    fn default() -> Self {
        Self::new()
    }
}

fn err(code: &str, message: String) -> DomainError {
    DomainError {
        code: code.into(),
        message,
        recoverable: true,
    }
}

fn req_str<'a>(args: &'a serde_json::Value, key: &str) -> Result<&'a str, DomainError> {
    args.get(key).and_then(|v| v.as_str()).ok_or_else(|| {
        err(
            "INVALID_ARGS",
            format!("Missing required string argument: {key}"),
        )
    })
}

fn opt_str(args: &serde_json::Value, key: &str) -> Option<String> {
    args.get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

fn opt_str_list(args: &serde_json::Value, key: &str) -> Vec<String> {
    args.get(key)
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

fn param(name: &str, r#type: &str, description: &str, optional: bool) -> ParamSpec {
    ParamSpec {
        name: name.into(),
        r#type: r#type.into(),
        description: description.into(),
        optional,
    }
}

fn to_json<T: serde::Serialize>(v: T) -> Result<serde_json::Value, DomainError> {
    serde_json::to_value(v)
        .map_err(|e| err("SERIALIZE_ERROR", format!("Serialize result: {e}")))
}

#[async_trait]
impl DomainPlugin for ModelPoolPlugin {
    fn name(&self) -> &str {
        "model_pool"
    }

    fn description(&self) -> &str {
        "模型池：供应商配置、API Key 管理、连通性检查"
    }

    fn actions(&self) -> Vec<ActionSpec> {
        vec![
            ActionSpec {
                name: "status".into(),
                description: "模型池状态（条目列表、配置路径）".into(),
                params: vec![],
                returns: "ModelPoolStatus".into(),
            },
            ActionSpec {
                name: "add".into(),
                description: "添加模型供应商条目".into(),
                params: vec![
                    param("label", "string", "条目标签", false),
                    param("provider", "string", "供应商名", false),
                    param("api_key", "string", "API Key", false),
                    param("model", "string", "模型名", false),
                    param("tags", "string[]", "标签", true),
                    param("base_url", "string", "自定义网关", true),
                ],
                returns: "ModelPoolEntry".into(),
            },
            ActionSpec {
                name: "remove".into(),
                description: "删除模型供应商条目".into(),
                params: vec![param("label", "string", "条目标签", false)],
                returns: "bool".into(),
            },
            ActionSpec {
                name: "update_key".into(),
                description: "更新条目的 API Key".into(),
                params: vec![
                    param("label", "string", "条目标签", false),
                    param("new_key", "string", "新 API Key", false),
                ],
                returns: "bool".into(),
            },
            ActionSpec {
                name: "check".into(),
                description: "检查供应商连通性".into(),
                params: vec![param("label", "string", "条目标签", false)],
                returns: "string".into(),
            },
        ]
    }

    async fn call(
        &self,
        action: &str,
        args: serde_json::Value,
    ) -> Result<serde_json::Value, DomainError> {
        use crate::domain::model_pool as pool;

        match action {
            "status" => {
                let status =
                    pool::get_status().map_err(|e| err("POOL_ERROR", e.to_string()))?;
                to_json(status)
            }
            "add" => {
                let label = req_str(&args, "label")?;
                let provider = req_str(&args, "provider")?;
                let api_key = req_str(&args, "api_key")?;
                let model = req_str(&args, "model")?;
                let tags = opt_str_list(&args, "tags");
                let base_url = opt_str(&args, "base_url");
                let entry = pool::add_entry(
                    label,
                    provider,
                    api_key,
                    model,
                    &tags,
                    base_url.as_deref(),
                )
                .map_err(|e| err("POOL_ERROR", e.to_string()))?;
                to_json(entry)
            }
            "remove" => {
                let label = req_str(&args, "label")?;
                let removed = pool::remove_entry(label)
                    .map_err(|e| err("POOL_ERROR", e.to_string()))?;
                Ok(serde_json::json!({ "removed": removed }))
            }
            "update_key" => {
                let label = req_str(&args, "label")?;
                // Accept both key names: ours (`new_key`) and llamacpp's
                // (`new_api_key`) — same backend, one contract.
                let new_key = opt_str(&args, "new_key")
                    .or_else(|| opt_str(&args, "new_api_key"))
                    .ok_or_else(|| err(
                        "INVALID_ARGS",
                        "Missing required string argument: new_key".into(),
                    ))?;
                let new_key = new_key.as_str();
                let updated = pool::update_api_key(label, new_key)
                    .map_err(|e| err("POOL_ERROR", e.to_string()))?;
                Ok(serde_json::json!({ "updated": updated }))
            }
            "check" => {
                let label = req_str(&args, "label")?;
                let report = pool::check_connectivity(label)
                    .await
                    .map_err(|e| err("POOL_ERROR", e.to_string()))?;
                Ok(serde_json::json!({ "report": report }))
            }
            _ => Err(err(
                "UNKNOWN_ACTION",
                format!("Unknown model_pool action: {action}"),
            )),
        }
    }
}
