//! Agent Plugin — Agent 生命周期：启动/停止/健康检查

use super::common::stub_action;
use crate::domain::{serde_json, ActionSpec, DomainError, DomainPlugin};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

static AGENT_STATE: LazyLock<Mutex<HashMap<String, serde_json::Value>>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert("running".into(), serde_json::json!(false));
    m.insert("started_at".into(), serde_json::json!(null));
    Mutex::new(m)
});

pub struct AgentPlugin;

#[async_trait]
impl DomainPlugin for AgentPlugin {
    fn name(&self) -> &str {
        "agent"
    }
    fn description(&self) -> &str {
        "Agent 生命周期：启动/停止/健康检查"
    }
    fn actions(&self) -> Vec<ActionSpec> {
        vec!["status", "start", "stop", "health", "app_version"]
            .iter()
            .map(|a| stub_action(a))
            .collect()
    }
    async fn call(
        &self,
        action: &str,
        _args: serde_json::Value,
    ) -> Result<serde_json::Value, DomainError> {
        match action {
            "app_version" => Ok(serde_json::json!(env!("CARGO_PKG_VERSION"))),
            "status" | "health" => {
                let state = AGENT_STATE.lock().map_err(|e| DomainError {
                    code: "LOCK_ERROR".into(),
                    message: e.to_string(),
                    recoverable: true,
                })?;
                let running = state
                    .get("running")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let started_at = state
                    .get("started_at")
                    .cloned()
                    .unwrap_or(serde_json::json!(null));
                Ok(serde_json::json!({
                    "status": if running { "running" } else { "stopped" },
                    "running": running,
                    "started_at": started_at,
                }))
            }
            "start" => {
                let mut state = AGENT_STATE.lock().map_err(|e| DomainError {
                    code: "LOCK_ERROR".into(),
                    message: e.to_string(),
                    recoverable: true,
                })?;
                if state
                    .get("running")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
                {
                    return Ok(
                        serde_json::json!({ "ok": true, "message": "Agent already running" }),
                    );
                }
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                state.insert("running".into(), serde_json::json!(true));
                state.insert("started_at".into(), serde_json::json!(now));
                Ok(serde_json::json!({ "ok": true, "started_at": now }))
            }
            "stop" => {
                let mut state = AGENT_STATE.lock().map_err(|e| DomainError {
                    code: "LOCK_ERROR".into(),
                    message: e.to_string(),
                    recoverable: true,
                })?;
                if !state
                    .get("running")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
                {
                    return Ok(serde_json::json!({ "ok": true, "message": "Agent not running" }));
                }
                state.insert("running".into(), serde_json::json!(false));
                state.insert("started_at".into(), serde_json::json!(null));
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
