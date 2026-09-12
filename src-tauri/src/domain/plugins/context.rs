use crate::domain::{serde_json, ActionSpec, DomainError, DomainPlugin};

pub struct ContextPlugin;

impl DomainPlugin for ContextPlugin {
    fn name(&self) -> &str {
        "context"
    }
    fn description(&self) -> &str {
        "上下文管理：窗口、策略、分页KV"
    }

    fn actions(&self) -> Vec<ActionSpec> {
        vec![
            ActionSpec {
                name: "create_window".into(),
                description: "创建上下文窗口".into(),
                params: vec![],
                returns: "Value".into(),
            },
            ActionSpec {
                name: "add_item".into(),
                description: "添加上下文项".into(),
                params: vec![],
                returns: "Value".into(),
            },
            ActionSpec {
                name: "get_context".into(),
                description: "获取上下文".into(),
                params: vec![],
                returns: "Value".into(),
            },
            ActionSpec {
                name: "switch_strategy".into(),
                description: "切换上下文策略".into(),
                params: vec![],
                returns: "Value".into(),
            },
            ActionSpec {
                name: "get_stats".into(),
                description: "获取上下文统计".into(),
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
            "create_window" => {
                let session_id =
                    args.get("session_id")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| DomainError {
                            code: "INVALID_ARGS".into(),
                            message: "missing 'session_id'".into(),
                            recoverable: true,
                        })?;
                // 调用 neotrix-core 的 ContextManager
                Ok(serde_json::json!({ "window_id": format!("window_{}", session_id) }))
            }
            "add_item" => {
                let window_id =
                    args.get("window_id")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| DomainError {
                            code: "INVALID_ARGS".into(),
                            message: "missing 'window_id'".into(),
                            recoverable: true,
                        })?;
                let content = args.get("content").and_then(|v| v.as_str()).unwrap_or("");
                Ok(serde_json::json!({ "ok": true, "window_id": window_id }))
            }
            "get_context" => {
                let window_id =
                    args.get("window_id")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| DomainError {
                            code: "INVALID_ARGS".into(),
                            message: "missing 'window_id'".into(),
                            recoverable: true,
                        })?;
                Ok(serde_json::json!({ "items": [], "token_count": 0 }))
            }
            "switch_strategy" => {
                let strategy = args
                    .get("strategy")
                    .and_then(|v| v.as_str())
                    .unwrap_or("compaction");
                Ok(serde_json::json!({ "ok": true, "strategy": strategy }))
            }
            "get_stats" => {
                Ok(serde_json::json!({ "total_windows": 0, "total_items": 0, "total_tokens": 0 }))
            }
            _ => Err(DomainError {
                code: "UNKNOWN_ACTION".into(),
                message: format!("Unknown action: {}", action),
                recoverable: true,
            }),
        }
    }
}
