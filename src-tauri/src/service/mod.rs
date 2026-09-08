use std::time::Duration;
use tauri::AppHandle;
use tauri::Manager;
use tauri::Emitter;
use tokio::time;

pub fn start(app_handle: &AppHandle) {
    tracing::info!("Starting NeoTrix background scheduler");
    let app_handle_clone = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        scheduler_loop(app_handle_clone).await;
    });
}

async fn scheduler_loop(app_handle: AppHandle) {
    let mut interval = time::interval(Duration::from_secs(30));
    loop {
        interval.tick().await;
        if let Err(e) = tick_health_check(&app_handle).await {
            tracing::warn!("Health check failed: {e}");
        }
    }
}

async fn tick_health_check(app_handle: &AppHandle) -> Result<(), String> {
    // Check if neotrix CLI is available
    let bin = super::commands::neotrix_cli::find_neotrix_binary();
    if bin.is_none() {
        tracing::debug!("neotrix CLI not found, skipping health check");
        return Ok(());
    }

    // Emit health status to frontend
    if let Some(window) = app_handle.get_webview_window("main") {
        let _ = window.emit("health-check", serde_json::json!({
            "status": "ok",
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }));
    }

    Ok(())
}
