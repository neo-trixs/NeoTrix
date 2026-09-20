//! Market Commands — 市场管理 Tauri 命令
//!
//! 提供市场搜索、安装、卸载等功能。

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::ipc::{self, IpcResponse};
use crate::market::schema::{MarketEntry, PluginManifest};
use crate::market::{MarketConfig, MarketEngine};

/// 市场状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketStatus {
    pub dsh_enabled: bool,
    pub github_enabled: bool,
    pub installed_count: usize,
    pub cache_dir: String,
    pub plugin_dir: String,
}

/// 搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketSearchResponse {
    pub entries: Vec<MarketEntry>,
    pub total: usize,
    pub source: String,
}

/// 市场引擎状态 (全局)
static MARKET_ENGINE: once_cell::sync::Lazy<Arc<Mutex<Option<MarketEngine>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(None)));

/// 初始化市场引擎
async fn get_engine() -> Result<tokio::sync::MutexGuard<'static, Option<MarketEngine>>, String> {
    let mut guard = MARKET_ENGINE.lock().await;

    if guard.is_none() {
        let config = MarketConfig::default();
        let mut engine = MarketEngine::new(config);
        engine.load_installed().map_err(|e| e.to_string())?;
        *guard = Some(engine);
    }

    Ok(guard)
}

// ═══════════════════════════════════════════════
// Tauri Commands
// ═══════════════════════════════════════════════

/// 获取市场状态
#[tauri::command]
pub async fn market_status() -> IpcResponse<MarketStatus> {
    let engine = match get_engine().await {
        Ok(g) => g,
        Err(e) => return ipc::err("ENGINE_INIT", e),
    };
    let installed = engine
        .as_ref()
        .map(|e| e.list_installed().len())
        .unwrap_or(0);

    ipc::ok(MarketStatus {
        dsh_enabled: true,
        github_enabled: true,
        installed_count: installed,
        cache_dir: crate::config::AppConfig::base_dir()
            .unwrap_or_default()
            .join("market")
            .join("cache")
            .to_string_lossy()
            .to_string(),
        plugin_dir: crate::config::AppConfig::base_dir()
            .unwrap_or_default()
            .join("plugins")
            .to_string_lossy()
            .to_string(),
    })
}

/// 搜索插件
#[tauri::command]
pub async fn market_search(
    query: String,
    category: Option<String>,
    page: Option<usize>,
    per_page: Option<usize>,
) -> IpcResponse<Vec<MarketSearchResponse>> {
    let engine = match get_engine().await {
        Ok(g) => g,
        Err(e) => return ipc::err("ENGINE_INIT", e),
    };
    let engine = match engine.as_ref() {
        Some(e) => e,
        None => return ipc::err("ENGINE_NOT_READY", "Market engine not initialized"),
    };

    let results = match engine
        .search(
            &query,
            category.as_deref(),
            page.unwrap_or(1),
            per_page.unwrap_or(20),
        )
        .await
    {
        Ok(r) => r,
        Err(e) => return ipc::err("SEARCH_FAILED", format!("{e}")),
    };

    ipc::ok(
        results
            .into_iter()
            .map(|r| MarketSearchResponse {
                entries: r.entries,
                total: r.total,
                source: r.source,
            })
            .collect(),
    )
}

/// 获取插件详情
#[tauri::command]
pub async fn market_get_detail(plugin_id: String, source: String) -> IpcResponse<MarketEntry> {
    let engine = match get_engine().await {
        Ok(g) => g,
        Err(e) => return ipc::err("ENGINE_INIT", e),
    };
    let engine = match engine.as_ref() {
        Some(e) => e,
        None => return ipc::err("ENGINE_NOT_READY", "Market engine not initialized"),
    };

    match engine.get_detail(&plugin_id, &source).await {
        Ok(entry) => ipc::ok(entry),
        Err(e) => ipc::err("DETAIL_FAILED", format!("{e}")),
    }
}

/// 下载插件
#[tauri::command]
pub async fn market_download(
    plugin_id: String,
    source: String,
    version: String,
) -> IpcResponse<String> {
    let engine = match get_engine().await {
        Ok(g) => g,
        Err(e) => return ipc::err("ENGINE_INIT", e),
    };
    let engine = match engine.as_ref() {
        Some(e) => e,
        None => return ipc::err("ENGINE_NOT_READY", "Market engine not initialized"),
    };

    match engine.download(&plugin_id, &source, &version).await {
        Ok(path) => ipc::ok(path.to_string_lossy().to_string()),
        Err(e) => ipc::err("DOWNLOAD_FAILED", format!("{e}")),
    }
}

/// 安装插件
#[tauri::command]
pub async fn market_install(
    plugin_id: String,
    source: String,
    version: String,
) -> IpcResponse<PluginManifest> {
    let mut engine_guard = match get_engine().await {
        Ok(g) => g,
        Err(e) => return ipc::err("ENGINE_INIT", e),
    };

    // 下载
    let cache_dir = crate::config::AppConfig::base_dir()
        .unwrap_or_default()
        .join("market")
        .join("cache");

    let download_path = {
        let engine = match engine_guard.as_ref() {
            Some(e) => e,
            None => return ipc::err("ENGINE_NOT_READY", "Market engine not initialized"),
        };
        match engine.download(&plugin_id, &source, &version).await {
            Ok(p) => p,
            Err(e) => return ipc::err("DOWNLOAD_FAILED", e),
        }
    };

    // 解析 manifest
    let manifest = if download_path.is_dir() {
        let toml_path = download_path.join("plugin.toml");
        if toml_path.exists() {
            match PluginManifest::from_file(&toml_path) {
                Ok(m) => m,
                Err(e) => return ipc::err("MANIFEST_PARSE", e),
            }
        } else {
            let json_path = download_path.join("manifest.json");
            if json_path.exists() {
                match PluginManifest::from_json_file(&json_path) {
                    Ok(m) => m,
                    Err(e) => return ipc::err("MANIFEST_PARSE", e),
                }
            } else {
                return ipc::err("NO_MANIFEST", "No manifest found in downloaded package");
            }
        }
    } else {
        // 尝试解压 zip
        // TODO: 实现 zip 解压
        return ipc::err("NOT_IMPLEMENTED", "Zip extraction not implemented yet");
    };

    // 安装
    if let Some(ref mut engine) = *engine_guard {
        if let Err(e) = engine.install(manifest.clone(), &download_path).await {
            return ipc::err("INSTALL_FAILED", format!("{e}"));
        }
    }

    ipc::ok(manifest)
}

/// 卸载插件
#[tauri::command]
pub async fn market_uninstall(plugin_id: String) -> IpcResponse<bool> {
    let mut engine_guard = match get_engine().await {
        Ok(g) => g,
        Err(e) => return ipc::err("ENGINE_INIT", e),
    };
    let engine = match engine_guard.as_mut() {
        Some(e) => e,
        None => return ipc::err("ENGINE_NOT_READY", "Market engine not initialized"),
    };

    match engine.uninstall(&plugin_id) {
        Ok(()) => ipc::ok(true),
        Err(e) => ipc::err("UNINSTALL_FAILED", format!("{e}")),
    }
}

/// 获取已安装插件列表
#[tauri::command]
pub async fn market_list_installed() -> IpcResponse<Vec<PluginManifest>> {
    let engine = match get_engine().await {
        Ok(g) => g,
        Err(e) => return ipc::err("ENGINE_INIT", e),
    };
    let engine = match engine.as_ref() {
        Some(e) => e,
        None => return ipc::err("ENGINE_NOT_READY", "Market engine not initialized"),
    };

    ipc::ok(engine.list_installed().into_iter().cloned().collect())
}

/// 检查更新
#[tauri::command]
pub async fn market_check_updates() -> IpcResponse<Vec<(String, String, String)>> {
    let engine = match get_engine().await {
        Ok(g) => g,
        Err(e) => return ipc::err("ENGINE_INIT", e),
    };
    let engine = match engine.as_ref() {
        Some(e) => e,
        None => return ipc::err("ENGINE_NOT_READY", "Market engine not initialized"),
    };

    match engine.check_updates().await {
        Ok(updates) => ipc::ok(updates),
        Err(e) => ipc::err("CHECK_FAILED", format!("{e}")),
    }
}

/// 设置市场配置
#[tauri::command]
pub async fn market_config(
    dsh_enabled: Option<bool>,
    dsh_api_endpoint: Option<String>,
    dsh_auth_token: Option<String>,
    github_enabled: Option<bool>,
    github_token: Option<String>,
) -> IpcResponse<MarketStatus> {
    // TODO: 更新配置并重新初始化引擎
    market_status().await
}
