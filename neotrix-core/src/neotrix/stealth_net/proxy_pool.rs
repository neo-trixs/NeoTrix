use std::cmp::Ordering;
use std::sync::Arc;
use std::sync::OnceLock;
use std::time::{Duration, Instant};


use rand::Rng;
use tokio::sync::{RwLock, Semaphore};

use crate::neotrix::stealth_net::config::load as cfg;
use crate::neotrix::stealth_net::ip_geo::IpGeoLocator;

pub use super::pool_types::{NodeSelectionStrategy, SpeedTier, ProxyNode};
pub use super::pool_strategies::StrategyLearner;
pub use super::pool_health::{base64_decode, FREE_PROXY_SCRAPERS, DEFAULT_SUBSCRIPTIONS};

const MAX_POOL_SIZE: usize = 200;
const HEALTH_CHECK_CONCURRENCY: usize = 100;
const HEALTH_CHECK_TIMEOUT_SECS: u64 = 3;

async fn check_proxy(url: &str) -> bool {
    !url.is_empty()
}

fn parse_proxy_url(line: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = line.splitn(2, ',').collect();
    if parts.len() == 2 {
        Some((parts[0].trim().to_string(), parts[1].trim().to_string()))
    } else {
        None
    }
}

pub struct ProxyPool {
    pub nodes: RwLock<Vec<ProxyNode>>,
    subscriptions: RwLock<Vec<String>>,
    min_nodes: u32,
    subs_file: std::path::PathBuf,
    geo_locator: IpGeoLocator,
    strategy: RwLock<NodeSelectionStrategy>,
    rr_idx: RwLock<usize>,
    learner: RwLock<StrategyLearner>,
}

impl Default for ProxyPool {
    fn default() -> Self {
        Self::new()
    }
}

impl ProxyPool {
    pub fn new() -> Self {
        let c = cfg();
        let home = std::env::home_dir().unwrap_or_default();
        let subs_file = home.join(".neotrix").join("subscriptions.json");
        let q_path = home.join(".neotrix").join("strategy_q.json");
        Self {
            nodes: RwLock::new(Vec::new()),
            subscriptions: RwLock::new(Vec::new()),
            min_nodes: c.pool.min_nodes,
            subs_file,
            geo_locator: IpGeoLocator::new(),
            strategy: RwLock::new(NodeSelectionStrategy::from_name(&c.pool.selection_strategy)),
            rr_idx: RwLock::new(0),
            learner: RwLock::new(StrategyLearner::load(&q_path)),
        }
    }

    pub fn with_min_nodes(mut self, n: u32) -> Self {
        self.min_nodes = n;
        self
    }

    pub fn ensure_subs_file(&self) {
        if !self.subs_file.exists() {
            let defaults = serde_json::json!(DEFAULT_SUBSCRIPTIONS);
            let _ = std::fs::create_dir_all(self.subs_file.parent().unwrap_or(std::path::Path::new("")));
            let _ = std::fs::write(&self.subs_file, serde_json::to_string_pretty(&defaults).expect("hardcoded json valid"));
        }
    }

    pub async fn load_subscriptions(&self) -> usize {
        self.ensure_subs_file();
        let content = match std::fs::read_to_string(&self.subs_file) {
            Ok(s) => s,
            Err(_) => return 0,
        };
        let subs: Vec<String> = serde_json::from_str(&content).unwrap_or_default();
        let count = subs.len();
        *self.subscriptions.write().await = subs;
        count
    }

    pub async fn reload_subscriptions(&self) -> usize {
        self.load_subscriptions().await
    }

    pub async fn add_subscription(&self, url: &str) {
        self.subscriptions.write().await.push(url.to_string());
        self.persist_subscriptions().await;
    }

    pub async fn remove_subscription(&self, url: &str) -> bool {
        let mut subs = self.subscriptions.write().await;
        let before = subs.len();
        subs.retain(|s| s != url);
        drop(subs);
        if before > 0 {
            self.persist_subscriptions().await;
        }
        before > 0
    }

    pub async fn list_subscriptions(&self) -> Vec<String> {
        self.subscriptions.read().await.clone()
    }

    async fn persist_subscriptions(&self) {
        let subs = self.subscriptions.read().await.clone();
        let json = serde_json::to_string_pretty(&subs).expect("hardcoded json valid");
        let _ = std::fs::write(&self.subs_file, json);
    }

    pub async fn heal_if_needed(&self) {
        let count = self.nodes.read().await.len();
        if count >= self.min_nodes as usize { return; }
        let subs = self.subscriptions.read().await.clone();
        if subs.is_empty() { return; }
        for url in &subs {
            let _ = self.fetch_subscription(url).await;
        }
    }

    pub async fn add(&self, url: &str, tag: &str) {
        self.add_batch_with_source(&[(url.to_string(), tag.to_string())], false).await;
    }

    pub async fn add_batch(&self, proxies: &[(String, String)]) {
        self.add_batch_with_source(proxies, false).await;
    }

    async fn add_batch_with_source(&self, proxies: &[(String, String)], from_subscription: bool) {
        let mut nodes = self.nodes.write().await;
        for (url, tag) in proxies {
            if nodes.len() >= MAX_POOL_SIZE { break; }
            nodes.push(ProxyNode {
                url: url.clone(),
                tag: tag.clone(),
                latency_ms: None,
                last_success: None,
                fail_count: 0,
                success_count: 0,
                from_subscription,
                geo_tag: None,
                ip_addr: None,
                timezone: None,
            });
        }
    }

    pub async fn ready(&self, n: usize) -> Vec<ProxyNode> {
        self.best_n(n, |n| n.latency_ms.is_some() && !n.is_stale()).await
    }

    async fn best_n<F>(&self, n: usize, filter: F) -> Vec<ProxyNode>
    where F: Fn(&ProxyNode) -> bool {
        let nodes = self.nodes.read().await;
        let mut scored: Vec<(f64, ProxyNode)> = nodes.iter()
            .filter(|n| filter(n))
            .map(|n| (n.score(), n.clone()))
            .collect();
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(Ordering::Equal));
        scored.truncate(n);
        scored.into_iter().map(|(_, n)| n).collect()
    }

    pub async fn select_fastest(&self) -> Option<ProxyNode> {
        self.ready(1).await.into_iter().next()
    }

    pub async fn select_node(&self) -> Option<ProxyNode> {
        let strategy = self.strategy.read().await.clone();
        match strategy {
            NodeSelectionStrategy::Adaptive => {
                self.select_fastest().await
            }
            NodeSelectionStrategy::Auto => {
                let learner = self.learner.read().await;
                if learner.has_enough_data() {
                    drop(learner);
                    self.select_fastest().await
                } else {
                    drop(learner);
                    self.select_fastest().await
                }
            }
            _ => self.select_node_with_strategy(&strategy).await,
        }
    }

    pub async fn select_node_for_host(&self, host: &str) -> Option<ProxyNode> {
        let strategy = self.strategy.read().await.clone();
        if strategy != NodeSelectionStrategy::Adaptive && strategy != NodeSelectionStrategy::Auto {
            return self.select_node().await;
        }
        let learner = self.learner.read().await;
        if matches!(strategy, NodeSelectionStrategy::Auto) && !learner.has_enough_data() {
            drop(learner);
            return self.select_fastest().await;
        }
        let sub = learner.select_strategy(host);
        drop(learner);
        self.select_node_with_strategy(&sub).await
    }

    pub async fn select_node_with_strategy(&self, strategy: &NodeSelectionStrategy) -> Option<ProxyNode> {
        let nodes = self.nodes.read().await;
        let candidates: Vec<&ProxyNode> = nodes.iter().filter(|n| n.latency_ms.is_some() && !n.is_stale()).collect();
        if candidates.is_empty() { return None; }

        match strategy {
            NodeSelectionStrategy::Fastest => {
                drop(nodes);
                self.select_fastest().await
            }
            NodeSelectionStrategy::LeastLatency => {
                candidates.iter()
                    .min_by(|a, b| a.latency_ms.partial_cmp(&b.latency_ms).unwrap_or(Ordering::Equal))
                    .map(|n| (*n).clone())
            }
            NodeSelectionStrategy::LeastFailure => {
                candidates.iter()
                    .max_by(|a, b| {
                        let ar = if a.success_count + a.fail_count > 0 {
                            a.success_count as f64 / (a.success_count + a.fail_count) as f64
                        } else { 0.5 };
                        let br = if b.success_count + b.fail_count > 0 {
                            b.success_count as f64 / (b.success_count + b.fail_count) as f64
                        } else { 0.5 };
                        ar.partial_cmp(&br).unwrap_or(Ordering::Equal)
                    })
                    .map(|n| (*n).clone())
            }
            NodeSelectionStrategy::WeightedRandom => {
                let mut rng = rand::thread_rng();
                let total: f64 = candidates.iter().map(|n| n.score().max(0.001)).sum();
                let mut roll = rng.gen::<f64>() * total;
                for n in &candidates {
                    roll -= n.score().max(0.001);
                    if roll <= 0.0 {
                        return Some((*n).clone());
                    }
                }
                candidates.last().map(|n| (*n).clone())
            }
            NodeSelectionStrategy::GeoPreferred(region) => {
                let region_lower = region.to_lowercase();
                let geo_indices: Vec<usize> = candidates.iter().enumerate()
                    .filter(|(_, n)| n.geo_tag.as_deref().map(|g| g.to_lowercase() == region_lower).unwrap_or(false))
                    .map(|(i, _)| i)
                    .collect();
                if geo_indices.is_empty() {
                    drop(nodes);
                    self.select_fastest().await
                } else {
                    let best_idx = geo_indices.iter()
                        .min_by(|a, b| {
                            let al = candidates[**a].latency_ms;
                            let bl = candidates[**b].latency_ms;
                            al.partial_cmp(&bl).unwrap_or(Ordering::Equal)
                        })
                        .copied()?;
                    Some(candidates[best_idx].clone())
                }
            }
            NodeSelectionStrategy::RoundRobin => {
                let mut idx = self.rr_idx.write().await;
                let node = candidates.get(*idx % candidates.len()).map(|n| (*n).clone());
                *idx = idx.wrapping_add(1);
                node
            }
            NodeSelectionStrategy::Adaptive => {
                drop(nodes);
                self.select_fastest().await
            }
            NodeSelectionStrategy::Auto => {
                drop(nodes);
                self.select_fastest().await
            }
        }
    }

    pub async fn set_strategy(&self, strategy: NodeSelectionStrategy) {
        *self.strategy.write().await = strategy;
    }

    pub async fn record_strategy_result(&self, host: &str, success: bool) {
        let mut learner = self.learner.write().await;
        let strategy = self.strategy.read().await.clone();
        if strategy == NodeSelectionStrategy::Adaptive || strategy == NodeSelectionStrategy::Auto {
            let sub = learner.select_strategy(host);
            learner.record_reward(host, &sub, success);
        }
        if learner.record_count % 20 < 5 {
            learner.save();
        }
    }

    pub async fn current_strategy(&self) -> NodeSelectionStrategy {
        self.strategy.read().await.clone()
    }

    pub async fn fast_nodes(&self) -> Vec<ProxyNode> {
        self.best_n(10, |n| n.speed_tier() == SpeedTier::Fast && !n.is_stale()).await
    }

    pub async fn prune_slow(&self) {
        let mut nodes = self.nodes.write().await;
        if nodes.len() <= MAX_POOL_SIZE { return; }
        let mut scored: Vec<(f64, usize)> = nodes.iter().enumerate()
            .map(|(i, n)| (n.score(), i)).collect();
        scored.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(Ordering::Equal));
        let to_remove: std::collections::HashSet<usize> = scored.iter()
            .take(nodes.len() - MAX_POOL_SIZE).map(|(_, i)| *i).collect();
        let mut idx = 0;
        nodes.retain(|_| {
            let keep = !to_remove.contains(&idx);
            idx += 1;
            keep
        });
    }

    pub async fn available_count(&self) -> usize {
        self.nodes.read().await.iter()
            .filter(|n| n.latency_ms.is_some())
            .count()
    }

    pub async fn total_count(&self) -> usize {
        self.nodes.read().await.len()
    }

    pub async fn health_check(&self) {
        let urls: Vec<String> = {
            self.nodes.read().await.iter().map(|n| n.url.clone()).collect()
        };
        let sem = Arc::new(Semaphore::new(HEALTH_CHECK_CONCURRENCY));
        let mut handles = Vec::with_capacity(urls.len());
        for url in &urls {
            let u = url.clone();
            let s = sem.clone();
            handles.push(tokio::spawn(async move {
                let _permit = s.acquire().await;
                let start = Instant::now();
                let ok = tokio::time::timeout(
                    Duration::from_secs(HEALTH_CHECK_TIMEOUT_SECS),
                    check_proxy(&u),
                ).await.unwrap_or(false);
                let latency = start.elapsed().as_millis() as f64;
                (u, ok, latency)
            }));
        }
        let results: Vec<(String, bool, f64)> = futures::future::join_all(handles)
            .await.into_iter().filter_map(|r| match r {
                Ok(v) => Some(v),
                Err(e) => {
                    log::warn!("[proxy-pool] health check task failed: {}", e);
                    None
                }
            }).collect();

        let mut nodes = self.nodes.write().await;
        let mut geo_targets = Vec::new();
        for (url, ok, latency) in &results {
            if let Some(node) = nodes.iter_mut().find(|n| n.url == *url) {
                if *ok {
                    node.latency_ms = Some(*latency);
                    node.last_success = Some(Instant::now());
                    node.success_count += 1;
                    if node.ip_addr.is_none() || node.geo_tag.is_none() {
                        geo_targets.push(node.url.clone());
                    }
                } else {
                    node.fail_count += 1;
                    if node.fail_count > 3 {
                        node.latency_ms = None;
                    }
                }
            }
        }
        drop(nodes);

        if !geo_targets.is_empty() {
            let mut batch_ips = Vec::new();
            let mut url_to_ip = Vec::new();
            for url in &geo_targets {
                if let Some(host) = IpGeoLocator::extract_host(url) {
                    if let Some(ip) = IpGeoLocator::resolve_to_ip(&host).await {
                        batch_ips.push(ip.clone());
                        url_to_ip.push((url.clone(), ip));
                    }
                }
            }
            let geo_results = self.geo_locator.lookup_batch(&batch_ips).await;
            let mut geo_map = std::collections::HashMap::new();
            for (ip, geo) in &geo_results {
                geo_map.insert(ip.clone(), geo.clone());
            }
            let mut nodes = self.nodes.write().await;
            for (url, ip) in &url_to_ip {
                if let Some(geo) = geo_map.get(ip) {
                    if let Some(node) = nodes.iter_mut().find(|n| n.url == *url) {
                        node.ip_addr = Some(ip.clone());
                        node.geo_tag = Some(geo.tag());
                        node.timezone = Some(geo.timezone.clone());
                    }
                }
            }
        }
    }

    pub async fn start_health_loop(self: Arc<Self>, interval_secs: u64) {
        loop {
            self.heal_if_needed().await;
            self.health_check().await;
            self.prune_slow().await;
            tokio::time::sleep(Duration::from_secs(interval_secs)).await;
        }
    }

    pub async fn fetch_subscription(&self, sub_url: &str) -> Result<usize, String> {
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .no_proxy()
            .build()
            .map_err(|e| format!("{}", e))?;
        let resp = client.get(sub_url)
            .send()
            .await
            .map_err(|e| format!("{}", e))?;
        let text = resp.text().await.map_err(|e| format!("{}", e))?;

        let decoded = if let Ok(d) = base64_decode(&text) { d } else { text.clone() };

        let mut proxies = Vec::new();
        for line in decoded.lines() {
            let line = line.trim();
            if line.is_empty() { continue; }

            if let Some((url, tag)) = parse_proxy_url(line) {
                proxies.push((url, tag));
            }
        }

        if proxies.is_empty() {
            return Ok(0);
        }

        let count = proxies.len();
        self.add_batch_with_source(&proxies, true).await;
        Ok(count)
    }
}

pub fn global_pool() -> Arc<ProxyPool> {
    static POOL: OnceLock<Arc<ProxyPool>> = OnceLock::new();
    POOL.get_or_init(|| Arc::new(ProxyPool::new())).clone()
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strategy_from_name_auto() {
        let s = NodeSelectionStrategy::from_name("auto");
        assert!(s.is_some());
    }

    #[tokio::test]
    async fn test_add_and_count() {
        let pool = ProxyPool::new();
        pool.add("socks5://127.0.0.1:1080", "localhost").await;
        assert_eq!(pool.total_count().await, 1);
    }

    #[tokio::test]
    async fn test_set_and_get_strategy() {
        let pool = ProxyPool::new();
        assert_eq!(pool.current_strategy().await, NodeSelectionStrategy::Auto);
        pool.set_strategy(NodeSelectionStrategy::RoundRobin).await;
        assert_eq!(pool.current_strategy().await, NodeSelectionStrategy::RoundRobin);
    }

    #[tokio::test]
    async fn test_strategy_roundtrip() {
        let pool = ProxyPool::new();
        pool.set_strategy(NodeSelectionStrategy::LeastLatency).await;
        assert_eq!(pool.current_strategy().await, NodeSelectionStrategy::LeastLatency);
        assert_eq!(pool.current_strategy().await.as_str(), "least_latency");
    }

    #[tokio::test]
    async fn test_select_node_fastest_returns_highest_score() {
        let pool = ProxyPool::new();
        pool.add("socks5://slow:1080", "slow").await;
        pool.add("socks5://fast:1080", "fast").await;

        let mut nodes = pool.nodes.write().await;
        let slow_idx = nodes.iter().position(|n| n.tag == "slow").unwrap();
        let fast_idx = nodes.iter().position(|n| n.tag == "fast").unwrap();
        nodes[slow_idx].latency_ms = Some(500.0);
        nodes[slow_idx].success_count = 10;
        nodes[slow_idx].last_success = Some(Instant::now());
        nodes[fast_idx].latency_ms = Some(50.0);
        nodes[fast_idx].success_count = 10;
        nodes[fast_idx].last_success = Some(Instant::now());
        drop(nodes);

        let best = pool.select_fastest().await.expect("should have a node");
        assert_eq!(best.tag, "fast");
    }

    #[tokio::test]
    async fn test_select_node_least_latency() {
        let pool = ProxyPool::new();
        pool.add("socks5://a:1080", "a").await;
        pool.add("socks5://b:1080", "b").await;
        let mut nodes = pool.nodes.write().await;
        let a_idx = nodes.iter().position(|n| n.tag == "a").unwrap();
        let b_idx = nodes.iter().position(|n| n.tag == "b").unwrap();
        nodes[a_idx].latency_ms = Some(100.0);
        nodes[a_idx].last_success = Some(Instant::now());
        nodes[b_idx].latency_ms = Some(200.0);
        nodes[b_idx].last_success = Some(Instant::now());
        drop(nodes);

        let n = pool.select_node_with_strategy(&NodeSelectionStrategy::LeastLatency).await;
        assert!(n.is_some());
        assert_eq!(n.unwrap().tag, "a");
    }

    #[tokio::test]
    async fn test_select_node_least_failure() {
        let pool = ProxyPool::new();
        pool.add("socks5://a:1080", "a").await;
        pool.add("socks5://b:1080", "b").await;
        let mut nodes = pool.nodes.write().await;
        let a_idx = nodes.iter().position(|n| n.tag == "a").unwrap();
        let b_idx = nodes.iter().position(|n| n.tag == "b").unwrap();
        nodes[a_idx].latency_ms = Some(100.0);
        nodes[a_idx].last_success = Some(Instant::now());
        nodes[b_idx].latency_ms = Some(100.0);
        nodes[b_idx].last_success = Some(Instant::now());
        nodes[a_idx].success_count = 10; nodes[a_idx].fail_count = 0;
        nodes[b_idx].success_count = 5; nodes[b_idx].fail_count = 5;
        drop(nodes);

        let n = pool.select_node_with_strategy(&NodeSelectionStrategy::LeastFailure).await;
        assert!(n.is_some());
        assert_eq!(n.unwrap().tag, "a");
    }

    #[tokio::test]
    async fn test_select_node_geo_preferred() {
        let pool = ProxyPool::new();
        pool.add("socks5://jp:1080", "jp").await;
        pool.add("socks5://us:1080", "us").await;
        let mut nodes = pool.nodes.write().await;
        for n in nodes.iter_mut() {
            n.latency_ms = Some(100.0);
            n.last_success = Some(Instant::now());
        }
        drop(nodes);
        // Manually set geo_tag
        let mut nodes = pool.nodes.write().await;
        for n in nodes.iter_mut() {
            n.geo_tag = match n.tag.as_str() {
                "jp" => Some("JP".into()),
                "us" => Some("US".into()),
                _ => None,
            };
        }
        drop(nodes);

        let n = pool.select_node_with_strategy(&NodeSelectionStrategy::GeoPreferred("JP".into())).await;
        assert!(n.is_some());
        assert_eq!(n.unwrap().tag, "jp");
    }

    #[tokio::test]
    async fn test_select_node_geo_preferred_fallback() {
        let pool = ProxyPool::new();
        pool.add("socks5://x:1080", "x").await;
        let mut nodes = pool.nodes.write().await;
        let x = nodes.iter_mut().find(|n| n.tag == "x").unwrap();
        x.latency_ms = Some(50.0);
        x.last_success = Some(Instant::now());
        drop(nodes);

        let n = pool.select_node_with_strategy(&NodeSelectionStrategy::GeoPreferred("JP".into())).await;
        assert!(n.is_some(), "should fallback to fastest when geo missing");
    }

    #[tokio::test]
    async fn test_select_node_round_robin_cycles() {
        let pool = ProxyPool::new();
        pool.add("socks5://a:1080", "a").await;
        pool.add("socks5://b:1080", "b").await;
        let mut nodes = pool.nodes.write().await;
        for n in nodes.iter_mut() {
            n.latency_ms = Some(10.0);
            n.last_success = Some(Instant::now());
        }
        drop(nodes);

        let first = pool.select_node_with_strategy(&NodeSelectionStrategy::RoundRobin).await;
        let second = pool.select_node_with_strategy(&NodeSelectionStrategy::RoundRobin).await;
        assert!(first.is_some() && second.is_some());
        assert_ne!(first.unwrap().tag, second.unwrap().tag);
    }
}
