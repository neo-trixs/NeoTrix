use std::sync::Arc;
use tauri::State;
use neotrix::neotrix::nt_core_error::NeoTrixError;
use neotrix::neotrix::nt_shield::permissions::{PermissionManager, PermissionRequest, AuditEntry};

#[tauri::command]
pub fn request_permission(
    req: PermissionRequest,
    manager: State<'_, Arc<PermissionManager>>,
) -> Result<PermissionRequest, NeoTrixError> {
    Ok(manager.request(req))
}

#[tauri::command]
pub fn respond_permission(
    request_id: String,
    approved: bool,
    manager: State<'_, Arc<PermissionManager>>,
) -> Result<(), NeoTrixError> {
    let reason = if approved {
        "Approved by user".to_string()
    } else {
        "Denied by user".to_string()
    };
    (if approved {
        manager.approve(&request_id, reason)
    } else {
        manager.deny(&request_id, reason)
    }).map_err(NeoTrixError::Brain)
}

#[tauri::command]
pub fn get_pending_permissions(
    manager: State<'_, Arc<PermissionManager>>,
) -> Vec<PermissionRequest> {
    manager.get_pending_requests()
}

#[tauri::command]
pub fn get_permission_audit_log(
    count: usize,
    manager: State<'_, Arc<PermissionManager>>,
) -> Vec<AuditEntry> {
    manager.get_audit_log(count)
}
