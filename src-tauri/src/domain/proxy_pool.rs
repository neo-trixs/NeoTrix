//! Proxy Pool Domain — business logic extracted from commands/proxy_pool.rs

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::app_error::{AppError, AppResult};
use crate::atomic_io;
use crate::config::AppConfig;

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

pub const VALID_STRATEGIES: &[&str] = &[
    "fastest",
    "least_latency",
    "least_failure",
    "weighted_random",
    "geo_preferred",
    "round_robin",
    "adaptive",
    "auto",
];

fn base_dir() -> AppResult<std::path::PathBuf> {
    AppConfig::base_dir().ok_or_else(|| AppError::Config {
        code: "PROXY_NO_BASE_DIR".into(),
        message: "Cannot determine base directory".into(),
    })
}

fn read_file_to_string(path: &Path) -> AppResult<String> {
    let bytes = atomic_io::read_with_fallback(path)?;
    String::from_utf8(bytes).map_err(|e| AppError::Io {
        code: "PROXY_INVALID_UTF8".into(),
        message: format!("Read {}: {}", path.display(), e),
        recoverable: false,
    })
}

// ═══════════════════════════════════════════════
// Subscriptions
// ═══════════════════════════════════════════════

pub fn load_subscriptions(base_dir: &Path) -> AppResult<Vec<String>> {
    let path = base_dir.join("subscriptions.json");
    if !path.exists() {
        return Ok(vec![]);
    }
    let content = read_file_to_string(&path)?;
    serde_json::from_str(&content).map_err(|e| AppError::Serde {
        code: "PROXY_PARSE_SUBS".into(),
        message: e.to_string(),
    })
}

pub fn save_subscriptions(base_dir: &Path, subs: &[String]) -> AppResult<()> {
    let path = base_dir.join("subscriptions.json");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    atomic_io::write_json_atomic(&path, subs)?;
    Ok(())
}

// ═══════════════════════════════════════════════
// Pool Config
// ═══════════════════════════════════════════════

pub fn load_pool_config(base_dir: &Path) -> AppResult<serde_json::Value> {
    let path = base_dir.join("config.toml");
    if !path.exists() {
        return Ok(serde_json::json!({
            "pool": {
                "selection_strategy": "adaptive",
                "min_nodes": 10
            }
        }));
    }
    let content = read_file_to_string(&path)?;
    Ok(serde_json::json!({ "raw": content }))
}

pub fn save_strategy(base_dir: &Path, strategy: &str) -> AppResult<()> {
    let config_path = base_dir.join("config.toml");

    let mut config_str = if config_path.exists() {
        read_file_to_string(&config_path).unwrap_or_default()
    } else {
        String::new()
    };

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
        std::fs::create_dir_all(parent)?;
    }
    atomic_io::write_atomic(&config_path, config_str.as_bytes())?;
    Ok(())
}

// ═══════════════════════════════════════════════
// Proxy Extraction
// ═══════════════════════════════════════════════

pub fn extract_direct_proxies(subs: &[String]) -> Vec<ProxyPoolEntry> {
    let mut nodes = vec![];
    for url in subs {
        if url.starts_with("http://") || url.starts_with("https://") {
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

// ═══════════════════════════════════════════════
// Cache helpers
// ═══════════════════════════════════════════════

fn cache_path(base_dir: &Path) -> std::path::PathBuf {
    base_dir.join("proxy_pool_cache.json")
}

fn load_cache(base_dir: &Path) -> AppResult<Vec<ProxyPoolEntry>> {
    let path = cache_path(base_dir);
    if !path.exists() {
        return Ok(vec![]);
    }
    let content = read_file_to_string(&path)?;
    serde_json::from_str(&content).map_err(|e| AppError::Serde {
        code: "PROXY_PARSE_CACHE".into(),
        message: e.to_string(),
    })
}

fn save_cache(base_dir: &Path, nodes: &[ProxyPoolEntry]) -> AppResult<()> {
    let path = cache_path(base_dir);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    atomic_io::write_json_atomic(&path, nodes)?;
    Ok(())
}

// ═══════════════════════════════════════════════
// Status / Snapshot
// ═══════════════════════════════════════════════

pub fn get_status(base_dir: &Path) -> AppResult<ProxyPoolStatus> {
    let subs = load_subscriptions(base_dir)?;

    let mut nodes = load_cache(base_dir)?;
    if nodes.is_empty() && !subs.is_empty() {
        let extracted = extract_direct_proxies(&subs);
        if !extracted.is_empty() {
            let _ = save_cache(base_dir, &extracted);
        }
        nodes = extracted;
    }

    let healthy = nodes.iter().filter(|n| n.fail_count < 3).count();
    let unhealthy = nodes.len() - healthy;

    let config = load_pool_config(base_dir)?;
    let strategy = config
        .get("pool")
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

pub fn get_snapshot(base_dir: &Path) -> AppResult<ProxyPoolSnapshot> {
    let status = get_status(base_dir)?;

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

    Ok(ProxyPoolSnapshot {
        total: status.total,
        healthy: status.healthy,
        avg_latency_ms: avg_latency,
        strategy: status.strategy,
        geo_distribution: geo_dist,
        speed_tiers,
    })
}

// ═══════════════════════════════════════════════
// Add / Remove Node
// ═══════════════════════════════════════════════

pub fn add_node(base_dir: &Path, url: &str, tag: Option<&str>) -> AppResult<ProxyPoolEntry> {
    let mut nodes = load_cache(base_dir)?;

    if nodes.iter().any(|n| n.url == url) {
        return Err(AppError::Duplicate {
            code: "PROXY_DUPLICATE".into(),
            message: "Proxy already exists".into(),
        });
    }

    let entry = ProxyPoolEntry {
        url: url.to_string(),
        tag: tag.unwrap_or("manual").to_string(),
        geo_tag: None,
        latency_ms: None,
        success_count: 0,
        fail_count: 0,
        speed_tier: "unknown".into(),
        from_subscription: false,
    };

    nodes.push(entry.clone());
    save_cache(base_dir, &nodes)?;
    Ok(entry)
}

pub fn remove_node(base_dir: &Path, url: &str) -> AppResult<bool> {
    let mut nodes = load_cache(base_dir)?;
    let original_len = nodes.len();
    nodes.retain(|n| n.url != url);
    if nodes.len() == original_len {
        return Ok(false);
    }
    save_cache(base_dir, &nodes)?;
    Ok(true)
}

// ═══════════════════════════════════════════════
// Add / Remove Subscription
// ═══════════════════════════════════════════════

pub fn add_subscription(base_dir: &Path, url: &str) -> AppResult<Vec<String>> {
    let mut subs = load_subscriptions(base_dir)?;
    if subs.contains(&url.to_string()) {
        return Err(AppError::Duplicate {
            code: "PROXY_DUPLICATE".into(),
            message: "Subscription already exists".into(),
        });
    }
    subs.push(url.to_string());
    save_subscriptions(base_dir, &subs)?;
    Ok(subs)
}

pub fn remove_subscription(base_dir: &Path, url: &str) -> AppResult<Vec<String>> {
    let mut subs = load_subscriptions(base_dir)?;
    let original_len = subs.len();
    subs.retain(|s| s != url);
    if subs.len() == original_len {
        return Err(AppError::NotFound {
            code: "PROXY_NOT_FOUND".into(),
            message: "Subscription not found".into(),
        });
    }
    save_subscriptions(base_dir, &subs)?;
    Ok(subs)
}

// ═══════════════════════════════════════════════
// Strategy
// ═══════════════════════════════════════════════

pub fn set_strategy(base_dir: &Path, strategy: &str) -> AppResult<String> {
    if !VALID_STRATEGIES.contains(&strategy) {
        return Err(AppError::InvalidInput {
            code: "PROXY_INVALID_STRATEGY".into(),
            message: format!(
                "Invalid strategy. Valid: {}",
                VALID_STRATEGIES.join(", ")
            ),
        });
    }
    save_strategy(base_dir, strategy)?;
    Ok(strategy.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    // ── extract_direct_proxies ───────────────────────

    #[test]
    fn extract_valid_http_url() {
        let subs = vec!["http://1.2.3.4:8080".into()];
        let nodes = extract_direct_proxies(&subs);
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].url, "http://1.2.3.4:8080");
        assert_eq!(nodes[0].speed_tier, "unknown");
        assert!(!nodes[0].from_subscription);
    }

    #[test]
    fn extract_valid_https_url() {
        let subs = vec!["https://10.0.0.1:443".into()];
        let nodes = extract_direct_proxies(&subs);
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].url, "https://10.0.0.1:443");
    }

    #[test]
    fn extract_multiple_valid_urls() {
        let subs = vec![
            "http://1.2.3.4:8080".into(),
            "https://5.6.7.8:443".into(),
            "http://9.10.11.12:3128".into(),
        ];
        let nodes = extract_direct_proxies(&subs);
        assert_eq!(nodes.len(), 3);
    }

    #[test]
    fn extract_skips_non_http() {
        let subs = vec!["socks5://1.2.3.4:1080".into(), "ftp://host:21".into()];
        let nodes = extract_direct_proxies(&subs);
        assert!(nodes.is_empty());
    }

    #[test]
    fn extract_skips_invalid_port() {
        let subs = vec!["http://1.2.3.4:99999".into()];
        let nodes = extract_direct_proxies(&subs);
        assert!(nodes.is_empty());
    }

    #[test]
    fn extract_skips_missing_port() {
        let subs = vec!["http://1.2.3.4".into()];
        let nodes = extract_direct_proxies(&subs);
        assert!(nodes.is_empty());
    }

    #[test]
    fn extract_skips_empty_host() {
        let subs = vec!["http://:8080".into()];
        let nodes = extract_direct_proxies(&subs);
        assert!(nodes.is_empty());
    }

    #[test]
    fn extract_empty_input() {
        let nodes = extract_direct_proxies(&[]);
        assert!(nodes.is_empty());
    }

    #[test]
    fn extract_tag_derived_from_host() {
        let subs = vec!["http://192.168.1.100:8080".into()];
        let nodes = extract_direct_proxies(&subs);
        assert!(nodes[0].tag.starts_with("sub-"));
    }

    // ── infer_geo_from_ip ────────────────────────────

    #[test]
    fn infer_geo_us() {
        assert_eq!(infer_geo_from_ip("1.0.0.1"), Some("US".into()));
        assert_eq!(infer_geo_from_ip("50.0.0.1"), Some("US".into()));
    }

    #[test]
    fn infer_geo_eu() {
        assert_eq!(infer_geo_from_ip("51.0.0.1"), Some("EU".into()));
        assert_eq!(infer_geo_from_ip("100.0.0.1"), Some("EU".into()));
    }

    #[test]
    fn infer_geo_as() {
        assert_eq!(infer_geo_from_ip("101.0.0.1"), Some("AS".into()));
        assert_eq!(infer_geo_from_ip("150.0.0.1"), Some("AS".into()));
    }

    #[test]
    fn infer_geo_sa() {
        assert_eq!(infer_geo_from_ip("151.0.0.1"), Some("SA".into()));
        assert_eq!(infer_geo_from_ip("200.0.0.1"), Some("SA".into()));
    }

    #[test]
    fn infer_geo_af() {
        assert_eq!(infer_geo_from_ip("201.0.0.1"), Some("AF".into()));
        assert_eq!(infer_geo_from_ip("255.0.0.1"), Some("AF".into()));
    }

    #[test]
    fn infer_geo_invalid_ip() {
        assert_eq!(infer_geo_from_ip("not-an-ip"), None);
        assert_eq!(infer_geo_from_ip(""), None);
        assert_eq!(infer_geo_from_ip("abc.def.ghi"), None);
    }

    // ── VALID_STRATEGIES ─────────────────────────────

    #[test]
    fn valid_strategies_contains_expected() {
        let expected = [
            "fastest",
            "least_latency",
            "least_failure",
            "weighted_random",
            "geo_preferred",
            "round_robin",
            "adaptive",
            "auto",
        ];
        for s in expected {
            assert!(
                VALID_STRATEGIES.contains(&s),
                "Missing strategy: {s}"
            );
        }
    }

    // ── set_strategy / save_strategy ─────────────────

    #[test]
    fn set_strategy_valid() {
        let dir = tempdir().unwrap();
        let result = set_strategy(dir.path(), "adaptive");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "adaptive");
    }

    #[test]
    fn set_strategy_invalid() {
        let dir = tempdir().unwrap();
        let result = set_strategy(dir.path(), "bogus_strategy");
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::InvalidInput { code, .. } => {
                assert_eq!(code, "PROXY_INVALID_STRATEGY");
            }
            other => panic!("Expected InvalidInput, got: {other:?}"),
        }
    }

    #[test]
    fn set_strategy_creates_config_file() {
        let dir = tempdir().unwrap();
        set_strategy(dir.path(), "round_robin").unwrap();
        let config_path = dir.path().join("config.toml");
        assert!(config_path.exists());
        let content = std::fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("round_robin"));
    }

    #[test]
    fn set_strategy_updates_existing() {
        let dir = tempdir().unwrap();
        set_strategy(dir.path(), "adaptive").unwrap();
        set_strategy(dir.path(), "fastest").unwrap();
        let content = std::fs::read_to_string(dir.path().join("config.toml")).unwrap();
        assert!(content.contains("fastest"));
        assert!(!content.contains("adaptive"));
    }

    // ── load_subscriptions ───────────────────────────

    #[test]
    fn load_subscriptions_missing_file() {
        let dir = tempdir().unwrap();
        let subs = load_subscriptions(dir.path()).unwrap();
        assert!(subs.is_empty());
    }

    #[test]
    fn load_subscriptions_valid_json() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("subscriptions.json");
        let data = vec!["http://a.com:80".into(), "https://b.com:443".into()];
        atomic_io::write_json_atomic(&path, &data).unwrap();
        let subs = load_subscriptions(dir.path()).unwrap();
        assert_eq!(subs.len(), 2);
    }

    #[test]
    fn load_subscriptions_invalid_json() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("subscriptions.json");
        std::fs::write(&path, "not json!!!").unwrap();
        let result = load_subscriptions(dir.path());
        assert!(result.is_err());
    }

    // ── save_subscriptions ───────────────────────────

    #[test]
    fn save_and_load_subscriptions_roundtrip() {
        let dir = tempdir().unwrap();
        let subs = vec!["http://x.com:80".into()];
        save_subscriptions(dir.path(), &subs).unwrap();
        let loaded = load_subscriptions(dir.path()).unwrap();
        assert_eq!(loaded, subs);
    }

    // ── load_pool_config ─────────────────────────────

    #[test]
    fn load_pool_config_missing_file_returns_default() {
        let dir = tempdir().unwrap();
        let config = load_pool_config(dir.path()).unwrap();
        let strategy = config["pool"]["selection_strategy"].as_str().unwrap();
        assert_eq!(strategy, "adaptive");
    }

    // ── add_node / remove_node ───────────────────────

    #[test]
    fn add_and_remove_node() {
        let dir = tempdir().unwrap();
        let entry = add_node(dir.path(), "http://1.2.3.4:8080", Some("test")).unwrap();
        assert_eq!(entry.url, "http://1.2.3.4:8080");
        assert_eq!(entry.tag, "test");

        let removed = remove_node(dir.path(), "http://1.2.3.4:8080").unwrap();
        assert!(removed);
    }

    #[test]
    fn add_node_duplicate_returns_error() {
        let dir = tempdir().unwrap();
        add_node(dir.path(), "http://1.2.3.4:8080", None).unwrap();
        let result = add_node(dir.path(), "http://1.2.3.4:8080", None);
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::Duplicate { code, .. } => assert_eq!(code, "PROXY_DUPLICATE"),
            other => panic!("Expected Duplicate, got: {other:?}"),
        }
    }

    #[test]
    fn remove_node_nonexistent_returns_false() {
        let dir = tempdir().unwrap();
        let removed = remove_node(dir.path(), "http://nope.com:99").unwrap();
        assert!(!removed);
    }

    // ── add_subscription / remove_subscription ────────

    #[test]
    fn add_and_remove_subscription() {
        let dir = tempdir().unwrap();
        let subs = add_subscription(dir.path(), "http://sub.com:80").unwrap();
        assert_eq!(subs.len(), 1);

        let subs = remove_subscription(dir.path(), "http://sub.com:80").unwrap();
        assert!(subs.is_empty());
    }

    #[test]
    fn add_subscription_duplicate_returns_error() {
        let dir = tempdir().unwrap();
        add_subscription(dir.path(), "http://sub.com:80").unwrap();
        let result = add_subscription(dir.path(), "http://sub.com:80");
        assert!(result.is_err());
    }

    #[test]
    fn remove_subscription_not_found_returns_error() {
        let dir = tempdir().unwrap();
        let result = remove_subscription(dir.path(), "http://nope.com:80");
        assert!(result.is_err());
        match result.unwrap_err() {
            AppError::NotFound { code, .. } => assert_eq!(code, "PROXY_NOT_FOUND"),
            other => panic!("Expected NotFound, got: {other:?}"),
        }
    }

    // ── get_status / get_snapshot ─────────────────────

    #[test]
    fn get_status_empty_dir() {
        let dir = tempdir().unwrap();
        let status = get_status(dir.path()).unwrap();
        assert_eq!(status.total, 0);
        assert_eq!(status.healthy, 0);
        assert_eq!(status.unhealthy, 0);
    }

    #[test]
    fn get_snapshot_empty_dir() {
        let dir = tempdir().unwrap();
        let snapshot = get_snapshot(dir.path()).unwrap();
        assert_eq!(snapshot.total, 0);
        assert_eq!(snapshot.avg_latency_ms, 0.0);
        assert!(snapshot.geo_distribution.is_empty());
        assert!(snapshot.speed_tiers.is_empty());
    }

    #[test]
    fn get_status_with_nodes() {
        let dir = tempdir().unwrap();
        add_node(dir.path(), "http://1.2.3.4:8080", None).unwrap();
        add_node(dir.path(), "http://5.6.7.8:8080", None).unwrap();
        let status = get_status(dir.path()).unwrap();
        assert_eq!(status.total, 2);
        assert_eq!(status.healthy, 2);
    }

    #[test]
    fn get_snapshot_geo_distribution() {
        let dir = tempdir().unwrap();
        // US IP
        let entry = ProxyPoolEntry {
            url: "http://1.0.0.1:80".into(),
            tag: "a".into(),
            geo_tag: Some("US".into()),
            latency_ms: Some(100),
            success_count: 0,
            fail_count: 0,
            speed_tier: "fast".into(),
            from_subscription: false,
        };
        let nodes = vec![entry];
        atomic_io::write_json_atomic(
            &dir.path().join("proxy_pool_cache.json"),
            &nodes,
        )
        .unwrap();

        let snapshot = get_snapshot(dir.path()).unwrap();
        assert_eq!(*snapshot.geo_distribution.get("US").unwrap(), 1);
        assert_eq!(*snapshot.speed_tiers.get("fast").unwrap(), 1);
        assert_eq!(snapshot.avg_latency_ms, 100.0);
    }
}
