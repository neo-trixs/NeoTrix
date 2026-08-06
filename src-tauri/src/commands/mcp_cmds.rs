use tauri::{command, Emitter, State};
use tauri_plugin_notification::NotificationExt;
use neotrix::neotrix::nt_core_error::NeoTrixError;
use neotrix::neotrix::nt_io_provider::{create_provider, LlmRequest};
use neotrix::neotrix::nt_shield::permissions::{PermissionAction, PermissionManager};
use std::sync::Arc;
use super::ProviderConfigPayload;
use super::agent_cmds::payload_to_provider_config;

#[command]
pub async fn test_provider(config: ProviderConfigPayload) -> Result<String, NeoTrixError> {
    if config.api_key.is_empty() || config.model.is_empty() {
        return Err(NeoTrixError::Config("API Key 和模型不能为空".into()));
    }
    let provider_config = payload_to_provider_config(&config);
    let provider = create_provider(provider_config);
    let request = LlmRequest::new(&config.model, "Hello");
    provider.complete(&request).await
        .map(|_| "ok".into())
        .map_err(|e| NeoTrixError::Brain(format!("测试失败: {}", e)))
}

#[command]
pub fn save_provider_config(config: ProviderConfigPayload) -> Result<String, NeoTrixError> {
    let path = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("neotrix")
        .join("provider.json");
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    // P1-2: never write the api_key to disk in plaintext. Store it in the OS
    // keyring (keyed per provider), and persist only the non-secret fields.
    if !config.api_key.is_empty() {
        let entry = keyring::Entry::new("neotrix", &format!("provider_key:{}", config.id))
            .map_err(|e| NeoTrixError::Keyring(e.to_string()))?;
        entry.set_password(&config.api_key).map_err(|e| NeoTrixError::Keyring(e.to_string()))?;
    }
    let mut disk = config.clone();
    disk.api_key.clear();
    let json = serde_json::to_string_pretty(&disk).map_err(|e| NeoTrixError::Serde(e.to_string()))?;
    // 不含明文 api_key，但仍 0o600 防御性写盘
    let mut opts = std::fs::OpenOptions::new();
    opts.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
    }
    let mut file = opts.open(&path).map_err(|e| NeoTrixError::Io(e.to_string()))?;
    use std::io::Write;
    file.write_all(json.as_bytes()).map_err(|e| NeoTrixError::Io(e.to_string()))?;
    file.flush().map_err(|e| NeoTrixError::Io(e.to_string()))?;
    Ok("saved".into())
}

/// Restore the api_key for a provider from the OS keyring (P1-2).
pub fn read_provider_key(provider_id: &str) -> Result<String, NeoTrixError> {
    let entry = keyring::Entry::new("neotrix", &format!("provider_key:{}", provider_id))
        .map_err(|e| NeoTrixError::Keyring(e.to_string()))?;
    entry.get_password().map_err(|e| NeoTrixError::Keyring(e.to_string()))
}

/// Delete a provider's api_key from the OS keyring (P1-2).
pub fn delete_provider_key(provider_id: &str) {
    if let Ok(entry) = keyring::Entry::new("neotrix", &format!("provider_key:{}", provider_id)) {
        let _ = entry.delete_credential();
    }
}

#[command]
pub fn send_notification(app: tauri::AppHandle, title: String, body: String) -> Result<(), NeoTrixError> {
    // body 可能含任务输出中的敏感串，降为 debug 并截断
    log::debug!("[notification] {}: {}", title, body.chars().take(200).collect::<String>());
    app.emit("task-complete", serde_json::json!({
        "title": &title,
        "body": &body,
    })).map_err(|e| NeoTrixError::Brain(format!("emit error: {}", e)))?;
    app.notification()
        .builder()
        .title(&title)
        .body(&body)
        .show()
        .map_err(|e| NeoTrixError::Brain(format!("notification error: {}", e)))?;
    Ok(())
}

#[command]
pub async fn execute_terminal_command(
    command: String,
    manager: State<'_, Arc<PermissionManager>>,
) -> Result<String, NeoTrixError> {
    // P0-1 gate: reject destructive/piped-download commands and cap length
    // before reaching `sh -c`. Errors are sanitized — the full command must
    // never be echoed back (may contain secrets).
    guard_shell_command(&command)?;
    // P0-6: every webview-triggered command execution lands in the permission
    // audit log so sensitive actions are traceable.
    manager.record(PermissionAction::CommandExec, &command, "executed");
    let output = tokio::process::Command::new("sh")
        .arg("-c")
        .arg(&command)
        .output()
        .await
        .map_err(|e| NeoTrixError::Command { cmd: "<redacted>".into(), exit_code: None, stderr: e.to_string() })?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let combined = if stderr.is_empty() { stdout } else { format!("{}\n{}", stdout, stderr) };
    Ok(combined)
}

/// Shell command guard (P0-1): hard-rejects destructive patterns, download-then-execute
/// pipes, and over-long commands. Pure function so the same gate guards every `sh -c`
/// entrypoint (execute_terminal_command, cli_command, tool_execute, MCP host).
pub fn guard_shell_command(command: &str) -> Result<(), NeoTrixError> {
    const MAX_LEN: usize = 2048;
    if command.len() > MAX_LEN {
        return Err(NeoTrixError::Shield(format!(
            "命令超过长度限制 {} 字符，已拦截", MAX_LEN
        )));
    }
    let c = command.trim();
    if c.is_empty() {
        return Err(NeoTrixError::Shield("空命令被拦截".into()));
    }
    // Destructive filesystem / system operations — never allow via webview.
    let destructive = [
        r"(^|[\s;&|])rm\s+(-[a-zA-Z]*\s+)*-rf?\s*/",
        r"(^|[\s;&|])mkfs(\s|\.)",
        r"dd\s+if=.*of=/dev/",
        r"(^|[\s;&|])shutdown\b",
        r"(^|[\s;&|])reboot\b",
        r"(^|[\s;&|])halt\b",
        r"chmod\s+(-[a-zA-Z]*\s+)*777\s+/",
        r"chown\s+(-[a-zA-Z]*\s+)*[^\s]+\s+/",
        r":\(\)\{",
        r"(^|[\s;&|])sudo\s+rm\b",
        r">\s*/dev/(sda|sdb|sdc|disk)",
    ];
    for pat in destructive {
        if regex::Regex::new(pat).map(|re| re.is_match(c)).unwrap_or(false) {
            return Err(NeoTrixError::Shield("检测到破坏性命令，已拦截".into()));
        }
    }
    // Download-then-execute pipes: curl|sh, wget|sh, base64 -d | sh, etc.
    let pipe_exec = r"(curl|wget|nc|base64\s+-d)\b.*\|\s*(sh|bash|zsh|python|perl|ruby)\b";
    if regex::Regex::new(pipe_exec).map(|re| re.is_match(c)).unwrap_or(false) {
        return Err(NeoTrixError::Shield("检测到下载后执行管道，已拦截".into()));
    }
    Ok(())
}

#[command]
pub async fn cli_command(input: String) -> Result<String, NeoTrixError> {
    guard_shell_command(&input)?;
    let output = tokio::process::Command::new("sh")
        .arg("-c")
        .arg(&input)
        .output()
        .await
        .map_err(|e| NeoTrixError::Command { cmd: "<redacted>".into(), exit_code: None, stderr: e.to_string() })?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    Ok(if stderr.is_empty() { stdout } else { format!("{}\n{}", stdout, stderr) })
}
