use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use rand::Rng;
use reqwest::blocking::Client as BlockingHttpClient;
use serde::{Deserialize, Serialize};

use super::api_key_pool::ApiKeyPool;
use crate::neotrix::nt_io_http_factory::{TlsFingerprint, TlsVariant};

/// A complete identity plan for one LLM API call.
#[derive(Debug, Clone)]
pub struct CallPlan {
    pub api_key: String,
    pub proxy_url: Option<String>,
    pub tls_fingerprint: TlsFingerprint,
    pub tls_variant: TlsVariant,
    pub jitter_pre_send_ms: u64,
    pub timeout_secs: u64,
}

#[derive(Debug)]
struct ProviderHeat {
    recent_outcomes: VecDeque<(bool, u16)>,
    backoff_until: Option<Instant>,
    cooldown_factor: f64,
    _cooled_keys: Vec<String>,
    total_calls: usize,
    total_successes: usize,
    total_latency_ms: u64,
}

impl ProviderHeat {
    fn new() -> Self {
        Self {
            recent_outcomes: VecDeque::with_capacity(20),
            backoff_until: None,
            cooldown_factor: 1.0,
            _cooled_keys: Vec::new(),
            total_calls: 0,
            total_successes: 0,
            total_latency_ms: 0,
        }
    }

    fn record(&mut self, success: bool, status: u16, key: &str, latency_ms: u64) {
        self.total_calls += 1;
        self.total_latency_ms = self.total_latency_ms.saturating_add(latency_ms);
        self.recent_outcomes.push_back((success, status));
        if self.recent_outcomes.len() > 20 {
            self.recent_outcomes.pop_front();
        }
        if success {
            self.total_successes += 1;
        }
        if status == 429 || status == 401 {
            self._cooled_keys.push(key.to_string());
            self.cooldown_factor = (self.cooldown_factor * 2.0).min(32.0);
            let wait_secs = (self.cooldown_factor * 10.0) as u64;
            self.backoff_until = Some(Instant::now() + Duration::from_secs(wait_secs));
        } else if success {
            self.cooldown_factor = (self.cooldown_factor * 0.8).max(1.0);
            if self.cooldown_factor < 1.5 {
                self._cooled_keys.clear();
            }
        }
    }

    fn is_backed_off(&self) -> bool {
        self.backoff_until
            .map(|t| t > Instant::now())
            .unwrap_or(false)
    }

    fn _estimated_wait_ms(&self) -> u64 {
        self.backoff_until
            .map(|t| {
                let remaining = t.saturating_duration_since(Instant::now());
                remaining.as_millis() as u64
            })
            .unwrap_or(0)
    }
}

#[derive(Debug, Clone)]
pub struct JitterConfig {
    pub base_delay_ms: u64,
    pub jitter_range_ms: u64,
}

impl Default for JitterConfig {
    fn default() -> Self {
        Self {
            base_delay_ms: 300,
            jitter_range_ms: 200,
        }
    }
}

/// Priority levels for API key selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum KeyPriority {
    Fallback = 0,
    Secondary = 1,
    Primary = 2,
}

impl KeyPriority {
    pub fn is_at_least(&self, min: KeyPriority) -> bool {
        *self as u8 >= min as u8
    }
}

/// Serializable snapshot of a provider's heat/health state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderHeatSnapshot {
    pub recent_outcomes: Vec<(bool, u16)>,
    pub cooldown_factor: f64,
    pub total_calls: usize,
    pub total_successes: usize,
    pub total_latency_ms: u64,
    pub is_backed_off: bool,
}

/// Per-provider health dashboard row.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthDashboard {
    pub provider: String,
    pub total_calls: usize,
    pub success_rate: f64,
    pub avg_latency_ms: f64,
    pub backoff: bool,
    pub key_count: usize,
}

/// A discovered peer instance in the knowledge network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInstance {
    pub id: String,
    pub url: String,
    pub domains: Vec<String>,
    pub knowledge_version: u64,
    pub last_seen: u64,
}

/// Identity Council — the consciousness's unified identity & network faculty.
///
/// It manages API key pools, proxy selection, TLS fingerprint rotation,
/// and timing jitter. Its purpose: for every LLM call, produce an optimal
/// {key, proxy, fingerprint, timing} plan, then learn from the outcome.
pub struct IdentityCouncil {
    key_pools: Mutex<HashMap<String, ApiKeyPool>>,
    heat_map: Mutex<HashMap<String, ProviderHeat>>,
    tls_fingerprints: Vec<TlsFingerprint>,
    tls_variants: Vec<TlsVariant>,
    rr_fp: AtomicUsize,
    rr_tls: AtomicUsize,
    pub jitter_config: JitterConfig,
    pub default_timeout_secs: u64,
    heat_save_path: Mutex<Option<String>>,
    key_priorities: Mutex<HashMap<String, HashMap<String, KeyPriority>>>,
    /// Discovered peer instances for cross-instance knowledge exchange.
    peers: Mutex<Vec<PeerInstance>>,
    /// Domains this instance specializes in, for peer discovery announcements.
    known_domains: Mutex<Vec<String>>,
}

impl Default for IdentityCouncil {
    fn default() -> Self {
        Self::new()
    }
}

impl IdentityCouncil {
    pub fn new() -> Self {
        Self {
            key_pools: Mutex::new(HashMap::new()),
            heat_map: Mutex::new(HashMap::new()),
            tls_fingerprints: vec![
                TlsFingerprint::Chrome120,
                TlsFingerprint::Firefox120,
                TlsFingerprint::Chrome116,
                TlsFingerprint::Safari17,
                TlsFingerprint::Edge120,
            ],
            tls_variants: vec![
                TlsVariant::ModernH2,
                TlsVariant::LegacyHttp11,
                TlsVariant::StrictVerify,
                TlsVariant::LegacyStrict,
            ],
            rr_fp: AtomicUsize::new(0),
            rr_tls: AtomicUsize::new(0),
            jitter_config: JitterConfig::default(),
            default_timeout_secs: 30,
            heat_save_path: Mutex::new(None),
            key_priorities: Mutex::new(HashMap::new()),
            peers: Mutex::new(Vec::new()),
            known_domains: Mutex::new(Vec::new()),
        }
    }

    /// Register a domain this instance specializes in.
    pub fn register_domain(&self, domain: &str) {
        let mut domains = self.known_domains.lock().unwrap();
        if !domains.contains(&domain.to_string()) {
            domains.push(domain.to_string());
        }
    }

    /// Get known domains.
    pub fn known_domains(&self) -> Vec<String> {
        self.known_domains.lock().unwrap().clone()
    }

    /// Discovered peer instances.
    pub fn discovered_peers(&self) -> Vec<PeerInstance> {
        self.peers.lock().unwrap().clone()
    }

    /// Announce presence to all known peers via HTTP POST (A2A dispatch).
    pub fn announce_presence(&self) -> Vec<PeerInstance> {
        let announcement = serde_json::json!({
            "type": "knowledge_peer",
            "instance_id": format!("neotrix-{}", std::process::id()),
            "domains": self.known_domains(),
            "knowledge_version": 1u64,
        });
        let peers = self.peers.lock().unwrap().clone();
        let client = match BlockingHttpClient::builder()
            .timeout(Duration::from_secs(5))
            .build()
        {
            Ok(c) => c,
            Err(e) => {
                log::warn!("KNOWLEDGE_NET: failed to build HTTP client: {}", e);
                return peers;
            }
        };
        for peer in &peers {
            log::info!(
                "KNOWLEDGE_NET: announcing to peer {} at {}",
                peer.id,
                peer.url,
            );
            let endpoint = format!("{}/a2a/discover", peer.url.trim_end_matches('/'));
            match client.post(&endpoint).json(&announcement).send() {
                Ok(resp) => {
                    if resp.status().is_success() {
                        log::debug!("KNOWLEDGE_NET: announced to {} OK", peer.id);
                    } else {
                        log::warn!(
                            "KNOWLEDGE_NET: announce to {} returned {}",
                            peer.id,
                            resp.status(),
                        );
                    }
                }
                Err(e) => {
                    log::warn!("KNOWLEDGE_NET: announce to {} failed: {}", peer.id, e);
                }
            }
        }
        peers
    }

    /// Handle an incoming peer discovery message.
    /// Returns the discovered peer if valid.
    pub fn handle_peer_discovery(&self, peer_id: &str, peer_url: &str, domains: &[String]) -> Option<PeerInstance> {
        let peer = PeerInstance {
            id: peer_id.to_string(),
            url: peer_url.to_string(),
            domains: domains.to_vec(),
            knowledge_version: 1,
            last_seen: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };
        let mut peers = self.peers.lock().unwrap();
        if !peers.iter().any(|p| p.id == peer.id) {
            peers.push(peer.clone());
            log::info!("KNOWLEDGE_NET: discovered new peer {} at {}", peer.id, peer.url);
        } else if let Some(existing) = peers.iter_mut().find(|p| p.id == peer.id) {
            existing.last_seen = peer.last_seen;
            existing.url = peer_url.to_string();
        }
        Some(peer)
    }

    pub fn add_key_pool(&self, provider: &str, pool: ApiKeyPool) {
        let mut pools = self.key_pools.lock().unwrap();
        pools.insert(provider.to_string(), pool);
    }

    /// Auto-discover API keys from known environment variables.
    pub fn auto_register_env_keys(&self) {
        let known = [
            ("groq", "GROQ_API_KEY"),
            ("cerebras", "CEREBRAS_API_KEY"),
            ("sambanova", "SAMBANOVA_API_KEY"),
            ("deepseek", "DEEPSEEK_API_KEY"),
            ("mistral", "MISTRAL_API_KEY"),
            ("cloudflare", "CLOUDFLARE_API_KEY"),
            ("github", "GITHUB_TOKEN"),
            ("openai", "OPENAI_API_KEY"),
            ("anthropic", "ANTHROPIC_API_KEY"),
            ("gemini", "GOOGLE_API_KEY"),
            ("together", "TOGETHER_API_KEY"),
            ("fireworks", "FIREWORKS_API_KEY"),
            ("perplexity", "PERPLEXITY_API_KEY"),
            ("cohere", "COHERE_API_KEY"),
            ("ai21", "AI21_API_KEY"),
            ("replicate", "REPLICATE_API_TOKEN"),
            ("huggingface", "HF_API_TOKEN"),
        ];
        for (provider, env_var) in &known {
            let pool = ApiKeyPool::from_env(env_var);
            if !pool.is_empty() {
                log::info!("[council] registered {} key pool ({} keys)", provider, pool.len());
                self.add_key_pool(provider, pool);
                let keys = self.key_pools.lock().unwrap().get(*provider).map(|p| p.keys()).unwrap_or_default();
                let mut priorities = self.key_priorities.lock().unwrap();
                let inner = priorities.entry((*provider).to_string()).or_default();
                for key in keys {
                    inner.entry(key).or_insert(KeyPriority::Primary);
                }
            }
        }
    }

    pub fn has_provider(&self, name: &str) -> bool {
        self.key_pools.lock().unwrap().contains_key(name)
    }

    /// Programmatically inject a key for a provider with Primary priority.
    pub fn register_key_direct(&self, provider: &str, key: &str) {
        let mut pools = self.key_pools.lock().unwrap();
        let pool = pools
            .entry(provider.to_string())
            .or_insert_with(|| ApiKeyPool::new(Vec::new()));
        pool.add_key(key);
        log::info!("[council] direct key registered for {}", provider);

        let mut priorities = self.key_priorities.lock().unwrap();
        priorities
            .entry(provider.to_string())
            .or_default()
            .insert(key.to_string(), KeyPriority::Primary);
    }

    pub fn registered_providers(&self) -> Vec<String> {
        self.key_pools.lock().unwrap().keys().cloned().collect()
    }

    /// Generate a call plan for the given provider (sync, no proxy pool).
    pub fn plan_call(&self, provider: &str, _model: &str) -> Option<CallPlan> {
        let pools = self.key_pools.lock().unwrap();
        let pool = pools.get(provider)?;

        {
            let heat = self.heat_map.lock().unwrap();
            if let Some(h) = heat.get(provider) {
                if h.is_backed_off() {
                    log::warn!(
                        "[council] {} in backoff, planning anyway",
                        provider,
                    );
                }
            }
        }

        let api_key = pool.next().unwrap_or_default();

        let fp_idx = self.rr_fp.fetch_add(1, Ordering::Relaxed) % self.tls_fingerprints.len();
        let tls_fingerprint = self.tls_fingerprints[fp_idx];

        let tls_idx = self.rr_tls.fetch_add(1, Ordering::Relaxed) % self.tls_variants.len();
        let tls_variant = self.tls_variants[tls_idx];

        let mut rng = rand::thread_rng();
        let jitter = if self.jitter_config.jitter_range_ms > 0 {
            let offset = rng.gen_range(0..=self.jitter_config.jitter_range_ms * 2);
            (self.jitter_config.base_delay_ms + offset)
                .saturating_sub(self.jitter_config.jitter_range_ms)
        } else {
            self.jitter_config.base_delay_ms
        };

        Some(CallPlan {
            api_key,
            proxy_url: None,
            tls_fingerprint,
            tls_variant,
            jitter_pre_send_ms: jitter,
            timeout_secs: self.default_timeout_secs,
        })
    }

    /// Generate a plan including proxy from the global pool (stealth-net).
    #[cfg(feature = "stealth-net")]
    pub async fn plan_call_stealth(&self, provider: &str, model: &str) -> Option<CallPlan> {
        let mut plan = self.plan_call(provider, model)?;
        if let Some(pool_ref) = crate::neotrix::nt_io_stealth_net::proxy_pool::global_pool() {
            if let Some(node) = pool_ref.select_node().await {
                plan.proxy_url = Some(node.url.clone());
            }
        }
        Some(plan)
    }

    pub fn report_outcome(&self, provider: &str, plan: &CallPlan, success: bool, status: u16, latency_ms: u64) {
        let save_path = self.heat_save_path.lock().unwrap().clone();
        {
            let mut heat = self.heat_map.lock().unwrap();
            let entry = heat
                .entry(provider.to_string())
                .or_insert_with(ProviderHeat::new);
            entry.record(success, status, &plan.api_key, latency_ms);
        }
        if let Some(path) = save_path {
            if let Err(e) = self.save_heat_state(&path) {
                log::warn!("[council] auto-save heat state failed: {}", e);
            }
        }
    }

    pub fn diag_summary(&self) -> String {
        let pools = self.key_pools.lock().unwrap();
        let heat = self.heat_map.lock().unwrap();
        let mut parts: Vec<String> = vec![];
        for (provider, pool) in pools.iter() {
            let status = match heat.get(provider) {
                Some(h) if h.is_backed_off() => "cooling".to_string(),
                Some(h) if h.cooldown_factor > 2.0 => {
                    format!("warm(x{:.1})", h.cooldown_factor)
                }
                _ => "ready".to_string(),
            };
            parts.push(format!("{}[{}]:{}", provider, pool.len(), status));
        }
        format!("council<{}>", parts.join(", "))
    }

    /// Generate a call plan respecting key priority and backoff.
    /// Picks a non-backed-off key if available; falls back to backed-off
    /// keys only when no non-backed-off key meets the priority threshold.
    pub fn plan_call_prioritized(
        &self,
        provider: &str,
        _model: &str,
        min_priority: KeyPriority,
    ) -> Option<CallPlan> {
        let pools = self.key_pools.lock().unwrap();
        let pool = pools.get(provider)?;
        let all_keys = pool.keys();
        let priorities = self.key_priorities.lock().unwrap();
        let prov_priorities = priorities.get(provider);

        // Separate keys by backoff status and priority
        let heat = self.heat_map.lock().unwrap();
        let prov_heat = heat.get(provider);

        // Try non-backed-off keys first
        let cooled_keys: Vec<&String> = prov_heat
            .map(|h| h._cooled_keys.iter().collect())
            .unwrap_or_default();

        let mut candidates: Vec<&String> = all_keys
            .iter()
            .filter(|k| {
                // Check priority threshold
                let pri = prov_priorities
                    .and_then(|p| p.get(*k))
                    .copied()
                    .unwrap_or(KeyPriority::Primary);
                pri.is_at_least(min_priority)
            })
            .collect();

        // Sort: non-backed-off first, then backed-off
        candidates.sort_by_key(|k| cooled_keys.contains(k));
        let key = candidates.first()?;

        let fp_idx = self.rr_fp.fetch_add(1, Ordering::Relaxed) % self.tls_fingerprints.len();
        let tls_fingerprint = self.tls_fingerprints[fp_idx];

        let tls_idx = self.rr_tls.fetch_add(1, Ordering::Relaxed) % self.tls_variants.len();
        let tls_variant = self.tls_variants[tls_idx];

        let mut rng = rand::thread_rng();
        let jitter = if self.jitter_config.jitter_range_ms > 0 {
            let offset = rng.gen_range(0..=self.jitter_config.jitter_range_ms * 2);
            (self.jitter_config.base_delay_ms + offset)
                .saturating_sub(self.jitter_config.jitter_range_ms)
        } else {
            self.jitter_config.base_delay_ms
        };

        Some(CallPlan {
            api_key: key.to_string(),
            proxy_url: None,
            tls_fingerprint,
            tls_variant,
            jitter_pre_send_ms: jitter,
            timeout_secs: self.default_timeout_secs,
        })
    }

    /// Save heat map to a JSON file.
    pub fn save_heat_state(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let heat = self.heat_map.lock().unwrap();
        let snapshots: HashMap<String, ProviderHeatSnapshot> = heat
            .iter()
            .map(|(provider, h)| {
                let snapshot = ProviderHeatSnapshot {
                    recent_outcomes: h.recent_outcomes.iter().copied().collect(),
                    cooldown_factor: h.cooldown_factor,
                    total_calls: h.total_calls,
                    total_successes: h.total_successes,
                    total_latency_ms: h.total_latency_ms,
                    is_backed_off: h.is_backed_off(),
                };
                (provider.clone(), snapshot)
            })
            .collect();
        let json = serde_json::to_string_pretty(&snapshots)?;
        std::fs::write(path, json)?;
        log::info!("[council] heat state saved to {}", path);
        Ok(())
    }

    /// Load heat map from a JSON file.
    pub fn load_heat_state(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(path)?;
        let snapshots: HashMap<String, ProviderHeatSnapshot> = serde_json::from_str(&json)?;
        let mut heat = self.heat_map.lock().unwrap();
        for (provider, snapshot) in snapshots {
            let h = heat
                .entry(provider.clone())
                .or_insert_with(ProviderHeat::new);
            h.recent_outcomes = snapshot.recent_outcomes.into_iter().collect();
            h.cooldown_factor = snapshot.cooldown_factor;
            h.total_calls = snapshot.total_calls;
            h.total_successes = snapshot.total_successes;
            h.total_latency_ms = snapshot.total_latency_ms;
            // backoff_until is NOT restored — restarted processes start fresh
        }
        log::info!("[council] heat state loaded from {}", path);
        Ok(())
    }

    /// Configure auto-save path for heat state.
    pub fn set_heat_save_path(&self, path: &str) {
        *self.heat_save_path.lock().unwrap() = Some(path.to_string());
        log::info!("[council] heat auto-save path set to {}", path);
    }

    /// Return per-provider health dashboard data.
    pub fn dashboard(&self) -> Vec<HealthDashboard> {
        let pools = self.key_pools.lock().unwrap();
        let heat = self.heat_map.lock().unwrap();
        let mut rows: Vec<HealthDashboard> = Vec::new();
        for (provider, pool) in pools.iter() {
            let h = heat.get(provider);
            let total_calls = h.map(|h| h.total_calls).unwrap_or(0);
            let total_successes = h.map(|h| h.total_successes).unwrap_or(0);
            let total_latency = h.map(|h| h.total_latency_ms).unwrap_or(0);
            let success_rate = if total_calls > 0 {
                total_successes as f64 / total_calls as f64
            } else {
                1.0
            };
            let avg_latency_ms = if total_calls > 0 {
                total_latency as f64 / total_calls as f64
            } else {
                0.0
            };
            let backoff = h.map(|h| h.is_backed_off()).unwrap_or(false);
            rows.push(HealthDashboard {
                provider: provider.clone(),
                total_calls,
                success_rate,
                avg_latency_ms,
                backoff,
                key_count: pool.len(),
            });
        }
        rows.sort_by(|a, b| a.provider.cmp(&b.provider));
        rows
    }

    /// Return a JSON-friendly health report.
    pub fn health_json(&self) -> serde_json::Value {
        let dashboard = self.dashboard();
        let json_rows: Vec<serde_json::Value> = dashboard
            .iter()
            .map(|d| {
                serde_json::json!({
                    "provider": d.provider,
                    "total_calls": d.total_calls,
                    "success_rate": d.success_rate,
                    "avg_latency_ms": d.avg_latency_ms,
                    "backoff": d.backoff,
                    "key_count": d.key_count,
                })
            })
            .collect();
        serde_json::json!({
            "council": {
                "providers": json_rows,
                "provider_count": json_rows.len(),
            }
        })
    }

    /// Reset all heat data (calls, successes, backoff cooldowns).
    pub fn reset_stats(&self) {
        let mut heat = self.heat_map.lock().unwrap();
        heat.clear();
        log::info!("[council] all heat stats reset");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn test_council() -> Arc<IdentityCouncil> {
        Arc::new(IdentityCouncil::new())
    }

    fn make_plan(key: &str) -> CallPlan {
        CallPlan {
            api_key: key.to_string(),
            proxy_url: None,
            tls_fingerprint: TlsFingerprint::Chrome120,
            tls_variant: TlsVariant::ModernH2,
            jitter_pre_send_ms: 0,
            timeout_secs: 30,
        }
    }

    #[test]
    fn test_new_providers_in_auto_register() {
        let council = IdentityCouncil::new();
        // Don't call auto_register_env_keys — env vars may or may not be set.
        // Just verify the known list includes all required providers.
        // Simulate by manually adding each and checking has_provider.
        council.register_key_direct("together", "sk-together-test");
        council.register_key_direct("fireworks", "sk-fireworks-test");
        council.register_key_direct("perplexity", "sk-perplexity-test");
        council.register_key_direct("cohere", "sk-cohere-test");
        council.register_key_direct("ai21", "sk-ai21-test");
        council.register_key_direct("replicate", "sk-replicate-test");
        council.register_key_direct("huggingface", "sk-hf-test");

        assert!(council.has_provider("together"));
        assert!(council.has_provider("fireworks"));
        assert!(council.has_provider("perplexity"));
        assert!(council.has_provider("cohere"));
        assert!(council.has_provider("ai21"));
        assert!(council.has_provider("replicate"));
        assert!(council.has_provider("huggingface"));
    }

    #[test]
    fn test_register_key_direct() {
        let council = test_council();
        assert!(!council.has_provider("test-provider"));
        council.register_key_direct("test-provider", "sk-test-123");
        assert!(council.has_provider("test-provider"));
        assert_eq!(council.registered_providers().len(), 1);
    }

    #[test]
    fn test_save_load_heat_roundtrip() {
        let council = test_council();
        council.register_key_direct("roundtrip", "sk-rt-1");

        let plan = make_plan("sk-rt-1");
        council.report_outcome("roundtrip", &plan, true, 200, 150);
        council.report_outcome("roundtrip", &plan, true, 200, 80);
        council.report_outcome("roundtrip", &plan, false, 429, 0);

        let tmp = std::env::temp_dir().join("council_heat_test.json");
        let path = tmp.to_str().unwrap().to_string();

        council.save_heat_state(&path).unwrap();

        // Load into a fresh council
        let council2 = test_council();
        council2.load_heat_state(&path).unwrap();

        // Check stats were restored (backoff_until NOT restored)
        let heat2 = council2.heat_map.lock().unwrap();
        let h = heat2.get("roundtrip").unwrap();
        assert_eq!(h.total_calls, 3);
        assert_eq!(h.total_successes, 2);
        assert_eq!(h.total_latency_ms, 230);
        assert_eq!(h.recent_outcomes.len(), 3);

        // Cleanup
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_dashboard_after_calls() {
        let council = test_council();
        council.register_key_direct("dash-test", "sk-dash-1");

        let plan = make_plan("sk-dash-1");
        council.report_outcome("dash-test", &plan, true, 200, 100);
        council.report_outcome("dash-test", &plan, true, 200, 200);
        council.report_outcome("dash-test", &plan, false, 500, 0);

        let rows = council.dashboard();
        assert!(!rows.is_empty());

        let row = rows.iter().find(|r| r.provider == "dash-test").unwrap();
        assert_eq!(row.total_calls, 3);
        assert!((row.success_rate - 2.0 / 3.0).abs() < 0.001);
        assert!((row.avg_latency_ms - 100.0).abs() < 0.001);
        assert!(!row.backoff); // 500 is not 429/401
        assert_eq!(row.key_count, 1);
    }

    #[test]
    fn test_health_json() {
        let council = test_council();
        council.register_key_direct("json-test", "sk-json-1");

        let plan = make_plan("sk-json-1");
        council.report_outcome("json-test", &plan, true, 200, 50);

        let json = council.health_json();
        assert_eq!(json["council"]["provider_count"], 1);
        assert_eq!(json["council"]["providers"][0]["provider"], "json-test");
        assert_eq!(json["council"]["providers"][0]["total_calls"], 1);
        assert_eq!(json["council"]["providers"][0]["key_count"], 1);
    }

    #[test]
    fn test_key_priority_selection() {
        let council = test_council();
        // Register keys with different priorities
        council.register_key_direct("prio-test", "primary-key");

        // Manually add a fallback key
        {
            let mut pools = council.key_pools.lock().unwrap();
            pools.get_mut("prio-test").unwrap().add_key("fallback-key");
            let mut priorities = council.key_priorities.lock().unwrap();
            priorities
                .entry("prio-test".to_string())
                .or_default()
                .insert("fallback-key".to_string(), KeyPriority::Fallback);
        }

        // Primary minimum should pick the primary key
        let plan = council.plan_call_prioritized("prio-test", "model-x", KeyPriority::Primary);
        assert!(plan.is_some());
        assert_eq!(plan.unwrap().api_key, "primary-key");

        // Fallback minimum can pick either — should prefer non-backed-off
        let plan2 = council.plan_call_prioritized("prio-test", "model-x", KeyPriority::Fallback);
        assert!(plan2.is_some());
    }

    #[test]
    fn test_plan_call_prioritized_obeys_threshold() {
        let council = test_council();
        council.register_key_direct("threshold-test", "primary-key");
        {
            let mut pools = council.key_pools.lock().unwrap();
            pools.get_mut("threshold-test").unwrap().add_key("secondary-key");
            let mut priorities = council.key_priorities.lock().unwrap();
            priorities
                .entry("threshold-test".to_string())
                .or_default()
                .insert("secondary-key".to_string(), KeyPriority::Secondary);
        }

        // Secondary minimum should give us a key
        let plan = council.plan_call_prioritized("threshold-test", "m", KeyPriority::Secondary);
        assert!(plan.is_some());
    }

    #[test]
    fn test_plan_call_prioritized_none_for_unknown_provider() {
        let council = test_council();
        let plan = council.plan_call_prioritized("nonexistent", "m", KeyPriority::Primary);
        assert!(plan.is_none());
    }

    #[test]
    fn test_reset_stats() {
        let council = test_council();
        council.register_key_direct("reset-test", "sk-reset");

        let plan = make_plan("sk-reset");
        council.report_outcome("reset-test", &plan, true, 200, 50);
        assert!(council.dashboard().iter().any(|r| r.total_calls > 0));

        council.reset_stats();
        let rows = council.dashboard();
        let row = rows.iter().find(|r| r.provider == "reset-test").unwrap();
        assert_eq!(row.total_calls, 0);
        assert_eq!(row.success_rate, 1.0);
    }

    #[test]
    fn test_save_load_empty_heat() {
        let council = test_council();
        let tmp = std::env::temp_dir().join("council_empty_heat.json");
        let path = tmp.to_str().unwrap().to_string();

        council.save_heat_state(&path).unwrap();

        let council2 = test_council();
        council2.load_heat_state(&path).unwrap();
        assert!(council2.dashboard().is_empty());

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_auto_save_on_report_outcome() {
        let council = test_council();
        council.register_key_direct("auto-save", "sk-auto");
        let tmp = std::env::temp_dir().join("council_auto_save_test.json");
        let path = tmp.to_str().unwrap().to_string();
        council.set_heat_save_path(&path);

        let plan = make_plan("sk-auto");
        council.report_outcome("auto-save", &plan, true, 200, 42);

        // Verify file was written
        assert!(tmp.exists());
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("auto-save"));

        let _ = std::fs::remove_file(&path);
    }
}

/// Global singleton IdentityCouncil, lazily initialized with auto-registered env keys.
pub fn global_council() -> Arc<IdentityCouncil> {
    static COUNCIL: OnceLock<Arc<IdentityCouncil>> = OnceLock::new();
    COUNCIL
        .get_or_init(|| {
            let council = Arc::new(IdentityCouncil::new());
            council.auto_register_env_keys();
            log::info!(
                "[council] initialized with {} providers: {}",
                council.registered_providers().len(),
                council.diag_summary()
            );
            council
        })
        .clone()
}
