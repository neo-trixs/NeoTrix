//! Market Commands — 市场管理 Tauri 命令
//!
//! 提供市场搜索、安装、卸载等功能。

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::market::{MarketConfig, MarketEngine};
use crate::market::schema::{MarketEntry, PluginManifest};

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
        engine.load_installed()?;
        *guard = Some(engine);
    }

    Ok(guard)
}

// ═══════════════════════════════════════════════
// Tauri Commands
// ═══════════════════════════════════════════════

/// 获取市场状态
#[tauri::command]
pub async fn market_status() -> Result<MarketStatus, String> {
    let engine = get_engine().await?;
    let installed = engine.as_ref().map(|e| e.list_installed().len()).unwrap_or(0);

    Ok(MarketStatus {
        dsh_enabled: true,
        github_enabled: true,
        installed_count: installed,
        cache_dir: dirs::home_dir()
            .unwrap_or_default()
            .join(".neotrix")
            .join("market")
            .join("cache")
            .to_string_lossy()
            .to_string(),
        plugin_dir: dirs::home_dir()
            .unwrap_or_default()
            .join(".neotrix")
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
) -> Result<Vec<MarketSearchResponse>, String> {
    let engine = get_engine().await?;
    let engine = engine.as_ref().ok_or("Market engine not initialized")?;

    let results = engine
        .search(
            &query,
            category.as_deref(),
            page.unwrap_or(1),
            per_page.unwrap_or(20),
        )
        .await?;

    Ok(results
        .into_iter()
        .map(|r| MarketSearchResponse {
            entries: r.entries,
            total: r.total,
            source: r.source,
        })
        .collect())
}

/// 获取插件详情
#[tauri::command]
pub async fn market_get_detail(
    plugin_id: String,
    source: String,
) -> Result<MarketEntry, String> {
    let engine = get_engine().await?;
    let engine = engine.as_ref().ok_or("Market engine not initialized")?;

    engine.get_detail(&plugin_id, &source).await
}

/// 下载插件
#[tauri::command]
pub async fn market_download(
    plugin_id: String,
    source: String,
    version: String,
) -> Result<String, String> {
    let engine = get_engine().await?;
    let engine = engine.as_ref().ok_or("Market engine not initialized")?;

    let path = engine.download(&plugin_id, &source, &version).await?;
    Ok(path.to_string_lossy().to_string())
}

/// 安装插件
#[tauri::command]
pub async fn market_install(
    plugin_id: String,
    source: String,
    version: String,
) -> Result<PluginManifest, String> {
    let mut engine_guard = get_engine().await?;

    // 下载
    let cache_dir = dirs::home_dir()
        .unwrap_or_default()
        .join(".neotrix")
        .join("market")
        .join("cache");

    let download_path = {
        let engine = engine_guard.as_ref().ok_or("Market engine not initialized")?;
        engine.download(&plugin_id, &source, &version).await?
    };

    // 解析 manifest
    let manifest = if download_path.is_dir() {
        let toml_path = download_path.join("plugin.toml");
        if toml_path.exists() {
            PluginManifest::from_file(&toml_path)?
        } else {
            let json_path = download_path.join("manifest.json");
            if json_path.exists() {
                PluginManifest::from_json_file(&json_path)?
            } else {
                return Err("No manifest found in downloaded package".into());
            }
        }
    } else {
        // 尝试解压 zip
        // TODO: 实现 zip 解压
        return Err("Zip extraction not implemented yet".into());
    };

    // 安装
    if let Some(ref mut engine) = *engine_guard {
        engine.install(manifest.clone(), &download_path).await?;
    }

    Ok(manifest)
}

/// 卸载插件
#[tauri::command]
pub async fn market_uninstall(plugin_id: String) -> Result<bool, String> {
    let mut engine_guard = get_engine().await?;
    let engine = engine_guard.as_mut().ok_or("Market engine not initialized")?;

    engine.uninstall(&plugin_id)?;
    Ok(true)
}

/// 获取已安装插件列表
#[tauri::command]
pub async fn market_list_installed() -> Result<Vec<PluginManifest>, String> {
    let engine = get_engine().await?;
    let engine = engine.as_ref().ok_or("Market engine not initialized")?;

    Ok(engine.list_installed().into_iter().cloned().collect())
}

/// 检查更新
#[tauri::command]
pub async fn market_check_updates() -> Result<Vec<(String, String, String)>, String> {
    let engine = get_engine().await?;
    let engine = engine.as_ref().ok_or("Market engine not initialized")?;

    engine.check_updates().await
}

/// 设置市场配置
#[tauri::command]
pub async fn market_config(
    dsh_enabled: Option<bool>,
    dsh_api_endpoint: Option<String>,
    dsh_auth_token: Option<String>,
    github_enabled: Option<bool>,
    github_token: Option<String>,
) -> Result<MarketStatus, String> {
    // TODO: 更新配置并重新初始化引擎
    market_status().await
}
