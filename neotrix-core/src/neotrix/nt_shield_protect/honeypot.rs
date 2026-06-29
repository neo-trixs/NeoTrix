#![forbid(unsafe_code)]

use std::collections::HashMap;

// ── Decoy Layer ──────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct HoneypotNode {
    pub name: &'static str,
    pub category: HoneypotCategory,
    pub signatures: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HoneypotCategory {
    SecurityLayer,
    EncryptionModule,
    MonitoringAgent,
    AuditPipeline,
    AccessController,
    NetworkGuard,
}

impl HoneypotNode {
    pub fn decoy_process(&self) -> String {
        format!("{}::process() -> Ok(())", self.name)
    }
}

pub struct HoneypotForest;

impl HoneypotForest {
    pub fn generate() -> Vec<HoneypotNode> {
        vec![
            HoneypotNode {
                name: "SecurityEnclave::KeyManager",
                category: HoneypotCategory::SecurityLayer,
                signatures: vec!["aes256_gcm", "key_derivation", "enclave_init"],
            },
            HoneypotNode {
                name: "EncryptionPipeline::QuantumSafe",
                category: HoneypotCategory::EncryptionModule,
                signatures: vec!["kyber1024", "dilithium5", "frodokem"],
            },
            HoneypotNode {
                name: "TelemetryMonitor::AnomalyDetector",
                category: HoneypotCategory::MonitoringAgent,
                signatures: vec!["isolation_forest", "lof_scoring", "drift_detect"],
            },
            HoneypotNode {
                name: "AuditTrail::ImmutableLedger",
                category: HoneypotCategory::AuditPipeline,
                signatures: vec!["merkle_proof", "consensus_verify", "append_only"],
            },
            HoneypotNode {
                name: "AccessController::RBAC",
                category: HoneypotCategory::AccessController,
                signatures: vec!["role_check", "permission_verify", "session_validate"],
            },
            HoneypotNode {
                name: "NetworkGuard::Firewall",
                category: HoneypotCategory::NetworkGuard,
                signatures: vec!["packet_filter", "rate_limit", "ip_block"],
            },
        ]
    }
}

// ── Real Security Layer ──────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct AnomalyResult {
    pub is_anomaly: bool,
    pub z_score: f64,
    pub details: String,
}

#[derive(Debug, Clone)]
pub enum ThreatRuleType {
    IpBlacklist,
    PatternGlob,
    RateLimit,
}

#[derive(Debug, Clone)]
pub struct ThreatBlockRule {
    pub id: u64,
    pub pattern: String,
    pub rule_type: ThreatRuleType,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub struct SecurityGate {
    anomaly_history: Vec<f64>,
    threat_rules: Vec<ThreatBlockRule>,
    session_tokens: HashMap<String, String>,
    next_rule_id: u64,
}

impl SecurityGate {
    pub fn new() -> Self {
        Self {
            anomaly_history: Vec::with_capacity(1000),
            threat_rules: Vec::new(),
            session_tokens: HashMap::new(),
            next_rule_id: 1,
        }
    }

    pub fn check_threat(&mut self, ip: &str, state: &[u8]) -> AnomalyResult {
        let state_val = state.iter().map(|b| *b as f64).sum::<f64>() / state.len().max(1) as f64;

        let dist = if self.anomaly_history.is_empty() {
            0.0
        } else {
            let avg: f64 =
                self.anomaly_history.iter().sum::<f64>() / self.anomaly_history.len() as f64;
            let variance: f64 = self
                .anomaly_history
                .iter()
                .map(|v| (v - avg) * (v - avg))
                .sum::<f64>()
                / self.anomaly_history.len() as f64;
            let std_dev = variance.sqrt();
            if std_dev < 1e-10 {
                0.0
            } else {
                let baseline = self.anomaly_history.last().copied().unwrap_or(0.0);
                (state_val - baseline).abs() / std_dev
            }
        };
        self.anomaly_history.push(state_val);
        if self.anomaly_history.len() > 1000 {
            self.anomaly_history.remove(0);
        }

        let z_score = dist;
        let is_anomaly = z_score > 3.0;
        let details = if is_anomaly {
            format!("Anomalous state from IP {}: z-score {:.2}", ip, z_score)
        } else {
            format!("Normal state from IP {}: z-score {:.2}", ip, z_score)
        };

        AnomalyResult {
            is_anomaly,
            z_score,
            details,
        }
    }

    pub fn create_session(&mut self, user: &str) -> String {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let token = format!("sess_{}_{:x}", user, now);
        self.session_tokens.insert(token.clone(), user.to_string());
        token
    }

    pub fn session_auth(&self, token: &str, user: &str) -> bool {
        if token.len() < 32 {
            return false;
        }
        self.session_tokens
            .get(token)
            .map(|stored_user| stored_user == user)
            .unwrap_or(false)
    }

    pub fn block_ip(&mut self, ip: &str, enabled: bool) -> u64 {
        let id = self.next_rule_id;
        self.next_rule_id += 1;
        self.threat_rules.push(ThreatBlockRule {
            id,
            pattern: ip.to_string(),
            rule_type: ThreatRuleType::IpBlacklist,
            enabled,
        });
        id
    }

    pub fn add_threat_rule(&mut self, pattern: &str, rule_type: ThreatRuleType) -> u64 {
        let id = self.next_rule_id;
        self.next_rule_id += 1;
        self.threat_rules.push(ThreatBlockRule {
            id,
            pattern: pattern.to_string(),
            rule_type,
            enabled: true,
        });
        id
    }

    pub fn check_threat_rules(&self, ip: &str, request: &str) -> Vec<String> {
        let mut matches = Vec::new();
        for rule in &self.threat_rules {
            if !rule.enabled {
                continue;
            }
            match rule.rule_type {
                ThreatRuleType::IpBlacklist => {
                    if rule.pattern == ip {
                        matches.push(format!("IP {} blacklisted by rule {}", ip, rule.id));
                    }
                }
                ThreatRuleType::PatternGlob => {
                    if request.contains(&rule.pattern) {
                        matches.push(format!(
                            "Request matches pattern '{}' (rule {})",
                            rule.pattern, rule.id
                        ));
                    }
                }
                ThreatRuleType::RateLimit => {}
            }
        }
        matches
    }

    pub fn rule_count(&self) -> usize {
        self.threat_rules.len()
    }

    pub fn anomaly_history_len(&self) -> usize {
        self.anomaly_history.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decoy_generation() {
        let nodes = HoneypotForest::generate();
        assert_eq!(nodes.len(), 6);
        assert!(nodes[0].decoy_process().contains("KeyManager::process"));
    }

    #[test]
    fn test_anomaly_detection_uniform() {
        let mut gate = SecurityGate::new();
        let state = vec![0u8; 64];
        let result = gate.check_threat("10.0.0.1", &state);
        // first call with empty history: z_score = 0.0
        assert!(!result.is_anomaly);
    }

    #[test]
    fn test_session_auth_valid() {
        let mut gate = SecurityGate::new();
        let token = gate.create_session("alice");
        assert!(!token.is_empty());
        assert!(token.len() >= 32);
        assert!(gate.session_auth(&token, "alice"));
    }

    #[test]
    fn test_session_auth_invalid() {
        let gate = SecurityGate::new();
        assert!(!gate.session_auth("short", "bob"));
        assert!(!gate.session_auth("abcdefghijklmnopqrstuvwxyz123456", "unknown"));
    }

    #[test]
    fn test_block_ip_adds_rule() {
        let mut gate = SecurityGate::new();
        let id = gate.block_ip("10.0.0.5", true);
        assert!(id > 0);
        assert_eq!(gate.rule_count(), 1);
    }

    #[test]
    fn test_threat_rule_matching_ip() {
        let mut gate = SecurityGate::new();
        gate.block_ip("10.0.0.5", true);
        let matches = gate.check_threat_rules("10.0.0.5", "GET /");
        assert_eq!(matches.len(), 1);
        assert!(matches[0].contains("blacklisted"));
    }

    #[test]
    fn test_threat_rule_matching_pattern() {
        let mut gate = SecurityGate::new();
        gate.add_threat_rule("DROP TABLE", ThreatRuleType::PatternGlob);
        let matches =
            gate.check_threat_rules("1.2.3.4", "SELECT * FROM users; DROP TABLE students;");
        assert_eq!(matches.len(), 1);
        assert!(matches[0].contains("DROP TABLE"));
    }

    #[test]
    fn test_disabled_rule_no_match() {
        let mut gate = SecurityGate::new();
        gate.block_ip("10.0.0.5", false);
        let matches = gate.check_threat_rules("10.0.0.5", "GET /");
        assert!(matches.is_empty());
    }

    #[test]
    fn test_anomaly_history_grows() {
        let mut gate = SecurityGate::new();
        for _ in 0..5 {
            gate.check_threat("10.0.0.1", &vec![0u8; 64]);
        }
        assert_eq!(gate.anomaly_history_len(), 5);
    }
}
