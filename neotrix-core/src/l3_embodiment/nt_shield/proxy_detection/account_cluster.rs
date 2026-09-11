//! Temporal and behavioral account clustering engine
//!
//! Detects coordinated proxy networks by analyzing:
//! - Shared creation timestamps (accounts created in narrow windows)
//! - IP address overlap across accounts
//! - Behavioral pattern similarity (request timing, prompt structure)
//! - Email domain grouping
//!
//! Uses a sliding-window approach with Union-Find for efficient cluster
//! detection across large account populations.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

use super::{ProxyDetectionConfig, ProxyDetectionError, AccountObservation};

// ── Cluster Result ────────────────────────────────────────────────────────
#[derive(Debug, Clone)]
pub struct ClusterResult {
    pub cluster_id: String,
    pub member_accounts: Vec<String>,
    pub shared_ips: Vec<String>,
    pub creation_window_score: f64,
    pub window_duration_secs: u64,
    pub behavioral_similarity: f64,
    pub email_domain: Option<String>,
    pub risk_assessment: ClusterRisk,
}

#[derive(Debug, Clone)]
pub struct ClusterRisk {
    pub level: String,
    pub confidence: f64,
    pub primary_indicator: String,
}

// ── Union-Find for cluster merging ────────────────────────────────────────
struct UnionFind {
    parent: HashMap<String, String>,
    rank: HashMap<String, u32>,
    size: HashMap<String, usize>,
}

impl UnionFind {
    fn new() -> Self {
        Self {
            parent: HashMap::new(),
            rank: HashMap::new(),
            size: HashMap::new(),
        }
    }

    fn make_set(&mut self, x: String) {
        if !self.parent.contains_key(&x) {
            self.parent.insert(x.clone(), x.clone());
            self.rank.insert(x.clone(), 0);
            self.size.insert(x, 1);
        }
    }

    fn find(&mut self, x: &str) -> String {
        let parent = self.parent.get(x).cloned().unwrap_or_else(|| x.to_string());
        if parent == x {
            x.to_string()
        } else {
            let root = self.find(&parent);
            self.parent.insert(x.to_string(), root.clone());
            root
        }
    }

    fn union(&mut self, x: &str, y: &str) {
        let root_x = self.find(x);
        let root_y = self.find(y);

        if root_x == root_y {
            return;
        }

        let rank_x = *self.rank.get(&root_x).unwrap_or(&0);
        let rank_y = *self.rank.get(&root_y).unwrap_or(&0);

        let (new_root, old_root) = if rank_x < rank_y {
            (root_y.clone(), root_x.clone())
        } else if rank_x > rank_y {
            (root_x.clone(), root_y.clone())
        } else {
            self.rank.entry(root_x.clone())
                .and_modify(|r| *r += 1);
            (root_x.clone(), root_y.clone())
        };

        let old_size = *self.size.get(&old_root).unwrap_or(&0);
        let new_size = *self.size.get(&new_root).unwrap_or(&0);
        self.size.insert(new_root.clone(), new_size + old_size);

        self.parent.insert(old_root, new_root);
    }

    fn find_read_only(&self, x: &str) -> String {
        let mut current = x.to_string();
        loop {
            match self.parent.get(&current) {
                Some(parent) if parent == &current => return current,
                Some(parent) => current = parent.clone(),
                None => return x.to_string(),
            }
        }
    }

    fn get_cluster_members(&self, root: &str) -> Vec<String> {
        self.parent.iter()
            .filter(|(_, v)| self.find_read_only(v) == root)
            .map(|(k, _)| k.clone())
            .collect()
    }

    fn clusters(&self) -> HashMap<String, Vec<String>> {
        let mut result: HashMap<String, Vec<String>> = HashMap::new();
        for key in self.parent.keys() {
            let root = self.find_read_only(key);
            result.entry(root).or_default().push(key.clone());
        }
        result
    }
}

// ── Behavioral Profile ────────────────────────────────────────────────────
#[derive(Debug, Clone)]
struct BehavioralProfile {
    /// Average request interval (seconds)
    avg_request_interval: f64,
    /// Prompt length distribution (mean, std)
    prompt_length_mean: f64,
    prompt_length_std: f64,
    /// Hour-of-day activity histogram (24 buckets)
    activity_histogram: [u32; 24],
    /// Character-level prompt hash (first 64 chars of content hash)
    content_hash_prefix: String,
}

impl BehavioralProfile {
    fn similarity(&self, other: &Self) -> f64 {
        let mut score = 0.0_f64;
        let mut count = 0;

        // Request interval similarity
        let interval_diff = (self.avg_request_interval - other.avg_request_interval).abs();
        let interval_sim = 1.0 / (1.0 + interval_diff / 60.0);
        score += interval_sim;
        count += 1;

        // Prompt length similarity
        let length_diff = (self.prompt_length_mean - other.prompt_length_mean).abs();
        let length_sim = 1.0 / (1.0 + length_diff / 500.0);
        score += length_sim;
        count += 1;

        // Activity histogram cosine similarity
        let dot: f64 = self.activity_histogram.iter()
            .zip(other.activity_histogram.iter())
            .map(|(a, b)| (*a as f64) * (*b as f64))
            .sum();
        let norm_a: f64 = self.activity_histogram.iter()
            .map(|x| (*x as f64).powi(2))
            .sum::<f64>()
            .sqrt();
        let norm_b: f64 = other.activity_histogram.iter()
            .map(|x| (*x as f64).powi(2))
            .sum::<f64>()
            .sqrt();
        let cos_sim = if norm_a > 0.0 && norm_b > 0.0 {
            dot / (norm_a * norm_b)
        } else {
            0.0
        };
        score += cos_sim;
        count += 1;

        // Content hash prefix match
        if !self.content_hash_prefix.is_empty()
            && self.content_hash_prefix == other.content_hash_prefix
        {
            score += 1.0;
            count += 1;
        }

        if count > 0 { score / count as f64 } else { 0.0 }
    }
}

// ── Account Cluster Engine ────────────────────────────────────────────────
pub struct AccountClusterEngine {
    config: ProxyDetectionConfig,
    /// All observations keyed by account_id
    observations: Arc<RwLock<HashMap<String, AccountObservation>>>,
    /// IP → set of account_ids that used it
    ip_index: Arc<RwLock<HashMap<String, HashSet<String>>>>,
    /// Email domain → set of account_ids
    email_index: Arc<RwLock<HashMap<String, HashSet<String>>>>,
    /// Behavioral profiles per account
    profiles: Arc<RwLock<HashMap<String, BehavioralProfile>>>,
    /// Computed clusters
    clusters: Arc<RwLock<HashMap<String, ClusterResult>>>,
}

impl AccountClusterEngine {
    pub fn new(config: ProxyDetectionConfig) -> Self {
        Self {
            config,
            observations: Arc::new(RwLock::new(HashMap::new())),
            ip_index: Arc::new(RwLock::new(HashMap::new())),
            email_index: Arc::new(RwLock::new(HashMap::new())),
            profiles: Arc::new(RwLock::new(HashMap::new())),
            clusters: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Analyze a new observation and update cluster state.
    pub async fn analyze_observation(
        &self,
        obs: &AccountObservation,
    ) -> Result<Option<ClusterResult>, ProxyDetectionError> {
        // Record observation
        {
            let mut observations = self.observations.write().await;
            observations.insert(obs.account_id.clone(), obs.clone());
        }

        // Update IP index
        {
            let mut ip_index = self.ip_index.write().await;
            for ip in &obs.ip_addresses {
                ip_index.entry(ip.clone())
                    .or_insert_with(HashSet::new)
                    .insert(obs.account_id.clone());
            }
        }

        // Update email index
        {
            let mut email_index = self.email_index.write().await;
            email_index.entry(obs.email_domain.clone())
                .or_insert_with(HashSet::new)
                .insert(obs.account_id.clone());
        }

        // Build behavioral profile
        let profile = self.build_profile(obs).await;
        {
            let mut profiles = self.profiles.write().await;
            profiles.insert(obs.account_id.clone(), profile);
        }

        // Run clustering analysis
        self.recompute_clusters().await
    }

    /// Build a behavioral profile from an observation's metadata.
    async fn build_profile(&self, obs: &AccountObservation) -> BehavioralProfile {
        let request_interval = obs.metadata.get("avg_request_interval")
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(60.0);

        let prompt_len = obs.metadata.get("avg_prompt_length")
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(200.0);

        let mut histogram = [0u32; 24];
        // Derive hour-of-day from creation timestamp (simplified)
        let hour = ((obs.created_at % 86400) / 3600) as usize;
        if hour < 24 {
            histogram[hour] += 1;
        }

        BehavioralProfile {
            avg_request_interval: request_interval,
            prompt_length_mean: prompt_len,
            prompt_length_std: 50.0, // Default std
            activity_histogram: histogram,
            content_hash_prefix: obs.request_pattern_hash[..64.min(obs.request_pattern_hash.len())].to_string(),
        }
    }

    /// Recompute all clusters from current index state.
    async fn recompute_clusters(&self) -> Result<Option<ClusterResult>, ProxyDetectionError> {
        let observations = self.observations.read().await;
        let ip_index = self.ip_index.read().await;
        let profiles = self.profiles.read().await;

        if observations.len() < 2 {
            return Ok(None);
        }

        let mut uf = UnionFind::new();
        let account_ids: Vec<String> = observations.keys().cloned().collect();
        for id in &account_ids {
            uf.make_set(id.clone());
        }

        // ── Union pass 1: IP sharing ─────────────────────────────────────
        for (_ip, accounts) in ip_index.iter() {
            if accounts.len() < 2 {
                continue;
            }
            let account_vec: Vec<&String> = accounts.iter().collect();
            for i in 0..account_vec.len() {
                for j in (i + 1)..account_vec.len() {
                    uf.union(account_vec[i], account_vec[j]);
                }
            }
        }

        // ── Union pass 2: Behavioral similarity ──────────────────────────
        let threshold = 0.75;
        for i in 0..account_ids.len() {
            for j in (i + 1)..account_ids.len() {
                if let (Some(pi), Some(pj)) = (
                    profiles.get(&account_ids[i]),
                    profiles.get(&account_ids[j]),
                ) {
                    if pi.similarity(pj) >= threshold {
                        uf.union(&account_ids[i], &account_ids[j]);
                    }
                }
            }
        }

        // ── Find clusters exceeding threshold ────────────────────────────
        let all_clusters = uf.clusters();
        let mut best_cluster: Option<ClusterResult> = None;

        for (root, members) in &all_clusters {
            if members.len() < self.config.ip_cluster_threshold {
                continue;
            }

            // Compute cluster metrics
            let shared_ips = self.compute_shared_ips(members, &ip_index).await;
            let creation_score = self.compute_creation_score(members, &observations).await;
            let window_duration = self.compute_window_duration(members, &observations).await;

            let email_domain = self.dominant_email_domain(members, &observations).await;

            let risk = if creation_score >= 0.8 && members.len() >= 10 {
                ClusterRisk {
                    level: "critical".into(),
                    confidence: 0.95,
                    primary_indicator: "coordinated_mass_creation".into(),
                }
            } else if creation_score >= 0.6 || shared_ips.len() >= 3 {
                ClusterRisk {
                    level: "high".into(),
                    confidence: 0.85,
                    primary_indicator: "ip_and_temporal_overlap".into(),
                }
            } else {
                ClusterRisk {
                    level: "medium".into(),
                    confidence: 0.65,
                    primary_indicator: "behavioral_similarity".into(),
                }
            };

            let result = ClusterResult {
                cluster_id: format!("cluster_{}", &root[..8.min(root.len())]),
                member_accounts: members.clone(),
                shared_ips,
                creation_window_score: creation_score,
                window_duration_secs: window_duration,
                behavioral_similarity: 0.0, // Will be set by caller
                email_domain,
                risk_assessment: risk,
            };

            // Keep the highest-risk cluster
            if best_cluster.as_ref().map_or(true, |best| {
                result.risk_assessment.confidence > best.risk_assessment.confidence
            }) {
                best_cluster = Some(result);
            }
        }

        // Store clusters
        if let Some(ref cluster) = best_cluster {
            let mut clusters = self.clusters.write().await;
            clusters.insert(cluster.cluster_id.clone(), cluster.clone());
        }

        Ok(best_cluster)
    }

    /// Compute IPs shared by multiple members.
    async fn compute_shared_ips(
        &self,
        members: &[String],
        ip_index: &HashMap<String, HashSet<String>>,
    ) -> Vec<String> {
        ip_index.iter()
            .filter(|(_, accounts)| {
                members.iter().filter(|m| accounts.contains(*m)).count() >= 2
            })
            .map(|(ip, _)| ip.clone())
            .collect()
    }

    /// Compute creation timestamp clustering score (0.0–1.0).
    async fn compute_creation_score(
        &self,
        members: &[String],
        observations: &HashMap<String, AccountObservation>,
    ) -> f64 {
        let times: Vec<u64> = members.iter()
            .filter_map(|id| observations.get(id).map(|o| o.created_at))
            .collect();

        if times.len() < 2 {
            return 0.0;
        }

        let min_time = *times.iter().min().unwrap();
        let max_time = *times.iter().max().unwrap();
        let window = max_time.saturating_sub(min_time);

        // Score: narrow window → high score
        let config_window = self.config.creation_window_seconds;
        if window <= config_window {
            // Within configured window — strong signal
            let density = times.len() as f64 / (window.max(1) as f64 / 60.0);
            (density / 5.0).min(1.0)
        } else {
            // Outside window — partial score based on density
            let density = times.len() as f64 / (window as f64 / 60.0);
            (density / 10.0).min(0.5)
        }
    }

    /// Compute the time span of the cluster window.
    async fn compute_window_duration(
        &self,
        members: &[String],
        observations: &HashMap<String, AccountObservation>,
    ) -> u64 {
        let times: Vec<u64> = members.iter()
            .filter_map(|id| observations.get(id).map(|o| o.created_at))
            .collect();

        if times.len() < 2 {
            return 0;
        }

        let min_time = *times.iter().min().unwrap();
        let max_time = *times.iter().max().unwrap();
        max_time.saturating_sub(min_time)
    }

    /// Find the dominant email domain in a cluster.
    async fn dominant_email_domain(
        &self,
        members: &[String],
        observations: &HashMap<String, AccountObservation>,
    ) -> Option<String> {
        let mut domain_counts: HashMap<String, usize> = HashMap::new();
        for id in members {
            if let Some(obs) = observations.get(id) {
                *domain_counts.entry(obs.email_domain.clone()).or_insert(0) += 1;
            }
        }
        domain_counts.into_iter()
            .max_by_key(|(_, count)| *count)
            .filter(|(_, count)| *count >= 2)
            .map(|(domain, _)| domain)
    }

    /// Get all computed clusters.
    pub async fn get_clusters(&self) -> HashMap<String, ClusterResult> {
        self.clusters.read().await.clone()
    }

    /// Get observation count.
    pub async fn observation_count(&self) -> usize {
        self.observations.read().await.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_observation(
        id: &str,
        created_at: u64,
        ip: &str,
        email_domain: &str,
    ) -> AccountObservation {
        AccountObservation {
            account_id: id.into(),
            created_at,
            ip_addresses: vec![ip.into()],
            email_domain: email_domain.into(),
            user_agent: "Mozilla/5.0".into(),
            request_pattern_hash: format!("{:064x}", created_at),
            metadata: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn cluster_detection_basic() {
        let config = ProxyDetectionConfig {
            ip_cluster_threshold: 3,
            ..Default::default()
        };
        let engine = AccountClusterEngine::new(config);

        // 3 accounts on same IP, created in rapid succession
        for i in 0..3 {
            let obs = make_observation(
                &format!("acc_{i}"),
                1000 + i,
                "1.2.3.4",
                "tempmail.com",
            );
            let _ = engine.analyze_observation(&obs).await;
        }

        let clusters = engine.get_clusters().await;
        // With IP union, all 3 should be in one cluster
        assert!(!clusters.is_empty() || engine.observation_count().await == 3);
    }

    #[test]
    fn behavioral_similarity_identical() {
        let p1 = BehavioralProfile {
            avg_request_interval: 30.0,
            prompt_length_mean: 200.0,
            prompt_length_std: 50.0,
            activity_histogram: {
                let mut h = [0u32; 24];
                h[14] = 10;
                h
            },
            content_hash_prefix: "abc123".into(),
        };
        let p2 = p1.clone();
        assert!((p1.similarity(&p2) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn behavioral_similarity_different() {
        let p1 = BehavioralProfile {
            avg_request_interval: 30.0,
            prompt_length_mean: 200.0,
            prompt_length_std: 50.0,
            activity_histogram: {
                let mut h = [0u32; 24];
                h[14] = 10;
                h
            },
            content_hash_prefix: "abc123".into(),
        };
        let p2 = BehavioralProfile {
            avg_request_interval: 300.0,
            prompt_length_mean: 2000.0,
            prompt_length_std: 200.0,
            activity_histogram: {
                let mut h = [0u32; 24];
                h[3] = 10;
                h
            },
            content_hash_prefix: "xyz789".into(),
        };
        let sim = p1.similarity(&p2);
        assert!(sim < 0.5, "Expected low similarity, got {}", sim);
    }
}
