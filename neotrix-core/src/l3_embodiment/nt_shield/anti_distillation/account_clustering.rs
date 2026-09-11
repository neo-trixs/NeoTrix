//! Account clustering for proxy network detection
//!
//! Detects clusters of accounts that share:
//! - Creation timestamps
//! - IP addresses/subnets
//! - Behavioral patterns
//! - Infrastructure fingerprints

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use super::{DetectionSignal, ThreatLevel, AntiDistillationConfig};

pub struct AccountClustering {
    config: AntiDistillationConfig,
    account_metadata: Arc<RwLock<HashMap<String, AccountInfo>>>,
    ip_to_accounts: Arc<RwLock<HashMap<String, HashSet<String>>>>,
    creation_windows: Arc<RwLock<Vec<CreationWindow>>>,
}

#[derive(Debug, Clone)]
struct AccountInfo {
    account_id: String,
    creation_time: u64,
    ip_addresses: Vec<String>,
    behavioral_hash: String,
}

#[derive(Debug, Clone)]
struct CreationWindow {
    start_time: u64,
    end_time: u64,
    account_ids: Vec<String>,
}

impl AccountClustering {
    pub fn new(config: AntiDistillationConfig) -> Self {
        Self {
            config,
            account_metadata: Arc::new(RwLock::new(HashMap::new())),
            ip_to_accounts: Arc::new(RwLock::new(HashMap::new())),
            creation_windows: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn check_cluster(
        &self,
        _account_id: &str,
        metadata: &HashMap<String, String>,
    ) -> Option<DetectionSignal> {
        let mut signals = Vec::new();

        // Check IP clustering
        if let Some(ip) = metadata.get("ip_address") {
            let ip_map = self.ip_to_accounts.read().await;
            if let Some(accounts) = ip_map.get(ip) {
                if accounts.len() >= self.config.cluster_min_accounts {
                    signals.push(DetectionSignal {
                        signal_type: "ip_cluster".to_string(),
                        confidence: 0.85,
                        threat_level: ThreatLevel::High,
                        details: format!(
                            "IP {} shared by {} accounts (threshold: {})",
                            ip, accounts.len(), self.config.cluster_min_accounts
                        ),
                    });
                }
            }
        }

        // Check creation window clustering
        if let Some(creation_time) = metadata.get("creation_time") {
            if let Ok(time) = creation_time.parse::<u64>() {
                let windows = self.creation_windows.read().await;
                for window in windows.iter() {
                    if time >= window.start_time && time <= window.end_time {
                        if window.account_ids.len() >= self.config.cluster_min_accounts {
                            signals.push(DetectionSignal {
                                signal_type: "creation_window_cluster".to_string(),
                                confidence: 0.9,
                                threat_level: ThreatLevel::Critical,
                                details: format!(
                                    "Account created in suspicious window: {} accounts in {} seconds",
                                    window.account_ids.len(),
                                    window.end_time - window.start_time
                                ),
                            });
                        }
                    }
                }
            }
        }

        // Check behavioral similarity
        if let Some(behavioral_hash) = metadata.get("behavioral_hash") {
            let accounts = self.account_metadata.read().await;
            let similar_count = accounts.values()
                .filter(|a| a.behavioral_hash == *behavioral_hash)
                .count();

            if similar_count >= self.config.cluster_min_accounts {
                signals.push(DetectionSignal {
                    signal_type: "behavioral_cluster".to_string(),
                    confidence: 0.88,
                    threat_level: ThreatLevel::High,
                    details: format!(
                        "Behavioral similarity cluster: {} accounts with identical behavior",
                        similar_count
                    ),
                });
            }
        }

        // Return highest threat signal
        signals.into_iter()
            .max_by_key(|s| match s.threat_level {
                ThreatLevel::Critical => 4,
                ThreatLevel::High => 3,
                ThreatLevel::Medium => 2,
                ThreatLevel::Low => 1,
            })
    }

    pub async fn record_account(&self, info: AccountInfo) {
        let mut accounts = self.account_metadata.write().await;
        let mut ip_map = self.ip_to_accounts.write().await;

        // Record account
        accounts.insert(info.account_id.clone(), info.clone());

        // Record IP mapping
        for ip in &info.ip_addresses {
            ip_map.entry(ip.clone())
                .or_insert_with(HashSet::new)
                .insert(info.account_id.clone());
        }
    }

    pub async fn record_creation_window(&self, window: CreationWindow) {
        let mut windows = self.creation_windows.write().await;
        windows.push(window);
    }
}
