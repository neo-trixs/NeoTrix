use crate::domain::{DomainPlugin, ActionSpec, DomainError, serde_json};

pub struct ChatPlugin;

impl DomainPlugin for ChatPlugin {
    fn name(&self) -> &str { "chat" }
    fn description(&self) -> &str { "对话管理：消息发送、流式、停止、历史" }

    fn actions(&self) -> Vec<ActionSpec> {
        vec![
            ActionSpec { name: "send".into(), description: "发送消息".into(), params: vec![], returns: "Value".into() },
            ActionSpec { name: "stop".into(), description: "停止生成".into(), params: vec![], returns: "Value".into() },
            ActionSpec { name: "history".into(), description: "获取历史".into(), params: vec![], returns: "Value".into() },
            ActionSpec { name: "compact".into(), description: "压缩上下文".into(), params: vec![], returns: "Value".into() },
            ActionSpec { name: "export".into(), description: "导出会话".into(), params: vec![], returns: "Value".into() },
        ]
    }

    fn call(&self, action: &str, _args: serde_json::Value) -> Result<serde_json::Value, DomainError> {
        match action {
            "send" | "stop" | "history" | "compact" | "export" => {
                Ok(serde_json::json!({ "ok": true, "stub": true }))
            }
            _ => Err(DomainError { code: "UNKNOWN_ACTION".into(), message: format!("Unknown action: {}", action), recoverable: true }),
        }
    }
}
