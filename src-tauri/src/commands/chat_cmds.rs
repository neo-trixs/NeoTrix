use crate::anthropic;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{LazyLock, Mutex};

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