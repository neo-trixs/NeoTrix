use neotrix::core::nt_core_agent::qr_code::generate_qr_svg;
use neotrix::core::nt_core_agent::remote_control::{MessageContent, RemoteControlServer};
use serde::Serialize;
use std::sync::Arc;

pub struct RemoteControlState {
    pub server: Arc<RemoteControlServer>,
}

#[derive(Serialize)]
pub struct RemoteSessionInfo {
    pub id: String,
    pub label: String,
    pub created_at: u64,
    pub state: String,
    pub relay_url: String,
    pub qr_url: String,
    pub qr_svg: Option<String>,
}

fn content_type(content: &MessageContent) -> &'static str {
    content.content_type()
}

#[tauri::command]
pub fn remote_start(
    state: tauri::State<'_, RemoteControlState>,
    label: Option<String>,
) -> Result<RemoteSessionInfo, String> {
    let session_id = state
        .server
        .create_session(label.unwrap_or_else(|| "Remote Session".into()));
    let id_str = session_id.0.clone();
    let qr_url = state.server.get_qr_url(&id_str);
    let qr_svg = Some(generate_qr_svg(&qr_url, 300));
    Ok(RemoteSessionInfo {
        id: id_str.clone(),
        label: "Remote Session".into(),
        created_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        state: "connected".into(),
        relay_url: state.server.get_relay_url(&id_str),
        qr_url,
        qr_svg,
    })
}

#[tauri::command]
pub fn remote_get_qr(
    state: tauri::State<'_, RemoteControlState>,
    session_id: String,
) -> Result<RemoteSessionInfo, String> {
    let sessions_arc = state.server.sessions();
    let sessions = sessions_arc.lock().unwrap();
    if let Some(session) = sessions.get(&session_id) {
        let qr_url = state.server.get_qr_url(&session_id);
        let qr_svg = Some(generate_qr_svg(&qr_url, 300));
        Ok(RemoteSessionInfo {
            id: session.id.0.clone(),
            label: session.label.clone(),
            created_at: session.created_at,
            state: format!("{:?}", session.state),
            relay_url: state.server.get_relay_url(&session_id),
            qr_url,
            qr_svg,
        })
    } else {
        Err("Session not found".into())
    }
}

#[tauri::command]
pub fn remote_status(
    state: tauri::State<'_, RemoteControlState>,
    session_id: String,
) -> Result<serde_json::Value, String> {
    let s_arc = state.server.sessions();
    let sessions = s_arc.lock().unwrap();
    if let Some(session) = sessions.get(&session_id) {
        Ok(serde_json::json!({
            "id": session.id.0,
            "label": session.label,
            "created_at": session.created_at,
            "last_active": session.last_active,
            "state": format!("{:?}", session.state),
            "queue_size": session.message_queue.len(),
        }))
    } else {
        Err("Session not found".into())
    }
}

#[tauri::command]
pub fn remote_poll(
    state: tauri::State<'_, RemoteControlState>,
    session_id: String,
    since_seq: u64,
) -> Result<Vec<serde_json::Value>, String> {
    let messages = state.server.poll_messages(&session_id, since_seq).map_err(|e| e.to_string())?;
    Ok(messages
        .into_iter()
        .map(|m| {
            serde_json::json!({
                "id": m.id,
                "seq": m.seq,
                "direction": format!("{:?}", m.direction),
                "content_type": content_type(&m.content),
                "content": serde_json::json!(m.content),
                "timestamp": m.timestamp,
            })
        })
        .collect())
}

#[tauri::command]
pub fn remote_send(
    state: tauri::State<'_, RemoteControlState>,
    session_id: String,
    text: String,
) -> Result<u64, String> {
    state.server.push_message(
        &session_id,
        MessageContent::Text(text),
        neotrix::core::nt_core_agent::remote_control::MessageDirection::Incoming,
    ).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn remote_stop(
    state: tauri::State<'_, RemoteControlState>,
    session_id: String,
) -> Result<(), String> {
    state.server.remove_session(&session_id);
    Ok(())
}
