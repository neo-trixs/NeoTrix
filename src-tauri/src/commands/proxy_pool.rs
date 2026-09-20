//! Proxy Pool — Tauri command wrappers
//!
//! Thin wrappers that delegate to `domain::proxy_pool::*`.

use crate::config::AppConfig;
use crate::domain::proxy_pool as pool;
use crate::ipc::{self, IpcResponse};

// Re-export types for existing consumers
pub use crate::domain::proxy_pool::{ProxyPoolEntry, ProxyPoolSnapshot, ProxyPoolStatus};

fn base_dir() -> std::path::PathBuf {
    AppConfig::base_dir().unwrap_or_default()
}

// ═══════════════════════════════════════════════
// Tauri Commands
// ═══════════════════════════════════════════════

#[tauri::command]
pub async fn proxy_pool_status() -> IpcResponse<ProxyPoolStatus> {
    match pool::get_status(&base_dir()) {
        Ok(status) => ipc::ok(status),
        Err(e) => ipc::err(e.code(), e.message()),
    }
}

#[tauri::command]
pub async fn proxy_pool_snapshot() -> IpcResponse<ProxyPoolSnapshot> {
    match pool::get_snapshot(&base_dir()) {
        Ok(snapshot) => ipc::ok(snapshot),
        Err(e) => ipc::err(e.code(), e.message()),
    }
}

#[tauri::command]
pub async fn proxy_pool_add(url: String, tag: String) -> IpcResponse<ProxyPoolEntry> {
    match pool::add_node(&base_dir(), &url, Some(&tag)) {
        Ok(entry) => ipc::ok(entry),
        Err(e) => ipc::err(e.code(), e.message()),
    }
}

#[tauri::command]
pub async fn proxy_pool_remove(url: String) -> IpcResponse<bool> {
    match pool::remove_node(&base_dir(), &url) {
        Ok(removed) => ipc::ok(removed),
        Err(e) => ipc::err(e.code(), e.message()),
    }
}

#[tauri::command]
pub async fn proxy_pool_add_subscription(url: String) -> IpcResponse<Vec<String>> {
    match pool::add_subscription(&base_dir(), &url) {
        Ok(subs) => ipc::ok(subs),
        Err(e) => ipc::err(e.code(), e.message()),
    }
}

#[tauri::command]
pub async fn proxy_pool_remove_subscription(url: String) -> IpcResponse<Vec<String>> {
    match pool::remove_subscription(&base_dir(), &url) {
        Ok(subs) => ipc::ok(subs),
        Err(e) => ipc::err(e.code(), e.message()),
    }
}

#[tauri::command]
pub async fn proxy_pool_set_strategy(strategy: String) -> IpcResponse<String> {
    match pool::set_strategy(&base_dir(), &strategy) {
        Ok(s) => ipc::ok(s),
        Err(e) => ipc::err(e.code(), e.message()),
    }
}

#[tauri::command]
pub async fn proxy_pool_list_strategies() -> IpcResponse<Vec<String>> {
    ipc::ok(
        pool::VALID_STRATEGIES
            .iter()
            .map(|s| s.to_string())
            .collect(),
    )
}
