//! Chat command — single entry point for all user interactions.

use tauri::State;
use crate::chat::{ChatResponse, ChatAction};
use crate::chat::router::IntentRouter;
use crate::commands::domain_cmd::DomainState;

/// Send a natural language message to NeoTrix.
#[tauri::command]
pub async fn chat_send(
    state: State<'_, DomainState>,
    message: String,
    session_id: Option<String>,
) -> Result<ChatResponse, String> {
    let registry = state.read().await;
    
    // Parse intent
    let intent = IntentRouter::parse(&message, &registry)
        .await
        .map_err(|e| e.message)?;
    
    // Execute intent
    let result = registry.call(&intent.domain, &intent.action, intent.args.clone())
        .await
        .map_err(|e| e.message)?;
    
    // Build response
    let response_message = intent.response_hint.unwrap_or_else(|| {
        format!("已完成操作: {} / {}", intent.domain, intent.action)
    });
    
    Ok(ChatResponse {
        message: response_message,
        actions: vec![ChatAction {
            domain: intent.domain,
            action: intent.action,
            result,
        }],
        state: None,
    })
}

/// Get help text for available commands.
#[tauri::command]
pub async fn chat_help() -> Result<String, String> {
    Ok(crate::chat::router::IntentRouter::help_text())
}
