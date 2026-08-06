use serde::Serialize;
use tauri::State;
use std::sync::Arc;

use neotrix::neotrix::nt_core_error::NeoTrixError;
use neotrix::neotrix::nt_shield::permissions::{PermissionAction, PermissionManager};
use super::mcp_cmds::guard_shell_command;

const REMOTE_HOSTS_FILE: &str = ".neotrix/remote-hosts.json";
const REMOTE_MAX_LEN: usize = 2048;

#[derive(Serialize, serde::Deserialize, Clone, Debug)]
pub struct RemoteHostConfig {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub user: String,
    pub auth_method: String,
    pub key_path: Option<String>,
}

fn hosts_path() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    format!("{}/{}", home, REMOTE_HOSTS_FILE)
}

fn load_hosts() -> Vec<RemoteHostConfig> {
    let path = hosts_path();
    if !std::path::Path::new(&path).exists() {
        return Vec::new();
    }
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|c| serde_json::from_str(&c).ok())
        .unwrap_or_default()
}

fn save_hosts(hosts: &[RemoteHostConfig]) {
    let path = hosts_path();
    if let Some(parent) = std::path::Path::new(&path).parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(hosts) {
        // P1-3: hosts 文件含 ssh 目标信息（host/user/key_path 引用），0o600 收紧权限
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            let mut opts = std::fs::OpenOptions::new();
            opts.write(true).create(true).truncate(true).mode(0o600);
            if let Ok(mut f) = opts.open(&path) {
                let _ = std::io::Write::write_all(&mut f, json.as_bytes());
                return;
            }
        }
        #[cfg(not(unix))]
        let _ = std::fs::write(&path, json);
    }
}

/// P1-3: ssh 目标串 user@host 走 argv（无 shell 注入面），但需拒绝以 `-` 开头的
/// 参数注入（ssh 会把 `-o` 之类当选项解析）与含空白/控制字符的 host/user。
fn validate_ssh_target(host: &str, user: &str) -> Result<(), NeoTrixError> {
    if host.trim().is_empty() || user.trim().is_empty() {
        return Err(NeoTrixError::Config("host 和 user 不能为空".into()));
    }
    for field in [host, user] {
        let t = field.trim();
        if t.starts_with('-') {
            return Err(NeoTrixError::Shield("host/user 不允许以 '-' 开头".into()));
        }
        if t.chars().any(|c| c.is_whitespace() || c.is_control()) {
            return Err(NeoTrixError::Shield("host/user 不允许包含空白或控制字符".into()));
        }
    }
    Ok(())
}

#[tauri::command]
pub fn list_remote_hosts() -> Vec<RemoteHostConfig> {
    load_hosts()
}

#[tauri::command]
pub fn add_remote_host(name: String, host: String, port: u16, user: String, auth_method: String, key_path: Option<String>) -> Result<RemoteHostConfig, String> {
    validate_ssh_target(&host, &user).map_err(|e| e.to_string())?;
    let mut hosts = load_hosts();
    let new_host = RemoteHostConfig {
        id: format!("host-{}", hosts.len() + 1),
        name,
        host,
        port,
        user,
        auth_method,
        key_path,
    };
    hosts.push(new_host.clone());
    save_hosts(&hosts);
    Ok(new_host)
}

#[tauri::command]
pub fn remove_remote_host(id: String) -> Result<(), String> {
    let mut hosts = load_hosts();
    hosts.retain(|h| h.id != id);
    save_hosts(&hosts);
    Ok(())
}

#[tauri::command]
pub fn test_remote_connection(id: String) -> Result<String, String> {
    let hosts = load_hosts();
    let host = hosts.iter().find(|h| h.id == id)
        .ok_or_else(|| format!("Host {} not found", id))?;
    validate_ssh_target(&host.host, &host.user).map_err(|e| e.to_string())?;

    let output = std::process::Command::new("ssh")
        .args([
            "-o", "ConnectTimeout=5",
            "-o", "BatchMode=yes",
            "-p", &host.port.to_string(),
            &format!("{}@{}", host.user, host.host),
            "echo ok",
        ])
        .output()
        .map_err(|e| format!("SSH connection failed: {}", e))?;

    if output.status.success() {
        Ok(format!("Connected to {} ({})", host.name, host.host))
    } else {
        Err(format!("Connection failed: {}", String::from_utf8_lossy(&output.stderr)))
    }
}

#[tauri::command]
pub fn execute_remote(id: String, command: String, manager: State<'_, Arc<PermissionManager>>) -> Result<String, String> {
    let hosts = load_hosts();
    let host = hosts.iter().find(|h| h.id == id)
        .ok_or_else(|| format!("Host {} not found", id))?;
    validate_ssh_target(&host.host, &host.user).map_err(|e| e.to_string())?;

    // P1-3: 与本地 shell 一致的门禁 — 破坏性/下载即执行/超长命令一律拒绝
    guard_shell_command(&command).map_err(|e| e.to_string())?;
    if command.chars().count() > REMOTE_MAX_LEN {
        return Err(NeoTrixError::Shield(format!(
            "远程命令超过长度限制 {} 字符，已拦截", REMOTE_MAX_LEN
        ))
        .to_string());
    }
    // 审计：远程执行同样经 PermissionManager 落审计日志（目标脱敏，命令原文
    // 只入审计、不回显到错误，避免密钥/敏感命令泄露）。
    manager.record(
        PermissionAction::CommandExec,
        &format!("ssh:{}@{}", host.user, host.host),
        &format!("remote-exec {} chars", command.chars().count()),
    );

    let output = std::process::Command::new("ssh")
        .args([
            "-o", "ConnectTimeout=10",
            "-p", &host.port.to_string(),
            &format!("{}@{}", host.user, host.host),
            &command,
        ])
        .output()
        .map_err(|e| format!("Remote execution failed: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(format!("Execution failed: {}", String::from_utf8_lossy(&output.stderr)))
    }
}
