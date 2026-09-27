//! `nt_cmd_core` — 晶体配对/状态/重载/能力/agent（灵魂面）.

use super::{NeobotCoreStatus, NeobotFreeItem, load_config, open_store};
use neotrix_neobot::{AgentRunResult, CoreStatus};
/// 免费发现 / 一键配对（缺 key 拒绝）

#[tauri::command]
pub fn neobot_core_free() -> Result<Vec<NeobotFreeItem>, String> {
    Ok(neotrix_neobot::discover_free()
        .into_iter()
        .map(|e| {
            let key_ok = e.key_env.is_empty()
                || std::env::var(e.key_env)
                    .map(|v| !v.trim().is_empty())
                    .unwrap_or(false);
            let via = match e.via {
                neotrix_neobot::FreeVia::Http => "http".to_owned(),
                neotrix_neobot::FreeVia::OpencodeCli => "cli".to_owned(),
            };
            NeobotFreeItem {
                provider: e.provider.to_owned(),
                model_id: e.model_id,
                display: e.display,
                key_env: e.key_env.to_owned(),
                key_ok,
                via,
            }
        })
        .collect())
}
/// 免费发现 / 一键配对（缺 key 拒绝）

#[tauri::command]
pub fn neobot_core_pair_free(model_id: String) -> Result<String, String> {
    let config = load_config()?;
    let store = open_store(&config)?;
    let found = neotrix_neobot::discover_free();
    let entry = found.iter().find(|e| e.model_id == model_id).ok_or_else(|| {
        format!("free model '{model_id}' not in discover list")
    })?;
    let (pair, count) = neotrix_neobot::pair_free(&store, entry).map_err(|err| err.to_string())?;
    Ok(format!(
        "soul embedded via {} ({} models, {})",
        entry.provider, count, pair.model
    ))
}
/// 配对行 + 活探 + capabilities 版本/工具数

#[tauri::command]
pub fn neobot_core_status() -> Result<NeobotCoreStatus, String> {    let config = load_config()?;
    let store = open_store(&config)?;
    match neotrix_neobot::core_status(&store).map_err(|err| err.to_string())? {
        CoreStatus::Unpaired => Ok(NeobotCoreStatus {
            paired: false,
            online: false,
            base_url: String::new(),
            model: String::new(),
            models: 0,
            latency_ms: -1,
            detail: "未嵌入（纯本地 App）".to_owned(),
            crystal_version: String::new(),
            tool_count: 0,
        }),
        CoreStatus::Online { pair, models, latency_ms, crystal_version, tool_count } => Ok(NeobotCoreStatus {
            paired: true,
            online: true,
            base_url: pair.base_url,
            model: pair.model,
            models,
            latency_ms,
            detail: format!("灵魂在线（{models} 模型，{latency_ms}ms）"),
            crystal_version,
            tool_count,
        }),
        CoreStatus::Offline { pair, reason } => Ok(NeobotCoreStatus {
            paired: true,
            online: false,
            base_url: pair.base_url,
            model: pair.model,
            models: 0,
            latency_ms: -1,
            detail: format!("灵魂离线（{}）", reason.chars().take(60).collect::<String>()),
            crystal_version: String::new(),
            tool_count: 0,
        }),
    }
}
/// POST /v1/admin/reload

#[tauri::command]
pub fn neobot_core_reload() -> Result<String, String> {
    let config = load_config()?;
    let store = open_store(&config)?;
    neotrix_neobot::core_reload(&store).map_err(|err| err.to_string())
}
/// GET /v1/capabilities 原始透传

#[tauri::command]
pub fn neobot_core_capabilities() -> Result<serde_json::Value, String> {
    let config = load_config()?;
    let store = open_store(&config)?;
    let pair = store
        .get_core_pair()
        .map_err(|err| err.to_string())?
        .ok_or_else(|| "unpaired".to_owned())?;
    neotrix_neobot::fetch_capabilities_raw(&pair.base_url, &pair.token_env)
        .map_err(|err| err.to_string())
}
/// POST /v1/agents/run（token 现读）

#[tauri::command]
pub async fn neobot_agent_run(
    goal: String,
    context: Option<String>,
) -> Result<AgentRunResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let config = load_config()?;
        let store = open_store(&config)?;
        neotrix_neobot::agent_run(&store, &goal, context.as_deref()).map_err(|err| err.to_string())
    })
    .await
    .map_err(|err| err.to_string())?
}
