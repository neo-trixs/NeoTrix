#![forbid(unsafe_code)]
//! Harness 统一网关 Tauri 命令 — 对标 harness/mcp-server 11工具 + openai/codex app-server
//!
//! 单入口 `harness_execute` 复用 ConsciousnessCore 的 CAPABILITY_ROUTES，
//! 前端仅调对话，其余隐藏。线程/审批为高级能力，前端按需调用。

use neotrix::core::nt_core_consciousness_core::{execute_task_loop, process_instruction, ExternalClosureConfig, LlmSolutionExecutor};
use neotrix::neotrix::nt_core_error::NeoTrixError;
use neotrix::neotrix::nt_harness::{HarnessExecuteRequest, HarnessGateway};
use serde_json::{json, Value};
use std::sync::{LazyLock, Mutex};
use tauri::{AppHandle, Emitter};

static GATEWAY: LazyLock<Mutex<HarnessGateway>> = LazyLock::new(|| Mutex::new(HarnessGateway::new()));

/// 统一执行（对话即OS）— 关键词→capability_tag→domain→harness_tool
#[tauri::command]
pub fn harness_execute(instruction: String, capability_tag: Option<String>, project: Option<String>) -> Result<Value, NeoTrixError> {
    let gateway = GATEWAY.lock().map_err(|e| NeoTrixError::Brain(format!("HarnessGateway poisoned: {e}")))?;
    let req = HarnessExecuteRequest { instruction, capability_tag, project, permission_mode: None };
    // 真实执行链路: 复用意识核心单例 process_instruction (进程内 CORE, 离线安全)
    let report = process_instruction(&req.instruction);
    let resp = gateway.execute_real(req, &report);
    serde_json::to_value(&resp).map_err(|e| NeoTrixError::Serde(e.to_string()))
}

/// 重路径真实执行（按需触发，区别于 harness_execute 的轻量路由）—
/// 经 `execute_task_loop` 跑完整闭环: 内置能力网执行 + 外部缺口 LLM 试错求解
/// (LlmSolutionExecutor)。离线/未配置 LLM 时 executor 返回 Failed, 报告仍透明返回
/// (外部缺口标注未解), 不 panic（降级到拆解→分配闭环）。
#[tauri::command]
pub async fn harness_run(
    app: AppHandle,
    instruction: String,
    run_id: Option<String>,
    capability_tag: Option<String>,
    project: Option<String>,
) -> Result<Value, NeoTrixError> {
    let _ = (capability_tag, project);
    // 阶段1: 拆解 + 能力网分配 (离线安全, 无 LLM) — 即时推送, 前端先渲染分配视图
    let alloc_report = process_instruction(&instruction);
    let _ = app.emit(
        "harness-progress",
        json!({ "run_id": run_id, "phase": "allocated", "report": &alloc_report }),
    );
    // 阶段2: 完整闭环 (内置能力网执行 + 外部缺口 LLM 试错求解) — 完成后推送全量报告
    let report = execute_task_loop(&instruction, &LlmSolutionExecutor, &ExternalClosureConfig::frugal());
    let _ = app.emit(
        "harness-progress",
        json!({ "run_id": run_id, "phase": "done", "report": &report }),
    );
    serde_json::to_value(&report).map_err(|e| NeoTrixError::Serde(e.to_string()))
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
