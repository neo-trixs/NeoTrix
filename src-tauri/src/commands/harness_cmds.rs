#![forbid(unsafe_code)]
//! Harness 统一网关 Tauri 命令 — 对标 harness/mcp-server 11工具 + openai/codex app-server
//!
//! 单入口 `harness_execute` 复用 ConsciousnessCore 的 CAPABILITY_ROUTES，
//! 前端仅调对话，其余隐藏。线程/审批为高级能力，前端按需调用。

use neotrix::neotrix::nt_core_error::NeoTrixError;
use neotrix::neotrix::nt_harness::{HarnessExecuteRequest, HarnessGateway};
use serde_json::Value;
use std::sync::{LazyLock, Mutex};

static GATEWAY: LazyLock<Mutex<HarnessGateway>> = LazyLock::new(|| Mutex::new(HarnessGateway::new()));

/// 统一执行（对话即OS）— 关键词→capability_tag→domain→harness_tool
#[tauri::command]
pub fn harness_execute(instruction: String, capability_tag: Option<String>, project: Option<String>) -> Result<Value, NeoTrixError> {
    let gateway = GATEWAY.lock().map_err(|e| NeoTrixError::Brain(format!("HarnessGateway poisoned: {e}")))?;
    let req = HarnessExecuteRequest { instruction, capability_tag, project, permission_mode: None };
    let resp = gateway.execute(req);
    serde_json::to_value(&resp).map_err(|e| NeoTrixError::Serde(e.to_string()))
}

/// 能力标签API地图（前端可审计，调试用；生产隐藏）
#[tauri::command]
pub fn harness_api_map() -> Result<Value, NeoTrixError> {
    let gateway = GATEWAY.lock().map_err(|e| NeoTrixError::Brain(format!("HarnessGateway poisoned: {e}")))?;
    serde_json::to_value(gateway.api_map()).map_err(|e| NeoTrixError::Serde(e.to_string()))
}

/// 11 统一工具目录（对标 harness/mcp-server registry dispatch）
#[tauri::command]
pub fn harness_tool_catalog() -> Result<Value, NeoTrixError> {
    let gateway = GATEWAY.lock().map_err(|e| NeoTrixError::Brain(format!("HarnessGateway poisoned: {e}")))?;
    serde_json::to_value(gateway.tool_catalog()).map_err(|e| NeoTrixError::Serde(e.to_string()))
}

/// 关键词探查（调试：输入片段返回命中 tag）
#[tauri::command]
pub fn harness_resolve(instruction: String) -> Result<Value, NeoTrixError> {
    let gateway = GATEWAY.lock().map_err(|e| NeoTrixError::Brain(format!("HarnessGateway poisoned: {e}")))?;
    let entry = gateway.resolve_instruction(&instruction);
    serde_json::to_value(&entry).map_err(|e| NeoTrixError::Serde(e.to_string()))
}

/// 推理路由状态（对标 grok-0.18 Router Settings）
#[tauri::command]
pub fn harness_router_status() -> Result<Value, NeoTrixError> {
    let gateway = GATEWAY.lock().map_err(|e| NeoTrixError::Brain(format!("HarnessGateway poisoned: {e}")))?;
    serde_json::to_value(&gateway.router.config).map_err(|e| NeoTrixError::Serde(e.to_string()))
}

#[tauri::command]
pub fn harness_router_set_provider(provider: String) -> Result<Value, NeoTrixError> {
    let mut gateway = GATEWAY.lock().map_err(|e| NeoTrixError::Brain(format!("HarnessGateway poisoned: {e}")))?;
    let p = match provider.to_lowercase().as_str() {
        "cursor" => neotrix::neotrix::nt_harness::router::InferenceProvider::Cursor,
        "claude" | "claude_code" => neotrix::neotrix::nt_harness::router::InferenceProvider::ClaudeCode,
        "codex" => neotrix::neotrix::nt_harness::router::InferenceProvider::Codex,
        "openrouter" => neotrix::neotrix::nt_harness::router::InferenceProvider::OpenRouter,
        "local" => neotrix::neotrix::nt_harness::router::InferenceProvider::Local,
        _ => return Err(NeoTrixError::Config(format!("unknown provider: {provider}"))),
    };
    gateway.router.set_default(p);
    serde_json::to_value(&gateway.router.config).map_err(|e| NeoTrixError::Serde(e.to_string()))
}

/// 沙箱状态（对标 harness lite-engine + grok local Docker）
#[tauri::command]
pub fn harness_sandbox_status() -> Result<Value, NeoTrixError> {
    let gateway = GATEWAY.lock().map_err(|e| NeoTrixError::Brain(format!("HarnessGateway poisoned: {e}")))?;
    serde_json::to_value(&gateway.sandbox).map_err(|e| NeoTrixError::Serde(e.to_string()))
}

/// App-Server 线程（对标 openai/codex app-server thread/turn）
#[tauri::command]
pub fn harness_thread_create(project: Option<String>) -> Result<Value, NeoTrixError> {
    let mut gateway = GATEWAY.lock().map_err(|e| NeoTrixError::Brain(format!("HarnessGateway poisoned: {e}")))?;
    let thr = gateway.threads.create_thread(project);
    serde_json::to_value(&thr).map_err(|e| NeoTrixError::Serde(e.to_string()))
}

#[tauri::command]
pub fn harness_thread_list() -> Result<Value, NeoTrixError> {
    let gateway = GATEWAY.lock().map_err(|e| NeoTrixError::Brain(format!("HarnessGateway poisoned: {e}")))?;
    let list: Vec<_> = gateway.threads.list_threads().into_iter().cloned().collect();
    serde_json::to_value(&list).map_err(|e| NeoTrixError::Serde(e.to_string()))
}

#[tauri::command]
pub fn harness_turn_start(thread_id: String, instruction: String) -> Result<Value, NeoTrixError> {
    let mut gateway = GATEWAY.lock().map_err(|e| NeoTrixError::Brain(format!("HarnessGateway poisoned: {e}")))?;
    // 先解析 tag，再开 turn
    let tag = gateway.resolve_instruction(&instruction).map(|e| e.capability_tag).unwrap_or_else(|| "orchestration".into());
    let turn = gateway.threads.start_turn(&thread_id, &instruction, &tag).ok_or_else(|| NeoTrixError::Config(format!("thread not found: {thread_id}")))?;
    serde_json::to_value(&turn).map_err(|e| NeoTrixError::Serde(e.to_string()))
}

#[tauri::command]
pub fn harness_approval_list() -> Result<Value, NeoTrixError> {
    let gateway = GATEWAY.lock().map_err(|e| NeoTrixError::Brain(format!("HarnessGateway poisoned: {e}")))?;
    let list: Vec<_> = gateway.threads.pending_approvals().into_iter().cloned().collect();
    serde_json::to_value(&list).map_err(|e| NeoTrixError::Serde(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_execute() {
        let r = harness_execute("帮我合并表格".into(), None, None).unwrap();
        assert_eq!(r["capability_tag"], "xlsx_consolidation");
    }
    #[test]
    fn test_api_map() {
        let v = harness_api_map().unwrap();
        assert!(v.as_array().unwrap().len() >= 20);
    }
    #[test]
    fn test_thread_flow() {
        let thr = harness_thread_create(Some("demo".into())).unwrap();
        let thr_id = thr["id"].as_str().unwrap().to_string();
        let turn = harness_turn_start(thr_id, "检索知识库".into()).unwrap();
        assert_eq!(turn["capability_tag"], "hybrid_retrieval");
    }
}
