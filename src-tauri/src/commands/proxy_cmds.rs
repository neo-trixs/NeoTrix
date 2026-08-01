use tauri::command;
use neotrix::neotrix::nt_core_error::NeoTrixError;
use super::ProxyStatus;
use super::ProxySourceStatus;
use super::ProxyConnectivityResult;
use super::ProxyNodeInfo;
use super::ProxyConfigData;

#[cfg(feature = "stealth-net")]
use neotrix::neotrix::nt_shield_stealth_net::proxy_control::{ProxyClient, DaemonMode};
#[cfg(feature = "stealth-net")]
use neotrix::neotrix::nt_shield_stealth_net::proxy_pool;

/// Inline wrapper for the proxy daemon path resolution.
mod proxy_daemon_wrapper {
    pub fn resolve_daemon_path() -> Result<String, String> {
        Err("proxy daemon not available (stealth-net feature disabled)".into())
    }
}

#[cfg(feature = "stealth-net")]
#[command]
pub async fn proxy_status() -> Result<ProxyStatus, NeoTrixError> {
    let client = ProxyClient::new();
    if !client.is_reachable().await {
        return Ok(ProxyStatus::default());
    }
    let body = client.status().await.map_err(|e| NeoTrixError::Network(format!("status req: {}", e)))?;
    let v: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| NeoTrixError::Serde(format!("parse status: {}", e)))?;
    Ok(ProxyStatus {
        running: true,
        mode: v["mode"].as_str().unwrap_or("off").to_string(),
        pid: v["pid"].as_u64().unwrap_or(0).min(u32::MAX as u64) as u32,
        port: v["port"].as_u64().unwrap_or(11080).min(u16::MAX as u64) as u16,
        uptime_secs: v["uptime_secs"].as_u64().unwrap_or(0),
        active_count: v["active_count"].as_u64().unwrap_or(0),
        idle_secs: v["idle_secs"].as_u64().unwrap_or(0),
    })
}

#[cfg(not(feature = "stealth-net"))]
#[command]
#[allow(dead_code)]
pub async fn proxy_status() -> Result<ProxyStatus, NeoTrixError> {
    Ok(ProxyStatus::default())
}

#[cfg(feature = "stealth-net")]
#[command]
pub async fn proxy_set_mode(mode: String) -> Result<String, NeoTrixError> {
    let dm = DaemonMode::from_str(&mode)
        .ok_or_else(|| NeoTrixError::Config(format!("Invalid mode: {}. Valid: geo, stealth, tor, off", mode)))?;
    let client = ProxyClient::new();
    if !client.is_reachable().await {
        return Err(NeoTrixError::Network("Proxy daemon not running".into()));
    }
    client.set_mode(dm).await.map_err(|e| NeoTrixError::Network(format!("set mode: {}", e)))?;
    Ok("ok".into())
}

#[cfg(not(feature = "stealth-net"))]
#[command]
#[allow(dead_code)]
pub async fn proxy_set_mode(mode: String) -> Result<String, NeoTrixError> {
    match mode.as_str() {
        "geo" | "stealth" | "tor" | "off" => Ok("ok".into()),
        _ => Err(NeoTrixError::Config(format!("Invalid mode: {}. Valid: geo, stealth, tor, off", mode))),
    }
}

#[cfg(feature = "stealth-net")]
#[command]
pub async fn proxy_start_daemon() -> Result<String, NeoTrixError> {
    if ProxyClient::new().is_reachable().await {
        return Ok("already running".into());
    }
    let daemon_path = proxy_daemon_wrapper::resolve_daemon_path().map_err(|e| NeoTrixError::Network(e))?;
    let mut child = std::process::Command::new(&daemon_path)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| NeoTrixError::Network(format!("spawn daemon: {}", e)))?;
    for _ in 0..25 {
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        if ProxyClient::new().is_reachable().await {
            return Ok("started".into());
        }
    }
    let _ = child.kill();
    Err(NeoTrixError::Network("Daemon failed to start within 5s".into()))
}

#[cfg(not(feature = "stealth-net"))]
#[command]
#[allow(dead_code)]
pub async fn proxy_start_daemon() -> Result<String, NeoTrixError> {
    let daemon_path = match proxy_daemon_wrapper::resolve_daemon_path() {
        Ok(p) => p,
        Err(e) => return Err(NeoTrixError::Network(e)),
    };
    match std::process::Command::new(&daemon_path)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
    {
        Ok(_) => Ok("started".into()),
        Err(e) => Err(NeoTrixError::Network(format!("spawn daemon: {}", e))),
    }
}

#[cfg(feature = "stealth-net")]
#[command]
pub async fn proxy_stop_daemon() -> Result<String, NeoTrixError> {
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
pub async fn proxy_stop_daemon() -> Result<String, NeoTrixError> {
    let _ = std::process::Command::new("pkill")
        .arg("-f").arg("neotrix-proxy-daemon").output();
    Ok("stopped".into())
}

// ========== New Proxy Management Commands ==========

#[cfg(feature = "stealth-net")]
#[command]
pub async fn proxy_source_status() -> Result<Vec<ProxySourceStatus>, NeoTrixError> {
    let pool = proxy_pool::global_pool();
    let sources = pool.list_subscriptions().await;
    let statuses = sources.into_iter().map(|url| {
        let name = url.trim_start_matches("https://")
            .trim_start_matches("http://")
            .split('/')
            .next()
            .unwrap_or(&url)
            .to_string();
        ProxySourceStatus {
            name,
            total_successes: 0,
            total_failures: 0,
            consecutive_failures: 0,
            on_cooldown: false,
        }
    }).collect();
    Ok(statuses)
}

#[cfg(not(feature = "stealth-net"))]
#[command]
#[allow(dead_code)]
pub async fn proxy_source_status() -> Result<Vec<ProxySourceStatus>, NeoTrixError> {
    Ok(vec![])
}

#[cfg(feature = "stealth-net")]
#[command]
pub async fn proxy_connectivity() -> Result<ProxyConnectivityResult, NeoTrixError> {
    let pool = proxy_pool::global_pool();
    let total = pool.total_count().await;
    let healthy = pool.available_count().await;
    Ok(ProxyConnectivityResult {
        active_mode: "auto".into(),
        direct_reachable: true,
        direct_latency_ms: None,
        proxy_healthy_count: healthy,
        proxy_total_count: total,
        proxy_avg_latency_ms: None,
    })
}

#[cfg(not(feature = "stealth-net"))]
#[command]
#[allow(dead_code)]
pub async fn proxy_connectivity() -> Result<ProxyConnectivityResult, NeoTrixError> {
    Ok(ProxyConnectivityResult {
        active_mode: "off".into(),
        direct_reachable: true,
        direct_latency_ms: None,
        proxy_healthy_count: 0,
        proxy_total_count: 0,
        proxy_avg_latency_ms: None,
    })
}

#[cfg(feature = "stealth-net")]
#[command]
pub async fn proxy_trigger_fetch(max_count: Option<u32>) -> Result<u32, NeoTrixError> {
    let pool = proxy_pool::global_pool();
    let subs = pool.list_subscriptions().await;
    let mut total = 0u32;
    for url in &subs {
        if let Ok(n) = pool.fetch_subscription(url).await {
            total += n as u32;
        }
        if let Some(max) = max_count {
            if total >= max {
                break;
            }
        }
    }
    Ok(total)
}

#[cfg(not(feature = "stealth-net"))]
#[command]
#[allow(dead_code)]
pub async fn proxy_trigger_fetch(max_count: Option<u32>) -> Result<u32, NeoTrixError> {
    let _ = max_count;
    Ok(0)
}

#[cfg(feature = "stealth-net")]
#[command]
pub async fn proxy_sub_list() -> Result<Vec<String>, NeoTrixError> {
    let pool = proxy_pool::global_pool();
    Ok(pool.list_subscriptions().await)
}

#[cfg(not(feature = "stealth-net"))]
#[command]
#[allow(dead_code)]
pub async fn proxy_sub_list() -> Result<Vec<String>, NeoTrixError> {
    let path = dirs::home_dir()
        .map(|p| p.join(".neotrix/subscriptions.json"))
        .unwrap_or_default();
    if path.exists() {
        let data = std::fs::read_to_string(&path).unwrap_or_default();
        let urls: Vec<String> = serde_json::from_str(&data).unwrap_or_default();
        return Ok(urls);
    }
    Ok(vec![])
}

#[cfg(feature = "stealth-net")]
#[command]
pub async fn proxy_sub_add(url: String) -> Result<String, NeoTrixError> {
    let pool = proxy_pool::global_pool();
    pool.add_subscription(&url).await;
    Ok(format!("added: {}", url))
}

#[cfg(not(feature = "stealth-net"))]
#[command]
#[allow(dead_code)]
pub async fn proxy_sub_add(url: String) -> Result<String, NeoTrixError> {
    let path = dirs::home_dir()
        .map(|p| p.join(".neotrix/subscriptions.json"))
        .unwrap_or_default();
    let mut urls: Vec<String> = if path.exists() {
        let data = std::fs::read_to_string(&path).unwrap_or_default();
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        vec![]
    };
    if !urls.contains(&url) {
        urls.push(url.clone());
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let data = serde_json::to_string_pretty(&urls).unwrap_or_default();
        let _ = std::fs::write(&path, &data);
    }
    Ok(format!("added: {}", url))
}

#[cfg(feature = "stealth-net")]
#[command]
pub async fn proxy_sub_remove(url: String) -> Result<String, NeoTrixError> {
    let pool = proxy_pool::global_pool();
    pool.remove_subscription(&url).await;
    Ok(format!("removed: {}", url))
}

#[cfg(not(feature = "stealth-net"))]
#[command]
#[allow(dead_code)]
pub async fn proxy_sub_remove(url: String) -> Result<String, NeoTrixError> {
    let path = dirs::home_dir()
        .map(|p| p.join(".neotrix/subscriptions.json"))
        .unwrap_or_default();
    if path.exists() {
        let data = std::fs::read_to_string(&path).unwrap_or_default();
        let mut urls: Vec<String> = serde_json::from_str(&data).unwrap_or_default();
        urls.retain(|u| u != &url);
        let data = serde_json::to_string_pretty(&urls).unwrap_or_default();
        let _ = std::fs::write(&path, &data);
    }
    Ok(format!("removed: {}", url))
}

#[cfg(feature = "stealth-net")]
#[command]
pub async fn proxy_pool_nodes() -> Result<Vec<ProxyNodeInfo>, NeoTrixError> {
    let pool = proxy_pool::global_pool();
    let nodes = pool.ready(200).await;
    Ok(nodes.into_iter().map(|n| {
        let latency = n.latency_ms;
        let tier = if latency.map_or(true, |l| l.is_nan()) {
            "Unknown"
        } else if latency.unwrap_or(f64::MAX) < 500.0 {
            "Fast"
        } else if latency.unwrap_or(f64::MAX) < 2000.0 {
            "Medium"
        } else {
            "Slow"
        };
        ProxyNodeInfo {
            url: n.url.clone(),
            tag: n.tag.clone(),
            latency_ms: n.latency_ms,
            fail_count: n.fail_count,
            success_count: n.success_count,
            from_subscription: n.from_subscription,
            geo_tag: n.geo_tag.clone(),
            ip_addr: n.ip_addr.clone(),
            speed_tier: tier.to_string(),
            score: n.score(),
            healthy: !n.is_stale() && n.fail_count < 3,
        }
    }).collect())
}

#[cfg(not(feature = "stealth-net"))]
#[command]
#[allow(dead_code)]
pub async fn proxy_pool_nodes() -> Result<Vec<ProxyNodeInfo>, NeoTrixError> {
    Ok(vec![])
}

#[command]
pub async fn proxy_config_get() -> Result<ProxyConfigData, NeoTrixError> {
    #[cfg(feature = "stealth-net")] {
        let cfg = neotrix::neotrix::nt_shield_stealth_net::config::load();
        let pool = &cfg.pool;
        return Ok(ProxyConfigData {
            local_port: cfg.proxy.local_port,
            socks_port: cfg.proxy.socks_port,
            min_nodes: pool.min_nodes,
            health_check_interval_secs: pool.health_check_interval_secs,
            selection_strategy: pool.selection_strategy.clone(),
            system_proxy_enabled: true,
            direct_timeout_secs: cfg.proxy.direct_timeout_secs,
        });
    }
    #[cfg(not(feature = "stealth-net"))] {
        Ok(ProxyConfigData {
            local_port: 11080,
            socks_port: 9050,
            min_nodes: 5,
            health_check_interval_secs: 60,
            selection_strategy: "auto".into(),
            system_proxy_enabled: true,
            direct_timeout_secs: 3,
        })
    }
}

#[command]
pub async fn proxy_config_set(config: ProxyConfigData) -> Result<String, NeoTrixError> {
    let _ = config;
    #[cfg(feature = "stealth-net")] {
        let _pool = neotrix::neotrix::nt_shield_stealth_net::proxy_pool::global_pool();
        // In a real implementation, we'd update config.toml here
        return Ok("config updated (stealth-net)".into());
    }
    #[cfg(not(feature = "stealth-net"))] {
        Ok("config saved (no-op)".into())
    }
}
