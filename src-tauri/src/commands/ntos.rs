//! ntos 壳窗口 — neotrix 晶体的第二切面。
//!
//! 同一进程、同一 IPC 内核（domain_call＋ntcode 全可用），
//! 与主控台共享 localStorage（同源），nt_username/nt_model_tier 互通。
//! 前端由 ntos 包构建至 `frontend/dist/ntos`（相对路径资源）。

use tauri::{AppHandle, Manager, WebviewUrl};

/// 确保 ntos 窗口存在（存在则聚焦），供 command 与 CLI 共用。
pub fn ensure_ntos_window(app: &AppHandle) -> Result<(), String> {
    if let Some(w) = app.get_webview_window("ntos") {
        w.set_focus().map_err(|e| e.to_string())?;
        tracing::info!("ntos 壳窗口已聚焦");
        return Ok(());
    }
    tauri::WebviewWindowBuilder::new(app, "ntos", WebviewUrl::App("ntos/index.html".into()))
        .title("ntos")
        .inner_size(1200.0, 800.0)
        .min_inner_size(900.0, 600.0)
        .center()
        .build()
        .map_err(|e| e.to_string())?;
    tracing::info!("ntos 壳窗口已创建（ntos/index.html，共享 neotrix 内核 IPC）");
    Ok(())
}

/// 打开（或聚焦）ntos 壳窗口。前端经 `open_ntos_window` 调用。
#[tauri::command]
pub async fn open_ntos_window(app: AppHandle) -> Result<(), String> {
    ensure_ntos_window(&app)
}
