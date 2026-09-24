use crate::domain::app_handle::get_app_handle;
use crate::domain::registry::DomainRegistry;
use crate::domain::{serde_json, ActionSpec, DomainError, DomainPlugin};
use async_trait::async_trait;
use neotrix::l5_cognition::consciousness_core::{
    dispatch::ConsciousTask,
    external_closure::{ExternalClosureConfig, SolutionExecutor},
};
use neotrix::l5_cognition::consciousness_core::core as consciousness_core;
use std::sync::Arc;
use tauri::Emitter;

/// GatewayV2 LLM 执行器 — 实现 consciousness core 的 SolutionExecutor trait
/// 通过 domain_call 统一调用，而非直接访问 GatewayV2
struct GatewayExecutor {
    registry: std::sync::Arc<tokio::sync::RwLock<DomainRegistry>>,
}

#[async_trait]
impl SolutionExecutor for GatewayExecutor {
    async fn execute(&self, task: &ConsciousTask) -> Result<String, String> {
        let system_prompt = format!(
            "你是 NeoTrix 意识核心的任务执行器。当前任务: {} (域: {}, 能力: {})\n\
             请直接执行此任务并返回结果。",
            task.summary, task.domain, task.capability_tag
        );

        let request = serde_json::json!({
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": task.summary}
            ],
            "temperature": 0.7,
            "max_tokens": crate::constants::DEFAULT_MAX_LLM_TOKENS,
        });

        // 新 trait 已是 async，直接 await（旧同步 attempt + block_in_place 已删除）。
        let registry = self.registry.read().await;
        match registry.call_async("agent", "complete", request).await {
            Ok(response) => {
                let content = response["content"].as_str().unwrap_or("");
                Ok(content.to_string())
            }
            Err(e) => Err(e.to_string()),
        }
    }
}

pub struct ChatPlugin {
    db_pool: Arc<crate::db_pool::DbPool>,
    registry: std::sync::Arc<tokio::sync::RwLock<DomainRegistry>>,
}

impl ChatPlugin {
    pub fn new(
        db_pool: Arc<crate::db_pool::DbPool>,
        registry: std::sync::Arc<tokio::sync::RwLock<DomainRegistry>>,
    ) -> Self {
        if let Err(e) = db_pool.init_schema(
            "CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                messages TEXT NOT NULL DEFAULT '[]',
                project TEXT NOT NULL DEFAULT '',
                sort_order INTEGER NOT NULL DEFAULT 0
            );",
        ) {
            tracing::warn!("schema init: {e}");
        }
        Self { db_pool, registry }
    }

    fn get_messages(&self, session_id: &str) -> Result<Vec<serde_json::Value>, DomainError> {
        let conn = self
            .db_pool
            .get()
            .map_err(|e| DomainError::from(anyhow::Error::from(e).context("acquiring DB connection for get_messages")))?;
        let messages_json: String = conn
            .query_row(
                "SELECT messages FROM sessions WHERE id = ?1",
                [session_id],
                |row| row.get(0),
            )
            .map_err(|e| DomainError::from(anyhow::Error::from(e).context(format!("querying messages for session {}", session_id))))?;

        serde_json::from_str(&messages_json).map_err(|e| DomainError::from(anyhow::Error::from(e).context("parsing session messages JSON")))
    }

    fn add_message(&self, session_id: &str, message: serde_json::Value) -> Result<(), DomainError> {
        let conn = self
            .db_pool
            .get()
            .map_err(|e| DomainError::from(anyhow::Error::from(e).context("acquiring DB connection for add_message")))?;

        let messages_json: String = conn
            .query_row(
                "SELECT messages FROM sessions WHERE id = ?1",
                [session_id],
                |row| row.get(0),
            )
            .map_err(|e| DomainError::from(anyhow::Error::from(e).context(format!("querying messages for session {} in add_message", session_id))))?;

        let mut messages: Vec<serde_json::Value> =
            serde_json::from_str(&messages_json).map_err(|e| DomainError::from(anyhow::Error::from(e).context("parsing messages JSON for append")))?;

        messages.push(message);

        let updated_json = serde_json::to_string(&messages).map_err(|e| DomainError::from(anyhow::Error::from(e).context("serializing updated messages")))?;

        conn.execute(
            "UPDATE sessions SET messages = ?1, updated_at = ?2 WHERE id = ?3",
            rusqlite::params![updated_json, chrono::Utc::now().timestamp(), session_id],
        )
        .map_err(|e| DomainError::from(anyhow::Error::from(e).context(format!("saving messages for session {}", session_id))))?;

        Ok(())
    }

    #[tracing::instrument(skip(self, content), fields(content_len = content.len()))]
    async fn call_llm(&self, content: &str) -> Result<String, DomainError> {
        // 发射流开始事件
        if let Some(app) = get_app_handle() {
            if let Err(e) = app.emit("neotrix_stream_start", "") {
                tracing::trace!("emit stream_start: {e}");
            }
        }

        // Emit reasoning event
        if let Some(app) = get_app_handle() {
            if let Err(e) = app.emit(
                "neotrix_stream_reasoning",
                serde_json::json!({
                    "text": format!("Starting reasoning for task: {}", content),
                }),
            ) {
                tracing::warn!("Failed to emit reasoning event: {}", e);
            }
        }

        // 使用 domain_call 统一路由
        let executor = GatewayExecutor {
            registry: self.registry.clone(),
        };

        let config = ExternalClosureConfig {
            max_attempts: 3,
            timeout_secs: 120,
        };

        // Emit tool event
        if let Some(app) = get_app_handle() {
            if let Err(e) = app.emit(
                "neotrix_stream_tool",
                serde_json::json!({
                    "tool_name": "consciousness_task_loop",
                    "status": "started",
                }),
            ) {
                tracing::warn!("Failed to emit tool event: {}", e);
            }
        }

        // 进程内单例闭环入口（内部处理 CORE 锁，锁毒化时返回默认空报告）。
        let report = consciousness_core::execute_task_loop(content, &executor, &config);

        // Emit tool completion event
        if let Some(app) = get_app_handle() {
            if let Err(e) = app.emit(
                "neotrix_stream_tool",
                serde_json::json!({
                    "tool_name": "consciousness_task_loop",
                    "status": "completed",
                }),
            ) {
                tracing::warn!("Failed to emit tool completion event: {}", e);
            }
        }

        // Emit reasoning event for task decomposition
        if let Some(app) = get_app_handle() {
            if let Err(e) = app.emit(
                "neotrix_stream_reasoning",
                serde_json::json!({
                    "text": format!("Task decomposed into {} subtasks", report.allocations.len()),
                }),
            ) {
                tracing::warn!("Failed to emit reasoning event: {}", e);
            }
        }

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
        if let Some(app) = get_app_handle() {
            if let Err(e) = app.emit("neotrix_stream_token", combined.clone()) {
                tracing::trace!("emit stream_token: {e}");
            }
            if let Err(e) = app.emit("neotrix_stream_end", combined.clone()) {
                tracing::trace!("emit stream_end: {e}");
            }
            if let Err(e) = app.emit(
                "neotrix_stream_done",
                serde_json::json!({
                    "cancelled": false,
                    "elapsed_ms": 0,
                    "content": combined,
                    "tasks_decomposed": report.allocations.len(),
                    "internal_executed": report.internal_count,
                    "external_gaps": report.external_gap_count,
                }),
            ) {
                tracing::trace!("emit stream_done: {e}");
            }
        }

        Ok(combined)
    }

    /// 流式 LLM 调用：通过 domain_call 统一调用获取流式响应
    /// mpsc::Receiver，逐 token emit neotrix_stream_token 事件，返回完整内容。
    #[tracing::instrument(skip(self, content), fields(content_len = content.len()))]
    async fn call_llm_stream(&self, content: &str) -> Result<String, DomainError> {
        if let Some(app) = get_app_handle() {
            if let Err(e) = app.emit("neotrix_stream_start", "") {
                tracing::trace!("emit stream_start: {e}");
            }
        }

        // Emit tool event
        if let Some(app) = get_app_handle() {
            if let Err(e) = app.emit(
                "neotrix_stream_tool",
                serde_json::json!({
                    "tool_name": "llm_stream",
                    "status": "started",
                }),
            ) {
                tracing::warn!("Failed to emit tool event: {}", e);
            }
        }

        let request = serde_json::json!({
            "messages": [
                {"role": "system", "content": "你是一个有帮助的AI助手。请用中文回答。"},
                {"role": "user", "content": content}
            ],
            "temperature": 0.7,
            "max_tokens": crate::constants::DEFAULT_MAX_LLM_TOKENS,
            "stream": true,
        });

        let mut full_content = String::new();

        let response = {
                let registry = self.registry.read().await;
                registry.call_async("agent", "stream", request).await
            }
            .map_err(|e| DomainError::from(anyhow::Error::from(e).context("initiating LLM stream request")))?;

        let token = response["content"].as_str().unwrap_or("");
        full_content.push_str(token);
        if let Some(app) = get_app_handle() {
            if let Err(e) = app.emit("neotrix_stream_token", token) {
                tracing::trace!("emit stream_token: {e}");
            }
        }

        // 流结束
        if let Some(app) = get_app_handle() {
            if let Err(e) = app.emit("neotrix_stream_end", &full_content) {
                tracing::trace!("emit stream_end: {e}");
            }
            if let Err(e) = app.emit(
                "neotrix_stream_done",
                serde_json::json!({
                    "cancelled": false,
                    "elapsed_ms": 0,
                    "content": full_content,
                    "tasks_decomposed": 0,
                    "internal_executed": 0,
                    "external_gaps": 0,
                }),
            ) {
                tracing::trace!("emit stream_done: {e}");
            }
        }

        // Emit tool completion event
        if let Some(app) = get_app_handle() {
            if let Err(e) = app.emit(
                "neotrix_stream_tool",
                serde_json::json!({
                    "tool_name": "llm_stream",
                    "status": "completed",
                }),
            ) {
                tracing::warn!("Failed to emit tool completion event: {}", e);
            }
        }

        Ok(full_content)
    }
}

#[async_trait]
impl DomainPlugin for ChatPlugin {
    fn name(&self) -> &str {
        "chat"
    }
    fn description(&self) -> &str {
        "对话管理：消息发送、流式、停止、历史"
    }

    fn actions(&self) -> Vec<ActionSpec> {
        vec![
            ActionSpec {
                name: "send_message_stream".into(),
                description: "发送消息（流式）".into(),
                params: vec![],
                returns: "Value".into(),
            },
            ActionSpec {
                name: "stop_stream".into(),
                description: "停止流式生成".into(),
                params: vec![],
                returns: "Value".into(),
            },
            ActionSpec {
                name: "get_session_messages".into(),
                description: "获取会话消息".into(),
                params: vec![],
                returns: "Value".into(),
            },
            ActionSpec {
                name: "send".into(),
                description: "发送消息".into(),
                params: vec![],
                returns: "Value".into(),
            },
            ActionSpec {
                name: "send_stream".into(),
                description: "发送消息（真实流式）".into(),
                params: vec![],
                returns: "Value".into(),
            },
            ActionSpec {
                name: "stop".into(),
                description: "停止生成".into(),
                params: vec![],
                returns: "Value".into(),
            },
            ActionSpec {
                name: "history".into(),
                description: "获取历史".into(),
                params: vec![],
                returns: "Value".into(),
            },
            ActionSpec {
                name: "compact".into(),
                description: "压缩上下文".into(),
                params: vec![],
                returns: "Value".into(),
            },
            ActionSpec {
                name: "export".into(),
                description: "导出会话".into(),
                params: vec![],
                returns: "Value".into(),
            },
            ActionSpec {
                name: "clear".into(),
                description: "清空会话".into(),
                params: vec![],
                returns: "Value".into(),
            },
            ActionSpec {
                name: "regenerate".into(),
                description: "重新生成".into(),
                params: vec![],
                returns: "Value".into(),
            },
            ActionSpec {
                name: "side_chat_get".into(),
                description: "获取副对话".into(),
                params: vec![],
                returns: "Value".into(),
            },
            ActionSpec {
                name: "side_chat_send".into(),
                description: "发送副对话消息".into(),
                params: vec![],
                returns: "Value".into(),
            },
            ActionSpec {
                name: "edit_message".into(),
                description: "编辑消息".into(),
                params: vec![],
                returns: "Value".into(),
            },
            ActionSpec {
                name: "delete_message".into(),
                description: "删除消息".into(),
                params: vec![],
                returns: "Value".into(),
            },
            ActionSpec {
                name: "provider_health".into(),
                description: "Provider 健康检查".into(),
                params: vec![],
                returns: "Value".into(),
            },
        ]
    }

    #[tracing::instrument(skip(self, args), fields(action = %action))]
    async fn call(
        &self,
        action: &str,
        args: serde_json::Value,
    ) -> Result<serde_json::Value, DomainError> {
        match action {
            "send_message_stream" | "send" => {
                let content = args
                    .get("content")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 content 参数".into(),
                        recoverable: true,
                    })?;
                let session_id = args
                    .get("session_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("default");

                // 保存用户消息
                let user_msg = serde_json::json!({
                    "id": format!("msg-{}", chrono::Utc::now().timestamp_millis()),
                    "content": content,
                    "role": "user",
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                });
                self.add_message(session_id, user_msg)
                    .map_err(|e| DomainError::from(anyhow::Error::from(e).context("saving user message")))?;

                // 调用本地 LLM
                let assistant_content = self.call_llm(content).await
                    .map_err(|e| DomainError::from(anyhow::Error::from(e).context("calling LLM for completion")))?;

                // 保存助手消息
                let assistant_msg = serde_json::json!({
                    "id": format!("msg-{}", chrono::Utc::now().timestamp_millis()),
                    "content": assistant_content,
                    "role": "assistant",
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                });
                self.add_message(session_id, assistant_msg.clone())
                    .map_err(|e| DomainError::from(anyhow::Error::from(e).context("saving assistant response")))?;

                // 返回给前端（字符串格式，适配 Promise<string>）
                Ok(serde_json::json!(assistant_content))
            }
            "send_stream" => {
                let content = args
                    .get("content")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError {
                        code: "INVALID_ARGS".into(),
                        message: "缺少 content 参数".into(),
                        recoverable: true,
                    })?;
                let session_id = args
                    .get("session_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("default");

                // 保存用户消息
                let user_msg = serde_json::json!({
                    "id": format!("msg-{}", chrono::Utc::now().timestamp_millis()),
                    "content": content,
                    "role": "user",
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                });
                self.add_message(session_id, user_msg)
                    .map_err(|e| DomainError::from(anyhow::Error::from(e).context("saving user message for stream")))?;

                // 流式调用 LLM
                let assistant_content = self.call_llm_stream(content).await
                    .map_err(|e| DomainError::from(anyhow::Error::from(e).context("calling LLM stream")))?;

                // 保存助手消息
                let assistant_msg = serde_json::json!({
                    "id": format!("msg-{}", chrono::Utc::now().timestamp_millis()),
                    "content": assistant_content,
                    "role": "assistant",
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                });
                self.add_message(session_id, assistant_msg)
                    .map_err(|e| DomainError::from(anyhow::Error::from(e).context("saving stream assistant response")))?;

                Ok(serde_json::json!(assistant_content))
            }
            "stop_stream" | "stop" => {
                if let Some(app) = get_app_handle() {
                    if let Err(e) = app.emit("neotrix_stream_cancel", "") {
                        tracing::trace!("emit stream_cancel: {e}");
                    }
                }
                Ok(serde_json::json!({ "ok": true }))
            }
            "get_session_messages" | "history" => {
                let session_id = args
                    .get("session_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("default");
                let messages = self.get_messages(session_id)
                    .map_err(|e| DomainError::from(anyhow::Error::from(e).context("fetching session history")))?;
                Ok(serde_json::json!({ "ok": true, "messages": messages }))
            }
            "compact" => Ok(serde_json::json!({ "ok": true })),
            "export" => {
                let session_id = args
                    .get("session_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("default");
                let messages = self.get_messages(session_id)
                    .map_err(|e| DomainError::from(anyhow::Error::from(e).context("fetching messages for export")))?;
                let md = messages
                    .iter()
                    .filter_map(|m| {
                        let role = m.get("role")?.as_str()?;
                        let content = m.get("content")?.as_str()?;
                        Some(format!("**{}**: {}\n\n", role, content))
                    })
                    .collect::<String>();
                Ok(serde_json::json!({ "ok": true, "markdown": md }))
            }
            "clear" => {
                let session_id = args
                    .get("session_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("default");
                let conn = self
                    .db_pool
                    .get()
                    .map_err(|e| DomainError::from(anyhow::Error::from(e).context("acquiring DB connection for clear")))?;
                conn.execute(
                    "UPDATE sessions SET messages = '[]', updated_at = ?1 WHERE id = ?2",
                    rusqlite::params![chrono::Utc::now().timestamp(), session_id],
                )
                .map_err(|e| DomainError::from(anyhow::Error::from(e).context(format!("clearing messages for session {}", session_id))))?;
                Ok(serde_json::json!({ "ok": true }))
            }
            "regenerate" => Ok(serde_json::json!({ "ok": true })),
            "provider_health" => {
                let status_list = {
                        let registry = self.registry.read().await;
                        registry
                            .call_async("agent", "provider_status", serde_json::json!({}))
                            .await
                    }
                    .unwrap_or_else(|_| serde_json::json!([]));

                let providers = status_list
                    .as_array()
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|status| {
                                let name = status.get("name")?.as_str()?.to_string();
                                let healthy = status.get("available")?.as_bool()?;
                                let score = status
                                    .get("composite_score")
                                    .and_then(|v| v.as_str())
                                    .and_then(|s| s.parse::<f64>().ok())
                                    .unwrap_or(0.0);
                                Some(serde_json::json!({
                                    "name": name,
                                    "healthy": healthy,
                                    "score": score,
                                }))
                            })
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();

                Ok(serde_json::json!({ "providers": providers }))
            }
            "side_chat_get" | "side_chat_send" => {
                Ok(serde_json::json!({ "ok": true, "messages": [] }))
            }
            "edit_message" => {
                let session_id = args
                    .get("session_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("default");
                let index = args.get("index").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                let content = args.get("content").and_then(|v| v.as_str()).unwrap_or("");
                let mut messages = self.get_messages(session_id)
                    .map_err(|e| DomainError::from(anyhow::Error::from(e).context("fetching messages for edit")))?;
                if index < messages.len() {
                    messages[index]["content"] = serde_json::json!(content);
                    let json = serde_json::to_string(&messages).unwrap_or_default();
                    let conn = self
                        .db_pool
                        .get()
                        .map_err(|e| DomainError::from(anyhow::Error::from(e).context("acquiring DB connection for edit")))?;
                    conn.execute(
                        "UPDATE sessions SET messages = ?1, updated_at = ?2 WHERE id = ?3",
                        rusqlite::params![json, chrono::Utc::now().timestamp(), session_id],
                    )
                    .ok();
                }
                Ok(serde_json::json!({ "ok": true, "index": index }))
            }
            "delete_message" => {
                let session_id = args
                    .get("session_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("default");
                let index = args.get("index").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                let mut messages = self.get_messages(session_id)
                    .map_err(|e| DomainError::from(anyhow::Error::from(e).context("fetching messages for delete")))?;
                if index < messages.len() {
                    messages.remove(index);
                    let json = serde_json::to_string(&messages).unwrap_or_default();
                    let conn = self
                        .db_pool
                        .get()
                        .map_err(|e| DomainError::from(anyhow::Error::from(e).context("acquiring DB connection for delete")))?;
                    conn.execute(
                        "UPDATE sessions SET messages = ?1, updated_at = ?2 WHERE id = ?3",
                        rusqlite::params![json, chrono::Utc::now().timestamp(), session_id],
                    )
                    .ok();
                }
                Ok(serde_json::json!({ "ok": true, "index": index }))
            }
            _ => Err(DomainError {
                code: "UNKNOWN_ACTION".into(),
                message: format!("Unknown action: {}", action),
                recoverable: true,
            }),
        }
    }
}
