//! ntcode streaming — 后台流式任务 + Tauri event 推送。
//!
//! 工作线程调用 neotrix-core 的模型 CLI 流式接口，
//! 通过 `app.emit("ntcode://chunk", ...)` 实时推送到前端。

use crate::ntcode::{ChatMessage, NtcodeSession, Role, SessionStatus};
use neotrix::l1_action::nt_model_cli::NtModelCliAsk;
use neotrix::l1_action::nt_free_pool::NtFreePoolAsk;
use neotrix::neotrix::nt_crystal_core::NtLlmAsk;
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

/// 流式推送的事件 payload。
#[derive(Clone, serde::Serialize)]
pub struct ChunkEvent {
    pub content: String,
}

#[derive(Clone, serde::Serialize)]
pub struct DoneEvent {
    pub summary: String,
    pub model: String,
}

#[derive(Clone, serde::Serialize)]
pub struct ErrorEvent {
    pub message: String,
}

#[derive(Clone, serde::Serialize)]
pub struct StatusEvent {
    pub status: String,
}

/// 启动流式生成任务。
///
/// 在后台 tokio task 中：
/// 1. 从池中选模型（定点 or 轮转）
/// 2. 调用 `ask_stream` 获取流式响应
/// 3. 每个 chunk 通过 `app.emit("ntcode://chunk", ...)` 推送
/// 4. 完成后推 `ntcode://done`
/// 5. 更新 session 状态
pub async fn spawn_streaming(
    app: AppHandle,
    session: Arc<tokio::sync::RwLock<NtcodeSession>>,
    pool: Arc<NtFreePoolAsk>,
    message: String,
) {
    // 标记开始流式
    {
        let mut s = session.write().await;
        s.push_user(&message);
        s.status = SessionStatus::Streaming;
        let _ = app.emit("ntcode://status", StatusEvent { status: "streaming".into() });
    }

    let cancel = Arc::new(tokio::sync::Notify::new());
    {
        let mut s = session.write().await;
        s.cancel = Some(cancel.clone());
    }

    let app_clone = app.clone();
    let session_clone = session.clone();

    tokio::spawn(async move {
        // 选择模型
        let model_id = pool.pinned().unwrap_or_else(|| {
            // 从池中取第一个可用模型
            pool.model_ids().first().cloned().unwrap_or_default()
        });

        if model_id.is_empty() {
            let _ = app_clone.emit("ntcode://error", ErrorEvent {
                message: "无可用模型".into(),
            });
            let mut s = session_clone.write().await;
            s.status = SessionStatus::Idle;
            s.cancel = None;
            return;
        }

        // 构建 LLM asker
        let asker = NtModelCliAsk::new()
            .with_model(model_id.clone())
            .with_timeout(Duration::from_secs(120));

        // 收集完整响应（ask_stream 为同步 trait，回调经 RefCell 收集）
        let full_response = std::cell::RefCell::new(String::new());
        let ask_result = {
            let session_snap = session_clone.read().await;
            // 构建上下文：最近的对话历史
            let context = build_context(&session_snap);
            asker.ask_stream(&context, &|chunk| {
                full_response.borrow_mut().push_str(chunk);
                let _ = app_clone.emit("ntcode://chunk", ChunkEvent {
                    content: chunk.to_string(),
                });
                true
            })
        };

        match ask_result {
            Ok(_reply) => {
                // 推送完成事件
                let _ = app_clone.emit("ntcode://done", DoneEvent {
                    summary: full_response.borrow().chars().take(100).collect(),
                    model: model_id.clone(),
                });
                // 更新 session
                let mut s = session_clone.write().await;
                s.push_assistant(&full_response.borrow(), Some(model_id));
                s.status = SessionStatus::Idle;
                s.cancel = None;
            }
            Err(e) => {
                let _ = app_clone.emit("ntcode://error", ErrorEvent {
                    message: format!("生成失败：{e}"),
                });
                let mut s = session_clone.write().await;
                s.status = SessionStatus::Idle;
                s.cancel = None;
            }
        }

        let _ = app_clone.emit("ntcode://status", StatusEvent { status: "idle".into() });
    });
}

/// 构建 LLM 上下文（最近 N 条消息 + 目标）。
fn build_context(session: &NtcodeSession) -> String {
    let mut ctx = format!("目标：{}\n\n", session.goal);
    // 取最近 10 条消息作为上下文
    let recent: Vec<&ChatMessage> = session.transcript.iter().rev().take(10).rev().collect();
    for msg in &recent {
        let role = match msg.role {
            Role::User => "用户",
            Role::Assistant => "助手",
            Role::System => "系统",
        };
        ctx.push_str(&format!("{}：{}\n", role, msg.content));
    }
    ctx.push_str("助手：");
    ctx
}

/// 停止当前流式生成。
pub async fn cancel_streaming(session: &tokio::sync::RwLock<NtcodeSession>) {
    let mut s = session.write().await;
    if let Some(cancel) = s.cancel.take() {
        cancel.notify_one();
    }
    s.status = SessionStatus::Idle;
}
