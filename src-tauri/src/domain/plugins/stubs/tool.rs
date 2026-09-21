//! Tool Plugin — 工具：MCP、harness、computer、voice

use super::common::stub_action;
use crate::domain::{serde_json, ActionSpec, DomainError, DomainPlugin};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

/// Pending approval queue (PI-style Allow/Ask/Deny).
///
/// Producers file requests here instead of executing directly (see
/// `request_approval`); the UI (`ApprovalPanel`) lists them and writes
/// back decisions via `approval_resolve`. State is in-memory: pending
/// approvals do not survive restarts by design (fail-closed on reboot).
static APPROVALS: LazyLock<Mutex<HashMap<String, serde_json::Value>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// File an approval request. Shared by the `approval_request` action and by
/// guardrailed executors (e.g. blocked `cli/exec` binaries auto-file here
/// instead of running). Returns the created entry.
pub(crate) fn request_approval(action: &str, detail: &str) -> serde_json::Value {
    let id = format!("appr-{}", uuid::Uuid::new_v4());
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let entry = serde_json::json!({
        "id": id,
        "action": action,
        "state": "pending",
        "detail": detail,
        "created_at": now,
    });
    if let Ok(mut q) = APPROVALS.lock() {
        q.insert(id, entry.clone());
    }
    entry
}

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

#[async_trait]
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
            "approval_request",
            "approval_list",
            "approval_resolve",
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
            "approval_request" => {
                let action = args
                    .get("action")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let detail = args
                    .get("detail")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                Ok(request_approval(action, detail))
            }
            "approval_list" => {
                let queue = APPROVALS.lock().map_err(|e| DomainError {
                    code: "LOCK_ERROR".into(),
                    message: e.to_string(),
                    recoverable: true,
                })?;
                let mut items: Vec<serde_json::Value> = queue.values().cloned().collect();
                // Newest first.
                items.sort_by(|a, b| {
                    b.get("created_at")
                        .and_then(|v| v.as_u64())
                        .cmp(&a.get("created_at").and_then(|v| v.as_u64()))
                });
                Ok(serde_json::json!({
                    "approvals": items,
                    "count": items.len(),
                }))
            }
            "approval_resolve" => {
                let id = args
                    .get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "Missing required string argument: id".into(),
                        recoverable: true,
                    })?;
                let decision = args
                    .get("decision")
                    .and_then(|v| v.as_str())
                    .unwrap_or("deny");
                let state = if decision == "approve" {
                    "approved"
                } else {
                    "denied"
                };
                let mut queue = APPROVALS.lock().map_err(|e| DomainError {
                    code: "LOCK_ERROR".into(),
                    message: e.to_string(),
                    recoverable: true,
                })?;
                match queue.get_mut(id) {
                    Some(entry) => {
                        if let Some(obj) = entry.as_object_mut() {
                            obj.insert("state".into(), serde_json::json!(state));
                        }
                        Ok(entry.clone())
                    }
                    None => Err(DomainError {
                        code: "NOT_FOUND".into(),
                        message: format!("Approval '{id}' not found"),
                        recoverable: true,
                    }),
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
