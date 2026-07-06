use std::sync::Mutex;
use tauri::command;
use neotrix::neotrix::nt_core_error::NeoTrixError;
use super::SessionInfo;

static SESSIONS: std::sync::LazyLock<Mutex<Vec<SessionInfo>>> =
    std::sync::LazyLock::new(|| Mutex::new(Vec::new()));

#[command]
pub fn session_list() -> Vec<SessionInfo> {
    vec![SessionInfo {
        id: "default".into(),
        name: "默认会话".into(),
        message_count: 0,
        created: 0,
    }]
}

#[command]
pub fn session_create(name: String) -> SessionInfo {
    SessionInfo {
        id: format!("s-{}", chrono::Utc::now().timestamp()),
        name,
        message_count: 0,
        created: chrono::Utc::now().timestamp(),
    }
}

#[command]
pub fn cmd_session_create(name: String) -> Result<String, NeoTrixError> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp();
    let info = SessionInfo { id: id.clone(), name, message_count: 0, created: now };
    SESSIONS.lock().map_err(|e| NeoTrixError::Brain(e.to_string()))?.push(info);
    Ok(id)
}

#[command]
pub fn cmd_session_switch(id: String) -> Result<(), NeoTrixError> {
    let sessions = SESSIONS.lock().map_err(|e| NeoTrixError::Brain(e.to_string()))?;
    if sessions.iter().any(|s| s.id == id) { Ok(()) }
    else { Err(NeoTrixError::Memory(format!("Session not found: {}", id))) }
}

#[command]
pub fn cmd_session_delete(id: String) -> Result<(), NeoTrixError> {
    SESSIONS.lock().map_err(|e| NeoTrixError::Brain(e.to_string()))?.retain(|s| s.id != id);
    Ok(())
}

#[command]
pub fn cmd_session_list() -> Result<Vec<SessionInfo>, NeoTrixError> {
    SESSIONS.lock().map(|s| s.clone()).map_err(|e| NeoTrixError::Brain(e.to_string()))
}

#[command]
pub fn cmd_session_fork(id: String) -> Result<String, NeoTrixError> {
    let mut sessions = SESSIONS.lock().map_err(|e| NeoTrixError::Brain(e.to_string()))?;
    let src = sessions
        .iter()
        .find(|s| s.id == id)
        .cloned()
        .ok_or_else(|| NeoTrixError::Memory(format!("Session not found: {}", id)))?;
    let new_id = uuid::Uuid::new_v4().to_string();
    let new_name = format!("{} (副本)", src.name);
    let forked = SessionInfo {
        id: new_id.clone(),
        name: new_name,
        message_count: src.message_count,
        created: chrono::Utc::now().timestamp(),
    };
    sessions.push(forked);
    Ok(new_id)
}

#[command]
pub fn cmd_session_export_json(id: String) -> Result<String, NeoTrixError> {
    let sessions = SESSIONS.lock().map_err(|e| NeoTrixError::Brain(e.to_string()))?;
    let src = sessions
        .iter()
        .find(|s| s.id == id)
        .cloned()
        .ok_or_else(|| NeoTrixError::Memory(format!("Session not found: {}", id)))?;
    let export = serde_json::json!({
        "format_version": 1,
        "sessions": [{
            "id": src.id,
            "name": src.name,
            "message_count": src.message_count,
            "created": src.created,
        }],
    });
    serde_json::to_string_pretty(&export).map_err(|e| NeoTrixError::Serde(e.to_string()))
}

#[command]
pub fn cmd_session_import_json(json: String) -> Result<String, NeoTrixError> {
    let value: serde_json::Value =
        serde_json::from_str(&json).map_err(|e| NeoTrixError::Serde(format!("解析失败: {}", e)))?;
    let version = value
        .get("format_version")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    if version != 1 {
        return Err(NeoTrixError::Serde(format!("不支持的格式版本: {}", version)));
    }
    let sessions_arr = value
        .get("sessions")
        .and_then(|v| v.as_array())
        .ok_or_else(|| NeoTrixError::Memory("缺少 sessions 字段".to_string()))?;
    let mut lock = SESSIONS.lock().map_err(|e| NeoTrixError::Brain(e.to_string()))?;
    let mut imported_ids = Vec::new();
    for item in sessions_arr {
        let name = item
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("imported");
        let msg_count = item
            .get("message_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;
        let created = item
            .get("created")
            .and_then(|v| v.as_i64())
            .unwrap_or_else(|| chrono::Utc::now().timestamp());
        let final_name = if lock.iter().any(|s| s.name == name) {
            format!("{} (导入)", name)
        } else {
            name.to_string()
        };
        let new_id = uuid::Uuid::new_v4().to_string();
        lock.push(SessionInfo {
            id: new_id.clone(),
            name: final_name,
            message_count: msg_count,
            created,
        });
        imported_ids.push(new_id);
    }
    Ok(imported_ids.join(","))
}
