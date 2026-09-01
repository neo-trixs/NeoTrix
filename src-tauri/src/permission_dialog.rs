use std::sync::Arc;
use tauri::State;

// Stub types for permission dialog
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PermissionRequest {
    pub id: String,
    pub action: String,
    pub target: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuditEntry {
    pub id: String,
    pub request_id: String,
    pub action: String,
    pub target: String,
    pub approved: bool,
    pub timestamp: i64,
}

pub struct PermissionManager;

impl PermissionManager {
    pub fn new() -> Self { Self }
    pub fn request(&self, req: PermissionRequest) -> PermissionRequest { req }
    pub fn approve(&self, id: &str) -> Result<AuditEntry, String> {
        Ok(AuditEntry { id: uuid::Uuid::new_v4().to_string(), request_id: id.into(), action: "approved".into(), target: String::new(), approved: true, timestamp: chrono::Utc::now().timestamp() })
    }
    pub fn deny(&self, id: &str) -> Result<AuditEntry, String> {
        Ok(AuditEntry { id: uuid::Uuid::new_v4().to_string(), request_id: id.into(), action: "denied".into(), target: String::new(), approved: false, timestamp: chrono::Utc::now().timestamp() })
    }
}

#[tauri::command]
pub fn request_permission(
    req: PermissionRequest,
    manager: State<'_, Arc<PermissionManager>>,
) -> Result<PermissionRequest, String> {
    Ok(manager.request(req))
}

#[tauri::command]
pub fn respond_permission(
    id: String,
    approved: bool,
    manager: State<'_, Arc<PermissionManager>>,
) -> Result<AuditEntry, String> {
    if approved { manager.approve(&id) } else { manager.deny(&id) }
}
