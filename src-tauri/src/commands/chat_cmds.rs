use crate::anthropic;
use crate::anthropic::types::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{LazyLock, Mutex};
use tauri::{AppHandle, Emitter};

static GENERATING: AtomicBool = AtomicBool::new(false);

static CONVERSATIONS: LazyLock<Mutex<Vec<ConversationItem>>> =
    LazyLock::new(|| Mutex::new(Vec::new()));

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

    let api_key = anthropic::client::get_api_key()?;
    let model_name = model.unwrap_or_else(|| "claude-sonnet-4-20250514".to_string());
    let message_id = format!("msg-{}", uuid::Uuid::new_v4());

    let messages = vec![Message {
        role: "user".to_string(),
        content: vec![ContentBlock::Text {
            text: content.clone(),
            r#type: Some("text".to_string()),
        }],
    }];

    let result = anthropic::client::send_message_stream(
        &api_key,
        &model_name,
        4096,
        messages,
        None,
        &app,
        &message_id,
    )
    .await;

    GENERATING.store(false, Ordering::SeqCst);

    let success = result.is_ok();
    if success {
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

    result.map(|_| message_id)
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
