use tauri::{command, Emitter};
use tauri_plugin_notification::NotificationExt;
use neotrix::neotrix::nt_io_provider::{create_provider, LlmRequest};
use super::ProviderConfigPayload;
use super::agent_cmds::{read_provider_config, payload_to_provider_config};

#[command]
pub async fn test_provider(config: ProviderConfigPayload) -> Result<String, String> {
    if config.api_key.is_empty() || config.model.is_empty() {
        return Err("API Key 和模型不能为空".into());
    }
    let provider_config = payload_to_provider_config(&config);
    let provider = create_provider(provider_config);
    let request = LlmRequest::new(&config.model, "Hello");
    provider.complete(&request).await
        .map(|_| "ok".into())
        .map_err(|e| format!("测试失败: {}", e))
}

#[command]
pub fn save_provider_config(config: ProviderConfigPayload) -> Result<String, String> {
    let path = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("neotrix")
        .join("provider.json");
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let json = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())?;
    Ok("saved".into())
}

#[command]
pub fn send_notification(app: tauri::AppHandle, title: String, body: String) -> Result<(), String> {
    log::info!("[notification] {}: {}", title, body);
    app.emit("task-complete", serde_json::json!({
        "title": &title,
        "body": &body,
    })).map_err(|e| format!("emit error: {}", e))?;
    app.notification()
        .builder()
        .title(&title)
        .body(&body)
        .show()
        .map_err(|e| format!("notification error: {}", e))?;
    Ok(())
}

#[command]
pub async fn execute_terminal_command(command: String) -> Result<String, String> {
    let output = tokio::process::Command::new("sh")
        .arg("-c")
        .arg(&command)
        .output()
        .await
        .map_err(|e| format!("命令执行失败: {}", e))?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let combined = if stderr.is_empty() { stdout } else { format!("{}\n{}", stdout, stderr) };
    Ok(combined)
}

#[command]
pub async fn cli_command(input: String) -> Result<String, String> {
    execute_terminal_command(input).await
}
