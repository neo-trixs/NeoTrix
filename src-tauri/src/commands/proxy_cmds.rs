use tauri::command;
use super::ProxyStatus;

#[cfg(feature = "stealth-net")]
use neotrix::neotrix::nt_shield_stealth_net::proxy_control::{ProxyClient, DaemonMode};
#[cfg(feature = "stealth-net")]
use std::path::PathBuf;

#[cfg(feature = "stealth-net")]
fn resolve_daemon_path() -> Result<PathBuf, String> {
    if let Ok(path) = std::env::var("NEOTRIX_PROXY_DAEMON_PATH") {
        let p = PathBuf::from(&path);
        if p.exists() {
            return Ok(p);
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let candidate = parent.join("neotrix-proxy-daemon");
            if candidate.exists() {
                return Ok(candidate);
            }
        }
    }
    Err("neotrix-proxy-daemon not found — set NEOTRIX_PROXY_DAEMON_PATH".into())
}

#[cfg(feature = "stealth-net")]
#[command]
pub async fn proxy_status() -> Result<ProxyStatus, String> {
    let client = ProxyClient::new();
    if !client.is_reachable().await {
        return Ok(ProxyStatus::default());
    }
    let body = client.status().await.map_err(|e| format!("status req: {}", e))?;
    let v: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| format!("parse status: {}", e))?;
    Ok(ProxyStatus {
        running: true,
        mode: v["mode"].as_str().unwrap_or("off").to_string(),
        pid: v["pid"].as_u64().unwrap_or(0) as u32,
        port: v["port"].as_u64().unwrap_or(11080) as u16,
        uptime_secs: v["uptime_secs"].as_u64().unwrap_or(0),
        active_count: v["active_count"].as_u64().unwrap_or(0),
        idle_secs: v["idle_secs"].as_u64().unwrap_or(0),
    })
}

#[cfg(not(feature = "stealth-net"))]
#[command]
#[allow(dead_code)]
pub async fn proxy_status() -> Result<ProxyStatus, String> {
    Ok(ProxyStatus::default())
}

#[cfg(feature = "stealth-net")]
#[command]
pub async fn proxy_set_mode(mode: String) -> Result<String, String> {
    let dm = DaemonMode::from_str(&mode)
        .ok_or_else(|| format!("Invalid mode: {}. Valid: geo, stealth, tor, off", mode))?;
    let client = ProxyClient::new();
    if !client.is_reachable().await {
        return Err("Proxy daemon not running".into());
    }
    client.set_mode(dm).await.map_err(|e| format!("set mode: {}", e))?;
    Ok("ok".into())
}

#[cfg(not(feature = "stealth-net"))]
#[command]
#[allow(dead_code)]
pub async fn proxy_set_mode(mode: String) -> Result<String, String> {
    match mode.as_str() {
        "geo" | "stealth" | "tor" | "off" => Ok("ok".into()),
        _ => Err(format!("Invalid mode: {}. Valid: geo, stealth, tor, off", mode)),
    }
}

#[cfg(feature = "stealth-net")]
#[command]
pub async fn proxy_start_daemon() -> Result<String, String> {
    if ProxyClient::new().is_reachable().await {
        return Ok("already running".into());
    }
    let daemon_path = resolve_daemon_path()?;
    let mut child = std::process::Command::new(&daemon_path)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| format!("spawn daemon: {}", e))?;
    for _ in 0..25 {
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        if ProxyClient::new().is_reachable().await {
            return Ok("started".into());
        }
    }
    let _ = child.kill();
    Err("Daemon failed to start within 5s".into())
}

#[cfg(not(feature = "stealth-net"))]
fn resolve_daemon_path() -> Result<std::path::PathBuf, String> {
    if let Ok(path) = std::env::var("NEOTRIX_PROXY_DAEMON_PATH") {
        let p = std::path::PathBuf::from(&path);
        if p.exists() {
            return Ok(p);
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let candidate = parent.join("neotrix-proxy-daemon");
            if candidate.exists() {
                return Ok(candidate);
            }
        }
    }
    Err("neotrix-proxy-daemon not found — set NEOTRIX_PROXY_DAEMON_PATH".into())
}

#[cfg(not(feature = "stealth-net"))]
#[command]
#[allow(dead_code)]
pub async fn proxy_start_daemon() -> Result<String, String> {
    let daemon_path = match resolve_daemon_path() {
        Ok(p) => p,
        Err(e) => return Err(e),
    };
    match std::process::Command::new(&daemon_path)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
    {
        Ok(_) => Ok("started".into()),
        Err(e) => Err(format!("spawn daemon: {}", e)),
    }
}

#[cfg(feature = "stealth-net")]
#[command]
pub async fn proxy_stop_daemon() -> Result<String, String> {
    let client = ProxyClient::new();
    if client.is_reachable().await {
        let _ = client.shutdown().await;
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }
    if ProxyClient::new().is_reachable().await {
        let _ = std::process::Command::new("pkill")
            .arg("-f").arg("neotrix-proxy-daemon").output();
    }
    Ok("stopped".into())
}

#[cfg(not(feature = "stealth-net"))]
#[command]
#[allow(dead_code)]
pub async fn proxy_stop_daemon() -> Result<String, String> {
    let _ = std::process::Command::new("pkill")
        .arg("-f").arg("neotrix-proxy-daemon").output();
    Ok("stopped".into())
}
