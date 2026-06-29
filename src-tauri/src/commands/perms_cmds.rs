use std::sync::Mutex;
use std::sync::atomic::AtomicU64;
use tauri::command;
use super::PermissionRequest;

static PERMISSION_COUNTER: AtomicU64 = AtomicU64::new(1);
static PENDING_PERMISSIONS: std::sync::LazyLock<Mutex<Vec<PermissionRequest>>> =
    std::sync::LazyLock::new(|| Mutex::new(Vec::new()));

#[command]
pub fn cmd_permission_request(action: String, target: String) -> Result<PermissionRequest, String> {
    let id = format!("perm-{}", PERMISSION_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst));
    let req = PermissionRequest { id: id.clone(), action, target, timestamp: chrono::Utc::now().timestamp() };
    PENDING_PERMISSIONS.lock().map_err(|e| e.to_string())?.push(req.clone());
    Ok(req)
}

#[command]
pub fn cmd_permission_approve(id: String) -> Result<(), String> {
    let mut perms = PENDING_PERMISSIONS.lock().map_err(|e| e.to_string())?;
    let len = perms.len();
    perms.retain(|p| p.id != id);
    if perms.len() == len { return Err(format!("Permission request not found: {}", id)); }
    Ok(())
}

#[command]
pub fn cmd_permission_deny(id: String) -> Result<(), String> {
    let mut perms = PENDING_PERMISSIONS.lock().map_err(|e| e.to_string())?;
    let len = perms.len();
    perms.retain(|p| p.id != id);
    if perms.len() == len { return Err(format!("Permission request not found: {}", id)); }
    Ok(())
}
