//! Chat command — single entry point for all user interactions.

use tauri::State;
use crate::chat::{ChatResponse, ChatAction};
use crate::chat::router::IntentRouter;
use crate::commands::domain_cmd::DomainState;
use crate::ipc::{self, IpcResponse};

/// Send a natural language message to NeoTrix.
///
/// Parses the message into a `(domain, action, args)` intent and dispatches
/// via `DomainRegistry::call_async`. Errors preserve `DomainError` codes
/// (same envelope as `domain_call`) so the frontend can match on them.
#[tauri::command]
pub async fn chat_send(
    state: State<'_, DomainState>,
    message: String,
    session_id: Option<String>,
) -> Result<IpcResponse<ChatResponse>, String> {
    let registry = state.read().await;

    // Parse intent
    let mut intent = IntentRouter::parse(&message, &registry)
        .await
        .map_err(|e| e.message)?;

    // `help` is answered locally — no domain plugin implements it.
    if intent.domain == "chat" && intent.action == "help" {
        let text = intent
            .response_hint
            .take()
            .unwrap_or_else(IntentRouter::help_text);
        return Ok(ipc::ok(ChatResponse {
            message: text,
            actions: vec![],
            state: None,
        }));
    }

    // Thread the conversation through when a session is provided.
    if let Some(sid) = session_id {
        if let Some(obj) = intent.args.as_object_mut() {
            obj.entry("session_id".to_string())
                .or_insert(serde_json::Value::String(sid));
        }
    }

    // Execute intent
    let result = match registry
        .call_async(&intent.domain, &intent.action, intent.args.clone())
        .await
    {
        Ok(data) => data,
        Err(e) => {
            return Ok(IpcResponse {
                ok: false,
                error: Some(crate::ipc::IpcError::new(e.code, e.message)),
                data: None,
            });
        }
    };

    // Build response
    let response_message = intent.response_hint.unwrap_or_else(|| {
        format!("已完成操作: {} / {}", intent.domain, intent.action)
    });

    Ok(ipc::ok(ChatResponse {
        message: response_message,
        actions: vec![ChatAction {
            domain: intent.domain,
            action: intent.action,
            result,
        }],
        state: None,
    }))
}

/// Get help text for available commands.
#[tauri::command]
pub async fn chat_help() -> Result<String, String> {
    Ok(IntentRouter::help_text())
}
