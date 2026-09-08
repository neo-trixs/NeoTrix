//! Proxy Pool — 对接 neotrix-core ProxyPool (nt_shield_stealth_net)
//!
//! 提供代理 IP 池的增删查状态查询，前端通过 domain_call 或直接调用。

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
fn load_subscriptions() -> Result<Vec<String>, String> {
    let path = dirs::home_dir()
        .unwrap_or_default()
        .join(".neotrix")
        .join("subscriptions.json");

    if !path.exists() {
        return Ok(vec![]);
    }

    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Read subscriptions: {e}"))?;

    serde_json::from_str(&content)
        .map_err(|e| format!("Parse subscriptions: {e}"))
}

/// 保存订阅文件
fn save_subscriptions(subs: &[String]) -> Result<(), String> {
    let path = dirs::home_dir()
        .unwrap_or_default()
        .join(".neotrix")
        .join("subscriptions.json");

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Create dir: {e}"))?;
    }

    let json = serde_json::to_string_pretty(subs)
        .map_err(|e| format!("Serialize: {e}"))?;

    std::fs::write(&path, json)
        .map_err(|e| format!("Write subscriptions: {e}"))
}

/// 读取代理池配置
fn load_pool_config() -> Result<serde_json::Value, String> {
    let path = dirs::home_dir()
        .unwrap_or_default()
        .join(".neotrix")
        .join("config.toml");

    if !path.exists() {
        return Ok(serde_json::json!({
            "pool": {
                "selection_strategy": "adaptive",
                "min_nodes": 10
            }
        }));
    }

    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Read config: {e}"))?;

    // TOML 转 JSON (简单处理)
    Ok(serde_json::json!({
        "raw": content
    }))
}

/// 保存代理池策略配置
fn save_strategy(strategy: &str) -> Result<(), String> {
    let config_path = dirs::home_dir()
        .unwrap_or_default()
        .join(".neotrix")
        .join("config.toml");

    // 读取现有配置或创建新的
    let mut config_str = if config_path.exists() {
        std::fs::read_to_string(&config_path)
            .unwrap_or_default()
    } else {
        String::new()
    };

    // 更新或插入 strategy
    if config_str.contains("selection_strategy") {
        config_str = config_str.lines().map(|line| {
            if line.contains("selection_strategy") {
                format!("selection_strategy = \"{}\"", strategy)
            } else {
                line.to_string()
            }
        }).collect::<Vec<_>>().join("\n");
    } else {
        config_str.push_str(&format!("\n[pool]\nselection_strategy = \"{}\"\n", strategy));
    }

    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Create dir: {e}"))?;
    }

    std::fs::write(&config_path, config_str)
        .map_err(|e| format!("Write config: {e}"))
}

// ═══════════════════════════════════════════════
// Tauri Commands
// ═══════════════════════════════════════════════

/// 获取代理池状态
#[tauri::command]
pub async fn proxy_pool_status() -> Result<ProxyPoolStatus, String> {
    let subs = load_subscriptions()?;

    // 代理池节点信息从缓存文件读取
    let cache_path = dirs::home_dir()
        .unwrap_or_default()
        .join(".neotrix")
        .join("proxy_pool_cache.json");

    let nodes: Vec<ProxyPoolEntry> = if cache_path.exists() {
        let content = std::fs::read_to_string(&cache_path)
            .map_err(|e| format!("Read cache: {e}"))?;
        serde_json::from_str(&content)
            .unwrap_or_default()
    } else {
        vec![]
    };

    let healthy = nodes.iter().filter(|n| n.fail_count < 3).count();
    let unhealthy = nodes.len() - healthy;

    let config = load_pool_config()?;
    let strategy = config.get("pool")
        .and_then(|p| p.get("selection_strategy"))
        .and_then(|s| s.as_str())
        .unwrap_or("adaptive")
        .to_string();

    Ok(ProxyPoolStatus {
        total: nodes.len(),
        healthy,
        unhealthy,
        strategy,
        nodes,
        subscriptions: subs,
    })
}

/// 获取代理池快照
#[tauri::command]
pub async fn proxy_pool_snapshot() -> Result<ProxyPoolSnapshot, String> {
    let status = proxy_pool_status().await?;

    let avg_latency = if status.nodes.is_empty() {
        0.0
    } else {
        let total: u64 = status.nodes.iter()
            .filter_map(|n| n.latency_ms)
            .sum();
        let count = status.nodes.iter()
            .filter(|n| n.latency_ms.is_some())
            .count();
        if count > 0 { total as f64 / count as f64 } else { 0.0 }
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

    Ok(ProxyPoolSnapshot {
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
pub async fn proxy_pool_add(url: String, tag: String) -> Result<ProxyPoolEntry, String> {
    let cache_path = dirs::home_dir()
        .unwrap_or_default()
        .join(".neotrix")
        .join("proxy_pool_cache.json");

    let mut nodes: Vec<ProxyPoolEntry> = if cache_path.exists() {
        let content = std::fs::read_to_string(&cache_path)
            .map_err(|e| format!("Read cache: {e}"))?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        vec![]
    };

    // 检查重复
    if nodes.iter().any(|n| n.url == url) {
        return Err("Proxy already exists".into());
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
    let json = serde_json::to_string_pretty(&nodes)
        .map_err(|e| format!("Serialize: {e}"))?;
    std::fs::write(&cache_path, json)
        .map_err(|e| format!("Write cache: {e}"))?;

    Ok(entry)
}

/// 删除代理节点
#[tauri::command]
pub async fn proxy_pool_remove(url: String) -> Result<bool, String> {
    let cache_path = dirs::home_dir()
        .unwrap_or_default()
        .join(".neotrix")
        .join("proxy_pool_cache.json");

    if !cache_path.exists() {
        return Ok(false);
    }

    let content = std::fs::read_to_string(&cache_path)
        .map_err(|e| format!("Read cache: {e}"))?;
    let mut nodes: Vec<ProxyPoolEntry> = serde_json::from_str(&content)
        .unwrap_or_default();

    let original_len = nodes.len();
    nodes.retain(|n| n.url != url);

    if nodes.len() == original_len {
        return Ok(false);
    }

    let json = serde_json::to_string_pretty(&nodes)
        .map_err(|e| format!("Serialize: {e}"))?;
    std::fs::write(&cache_path, json)
        .map_err(|e| format!("Write cache: {e}"))?;

    Ok(true)
}

/// 添加订阅源
#[tauri::command]
pub async fn proxy_pool_add_subscription(url: String) -> Result<Vec<String>, String> {
    let mut subs = load_subscriptions()?;

    if subs.contains(&url) {
        return Err("Subscription already exists".into());
    }

    subs.push(url);
    save_subscriptions(&subs)?;
    Ok(subs)
}

/// 删除订阅源
#[tauri::command]
pub async fn proxy_pool_remove_subscription(url: String) -> Result<Vec<String>, String> {
    let mut subs = load_subscriptions()?;
    let original_len = subs.len();

    subs.retain(|s| s != &url);

    if subs.len() == original_len {
        return Err("Subscription not found".into());
    }

    save_subscriptions(&subs)?;
    Ok(subs)
}

/// 设置选择策略
#[tauri::command]
pub async fn proxy_pool_set_strategy(strategy: String) -> Result<String, String> {
    let valid_strategies = ["fastest", "least_latency", "least_failure",
        "weighted_random", "geo_preferred", "round_robin", "adaptive", "auto"];

    if !valid_strategies.contains(&strategy.as_str()) {
        return Err(format!("Invalid strategy. Valid: {}", valid_strategies.join(", ")));
    }

    save_strategy(&strategy)?;
    Ok(strategy)
}

/// 获取可用策略列表
#[tauri::command]
pub async fn proxy_pool_list_strategies() -> Result<Vec<String>, String> {
    Ok(vec![
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
