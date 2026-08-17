use crate::anthropic;
use crate::anthropic::types::*;
use neotrix::neotrix::nt_io_provider::{
    create_provider, estimate_messages_tokens, GatewayV2, LlmProviderType, LlmRequest,
    Message as CoreMessage, ProviderCategory, ProviderConfig, Role,
};
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock, Mutex};
use tauri::{AppHandle, Emitter};

static GENERATING: AtomicBool = AtomicBool::new(false);

static CONVERSATIONS: LazyLock<Mutex<Vec<ConversationItem>>> =
    LazyLock::new(|| Mutex::new(Vec::new()));

/// P2-G2: 每会话消息历史 (桌面侧有状态恢复)。上限受 token 预算 + 条数双控。
static HISTORY: LazyLock<Mutex<HashMap<String, VecDeque<Message>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// P2-G3: 桌面聊天并入统一 GatewayV2 (Anthropic provider)。
/// 缓存 key 指纹: api_key 变更时重建网关, 避免旧 key 泄漏。
static GATEWAY: LazyLock<Mutex<Option<(String, Arc<GatewayV2>)>>> =
    LazyLock::new(|| Mutex::new(None));

const HISTORY_MAX_TOKENS: usize = 24_000;
const HISTORY_MAX_MSGS: usize = 40;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConversationItem {
    pub id: String,
    pub title: String,
    pub model: String,
    pub pinned: bool,
    pub created_at: i64,
    pub updated_at: i64,
    pub message_count: usize,
}

#[tauri::command]
pub fn save_api_key(key: String) -> Result<(), String> {
    anthropic::client::save_api_key(&key)
}

#[tauri::command]
pub fn has_api_key() -> bool {
    anthropic::client::has_api_key()
}

#[tauri::command]
pub fn delete_api_key() -> Result<(), String> {
    anthropic::client::delete_api_key()
}

/// 将桌面 Message (role + 文本 block) 转为核心 Message (role 枚举 + 拼接文本)。
fn to_core_message(m: &Message) -> CoreMessage {
    let content = m
        .content
        .iter()
        .filter_map(|b| match b {
            ContentBlock::Text { text, .. } => Some(text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n");
    let role = match m.role.as_str() {
        "assistant" => Role::Assistant,
        "system" => Role::System,
        _ => Role::User,
    };
    CoreMessage::new(role, &content)
}

/// 历史双控裁剪: 条数超限直接丢最旧; token 超预算逐条丢最旧 (保留最近请求)。
fn trim_history(msgs: &mut VecDeque<Message>) {
    while msgs.len() > HISTORY_MAX_MSGS {
        msgs.pop_front();
    }
    loop {
        if msgs.len() <= 1 {
            break;
        }
        let core: Vec<CoreMessage> = msgs.iter().map(to_core_message).collect();
        if estimate_messages_tokens(&core) <= HISTORY_MAX_TOKENS {
            break;
        }
        msgs.pop_front();
    }
}

/// P2-G3: 懒构建并入 Gateway 的桌面网关 (Anthropic provider, 带响应缓存)。
fn gateway_for(api_key: &str) -> Result<Arc<GatewayV2>, String> {
    let mut guard = GATEWAY.lock().map_err(|e| e.to_string())?;
    if let Some((key, gw)) = guard.as_ref() {
        if key == api_key {
            return Ok(gw.clone());
        }
    }
    let provider = create_provider(ProviderConfig {
        provider_type: LlmProviderType::Anthropic,
        api_key: Some(api_key.to_string()),
        ..Default::default()
    });
    let mut gateway = GatewayV2::new();
    gateway.register_provider_with_category("anthropic", provider, false, ProviderCategory::Cloud);
    gateway.enable_response_cache(true);
    let gw = Arc::new(gateway);
    *guard = Some((api_key.to_string(), gw.clone()));
    Ok(gw)
}

#[tauri::command]
pub async fn send_message(
    app: AppHandle,
    conversation_id: String,
    content: String,
    model: Option<String>,
) -> Result<String, String> {
    if GENERATING.swap(true, Ordering::SeqCst) {
        return Err("already generating".to_string());
    }

    let api_key = match anthropic::client::get_api_key() {
        Ok(k) => k,
        Err(e) => {
            GENERATING.store(false, Ordering::SeqCst);
            return Err(e);
        }
    };
    let model_name = model.unwrap_or_else(|| "claude-sonnet-4-20250514".to_string());
    let message_id = format!("msg-{}", uuid::Uuid::new_v4());

    // P2-G2: 载入会话历史, 追加当前用户消息后裁剪
    let history = {
        let mut hist = HISTORY.lock().map_err(|e| e.to_string())?;
        let entry = hist.entry(conversation_id.clone()).or_default();
        entry.push_back(Message {
            role: "user".to_string(),
            content: vec![ContentBlock::Text {
                text: content.clone(),
                r#type: Some("text".to_string()),
            }],
        });
        trim_history(entry);
        entry.clone()
    };

    // P2-G3: 统一网关通道 (Anthropic provider)
    let gateway = match gateway_for(&api_key) {
        Ok(g) => g,
        Err(e) => {
            GENERATING.store(false, Ordering::SeqCst);
            return Err(e);
        }
    };

    let mut request = LlmRequest::new(&format!("anthropic/{}", model_name), &content);
    request.messages = history.iter().map(to_core_message).collect();
    request.temperature = None;
    request.max_tokens = 4096;
    // P0-4: 稳定前缀 (除末条当前请求外) 标注 prefix cache, 多轮对话命中 Anthropic 缓存
    request.cacheable_prefix_tokens = if request.messages.len() > 1 {
        Some(estimate_messages_tokens(&request.messages[..request.messages.len() - 1]))
    } else {
        None
    };

    let mut rx = match gateway.stream_complete_with_selection(&request).await {
        Ok(rx) => rx,
        Err(e) => {
            GENERATING.store(false, Ordering::SeqCst);
            return Err(format!("gateway: {}", e));
        }
    };

    let mut assistant = String::new();
    let mut failed: Option<String> = None;
    while let Some(item) = rx.recv().await {
        match item {
            Ok(resp) => {
                let delta = resp.content;
                if !delta.is_empty() {
                    assistant.push_str(&delta);
                    let _ = app.emit("stream-chunk", serde_json::json!({
                        "messageId": message_id,
                        "delta": delta,
                    }));
                }
            }
            Err(e) => {
                let msg = format!("stream: {}", e);
                failed = Some(msg.clone());
                let _ = app.emit("stream-error", serde_json::json!({
                    "messageId": message_id,
                    "error": msg,
                }));
            }
        }
    }

    GENERATING.store(false, Ordering::SeqCst);

    let success = failed.is_none();
    if success {
        // P2-G2: 回写 assistant 回复到会话历史
        if let Ok(mut hist) = HISTORY.lock() {
            if let Some(h) = hist.get_mut(&conversation_id) {
                h.push_back(Message {
                    role: "assistant".to_string(),
                    content: vec![ContentBlock::Text {
                        text: assistant,
                        r#type: Some("text".to_string()),
                    }],
                });
                trim_history(h);
            }
        }
        let mut convs = CONVERSATIONS.lock().unwrap_or_else(|e| e.into_inner());
        let now = chrono::Utc::now().timestamp();
        if let Some(conv) = convs.iter_mut().find(|c| c.id == conversation_id) {
            conv.message_count += 1;
            conv.updated_at = now;
        } else {
            convs.push(ConversationItem {
                id: conversation_id.clone(),
                title: content.chars().take(80).collect(),
                model: model_name.clone(),
                pinned: false,
                created_at: now,
                updated_at: now,
                message_count: 1,
            });
        }
    }

    let _ = app.emit("stream-done", serde_json::json!({
        "messageId": message_id,
        "conversationId": conversation_id,
        "success": success,
    }));

    result_err(success, failed).map(|_| message_id)
}

/// 统一成功/失败出口。
fn result_err(success: bool, failed: Option<String>) -> Result<(), String> {
    if success {
        Ok(())
    } else {
        Err(failed.unwrap_or_else(|| "generation failed".to_string()))
    }
}

#[tauri::command]
pub fn stop_generation() -> Result<(), String> {
    if GENERATING.load(Ordering::SeqCst) {
        GENERATING.store(false, Ordering::SeqCst);
        Ok(())
    } else {
        Err("no active generation".to_string())
    }
}

#[tauri::command]
pub fn list_conversations() -> Vec<ConversationItem> {
    CONVERSATIONS.lock().map(|c| c.clone()).unwrap_or_default()
}