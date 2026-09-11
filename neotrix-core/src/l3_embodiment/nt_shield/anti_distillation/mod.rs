//! Anti-distillation defense module
//!
//! Protects against unauthorized extraction of model capabilities:
//! - CoT cross-session replay detection
//! - Account clustering for proxy networks
//! - Request pattern anomaly detection
//! - Reasoning trace protection

pub mod extraction_detector;
pub mod session_replay_guard;
pub mod account_clustering;
pub mod reasoning_protector;

use std::collections::HashMap;

/// Threat level for detected distillation attempts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ThreatLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Detection signal from anti-distillation checks
#[derive(Debug, Clone)]
pub struct DetectionSignal {
    pub signal_type: String,
    pub confidence: f64,
    pub threat_level: ThreatLevel,
    pub details: String,
}

/// Configuration for anti-distillation module
#[derive(Debug, Clone)]
pub struct AntiDistillationConfig {
    /// Enable CoT extraction detection
    pub enable_cot_detection: bool,
    /// Enable account clustering
    pub enable_account_clustering: bool,
    /// Request rate threshold (requests per minute)
    pub rate_threshold: u32,
    /// Minimum accounts for cluster detection
    pub cluster_min_accounts: usize,
    /// Time window for cluster detection (seconds)
    pub cluster_time_window: u64,
}

impl Default for AntiDistillationConfig {
    fn default() -> Self {
        Self {
            enable_cot_detection: true,
            enable_account_clustering: true,
            rate_threshold: 100,
            cluster_min_accounts: 5,
            cluster_time_window: 300,
        }
    }
}

/// Main anti-distillation engine
pub struct AntiDistillationEngine {
    config: AntiDistillationConfig,
    extraction_detector: extraction_detector::ExtractionDetector,
    session_replay_guard: session_replay_guard::SessionReplayGuard,
    account_clustering: account_clustering::AccountClustering,
    reasoning_protector: reasoning_protector::ReasoningProtector,
}

impl AntiDistillationEngine {
    pub fn new(config: AntiDistillationConfig) -> Self {
        Self {
            config: config.clone(),
            extraction_detector: extraction_detector::ExtractionDetector::new(config.clone()),
            session_replay_guard: session_replay_guard::SessionReplayGuard::new(),
            account_clustering: account_clustering::AccountClustering::new(config.clone()),
            reasoning_protector: reasoning_protector::ReasoningProtector::new(),
        }
    }

    /// Analyze a request for distillation signals
    pub async fn analyze_request(
        &self,
        _request_id: &str,
        account_id: &str,
        prompt: &str,
        metadata: &HashMap<String, String>,
    ) -> Vec<DetectionSignal> {
        let mut signals = Vec::new();

        // Check extraction patterns
        if let Some(signal) = self.extraction_detector.detect(prompt, metadata).await {
            signals.push(signal);
        }

        // Check session replay
        if let Some(signal) = self.session_replay_guard
            .check_replay(account_id, prompt, metadata)
            .await
        {
            signals.push(signal);
        }

        // Check account clustering
        if let Some(signal) = self.account_clustering
            .check_cluster(account_id, metadata)
            .await
        {
            signals.push(signal);
        }

        signals
    }

    /// Protect reasoning traces before response
    pub fn protect_reasoning(&self, reasoning: &str, signature: &str) -> String {
        self.reasoning_protector.protect(reasoning, signature)
    }

    /// Get overall threat assessment
    pub fn assess_threat(signals: &[DetectionSignal]) -> ThreatLevel {
        signals.iter()
            .map(|s| s.threat_level)
            .max()
            .unwrap_or(ThreatLevel::Low)
    }
}
