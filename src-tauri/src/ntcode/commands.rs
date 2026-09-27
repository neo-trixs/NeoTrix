//! ntcode Tauri commands — 9 个 IPC 命令供前端调用。

use crate::ipc::{self, IpcResponse};
use crate::ntcode::streaming;
use crate::ntcode::{
    ConversationSummary, ModelInfo, NtcodeSession, NtcodeState, SessionStatus,
};
use neotrix::l1_action::nt_conversation;
use neotrix::l1_action::nt_free_pool::NtFreePoolAsk;
use neotrix::l1_action::nt_io::nt_io_provider::catalog::cli_free_source::CliFreeSource;
use neotrix::l1_action::nt_io::nt_io_provider::catalog::model_pool::UnifiedModelPool;
use std::sync::Arc;
use tauri::{AppHandle, State};

// ============================================================================
// 模型管理
// ============================================================================

/// 获取可用模型列表。
#[tauri::command]
pub async fn ntcode_models() -> Result<IpcResponse<Vec<ModelInfo>>, String> {
    let mut pool = UnifiedModelPool::default_pool();
    pool.add_source(Box::new(CliFreeSource::new()));
    let entries = pool.refresh();

    let models: Vec<ModelInfo> = entries
        .iter()
        .map(|e| ModelInfo {
            id: e.id.clone(),
            name: e.display_name.clone(),
            source: e.source.clone(),
            tier: e.tier.clone(),
            is_free: e.is_free,
        })
        .collect();

    Ok(ipc::ok(models))
}

/// 切换当前模型。
#[tauri::command]
pub async fn ntcode_switch_model(
    state: State<'_, NtcodeState>,
    model: String,
) -> Result<IpcResponse<()>, String> {
    let mut session = state.inner.write().await;
    session.model = Some(model.clone());
    Ok(ipc::ok(()))
}

// ============================================================================
// 对话管理
// ============================================================================

/// 对话列表。
#[tauri::command]
pub async fn ntcode_conversations() -> Result<IpcResponse<Vec<ConversationSummary>>, String> {
    let convs = nt_conversation::list();
    let summaries: Vec<ConversationSummary> = convs
        .iter()
        .map(|c| ConversationSummary {
            id: c.id.clone(),
            goal: c.goal.clone(),
            message_count: c.transcript.len(),
            model: c.model.clone(),
            updated_at: c.updated_at,
        })
        .collect();
    Ok(ipc::ok(summaries))
}

/// 打开指定对话。
#[tauri::command]
pub async fn ntcode_open(
    state: State<'_, NtcodeState>,
    id: String,
) -> Result<IpcResponse<()>, String> {
    let conv = nt_conversation::load(&id).map_err(|e| e)?;
    let mut session = state.inner.write().await;
    session.conversation_id = Some(conv.id);
    session.goal = conv.goal;
    session.model = conv.model;
    // 将持久化的 transcript 转换为 ChatMessage
    session.transcript = conv
        .transcript
        .into_iter()
        .enumerate()
        .map(|(i, line)| {
            let role = if i % 2 == 0 {
                crate::ntcode::Role::User
            } else {
                crate::ntcode::Role::Assistant
            };
            crate::ntcode::ChatMessage {
                role,
                content: line,
                timestamp: conv.updated_at,
                model: session.model.clone(),
            }
        })
        .collect();
    Ok(ipc::ok(()))
}

/// 新建对话。
#[tauri::command]
pub async fn ntcode_new(
    state: State<'_, NtcodeState>,
    goal: String,
) -> Result<IpcResponse<()>, String> {
    let mut session = state.inner.write().await;
    // 保存旧对话
    if session.conversation_id.is_some() {
        save_session(&session).await;
    }
    *session = NtcodeSession::new(&goal);
    Ok(ipc::ok(()))
}

/// 保存当前对话。
#[tauri::command]
pub async fn ntcode_save(
    state: State<'_, NtcodeState>,
) -> Result<IpcResponse<()>, String> {
    let session = state.inner.read().await;
    save_session(&session).await;
    Ok(ipc::ok(()))
}

/// 获取当前状态。
#[tauri::command]
pub async fn ntcode_status(
    state: State<'_, NtcodeState>,
) -> Result<IpcResponse<SessionStatus>, String> {
    let session = state.inner.read().await;
    Ok(ipc::ok(session.status))
}

// ============================================================================
// 消息发送（流式）
// ============================================================================

/// 发送消息，流式响应通过 `ntcode://chunk` event 推送。
#[tauri::command]
pub async fn ntcode_send(
    app: AppHandle,
    state: State<'_, NtcodeState>,
    message: String,
    pool_state: State<'_, NtcodePoolState>,
) -> Result<IpcResponse<()>, String> {
    // 检查是否正在生成
    {
        let session = state.inner.read().await;
        if session.status == SessionStatus::Streaming {
            return Ok(IpcResponse {
                ok: false,
                error: Some(crate::ipc::IpcError::new(
                    "BUSY".to_string(),
                    "正在生成中，请等待完成或按停止".to_string(),
                )),
                data: None,
            });
        }
    }

    // 设置目标（如果还没有）
    {
        let mut session = state.inner.write().await;
        if session.goal.is_empty() {
            session.goal = message.clone();
        }
    }

    streaming::spawn_streaming(
        app,
        state.inner.clone(),
        pool_state.pool.clone(),
        message,
    )
    .await;

    Ok(ipc::ok(()))
}

/// 停止当前生成。
#[tauri::command]
pub async fn ntcode_stop(
    state: State<'_, NtcodeState>,
) -> Result<IpcResponse<()>, String> {
    streaming::cancel_streaming(&state.inner).await;
    Ok(ipc::ok(()))
}

// ============================================================================
// 池状态（Tauri managed）
// ============================================================================

/// 模型池状态（Tauri managed，启动时初始化）。
pub struct NtcodePoolState {
    pub pool: Arc<NtFreePoolAsk>,
}

// ============================================================================
// 内部辅助
// ============================================================================

/// 保存 session 到磁盘。
async fn save_session(session: &NtcodeSession) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let conv = nt_conversation::Conversation {
        id: session
            .conversation_id
            .clone()
            .unwrap_or_else(|| format!("conv-{now:x}")),
        goal: session.goal.clone(),
        transcript: session.transcript.iter().map(|m| m.content.clone()).collect(),
        model: session.model.clone(),
        created_at: session
            .conversation_id
            .as_ref()
            .and_then(|id| nt_conversation::load(id).ok().map(|c| c.created_at))
            .unwrap_or(now),
        updated_at: now,
    };

    let _ = nt_conversation::save(&conv);
}
