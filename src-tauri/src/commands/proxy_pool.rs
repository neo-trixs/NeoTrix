//! Proxy Pool — 对接 neotrix-core ProxyPool (nt_shield_stealth_net)
//!
//! 提供代理 IP 池的增删查状态查询，前端通过 domain_call 或直接调用。

use anyhow::{Context, Result as AnyhowResult};
use crate::atomic_io;
use crate::config::AppConfig;
use crate::ipc::{self, IpcResponse};
use serde::{Deserialize, Serialize};

/// 代理池条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyPoolEntry {
    pub url: String,
    pub tag: String,
    pub geo_tag: Option<String>,
    pub latency_ms: Option<u64>,
    pub success_count: u64,
    pub fail_count: u64,
    pub speed_tier: String,
    pub from_subscription: bool,
}

/// 代理池状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyPoolStatus {
    pub total: usize,
    pub healthy: usize,
    pub unhealthy: usize,
    pub strategy: String,
    pub nodes: Vec<ProxyPoolEntry>,
    pub subscriptions: Vec<String>,
}

/// 代理池快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyPoolSnapshot {
    pub total: usize,
    pub healthy: usize,
    pub avg_latency_ms: f64,
    pub strategy: String,
    pub geo_distribution: std::collections::HashMap<String, usize>,
    pub speed_tiers: std::collections::HashMap<String, usize>,
}

/// 从 ~/.neotrix/ 读取订阅文件
fn load_subscriptions() -> AnyhowResult<Vec<String>> {
    let path = AppConfig::base_dir()
        .unwrap_or_default()
        .join("subscriptions.json");

    if !path.exists() {
        return Ok(vec![]);
    }

    let content = std::fs::read_to_string(&path).context("Read subscriptions")?;

    serde_json::from_str(&content).context("Parse subscriptions")
}

/// 保存订阅文件
fn save_subscriptions(subs: &[String]) -> AnyhowResult<()> {
    let path = AppConfig::base_dir()
        .unwrap_or_default()
        .join("subscriptions.json");

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).context("Create dir")?;
    }

    atomic_io::write_json_atomic(&path, subs).context("Write subscriptions")
}

/// 读取代理池配置
fn load_pool_config() -> AnyhowResult<serde_json::Value> {
    let path = AppConfig::base_dir()
        .unwrap_or_default()
        .join("config.toml");

    if !path.exists() {
        return Ok(serde_json::json!({
            "pool": {
                "selection_strategy": "adaptive",
                "min_nodes": 10
            }
        }));
    }

    let content = std::fs::read_to_string(&path).context("Read config")?;

    // TOML 转 JSON (简单处理)
    Ok(serde_json::json!({
        "raw": content
    }))
}

/// 保存代理池策略配置
fn save_strategy(strategy: &str) -> AnyhowResult<()> {
    let config_path = AppConfig::base_dir()
        .unwrap_or_default()
        .join("config.toml");

    // 读取现有配置或创建新的
    let mut config_str = if config_path.exists() {
        std::fs::read_to_string(&config_path).unwrap_or_default()
    } else {
        String::new()
    };

    // 更新或插入 strategy
    if config_str.contains("selection_strategy") {
        config_str = config_str
            .lines()
            .map(|line| {
                if line.contains("selection_strategy") {
                    format!("selection_strategy = \"{}\"", strategy)
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
    } else {
        config_str.push_str(&format!(
            "\n[pool]\nselection_strategy = \"{}\"\n",
            strategy
        ));
    }

    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent).context("Create dir")?;
    }

    atomic_io::write_atomic(&config_path, config_str.as_bytes()).context("Write config")
}

// ═══════════════════════════════════════════════
// Tauri Commands
// ═══════════════════════════════════════════════

/// 获取代理池状态
#[tauri::command]
pub async fn proxy_pool_status() -> IpcResponse<ProxyPoolStatus> {
    let subs = match load_subscriptions() {
        Ok(s) => s,
        Err(e) => return ipc::err("PROXY_LOAD_FAILED", format!("{e}")),
    };

    // 代理池节点信息从缓存文件读取
    let cache_path = AppConfig::base_dir()
        .unwrap_or_default()
        .join("proxy_pool_cache.json");

    let nodes: Vec<ProxyPoolEntry> = if cache_path.exists() {
        let content = match std::fs::read_to_string(&cache_path) {
            Ok(c) => c,
            Err(e) => return ipc::err("PROXY_READ_FAILED", &format!("Read cache: {e}")),
        };
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        // Cache 不存在，从 subscriptions.json 中提取直连代理节点
        let extracted = extract_direct_proxies(&subs);
        if !extracted.is_empty() {
            // 写入缓存供下次使用
            if let Some(parent) = cache_path.parent() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    tracing::warn!("proxy_pool: create cache dir failed: {e}");
                }
            }
            if let Err(e) = atomic_io::write_json_atomic(&cache_path, &extracted) {
                tracing::warn!("proxy_pool: write cache failed: {e}");
            }
        }
        extracted
    };

    let healthy = nodes.iter().filter(|n| n.fail_count < 3).count();
    let unhealthy = nodes.len() - healthy;

    let config = match load_pool_config() {
        Ok(c) => c,
        Err(e) => return ipc::err("PROXY_LOAD_FAILED", format!("{e}")),
    };
    let strategy = config
        .get("pool")
        .and_then(|p| p.get("selection_strategy"))
        .and_then(|s| s.as_str())
        .unwrap_or("adaptive")
        .to_string();

    ipc::ok(ProxyPoolStatus {
        total: nodes.len(),
        healthy,
        unhealthy,
        strategy,
        nodes,
        subscriptions: subs,
    })
}

/// 从订阅列表中提取直连代理节点 (http://ip:port 格式)
fn extract_direct_proxies(subs: &[String]) -> Vec<ProxyPoolEntry> {
    let mut nodes = vec![];
    for url in subs {
        if url.starts_with("http://") || url.starts_with("https://") {
            // 验证格式: 必须包含 ip:port
            let stripped = url
                .trim_start_matches("https://")
                .trim_start_matches("http://");
            if let Some(colon_pos) = stripped.rfind(':') {
                let host = &stripped[..colon_pos];
                let port_str = &stripped[colon_pos + 1..];
                if port_str.parse::<u16>().is_ok() && !host.is_empty() {
                    let geo_tag = infer_geo_from_ip(host);
                    nodes.push(ProxyPoolEntry {
                        url: url.clone(),
                        tag: format!("sub-{}", &host[host.len().saturating_sub(12)..]),
                        geo_tag,
                        latency_ms: None,
                        success_count: 0,
                        fail_count: 0,
                        speed_tier: "unknown".into(),
                        from_subscription: true,
                    });
                }
            }
        }
    }
    nodes
}

/// 根据 IP 首位推断地理区域 (简化版)
fn infer_geo_from_ip(ip: &str) -> Option<String> {
    let first_octet = ip.split('.').next()?.parse::<u8>().ok()?;
    match first_octet {
        1..=50 => Some("US".into()),
        51..=100 => Some("EU".into()),
        101..=150 => Some("AS".into()),
        151..=200 => Some("SA".into()),
        201..=255 => Some("AF".into()),
        _ => None,
    }
}

/// 获取代理池快照
#[tauri::command]
pub async fn proxy_pool_snapshot() -> IpcResponse<ProxyPoolSnapshot> {
    let status = match proxy_pool_status().await {
        IpcResponse { ok: true, data: Some(d), .. } => d,
        IpcResponse { error: Some(e), .. } => return IpcResponse { ok: false, error: Some(e), data: None },
        _ => return ipc::err("PROXY_SNAPSHOT_FAILED", "unexpected response"),
    };

    let avg_latency = if status.nodes.is_empty() {
        0.0
    } else {
        let total: u64 = status.nodes.iter().filter_map(|n| n.latency_ms).sum();
        let count = status
            .nodes
            .iter()
            .filter(|n| n.latency_ms.is_some())
            .count();
        if count > 0 {
            total as f64 / count as f64
        } else {
            0.0
        }
    };

    let mut geo_dist = std::collections::HashMap::new();
    for node in &status.nodes {
        let geo = node.geo_tag.clone().unwrap_or_else(|| "unknown".into());
        *geo_dist.entry(geo).or_insert(0) += 1;
    }

    let mut speed_tiers = std::collections::HashMap::new();
    for node in &status.nodes {
        *speed_tiers.entry(node.speed_tier.clone()).or_insert(0) += 1;
    }

    ipc::ok(ProxyPoolSnapshot {
        total: status.total,
        healthy: status.healthy,
        avg_latency_ms: avg_latency,
        strategy: status.strategy,
        geo_distribution: geo_dist,
        speed_tiers,
    })
}

/// 添加代理节点
#[tauri::command]
pub async fn proxy_pool_add(url: String, tag: String) -> IpcResponse<ProxyPoolEntry> {
    let cache_path = AppConfig::base_dir()
        .unwrap_or_default()
        .join("proxy_pool_cache.json");

    let mut nodes: Vec<ProxyPoolEntry> = if cache_path.exists() {
        let content = match std::fs::read_to_string(&cache_path) {
            Ok(c) => c,
            Err(e) => return ipc::err("PROXY_READ_FAILED", &format!("Read cache: {e}")),
        };
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        vec![]
    };

    // 检查重复
    if nodes.iter().any(|n| n.url == url) {
        return ipc::err("PROXY_DUPLICATE", "Proxy already exists");
    }

    let entry = ProxyPoolEntry {
        url: url.clone(),
        tag,
        geo_tag: None,
        latency_ms: None,
        success_count: 0,
        fail_count: 0,
        speed_tier: "unknown".into(),
        from_subscription: false,
    };

    nodes.push(entry.clone());

    // 保存到缓存
    if let Err(e) = atomic_io::write_json_atomic(&cache_path, &nodes) {
        return ipc::err("PROXY_WRITE_FAILED", &format!("Write cache: {e}"));
    }

    ipc::ok(entry)
}

/// 删除代理节点
#[tauri::command]
pub async fn proxy_pool_remove(url: String) -> IpcResponse<bool> {
    let cache_path = AppConfig::base_dir()
        .unwrap_or_default()
        .join("proxy_pool_cache.json");

    if !cache_path.exists() {
        return ipc::ok(false);
    }

    let content = match std::fs::read_to_string(&cache_path) {
        Ok(c) => c,
        Err(e) => return ipc::err("PROXY_READ_FAILED", &format!("Read cache: {e}")),
    };
    let mut nodes: Vec<ProxyPoolEntry> = serde_json::from_str(&content).unwrap_or_default();

    let original_len = nodes.len();
    nodes.retain(|n| n.url != url);

    if nodes.len() == original_len {
        return ipc::ok(false);
    }

    if let Err(e) = atomic_io::write_json_atomic(&cache_path, &nodes) {
        return ipc::err("PROXY_WRITE_FAILED", &format!("Write cache: {e}"));
    }

    ipc::ok(true)
}

/// 添加订阅源
#[tauri::command]
pub async fn proxy_pool_add_subscription(url: String) -> IpcResponse<Vec<String>> {
    let mut subs = match load_subscriptions() {
        Ok(s) => s,
        Err(e) => return ipc::err("PROXY_LOAD_FAILED", format!("{e}")),
    };

    if subs.contains(&url) {
        return ipc::err("PROXY_DUPLICATE", "Subscription already exists");
    }

    subs.push(url);
    if let Err(e) = save_subscriptions(&subs) {
        return ipc::err("PROXY_SAVE_FAILED", format!("{e}"));
    }
    ipc::ok(subs)
}

/// 删除订阅源
#[tauri::command]
pub async fn proxy_pool_remove_subscription(url: String) -> IpcResponse<Vec<String>> {
    let mut subs = match load_subscriptions() {
        Ok(s) => s,
        Err(e) => return ipc::err("PROXY_LOAD_FAILED", format!("{e}")),
    };
    let original_len = subs.len();

    subs.retain(|s| s != &url);

    if subs.len() == original_len {
        return ipc::err("PROXY_NOT_FOUND", "Subscription not found");
    }

    if let Err(e) = save_subscriptions(&subs) {
        return ipc::err("PROXY_SAVE_FAILED", format!("{e}"));
    }
    ipc::ok(subs)
}

/// 设置选择策略
#[tauri::command]
pub async fn proxy_pool_set_strategy(strategy: String) -> IpcResponse<String> {
    let valid_strategies = [
        "fastest",
        "least_latency",
        "least_failure",
        "weighted_random",
        "geo_preferred",
        "round_robin",
        "adaptive",
        "auto",
    ];

    if !valid_strategies.contains(&strategy.as_str()) {
        return ipc::err(
            "PROXY_INVALID_STRATEGY",
            &format!("Invalid strategy. Valid: {}", valid_strategies.join(", ")),
        );
    }

    if let Err(e) = save_strategy(&strategy) {
        return ipc::err("PROXY_SAVE_FAILED", format!("{e}"));
    }
    ipc::ok(strategy)
}

/// 获取可用策略列表
#[tauri::command]
pub async fn proxy_pool_list_strategies() -> IpcResponse<Vec<String>> {
    ipc::ok(vec![
        "fastest".into(),
        "least_latency".into(),
        "least_failure".into(),
        "weighted_random".into(),
        "geo_preferred".into(),
        "round_robin".into(),
        "adaptive".into(),
        "auto".into(),
    ])
}
