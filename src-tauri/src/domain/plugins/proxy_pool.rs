//! ProxyPool domain plugin — thin adapter over `crate::domain::proxy_pool`.
//!
//! Exposes the proxy-pool business logic (nodes, subscriptions, strategy)
//! through the `DomainRegistry` so `domain_call` and `chat_send` can reach it.

use crate::domain::{serde_json, ActionSpec, DomainError, DomainPlugin, ParamSpec};
use async_trait::async_trait;
use std::path::PathBuf;

/// 代理池域插件
pub struct ProxyPoolPlugin;

impl ProxyPoolPlugin {
    pub fn new() -> Self {
        Self
    }

    fn base_dir() -> Result<PathBuf, DomainError> {
        crate::config::AppConfig::base_dir().ok_or_else(|| DomainError {
            code: "PROXY_NO_BASE_DIR".into(),
            message: "Cannot determine base directory".into(),
            recoverable: false,
        })
    }
}

impl Default for ProxyPoolPlugin {
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
impl DomainPlugin for ProxyPoolPlugin {
    fn name(&self) -> &str {
        "proxy_pool"
    }

    fn description(&self) -> &str {
        "代理池：节点管理、订阅源、负载均衡策略"
    }

    fn actions(&self) -> Vec<ActionSpec> {
        vec![
            ActionSpec {
                name: "status".into(),
                description: "代理池状态（节点、健康度、策略、订阅）".into(),
                params: vec![],
                returns: "ProxyPoolStatus".into(),
            },
            ActionSpec {
                name: "snapshot".into(),
                description: "代理池聚合快照（地理分布、速度分级）".into(),
                params: vec![],
                returns: "ProxyPoolSnapshot".into(),
            },
            ActionSpec {
                name: "add".into(),
                description: "添加代理节点".into(),
                params: vec![
                    param("url", "string", "代理 URL", false),
                    param("tag", "string", "节点标签", true),
                ],
                returns: "ProxyPoolEntry".into(),
            },
            ActionSpec {
                name: "remove".into(),
                description: "删除代理节点".into(),
                params: vec![param("url", "string", "代理 URL", false)],
                returns: "bool".into(),
            },
            ActionSpec {
                name: "add_subscription".into(),
                description: "添加订阅源".into(),
                params: vec![param("url", "string", "订阅 URL", false)],
                returns: "string[]".into(),
            },
            ActionSpec {
                name: "remove_subscription".into(),
                description: "删除订阅源".into(),
                params: vec![param("url", "string", "订阅 URL", false)],
                returns: "string[]".into(),
            },
            ActionSpec {
                name: "set_strategy".into(),
                description: "设置负载均衡策略".into(),
                params: vec![param("strategy", "string", "策略名", false)],
                returns: "string".into(),
            },
            ActionSpec {
                name: "list_strategies".into(),
                description: "列出可用策略".into(),
                params: vec![],
                returns: "string[]".into(),
            },
        ]
    }

    async fn call(
        &self,
        action: &str,
        args: serde_json::Value,
    ) -> Result<serde_json::Value, DomainError> {
        use crate::domain::proxy_pool as pool;

        let base = Self::base_dir()?;
        match action {
            "status" => {
                let status = pool::get_status(&base)
                    .map_err(|e| err("POOL_ERROR", e.to_string()))?;
                to_json(status)
            }
            "snapshot" => {
                let snapshot = pool::get_snapshot(&base)
                    .map_err(|e| err("POOL_ERROR", e.to_string()))?;
                to_json(snapshot)
            }
            "add" => {
                let url = req_str(&args, "url")?;
                let tag = opt_str(&args, "tag");
                let entry = pool::add_node(&base, url, tag.as_deref())
                    .map_err(|e| err("POOL_ERROR", e.to_string()))?;
                to_json(entry)
            }
            "remove" => {
                let url = req_str(&args, "url")?;
                let removed = pool::remove_node(&base, url)
                    .map_err(|e| err("POOL_ERROR", e.to_string()))?;
                Ok(serde_json::json!({ "removed": removed }))
            }
            "add_subscription" => {
                let url = req_str(&args, "url")?;
                let subs = pool::add_subscription(&base, url)
                    .map_err(|e| err("POOL_ERROR", e.to_string()))?;
                to_json(subs)
            }
            "remove_subscription" => {
                let url = req_str(&args, "url")?;
                let subs = pool::remove_subscription(&base, url)
                    .map_err(|e| err("POOL_ERROR", e.to_string()))?;
                to_json(subs)
            }
            "set_strategy" => {
                let strategy = req_str(&args, "strategy")?;
                let applied = pool::set_strategy(&base, strategy)
                    .map_err(|e| err("POOL_ERROR", e.to_string()))?;
                Ok(serde_json::json!({ "strategy": applied }))
            }
            "list_strategies" => to_json(pool::VALID_STRATEGIES),
            _ => Err(err(
                "UNKNOWN_ACTION",
                format!("Unknown proxy_pool action: {action}"),
            )),
        }
    }
}
