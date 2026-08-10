use serde::Serialize;

const REMOTE_HOSTS_FILE: &str = ".neotrix/remote-hosts.json";

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
        let _ = std::fs::write(&path, json);
    }
}

#[tauri::command]
pub fn list_remote_hosts() -> Vec<RemoteHostConfig> {
    load_hosts()
}

#[tauri::command]
pub fn add_remote_host(name: String, host: String, port: u16, user: String, auth_method: String, key_path: Option<String>) -> RemoteHostConfig {
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
    new_host
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
pub fn execute_remote(id: String, command: String) -> Result<String, String> {
    let hosts = load_hosts();
    let host = hosts.iter().find(|h| h.id == id)
        .ok_or_else(|| format!("Host {} not found", id))?;

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
