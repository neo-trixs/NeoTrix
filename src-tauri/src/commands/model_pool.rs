//! Model Pool — Tauri command wrappers (delegates to `domain::model_pool`)

use crate::domain::model_pool as mp;
use crate::ipc;
use crate::ipc::IpcResponse;
pub use mp::{ModelPoolEntry, ModelPoolStatus};

#[tauri::command]
pub async fn model_pool_status() -> IpcResponse<ModelPoolStatus> {
    match mp::get_status() {
        Ok(s) => ipc::ok(s),
        Err(e) => ipc::err("POOL_READ_FAILED", format!("{e}")),
    }
}

#[tauri::command]
pub async fn model_pool_add(
    label: String,
    provider: String,
    api_key: String,
    model: String,
    tags: Vec<String>,
    base_url: Option<String>,
) -> IpcResponse<ModelPoolEntry> {
    match mp::add_entry(
        &label,
        &provider,
        &api_key,
        &model,
        &tags,
        base_url.as_deref(),
    ) {
        Ok(entry) => ipc::ok(entry),
        Err(e) => ipc::err("POOL_WRITE_FAILED", format!("{e}")),
    }
}

#[tauri::command]
pub async fn model_pool_remove(label: String) -> IpcResponse<bool> {
    match mp::remove_entry(&label) {
        Ok(b) => ipc::ok(b),
        Err(e) => ipc::err("POOL_WRITE_FAILED", format!("{e}")),
    }
}

#[tauri::command]
pub async fn model_pool_update_key(label: String, new_api_key: String) -> Result<bool, String> {
    mp::update_api_key(&label, &new_api_key).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn model_pool_check(label: String) -> Result<String, String> {
    mp::check_connectivity(&label)
        .await
        .map_err(|e| e.to_string())
}
