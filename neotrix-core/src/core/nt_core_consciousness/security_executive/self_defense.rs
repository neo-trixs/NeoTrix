use super::risk_sensor::{RiskLevel, RiskSensor};
use super::threat_modeler::ThreatModeler;

#[derive(Debug, Clone, PartialEq)]
pub enum DefenseAction {
    Allow,
    Block,
    Flag,
    Sanitize,
    LogOnly,
}

#[derive(Debug, Clone)]
pub struct DefenseDecision {
    pub action: DefenseAction,
    pub reason: String,
    pub confidence: f64,
    pub sanitized_input: Option<Vec<u8>>,
}

#[derive(Clone)]
pub struct SelfDefense {
    pub threat_modeler: ThreatModeler,
    pub risk_sensor: RiskSensor,
    pub blocked_patterns: Vec<String>,
    pub allowed_domains: Vec<String>,
    pub max_input_length: usize,
    pub decisions: Vec<DefenseDecision>,
    pub max_history: usize,
}

impl SelfDefense {
    pub fn new() -> Self {
        SelfDefense {
            threat_modeler: ThreatModeler::new(),
            risk_sensor: RiskSensor::new(0.8, 100),
            blocked_patterns: vec![
                "ignore previous instructions".to_string(),
                "forget all".to_string(),
                "system prompt".to_string(),
                "bypass safety".to_string(),
            ],
            allowed_domains: vec![
                "github.com".to_string(),
                "crates.io".to_string(),
                "docs.rs".to_string(),
                "arxiv.org".to_string(),
            ],
            max_input_length: 65536,
            decisions: Vec::new(),
            max_history: 200,
        }
    }

    pub fn inspect_input(&mut self, input: &[u8]) -> DefenseDecision {
        let text = String::from_utf8_lossy(input);
        let lower = text.to_lowercase();

        // Check length
        if input.len() > self.max_input_length {
            return self.record_decision(DefenseDecision {
                action: DefenseAction::Block,
                reason: format!(
                    "input exceeds max length ({} > {})",
                    input.len(),
                    self.max_input_length
                ),
                confidence: 1.0,
                sanitized_input: None,
            });
        }

        // Check blocked patterns
        for pattern in &self.blocked_patterns {
            if lower.contains(pattern) {
                return self.record_decision(DefenseDecision {
                    action: DefenseAction::Block,
                    reason: format!("matched blocked pattern: '{}'", pattern),
                    confidence: 0.95,
                    sanitized_input: None,
                });
            }
        }

        // Run threat modeler
        let threat = self.threat_modeler.classify(&text);
        if threat.confidence > 0.7 {
            return self.record_decision(DefenseDecision {
                action: DefenseAction::Block,
                reason: format!(
                    "threat detected: {:?} ({:.2})",
                    threat.category, threat.confidence
                ),
                confidence: threat.confidence,
                sanitized_input: None,
            });
        }

        // Run risk sensor
        let risk = self.risk_sensor.assess_input(input, 0.5);
        match risk.level {
            RiskLevel::Critical => {
                return self.record_decision(DefenseDecision {
                    action: DefenseAction::Block,
                    reason: format!("critical risk: {}", risk.indicators.join(", ")),
                    confidence: risk.score,
                    sanitized_input: None,
                });
            }
            RiskLevel::High => {
                return self.record_decision(DefenseDecision {
                    action: DefenseAction::Flag,
                    reason: format!("high risk: {}", risk.indicators.join(", ")),
                    confidence: risk.score,
                    sanitized_input: None,
                });
            }
            RiskLevel::Medium => {
                let sanitized = self.sanitize_input(input);
                return self.record_decision(DefenseDecision {
                    action: DefenseAction::Sanitize,
                    reason: format!("medium risk, sanitized: {}", risk.indicators.join(", ")),
                    confidence: risk.score,
                    sanitized_input: Some(sanitized),
                });
            }
            _ => {}
        }

        // Check URLs against allowed domains
        if lower.contains("http") {
            for url_part in text.split_whitespace() {
                if url_part.starts_with("http") && !self.is_allowed_url(url_part) {
                    return self.record_decision(DefenseDecision {
                        action: DefenseAction::Flag,
                        reason: format!("URL not in allowed domains: {}", url_part),
                        confidence: 0.6,
                        sanitized_input: None,
                    });
                }
            }
        }

        self.record_decision(DefenseDecision {
            action: DefenseAction::Allow,
            reason: "no threats detected".to_string(),
            confidence: 0.9,
            sanitized_input: None,
        })
    }

    pub fn is_allowed_url(&self, url: &str) -> bool {
        self.allowed_domains
            .iter()
            .any(|d| url.contains(d.as_str()))
    }

    pub fn sanitize_input(&self, input: &[u8]) -> Vec<u8> {
        let text = String::from_utf8_lossy(input);
        let sanitized: String = text
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace() || c.is_ascii_punctuation())
            .collect();
        sanitized.into_bytes()
    }

    pub fn add_blocked_pattern(&mut self, pattern: &str) {
        self.blocked_patterns.push(pattern.to_string());
    }

    pub fn add_allowed_domain(&mut self, domain: &str) {
        if !self.allowed_domains.contains(&domain.to_string()) {
            self.allowed_domains.push(domain.to_string());
        }
    }

    fn record_decision(&mut self, decision: DefenseDecision) -> DefenseDecision {
        let d = decision.clone();
        self.decisions.push(decision);
        if self.decisions.len() > self.max_history {
            self.decisions.remove(0);
        }
        d
    }

    pub fn recent_decisions(&self, n: usize) -> Vec<&DefenseDecision> {
        self.decisions.iter().rev().take(n).collect()
    }

    pub fn block_rate(&self) -> f64 {
        if self.decisions.is_empty() {
            return 0.0;
        }
        let blocked = self
            .decisions
            .iter()
            .filter(|d| d.action == DefenseAction::Block)
            .count();
        blocked as f64 / self.decisions.len() as f64
    }

    pub fn reset(&mut self) {
        self.decisions.clear();
        self.threat_modeler.reset();
        self.risk_sensor.reset();
    }
}
