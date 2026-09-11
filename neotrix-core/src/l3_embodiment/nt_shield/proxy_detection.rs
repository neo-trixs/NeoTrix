//! Proxy Detection - 代理网络检测
//!
//! IP信誉分析 + 账户聚类 + 基础设施映射

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

/// 检测结果
#[derive(Debug, Clone)]
pub struct ProxyDetectionResult {
    pub is_proxy: bool,
    pub confidence: f64,
    pub threat_level: ThreatLevel,
    pub signals: Vec<DetectionSignal>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreatLevel {
    Safe,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub struct DetectionSignal {
    pub signal_type: String,
    pub confidence: f64,
    pub details: String,
}

/// 代理检测引擎
pub struct ProxyDetectionEngine {
    known_proxy_ips: HashSet<String>,
    suspicious_asns: HashSet<String>,
    account_clusters: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl ProxyDetectionEngine {
    pub fn new() -> Self {
        let mut known_proxy_ips = HashSet::new();
        // 添加已知代理IP段
        known_proxy_ips.insert("38.129.138.0/24".to_string());
        known_proxy_ips.insert("157.180.93.0/24".to_string());

        let mut suspicious_asns = HashSet::new();
        suspicious_asns.insert("AS26042".to_string());
        suspicious_asns.insert("AS16276".to_string());

        Self {
            known_proxy_ips,
            suspicious_asns,
            account_clusters: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 检测代理
    pub async fn detect(&self, ip: &str, metadata: &HashMap<String, String>) -> ProxyDetectionResult {
        let mut signals = Vec::new();
        let mut max_threat = ThreatLevel::Safe;

        // 1. IP信誉检测
        if self.check_ip_reputation(ip) {
            signals.push(DetectionSignal {
                signal_type: "known_proxy_ip".to_string(),
                confidence: 0.95,
                details: format!("IP {} is in known proxy list", ip),
            });
            max_threat = ThreatLevel::Critical;
        }

        // 2. ASN检测
        if let Some(asn) = metadata.get("asn") {
            if self.suspicious_asns.contains(asn) {
                signals.push(DetectionSignal {
                    signal_type: "suspicious_asn".to_string(),
                    confidence: 0.85,
                    details: format!("Suspicious ASN: {}", asn),
                });
                if max_threat < ThreatLevel::High {
                    max_threat = ThreatLevel::High;
                }
            }
        }

        // 3. 账户聚类检测
        if let Some(cluster_id) = metadata.get("cluster_id") {
            if self.check_account_cluster(cluster_id, ip).await {
                signals.push(DetectionSignal {
                    signal_type: "account_cluster".to_string(),
                    confidence: 0.9,
                    details: format!("IP {} belongs to account cluster", ip),
                });
                if max_threat < ThreatLevel::High {
                    max_threat = ThreatLevel::High;
                }
            }
        }

        // 4. 时间窗口检测
        if let Some(creation_time) = metadata.get("creation_time") {
            if self.check_time_window(creation_time) {
                signals.push(DetectionSignal {
                    signal_type: "suspicious_timing".to_string(),
                    confidence: 0.7,
                    details: "Account created in suspicious time window".to_string(),
                });
                if max_threat < ThreatLevel::Medium {
                    max_threat = ThreatLevel::Medium;
                }
            }
        }

        // 5. 行为相似性检测
        if let Some(behavior_hash) = metadata.get("behavior_hash") {
            if self.check_behavior_similarity(behavior_hash).await {
                signals.push(DetectionSignal {
                    signal_type: "behavior_similarity".to_string(),
                    confidence: 0.8,
                    details: "Behavioral similarity with known proxy accounts".to_string(),
                });
                if max_threat < ThreatLevel::Medium {
                    max_threat = ThreatLevel::Medium;
                }
            }
        }

        let confidence = signals.iter().map(|s| s.confidence).fold(0.0, f64::max);
        let is_proxy = max_threat >= ThreatLevel::Medium;

        ProxyDetectionResult {
            is_proxy,
            confidence,
            threat_level: max_threat,
            signals,
        }
    }

    /// 检查IP信誉
    fn check_ip_reputation(&self, ip: &str) -> bool {
        // 简化实现 - 实际应查询IP信誉数据库
        self.known_proxy_ips.iter().any(|proxy| ip.starts_with(proxy.split('.').next().unwrap_or("")))
    }

    /// 检查账户聚类
    async fn check_account_cluster(&self, cluster_id: &str, ip: &str) -> bool {
        let clusters = self.account_clusters.read().await;
        if let Some(accounts) = clusters.get(cluster_id) {
            accounts.len() >= 5 // 至少5个账户
        } else {
            false
        }
    }

    /// 检查时间窗口
    fn check_time_window(&self, creation_time: &str) -> bool {
        // 简化实现 - 实际应解析时间戳
        creation_time.contains("2026") // 2026年创建的账户
    }

    /// 检查行为相似性
    async fn check_behavior_similarity(&self, behavior_hash: &str) -> bool {
        // 简化实现 - 实际应查询行为数据库
        behavior_hash.len() > 10
    }

    /// 记录账户
    pub async fn record_account(&self, cluster_id: &str, ip: &str) {
        let mut clusters = self.account_clusters.write().await;
        clusters.entry(cluster_id.to_string())
            .or_insert_with(Vec::new)
            .push(ip.to_string());
    }
}

impl Default for ProxyDetectionEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_detect_proxy() {
        let engine = ProxyDetectionEngine::new();
        let mut metadata = HashMap::new();
        metadata.insert("asn".to_string(), "AS26042".to_string());
        
        let result = engine.detect("38.129.138.1", &metadata).await;
        assert!(result.is_proxy);
        assert!(result.confidence > 0.8);
    }

    #[tokio::test]
    async fn test_detect_safe() {
        let engine = ProxyDetectionEngine::new();
        let metadata = HashMap::new();
        
        let result = engine.detect("8.8.8.8", &metadata).await;
        assert!(!result.is_proxy);
    }
}
