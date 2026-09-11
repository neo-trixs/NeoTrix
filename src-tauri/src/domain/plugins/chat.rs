use crate::domain::{DomainPlugin, ActionSpec, DomainError, serde_json};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock, Arc};
use rusqlite::Connection;
use tauri::{AppHandle, Emitter};
use neotrix::core::nt_core_consciousness_core::{
    ConsciousTask, SolutionExecutor, AttemptOutcome,
    ExternalClosureConfig, CORE,
};
use neotrix::l1_action::nt_io::nt_io_provider::gateway::GatewayV2;
use neotrix::l1_action::nt_io::nt_io_provider::types::{LlmRequest, LlmResponse, LlmError, Usage, FinishReason};
use neotrix::l1_action::nt_io::nt_io_provider::factory::create_gateway_async;

static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();
static GATEWAY: OnceLock<Arc<GatewayV2>> = OnceLock::new();

pub fn set_app_handle(app: AppHandle) {
    let _ = APP_HANDLE.set(app);
}

/// 获取或初始化 GatewayV2
async fn get_gateway() -> &'static GatewayV2 {
    GATEWAY.get_or_init(|| {
        // 同步创建一个基础 gateway，异步初始化在首次调用时完成
        Arc::new(GatewayV2::new())
    })
}

/// GatewayV2 LLM 执行器 — 实现 consciousness core 的 SolutionExecutor trait
struct GatewayExecutor {
    gateway: Arc<GatewayV2>,
}

impl SolutionExecutor for GatewayExecutor {
    fn attempt(&self, task: &ConsciousTask, grounding: &str, _attempt_no: u32) -> AttemptOutcome {
        let system_prompt = format!(
            "你是 NeoTrix 意识核心的任务执行器。当前任务: {} (域: {}, 能力: {})\n\
             上下文: {}\n\
             请直接执行此任务并返回结果。",
            task.summary, task.domain, task.capability_tag, grounding
        );

        let request = LlmRequest {
            model: String::new(), // Gateway 会自动选择
            messages: vec![
                neotrix::l1_action::nt_io::nt_io_provider::types::ChatMessage {
                    role: "system".to_string(),
                    content: system_prompt,
                },
                neotrix::l1_action::nt_io::nt_io_provider::types::ChatMessage {
                    role: "user".to_string(),
                    content: task.summary.clone(),
                },
            ],
            temperature: Some(0.7),
            max_tokens: Some(2048),
            stream: false,
            tools: None,
        };

        // 使用 tokio runtime 执行异步操作
        let rt = tokio::runtime::Handle::current();
        match rt.block_on(self.gateway.complete_raw(&request)) {
            Ok(response) => AttemptOutcome::Solved {
                solution: response.content,
                tokens_used: response.usage.total_tokens,
            },
            Err(e) => AttemptOutcome::Failed {
                error: e.to_string(),
                tokens_used: 0,
            },
        }
    }
}

pub struct ChatPlugin {
    db_path: PathBuf,
    _db: Mutex<()>,
}

impl ChatPlugin {
    pub fn new() -> Self {
        let db_path = dirs::home_dir()
            .map(|h| h.join(".neotrix").join("desktop.db"))
            .unwrap_or_else(|| PathBuf::from(".neotrix/desktop.db"));
        Self {
            db_path,
            _db: Mutex::new(()),
        }
    }

    fn open_db(&self) -> Result<Connection, DomainError> {
        let conn = Connection::open(&self.db_path)
            .map_err(|e| DomainError { code: "DB_ERROR".into(), message: format!("打开数据库失败: {}", e), recoverable: true })?;
        conn.pragma_update(None, "journal_mode", "WAL")
            .map_err(|e| DomainError { code: "DB_ERROR".into(), message: format!("启用 WAL 失败: {}", e), recoverable: true })?;
        let _ = conn.busy_timeout(std::time::Duration::from_secs(5));
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                messages TEXT NOT NULL DEFAULT '[]',
                project TEXT NOT NULL DEFAULT '',
                sort_order INTEGER NOT NULL DEFAULT 0
            );",
        )
        .map_err(|e| DomainError { code: "DB_ERROR".into(), message: format!("初始化表失败: {}", e), recoverable: true })?;
        Ok(conn)
    }

    fn get_messages(&self, session_id: &str) -> Result<Vec<serde_json::Value>, DomainError> {
        let conn = self.open_db()?;
        let messages_json: String = conn
            .query_row(
                "SELECT messages FROM sessions WHERE id = ?1",
                [session_id],
                |row| row.get(0),
            )
            .map_err(|e| DomainError { code: "NOT_FOUND".into(), message: format!("会话不存在: {}", e), recoverable: false })?;
        
        serde_json::from_str(&messages_json)
            .map_err(|e| DomainError { code: "PARSE_ERROR".into(), message: format!("解析消息失败: {}", e), recoverable: true })
    }

    fn add_message(&self, session_id: &str, message: serde_json::Value) -> Result<(), DomainError> {
        let conn = self.open_db()?;
        let mut messages = self.get_messages(session_id)?;
        messages.push(message);
        
        let messages_json = serde_json::to_string(&messages)
            .map_err(|e| DomainError { code: "SERIALIZE_ERROR".into(), message: format!("序列化消息失败: {}", e), recoverable: true })?;
        
        conn.execute(
            "UPDATE sessions SET messages = ?1, updated_at = ?2 WHERE id = ?3",
            rusqlite::params![messages_json, chrono::Utc::now().timestamp(), session_id],
        )
        .map_err(|e| DomainError { code: "DB_ERROR".into(), message: format!("保存消息失败: {}", e), recoverable: true })?;
        
        Ok(())
    }

    fn call_llm(&self, content: &str) -> Result<String, DomainError> {
        // 发射流开始事件
        if let Some(app) = APP_HANDLE.get() {
            let _ = app.emit("neocodex_stream_start", "");
        }

        // 使用 GatewayV2 统一路由
        let executor = {
            let rt = tokio::runtime::Handle::current();
            let gateway = rt.block_on(async {
                Arc::new(create_gateway_async().await)
            });
            GatewayExecutor { gateway }
        };
        
        let config = ExternalClosureConfig {
            max_attempts: 3,
            token_budget: 4096,
            max_llm_tokens: 2048,
            acquire_knowledge: false,
        };

        let report = {
            let mut core = CORE.write().map_err(|e| DomainError {
                code: "CORE_LOCK".into(),
                message: format!("获取意识核心锁失败: {}", e),
                recoverable: true,
            })?;
            core.execute_task_loop(content, &executor, &config)
        };

        // 聚合所有子任务的结果
        let mut results = Vec::new();
        for result in &report.internal_results {
            results.push(result.output.clone());
        }
        for gap in &report.external_gaps {
            results.push(format!("[外部缺口] {}", gap));
        }

        let combined = if results.is_empty() {
            "[意识核心未返回结果]".to_string()
        } else {
            results.join("\n\n")
        };

        // 发射流式 token 事件
        if let Some(app) = APP_HANDLE.get() {
            let _ = app.emit("neocodex_stream_token", combined.clone());
            let _ = app.emit("neocodex_stream_end", combined.clone());
            let _ = app.emit("neocodex_stream_done", serde_json::json!({
                "cancelled": false,
                "elapsed_ms": 0,
                "content": combined,
                "tasks_decomposed": report.allocations.len(),
                "internal_executed": report.internal_count,
                "external_gaps": report.external_gap_count,
            }));
        }

        Ok(combined)
    }
}

impl DomainPlugin for ChatPlugin {
    fn name(&self) -> &str { "chat" }
    fn description(&self) -> &str { "对话管理：消息发送、流式、停止、历史" }

    fn actions(&self) -> Vec<ActionSpec> {
        vec![
            ActionSpec { name: "send_message_stream".into(), description: "发送消息（流式）".into(), params: vec![], returns: "Value".into() },
            ActionSpec { name: "stop_stream".into(), description: "停止流式生成".into(), params: vec![], returns: "Value".into() },
            ActionSpec { name: "get_session_messages".into(), description: "获取会话消息".into(), params: vec![], returns: "Value".into() },
            ActionSpec { name: "send".into(), description: "发送消息".into(), params: vec![], returns: "Value".into() },
            ActionSpec { name: "stop".into(), description: "停止生成".into(), params: vec![], returns: "Value".into() },
            ActionSpec { name: "history".into(), description: "获取历史".into(), params: vec![], returns: "Value".into() },
            ActionSpec { name: "compact".into(), description: "压缩上下文".into(), params: vec![], returns: "Value".into() },
            ActionSpec { name: "export".into(), description: "导出会话".into(), params: vec![], returns: "Value".into() },
            ActionSpec { name: "clear".into(), description: "清空会话".into(), params: vec![], returns: "Value".into() },
            ActionSpec { name: "regenerate".into(), description: "重新生成".into(), params: vec![], returns: "Value".into() },
            ActionSpec { name: "side_chat_get".into(), description: "获取副对话".into(), params: vec![], returns: "Value".into() },
            ActionSpec { name: "side_chat_send".into(), description: "发送副对话消息".into(), params: vec![], returns: "Value".into() },
            ActionSpec { name: "edit_message".into(), description: "编辑消息".into(), params: vec![], returns: "Value".into() },
            ActionSpec { name: "delete_message".into(), description: "删除消息".into(), params: vec![], returns: "Value".into() },
        ]
    }

    fn call(&self, action: &str, args: serde_json::Value) -> Result<serde_json::Value, DomainError> {
        match action {
            "send_message_stream" | "send" => {
                let content = args.get("content")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError { code: "INVALID_ARGS".into(), message: "缺少 content 参数".into(), recoverable: true })?;
                let session_id = args.get("session_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("default");
                
                // 保存用户消息
                let user_msg = serde_json::json!({
                    "id": format!("msg-{}", chrono::Utc::now().timestamp_millis()),
                    "content": content,
                    "role": "user",
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                });
                self.add_message(session_id, user_msg)?;
                
                // 调用本地 LLM
                let assistant_content = self.call_llm(content)?;
                
                // 保存助手消息
                let assistant_msg = serde_json::json!({
                    "id": format!("msg-{}", chrono::Utc::now().timestamp_millis()),
                    "content": assistant_content,
                    "role": "assistant",
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                });
                self.add_message(session_id, assistant_msg.clone())?;
                
                // 返回给前端（字符串格式，适配 Promise<string>）
                Ok(serde_json::json!(assistant_content))
            }
            "stop_stream" | "stop" => {
                if let Some(app) = APP_HANDLE.get() {
                    let _ = app.emit("neocodex_stream_cancel", "");
                }
                Ok(serde_json::json!({ "ok": true }))
            }
            "get_session_messages" | "history" => {
                let session_id = args.get("session_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("default");
                let messages = self.get_messages(session_id)?;
                Ok(serde_json::json!({ "ok": true, "messages": messages }))
            }
            "compact" => {
                Ok(serde_json::json!({ "ok": true }))
            }
            "export" => {
                let session_id = args.get("session_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("default");
                let messages = self.get_messages(session_id)?;
                let md = messages.iter()
                    .filter_map(|m| {
                        let role = m.get("role")?.as_str()?;
                        let content = m.get("content")?.as_str()?;
                        Some(format!("**{}**: {}\n\n", role, content))
                    })
                    .collect::<String>();
                Ok(serde_json::json!({ "ok": true, "markdown": md }))
            }
            "clear" => {
                let session_id = args.get("session_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("default");
                let conn = self.open_db()?;
                conn.execute(
                    "UPDATE sessions SET messages = '[]', updated_at = ?1 WHERE id = ?2",
                    rusqlite::params![chrono::Utc::now().timestamp(), session_id],
                )
                .map_err(|e| DomainError { code: "DB_ERROR".into(), message: format!("清空失败: {}", e), recoverable: true })?;
                Ok(serde_json::json!({ "ok": true }))
            }
            "regenerate" => {
                Ok(serde_json::json!({ "ok": true }))
            }
            "side_chat_get" | "side_chat_send" => {
                Ok(serde_json::json!({ "ok": true, "messages": [] }))
            }
            "edit_message" => {
                let session_id = args.get("session_id").and_then(|v| v.as_str()).unwrap_or("default");
                let index = args.get("index").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                let content = args.get("content").and_then(|v| v.as_str()).unwrap_or("");
                let mut messages = self.get_messages(session_id)?;
                if index < messages.len() {
                    messages[index]["content"] = serde_json::json!(content);
                    let json = serde_json::to_string(&messages).unwrap_or_default();
                    let conn = self.open_db()?;
                    conn.execute(
                        "UPDATE sessions SET messages = ?1, updated_at = ?2 WHERE id = ?3",
                        rusqlite::params![json, chrono::Utc::now().timestamp(), session_id],
                    ).ok();
                }
                Ok(serde_json::json!({ "ok": true, "index": index }))
            }
            "delete_message" => {
                let session_id = args.get("session_id").and_then(|v| v.as_str()).unwrap_or("default");
                let index = args.get("index").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                let mut messages = self.get_messages(session_id)?;
                if index < messages.len() {
                    messages.remove(index);
                    let json = serde_json::to_string(&messages).unwrap_or_default();
                    let conn = self.open_db()?;
                    conn.execute(
                        "UPDATE sessions SET messages = ?1, updated_at = ?2 WHERE id = ?3",
                        rusqlite::params![json, chrono::Utc::now().timestamp(), session_id],
                    ).ok();
                }
                Ok(serde_json::json!({ "ok": true, "index": index }))
            }
            _ => Err(DomainError { code: "UNKNOWN_ACTION".into(), message: format!("Unknown action: {}", action), recoverable: true }),
        }
    }
}
