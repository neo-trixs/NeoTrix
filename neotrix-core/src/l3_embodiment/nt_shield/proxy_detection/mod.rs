//! Proxy network detection engine
//!
//! Detects proxy/VPN/coordinated abuse patterns from Anthropic's distillation report:
//! - Shared creation timestamps across accounts
//! - Residential proxy and datacenter IP clustering
//! - Disposable email domain correlation
//! - Temporal and behavioral clustering
//! - Domain/IP/account infrastructure mapping
//!
//! Architecture follows NT-SHIELD `L3 Embodiment` conventions:
//! - `DetectionSignal` unified with `anti_distillation` module
//! - `ThreatLevel` severity escalation (Low → Critical)
//! - Async-first with tokio for concurrent IP/ASN lookups

pub mod ip_fingerprint;
pub mod account_cluster;
pub mod infrastructure_mapper;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use thiserror::Error;

// ── Detection types (mirrors anti_distillation for module independence) ───
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ThreatLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl ThreatLevel {
    pub fn numeric(&self) -> u8 {
        match self {
            Self::Low => 1,
            Self::Medium => 2,
            Self::High => 3,
            Self::Critical => 4,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DetectionSignal {
    pub signal_type: String,
    pub confidence: f64,
    pub threat_level: ThreatLevel,
    pub details: String,
}

// ── Errors ────────────────────────────────────────────────────────────────
#[derive(Debug, Error)]
pub enum ProxyDetectionError {
    #[error("IP lookup failed: {0}")]
    IpLookupFailed(String),

    #[error("ASN database unavailable: {0}")]
    AsnDatabaseUnavailable(String),

    #[error("Cluster analysis timeout after {0}ms")]
    ClusterAnalysisTimeout(u64),

    #[error("Infrastructure map corrupted: {0}")]
    InfrastructureMapCorrupted(String),
}

// ── Configuration ─────────────────────────────────────────────────────────
#[derive(Debug, Clone)]
pub struct ProxyDetectionConfig {
    /// Enable IP reputation/ASN fingerprinting
    pub enable_ip_fingerprint: bool,
    /// Enable temporal+behavioral account clustering
    pub enable_account_clustering: bool,
    /// Enable domain/IP/account correlation mapping
    pub enable_infrastructure_mapping: bool,
    /// Minimum accounts sharing an IP before flagging
    pub ip_cluster_threshold: usize,
    /// Time window (seconds) for creation-timestamp clustering
    pub creation_window_seconds: u64,
    /// Minimum accounts in a creation window to flag
    pub creation_window_threshold: usize,
    /// Disposable email domains (built-in + extensible)
    pub disposable_email_domains: Vec<String>,
    /// Known datacenter IP CIDRs (simplified: /24 prefixes)
    pub datacenter_prefixes: Vec<String>,
    /// Maximum concurrent IP lookups
    pub max_concurrent_lookups: usize,
}

impl Default for ProxyDetectionConfig {
    fn default() -> Self {
        Self {
            enable_ip_fingerprint: true,
            enable_account_clustering: true,
            enable_infrastructure_mapping: true,
            ip_cluster_threshold: 5,
            creation_window_seconds: 300,
            creation_window_threshold: 10,
            disposable_email_domains: vec![
                "guerrillamail.com".into(),
                "tempmail.com".into(),
                "throwaway.email".into(),
                "yopmail.com".into(),
                "mailinator.com".into(),
                "temp-mail.org".into(),
                "10minutemail.com".into(),
                "trashmail.com".into(),
                "dispostable.com".into(),
                "sharklasers.com".into(),
                "guerrillamailblock.com".into(),
                "grr.la".into(),
                "disposable-email addresses.com".into(),
                "fakeinbox.com".into(),
                "tempinbox.com".into(),
            ],
            datacenter_prefixes: vec![
                "104.248.0.0".into(),
                "157.230.0.0".into(),
                "167.71.0.0".into(),
                "165.227.0.0".into(),
                "134.122.0.0".into(),
                "161.35.0.0".into(),
                "159.65.0.0".into(),
                "137.184.0.0".into(),
                "209.250.0.0".into(),
                "139.59.0.0".into(),
            ],
            max_concurrent_lookups: 32,
        }
    }
}

// ── Proxy Network Signature (Anthropic report indicators) ─────────────────
#[derive(Debug, Clone)]
pub struct ProxySignature {
    /// IPs sharing creation timestamp windows
    pub shared_creation_ips: Vec<String>,
    /// Accounts created within the same window
    pub cluster_accounts: Vec<String>,
    /// Disposable email domains used
    pub disposable_domains: Vec<String>,
    /// Datacenter IP count (vs residential)
    pub datacenter_ip_count: usize,
    /// Residential proxy indicators
    pub residential_proxy_score: f64,
    /// Overall confidence (0.0–1.0)
    pub confidence: f64,
    /// Escalated threat level
    pub threat_level: ThreatLevel,
}

impl ProxySignature {
    /// Default (clean) signature
    pub fn clean() -> Self {
        Self {
            shared_creation_ips: Vec::new(),
            cluster_accounts: Vec::new(),
            disposable_domains: Vec::new(),
            datacenter_ip_count: 0,
            residential_proxy_score: 0.0,
            confidence: 0.0,
            threat_level: ThreatLevel::Low,
        }
    }

    /// Aggregate sub-scores into overall confidence and threat level
    pub fn finalize(&mut self) {
        let mut score = 0.0_f64;

        // Shared creation timestamps — strong signal
        if !self.shared_creation_ips.is_empty() {
            score += 0.35;
        }

        // Cluster size
        let cluster_ratio = self.cluster_accounts.len() as f64 / 20.0;
        score += cluster_ratio.min(0.25);

        // Disposable emails
        if !self.disposable_domains.is_empty() {
            score += 0.15;
        }

        // Datacenter IPs
        if self.datacenter_ip_count > 0 {
            let dc_ratio = self.datacenter_ip_count as f64 / 10.0;
            score += dc_ratio.min(0.15);
        }

        // Residential proxy score
        score += self.residential_proxy_score * 0.10;

        self.confidence = score.min(1.0);

        self.threat_level = match self.confidence {
            c if c >= 0.8 => ThreatLevel::Critical,
            c if c >= 0.6 => ThreatLevel::High,
            c if c >= 0.35 => ThreatLevel::Medium,
            _ => ThreatLevel::Low,
        };
    }
}

// ── Account Observation ───────────────────────────────────────────────────
#[derive(Debug, Clone)]
pub struct AccountObservation {
    pub account_id: String,
    pub created_at: u64,
    pub ip_addresses: Vec<String>,
    pub email_domain: String,
    pub user_agent: String,
    pub request_pattern_hash: String,
    pub metadata: HashMap<String, String>,
}

// ── Analysis Result ───────────────────────────────────────────────────────
#[derive(Debug, Clone)]
pub struct ProxyAnalysisResult {
    pub signature: ProxySignature,
    pub signals: Vec<DetectionSignal>,
    pub ip_fingerprints: Vec<ip_fingerprint::IpFingerprint>,
    pub cluster_id: Option<String>,
    pub infrastructure_map: Option<infrastructure_mapper::InfrastructureMap>,
}

// ── ProxyDetectionEngine ──────────────────────────────────────────────────
pub struct ProxyDetectionEngine {
    config: ProxyDetectionConfig,
    ip_engine: ip_fingerprint::IpFingerprintEngine,
    cluster_engine: account_cluster::AccountClusterEngine,
    infra_mapper: infrastructure_mapper::InfrastructureMapper,
    /// In-memory observation buffer (account_id → observation)
    observations: Arc<RwLock<HashMap<String, AccountObservation>>>,
}

impl ProxyDetectionEngine {
    pub fn new(config: ProxyDetectionConfig) -> Self {
        let ip_engine = ip_fingerprint::IpFingerprintEngine::new(config.clone());
        let cluster_engine = account_cluster::AccountClusterEngine::new(config.clone());
        let infra_mapper = infrastructure_mapper::InfrastructureMapper::new(config.clone());

        Self {
            config,
            ip_engine,
            cluster_engine,
            infra_mapper,
            observations: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Analyze a single account observation for proxy network indicators.
    ///
    /// Runs all three sub-engines concurrently and merges results into a
    /// unified `ProxySignature`.
    pub async fn analyze(
        &self,
        observation: AccountObservation,
    ) -> Result<ProxyAnalysisResult, ProxyDetectionError> {
        // Buffer the observation
        {
            let mut obs = self.observations.write().await;
            obs.insert(observation.account_id.clone(), observation.clone());
        }

        let mut signals: Vec<DetectionSignal> = Vec::new();
        let mut signature = ProxySignature::clean();

        // ── Phase 1: IP fingerprinting (concurrent) ──────────────────────
        let ip_fingerprints = if self.config.enable_ip_fingerprint {
            let fingerprints = self.ip_engine
                .analyze_ips(&observation.ip_addresses)
                .await?;
            for fp in &fingerprints {
                if fp.is_datacenter {
                    signature.datacenter_ip_count += 1;
                }
                signature.residential_proxy_score += fp.residential_proxy_probability;
            }
            if !fingerprints.is_empty() {
                signature.residential_proxy_score /= fingerprints.len() as f64;
            }
            signals.extend(self.ip_engine.build_signals(&fingerprints));
            fingerprints
        } else {
            Vec::new()
        };

        // ── Phase 2: Account clustering ──────────────────────────────────
        let cluster_id = if self.config.enable_account_clustering {
            let cluster_result = self.cluster_engine
                .analyze_observation(&observation)
                .await?;
            if let Some(ref cluster) = cluster_result {
                signature.cluster_accounts = cluster.member_accounts.clone();
                signature.shared_creation_ips = cluster.shared_ips.clone();
                signals.extend(cluster_engine_signals(cluster));
            }
            cluster_result.map(|c| c.cluster_id)
        } else {
            None
        };

        // ── Phase 3: Infrastructure mapping ──────────────────────────────
        let infra_map = if self.config.enable_infrastructure_mapping {
            let obs = self.observations.read().await;
            let map = self.infra_mapper
                .build_map(obs.values().collect())
                .await?;
            signals.extend(self.infra_mapper.build_signals(&map));
            Some(map)
        } else {
            None
        };

        // ── Phase 4: Disposable email check ──────────────────────────────
        if self.config.disposable_email_domains
            .iter()
            .any(|d| d == &observation.email_domain)
        {
            signature.disposable_domains.push(observation.email_domain.clone());
            signals.push(DetectionSignal {
                signal_type: "disposable_email".to_string(),
                confidence: 0.85,
                threat_level: ThreatLevel::High,
                details: format!(
                    "Disposable email domain detected: {}",
                    observation.email_domain
                ),
            });
        }

        // ── Finalize ─────────────────────────────────────────────────────
        signature.finalize();

        Ok(ProxyAnalysisResult {
            signature,
            signals,
            ip_fingerprints,
            cluster_id,
            infrastructure_map: infra_map,
        })
    }

    /// Batch-analyze multiple observations with bounded concurrency.
    pub async fn analyze_batch(
        &self,
        observations: Vec<AccountObservation>,
    ) -> Vec<Result<ProxyAnalysisResult, ProxyDetectionError>> {
        use futures::stream::{self, StreamExt};

        let concurrency = self.config.max_concurrent_lookups;
        stream::iter(observations)
            .map(|obs| {
                let engine = self;
                async move { engine.analyze(obs).await }
            })
            .buffer_unordered(concurrency)
            .collect()
            .await
    }

    /// Get current observation count (for monitoring).
    pub async fn observation_count(&self) -> usize {
        self.observations.read().await.len()
    }

    /// Purge observations older than `max_age_seconds`.
    pub async fn purge_stale(&self, max_age_seconds: u64, now: u64) {
        let mut obs = self.observations.write().await;
        obs.retain(|_, v| now.saturating_sub(v.created_at) < max_age_seconds);
    }
}

/// Convert cluster result to DetectionSignals (local helper).
fn cluster_engine_signals(
    cluster: &account_cluster::ClusterResult,
) -> Vec<DetectionSignal> {
    let mut signals = Vec::new();

    if !cluster.shared_ips.is_empty() {
        signals.push(DetectionSignal {
            signal_type: "shared_ip_cluster".to_string(),
            confidence: 0.9,
            threat_level: ThreatLevel::Critical,
            details: format!(
                "{} IPs shared across {} accounts in cluster {}",
                cluster.shared_ips.len(),
                cluster.member_accounts.len(),
                cluster.cluster_id,
            ),
        });
    }

    if cluster.creation_window_score > 0.5 {
        signals.push(DetectionSignal {
            signal_type: "creation_timestamp_cluster".to_string(),
            confidence: cluster.creation_window_score,
            threat_level: ThreatLevel::Critical,
            details: format!(
                "Accounts in cluster {} created within {}s window (score: {:.2})",
                cluster.cluster_id,
                cluster.window_duration_secs,
                cluster.creation_window_score,
            ),
        });
    }

    signals
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proxy_signature_finalize_low_threat() {
        let mut sig = ProxySignature::clean();
        sig.finalize();
        assert_eq!(sig.threat_level, ThreatLevel::Low);
        assert!((sig.confidence - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn proxy_signature_finalize_critical() {
        let mut sig = ProxySignature {
            shared_creation_ips: vec!["1.2.3.0/24".into()],
            cluster_accounts: (0..20).map(|i| format!("acc_{i}")).collect(),
            disposable_domains: vec!["temp.com".into()],
            datacenter_ip_count: 5,
            residential_proxy_score: 0.8,
            confidence: 0.0,
            threat_level: ThreatLevel::Low,
        };
        sig.finalize();
        assert_eq!(sig.threat_level, ThreatLevel::Critical);
        assert!(sig.confidence >= 0.8);
    }

    #[tokio::test]
    async fn engine_observation_buffer() {
        let config = ProxyDetectionConfig::default();
        let engine = ProxyDetectionEngine::new(config);

        let obs = AccountObservation {
            account_id: "a1".into(),
            created_at: 1000,
            ip_addresses: vec!["1.2.3.4".into()],
            email_domain: "gmail.com".into(),
            user_agent: "Mozilla/5.0".into(),
            request_pattern_hash: "abc".into(),
            metadata: HashMap::new(),
        };

        let _ = engine.analyze(obs).await.unwrap();
        assert_eq!(engine.observation_count().await, 1);
    }
}
