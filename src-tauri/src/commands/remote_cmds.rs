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

fn load_hosts() -> Result<Vec<RemoteHostConfig>, String> {
    let path = hosts_path();
    if !std::path::Path::new(&path).exists() {
        return Ok(Vec::new());
    }
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read remote hosts file {}: {}", path, e))?;
    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse remote hosts file {}: {}", path, e))
}

fn save_hosts(hosts: &[RemoteHostConfig]) -> Result<(), String> {
    let path = hosts_path();
    if let Some(parent) = std::path::Path::new(&path).parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create {}: {}", parent.display(), e))?;
    }
    let json = serde_json::to_string_pretty(hosts)
        .map_err(|e| format!("Failed to serialize remote hosts: {}", e))?;
    std::fs::write(&path, json)
        .map_err(|e| format!("Failed to write remote hosts file {}: {}", path, e))?;
    Ok(())
}

/// 远程命令白名单校验：仅允许单条简单命令 + 参数数组。
/// ssh 会把 command 交给远端 shell 解析，因此禁止一切 shell 元字符
/// （命令链/重定向/管道/子shell/展开/后台符/引号/通配符），
/// 且命令本身必须在白名单内。黑名单可被 `\t`/反斜杠/`&` 绕过，白名单不可。
pub(crate) fn validate_remote_command(command: &str) -> Result<(), String> {
    let trimmed = command.trim();
    if trimmed.is_empty() {
        return Err("Empty command".into());
    }
    // 禁止 shell 元字符集合（含空白变体绕过）
    for (pat, desc) in [
        (";", "command chaining (;)"),
        ("&&", "command chaining (&&)"),
        ("||", "command chaining (||)"),
        ("|", "pipe (|)"),
        (">", "redirection (>)"),
        ("<", "redirection (<)"),
        ("`", "backtick substitution"),
        ("$", "variable/command substitution ($)"),
        ("\n", "newline in command"),
        ("\t", "tab in command"),
        ("&", "background operator (&)"),
        ("\\", "escape/backslash"),
        ("*", "glob wildcard (*)"),
        ("?", "glob wildcard (?)"),
        ("[", "glob wildcard ([)"),
        ("{", "brace expansion ({)"),
        ("(", "subshell (()"),
        ("!", "history expansion (!)"),
        ("~", "tilde expansion (~)"),
        ("'", "single quote"),
        ("\"", "double quote"),
    ] {
        if trimmed.contains(pat) {
            return Err(format!(
                "Command rejected: {} not allowed in remote commands",
                desc
            ));
        }
    }
    // 命令必须在白名单内（运维常用只读/诊断命令）
    let first = trimmed.split_whitespace().next().unwrap_or("");
    const ALLOWED: &[&str] = &[
        "ls", "pwd", "whoami", "uname", "date", "uptime", "df", "du", "free",
        "ps", "cat", "head", "tail", "grep", "find", "stat", "id", "hostname",
        "echo", "wc", "which", "env", "netstat", "ss", "ip", "ifconfig",
        "curl", "wget", "git", "docker", "systemctl", "service", "journalctl",
    ];
    if !ALLOWED.contains(&first) {
        return Err(format!(
            "Command rejected: '{}' is not in the allowed command whitelist",
            first
        ));
    }
    Ok(())
}

#[tauri::command]
pub fn list_remote_hosts() -> Result<Vec<RemoteHostConfig>, String> {
    load_hosts()
}

#[tauri::command]
pub fn add_remote_host(name: String, host: String, port: u16, user: String, auth_method: String, key_path: Option<String>) -> Result<RemoteHostConfig, String> {
    let mut hosts = load_hosts()?;
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
    save_hosts(&hosts)?;
    Ok(new_host)
}

#[tauri::command]
pub fn remove_remote_host(id: String) -> Result<(), String> {
    let mut hosts = load_hosts()?;
    hosts.retain(|h| h.id != id);
    save_hosts(&hosts)
}

#[tauri::command]
pub fn test_remote_connection(id: String) -> Result<String, String> {
    let hosts = load_hosts()?;
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
    // 命令白名单校验：拒绝危险 shell 模式（命令链/重定向/管道/提权）
    validate_remote_command(&command)?;

    let hosts = load_hosts()?;
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
