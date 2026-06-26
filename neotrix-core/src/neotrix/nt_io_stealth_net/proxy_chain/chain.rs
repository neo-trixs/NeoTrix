use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use futures::future::join_all;
use rand::Rng;
use tokio::sync::RwLock;
use tokio::time::sleep;

use super::super::rotation_coordinator::{RotationCoordinator, RotationDomain};
use super::types::{
    ProxyHealth, ProxyNode, ProxyProtocol, ProxySelectionRule, CONNECT_TIMEOUT_SECS,
    DEFAULT_ROTATION_INTERVAL_SECS, FAILOVER_LATENCY_THRESHOLD_MS, FAILOVER_THRESHOLD_SUCCESS_RATE,
    PROBE_INTERVAL_MS, QUICK_PROBE_TIMEOUT_MS,
};

/// 动态代理链 — 每15秒轮转每个节点的代理，策略也动态切换
#[derive(Debug)]
pub struct DynamicProxyChain {
    name: String,
    layers: Vec<RwLock<Vec<ProxyNode>>>,
    active: RwLock<Vec<usize>>,
    rotation_interval_secs: u64,
    running: AtomicBool,
    rotation_count: AtomicU64,
    current_rule: RwLock<ProxySelectionRule>,
    rule_rotation_count: AtomicU64,
    latency_cache: RwLock<HashMap<String, f64>>,
    coordinator: RwLock<Option<Arc<RotationCoordinator>>>,
}

impl DynamicProxyChain {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            layers: Vec::new(),
            active: RwLock::new(Vec::new()),
            rotation_interval_secs: DEFAULT_ROTATION_INTERVAL_SECS,
            running: AtomicBool::new(false),
            rotation_count: AtomicU64::new(0),
            current_rule: RwLock::new(ProxySelectionRule::Random),
            rule_rotation_count: AtomicU64::new(0),
            latency_cache: RwLock::new(HashMap::new()),
            coordinator: RwLock::new(None),
        }
    }

    pub async fn update_latency(&self, label: &str, latency_ms: f64) {
        self.latency_cache
            .write()
            .await
            .insert(label.to_string(), latency_ms);
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn rotation_interval(&self) -> u64 {
        self.rotation_interval_secs
    }

    pub async fn set_coordinator(&self, coord: Arc<RotationCoordinator>) {
        *self.coordinator.write().await = Some(coord);
    }

    pub fn with_rotation_interval(mut self, secs: u64) -> Self {
        self.rotation_interval_secs = secs;
        self
    }

    pub fn add_layer(&mut self, nodes: Vec<ProxyNode>) {
        self.layers.push(RwLock::new(nodes));
    }

    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }

    pub async fn add_layer_adaptive(&mut self, new_nodes: Vec<ProxyNode>) {
        let last_label = self.current_exit_label().await;
        if let Some(label) = last_label {
            if let Some(old) = parse_url_to_node(&label) {
                let mut pool = vec![old];
                pool.extend(new_nodes);
                self.layers.push(RwLock::new(pool));
                let mut active = self.active.write().await;
                active.push(0);
            }
        }
    }

    pub async fn remove_layer_adaptive(&mut self) -> bool {
        if self.layers.len() <= 1 {
            return false;
        }
        self.layers.pop();
        let mut active = self.active.write().await;
        if !active.is_empty() {
            active.pop();
        }
        true
    }

    pub async fn rotate_rule(&self) {
        let new_rule = ProxySelectionRule::random();
        *self.current_rule.write().await = new_rule;
        self.rule_rotation_count.fetch_add(1, Ordering::Relaxed);
    }

    pub async fn current_rule(&self) -> ProxySelectionRule {
        *self.current_rule.read().await
    }

    async fn select_nodes_by_rule(&self) -> Vec<usize> {
        let rule = *self.current_rule.read().await;
        let mut rng = rand::thread_rng();
        let mut active = Vec::new();
        let latency_cache = self.latency_cache.read().await;

        for layer in &self.layers {
            let pool = layer.read().await;
            if pool.is_empty() {
                active.push(0);
                continue;
            }
            let idx = match rule {
                ProxySelectionRule::Random => rng.gen_range(0..pool.len()),
                ProxySelectionRule::Weighted => {
                    let total_weight: f64 = pool.iter().map(|n| n.weight).sum();
                    if total_weight <= 0.0 {
                        rng.gen_range(0..pool.len())
                    } else {
                        let mut roll = rng.gen::<f64>() * total_weight;
                        pool.iter()
                            .position(|n| {
                                roll -= n.weight;
                                roll <= 0.0
                            })
                            .unwrap_or_else(|| rng.gen_range(0..pool.len()))
                    }
                }
                ProxySelectionRule::LowestLatency => {
                    let mut best = 0usize;
                    let mut best_lat = f64::MAX;
                    for (i, node) in pool.iter().enumerate() {
                        let lat = latency_cache.get(&node.label).copied().unwrap_or(f64::MAX);
                        if lat < best_lat {
                            best_lat = lat;
                            best = i;
                        }
                    }
                    best
                }
                ProxySelectionRule::HighestSuccess => {
                    let mut best = 0usize;
                    let mut best_w = f64::MIN;
                    for (i, node) in pool.iter().enumerate() {
                        if node.weight > best_w {
                            best_w = node.weight;
                            best = i;
                        }
                    }
                    best
                }
                ProxySelectionRule::GeoRoundRobin => {
                    (self.rotation_count.load(Ordering::Relaxed) as usize) % pool.len()
                }
            };
            active.push(idx);
        }
        active
    }

    pub async fn rotate_all(&self) {
        let count = self.rotation_count.fetch_add(1, Ordering::Relaxed);
        if count.is_multiple_of(3) {
            self.rotate_rule().await;
        }
        let new_active = self.select_nodes_by_rule().await;
        let mut active = self.active.write().await;
        *active = new_active;
    }

    pub async fn current_chain_urls(&self) -> Vec<String> {
        let active = self.active.read().await;
        let mut urls = Vec::new();
        for (layer_idx, &node_idx) in active.iter().enumerate() {
            if layer_idx < self.layers.len() {
                let pool = self.layers[layer_idx].read().await;
                if node_idx < pool.len() {
                    urls.push(pool[node_idx].secret_url());
                }
            }
        }
        urls
    }

    pub async fn current_exit_url(&self) -> Option<String> {
        let urls = self.current_chain_urls().await;
        urls.last().cloned()
    }

    pub async fn current_exit_label(&self) -> Option<String> {
        let active = self.active.read().await;
        let last_idx = active.len().checked_sub(1)?;
        let node_idx = active.get(last_idx).copied()?;
        let layer = self.layers.get(last_idx)?;
        let pool = layer.read().await;
        pool.get(node_idx).map(|n| n.label.clone())
    }

    pub async fn current_exit_geo(&self) -> Option<String> {
        let active = self.active.read().await;
        let last_idx = active.len().checked_sub(1)?;
        let node_idx = active.get(last_idx).copied()?;
        let layer = self.layers.get(last_idx)?;
        let pool = layer.read().await;
        pool.get(node_idx).and_then(|n| n.geo_tag.clone())
    }

    pub async fn current_entry_url(&self) -> Option<String> {
        let urls = self.current_chain_urls().await;
        urls.first().cloned()
    }

    pub async fn start_rotation_loop(self: Arc<Self>) {
        if self.running.swap(true, Ordering::AcqRel) {
            return;
        }
        self.rotate_all().await;

        loop {
            if !self.running.load(Ordering::Acquire) {
                break;
            }
            let coord_opt = self.coordinator.read().await.clone();
            if let Some(coord) = coord_opt {
                let secs = coord
                    .seconds_until_rotation(RotationDomain::ProxyChain)
                    .await;
                sleep(Duration::from_secs_f64(secs.clamp(0.5, 10.0))).await;
                if coord.should_rotate(RotationDomain::ProxyChain).await {
                    self.rotate_all().await;
                    coord.mark_rotated(RotationDomain::ProxyChain).await;
                }
            } else {
                sleep(Duration::from_secs(self.rotation_interval_secs)).await;
                self.rotate_all().await;
            }
        }
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::Release);
    }

    pub fn rotation_count(&self) -> u64 {
        self.rotation_count.load(Ordering::Relaxed)
    }

    pub async fn describe(&self) -> DynamicChainSummary {
        let active = self.active.read().await;
        let mut layer_info = Vec::new();
        for i in 0..self.layers.len() {
            let layer = self.layers[i].read().await;
            let pool_size = layer.len();
            let idx = active.get(i).copied().unwrap_or(0);
            let label = if idx < layer.len() {
                layer[idx].label.clone()
            } else {
                "none".into()
            };
            layer_info.push(format!(
                "layer[{}] {} ({}/{} avail)",
                i,
                label,
                idx + 1,
                pool_size
            ));
        }

        DynamicChainSummary {
            name: self.name.clone(),
            layer_count: self.layers.len(),
            rotation_interval_secs: self.rotation_interval_secs,
            rotation_count: self.rotation_count(),
            active: active.clone(),
            layers: layer_info,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DynamicChainSummary {
    pub name: String,
    pub layer_count: usize,
    pub rotation_interval_secs: u64,
    pub rotation_count: u64,
    pub active: Vec<usize>,
    pub layers: Vec<String>,
}

/// 代理池 — 管理多个动态链 + 健康检测
pub struct ProxyPool {
    chains: RwLock<Vec<Arc<DynamicProxyChain>>>,
    health: RwLock<HashMap<String, ProxyHealth>>,
    active_index: AtomicU64,
    running: AtomicBool,
}

impl Default for ProxyPool {
    fn default() -> Self {
        Self::new()
    }
}

impl ProxyPool {
    pub fn new() -> Self {
        Self {
            chains: RwLock::new(Vec::new()),
            health: RwLock::new(HashMap::new()),
            active_index: AtomicU64::new(0),
            running: AtomicBool::new(true),
        }
    }

    pub async fn add_chain(&self, chain: Arc<DynamicProxyChain>) {
        let mut chains = self.chains.write().await;
        chains.push(chain);
    }

    pub async fn chains_len(&self) -> usize {
        self.chains.read().await.len()
    }

    pub async fn get_active_chain(&self) -> Option<Arc<DynamicProxyChain>> {
        let chains = self.chains.read().await;
        if chains.is_empty() {
            return None;
        }
        let idx = self.active_index.load(Ordering::Relaxed) as usize;
        chains.get(idx % chains.len()).cloned()
    }

    pub async fn rotate_chain(&self) {
        self.active_index.fetch_add(1, Ordering::Relaxed);
    }

    pub fn stop_probe_loop(&self) {
        self.running.store(false, Ordering::Relaxed);
    }

    async fn check_node_inner(&self, node: &ProxyNode, timeout: Duration) -> ProxyHealth {
        let start = Instant::now();
        let is_ok = tokio::time::timeout(
            timeout,
            tokio::net::TcpStream::connect(format!("{}:{}", node.host, node.port)),
        )
        .await
        .is_ok();

        let latency = start.elapsed().as_millis() as f64;
        let label = node.label.clone();
        let mut health = self.health.write().await;
        let entry = health.entry(label.clone()).or_insert(ProxyHealth {
            node_label: label.clone(),
            last_check: None,
            success_count: 0,
            fail_count: 0,
            avg_latency_ms: 0.0,
        });
        entry.last_check = Some(Instant::now());
        if is_ok {
            entry.success_count += 1;
            entry.avg_latency_ms = (entry.avg_latency_ms * 0.7) + (latency * 0.3);
        } else {
            entry.fail_count += 1;
        }

        let chains = self.chains.read().await;
        for chain in chains.iter() {
            chain.update_latency(&label, entry.avg_latency_ms).await;
        }

        ProxyHealth {
            node_label: label,
            last_check: Some(Instant::now()),
            success_count: entry.success_count,
            fail_count: entry.fail_count,
            avg_latency_ms: entry.avg_latency_ms,
        }
    }

    pub async fn check_node(&self, node: &ProxyNode) -> ProxyHealth {
        self.check_node_inner(node, Duration::from_secs(CONNECT_TIMEOUT_SECS))
            .await
    }

    pub async fn quick_check_node(&self, node: &ProxyNode) -> ProxyHealth {
        self.check_node_inner(node, Duration::from_millis(QUICK_PROBE_TIMEOUT_MS))
            .await
    }

    pub async fn check_all(&self) -> Vec<ProxyHealth> {
        let chains = self.chains.read().await;
        let mut results = Vec::new();
        for chain in chains.iter() {
            let urls = chain.current_chain_urls().await;
            for url_str in &urls {
                if let Some(node) = parse_url_to_node(url_str) {
                    let health = self.check_node(&node).await;
                    results.push(health);
                }
            }
        }
        results
    }

    pub async fn check_all_parallel(&self) -> Vec<ProxyHealth> {
        let chains = self.chains.read().await;
        let mut all_nodes = Vec::new();
        let mut node_chain_map: Vec<(String, usize)> = Vec::new();

        for (ci, chain) in chains.iter().enumerate() {
            let urls = chain.current_chain_urls().await;
            for url_str in &urls {
                if let Some(node) = parse_url_to_node(url_str) {
                    all_nodes.push(node);
                    node_chain_map.push((url_str.clone(), ci));
                }
            }
        }

        if all_nodes.is_empty() {
            return Vec::new();
        }

        let tasks: Vec<_> = all_nodes
            .iter()
            .map(|node| self.quick_check_node(node))
            .collect();
        join_all(tasks).await
    }

    pub async fn start_probe_loop(self: Arc<Self>, interval_ms: u64) {
        let actual_interval = if interval_ms == 0 {
            PROBE_INTERVAL_MS
        } else {
            interval_ms
        };
        self.running.store(true, Ordering::Relaxed);
        while self.running.load(Ordering::Relaxed) {
            sleep(Duration::from_millis(actual_interval)).await;
            let results = self.check_all_parallel().await;
            self.auto_failover(&results).await;
        }
    }

    pub async fn auto_failover(&self, results: &[ProxyHealth]) {
        let chains = self.chains.read().await;
        let active_idx = self.active_index.load(Ordering::Relaxed) as usize;
        if chains.is_empty() {
            return;
        }
        let current_chain = &chains[active_idx % chains.len()];
        let current_urls = current_chain.current_chain_urls().await;

        let mut total_latency = 0.0f64;
        let mut failed_nodes = 0usize;
        let mut checked = 0usize;

        for result in results {
            if parse_url_to_node(&result.node_label).is_some_and(|parsed| {
                let key = format!("{}:{}", parsed.host, parsed.port);
                current_urls.iter().any(|u| u.contains(&key))
            }) {
                checked += 1;
                total_latency += result.avg_latency_ms;
                if result.fail_count > result.success_count
                    && result.success_count + result.fail_count > 2
                {
                    failed_nodes += 1;
                }
            }
        }

        if checked == 0 {
            return;
        }

        let avg_latency = total_latency / checked as f64;
        let fail_ratio = failed_nodes as f64 / checked as f64;
        let success_rate = 1.0 - fail_ratio;

        if success_rate < FAILOVER_THRESHOLD_SUCCESS_RATE
            || avg_latency > FAILOVER_LATENCY_THRESHOLD_MS
        {
            drop(chains);
            self.rotate_chain().await;
        }
    }

    pub async fn health_summary(&self) -> ProxyPoolSummary {
        let health = self.health.read().await;
        let chains = self.chains.read().await;
        let total_nodes: usize = chains.iter().map(|c| c.layer_count()).sum();
        let healthy_nodes = health.values().filter(|h| h.is_healthy(0.5)).count();
        ProxyPoolSummary {
            chain_count: chains.len(),
            total_nodes,
            healthy_nodes,
            active_chain_index: self.active_index.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProxyPoolSummary {
    pub chain_count: usize,
    pub total_nodes: usize,
    pub healthy_nodes: usize,
    pub active_chain_index: u64,
}

/// 从 URL 字符串解析为 ProxyNode（用于健康检查）
pub(crate) fn parse_url_to_node(url_str: &str) -> Option<ProxyNode> {
    let parsed = match url::Url::parse(url_str) {
        Ok(u) => u,
        Err(e) => {
            log::warn!("[proxy-chain] parse URL: {}", e);
            return None;
        }
    };
    let protocol = match parsed.scheme() {
        "http" => ProxyProtocol::Http,
        "https" => ProxyProtocol::Https,
        "socks4" => ProxyProtocol::Socks4,
        "socks5" => ProxyProtocol::Socks5,
        _ => return None,
    };
    let host = parsed.host_str()?.to_string();
    let port = parsed.port().unwrap_or_else(|| protocol.default_port());
    // OWASP: 标签不包含认证凭证
    let label = format!("{}://{}:{}", protocol.as_url_scheme(), host, port);
    let username = parsed.username().to_string();
    let password = parsed.password().map(|s| s.to_string());
    let has_username = !username.is_empty();
    let username = if has_username { Some(username) } else { None };
    Some(ProxyNode {
        protocol,
        host,
        port,
        username,
        password: password.filter(|_| has_username),
        geo_tag: None,
        label,
        weight: 1.0,
    })
}
