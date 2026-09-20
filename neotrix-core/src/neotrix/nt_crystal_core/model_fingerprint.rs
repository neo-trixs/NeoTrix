//! ModelFingerprintVerifier — 模型指纹监控
//!
//! 基于 is-gpt-nerfed: 通过探针问题检测模型降级或切换。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeQuestion {
    pub id: String,
    pub question: String,
    pub expected_pattern: String,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FingerprintBaseline {
    pub model_name: String,
    pub probe_responses: HashMap<String, String>,
    pub response_hashes: HashMap<String, u64>,
    pub registered_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub model_name: String,
    pub is_genuine: bool,
    pub match_rate: f64,
    pub anomalies: Vec<String>,
    pub verified_at: u64,
}

pub struct ModelFingerprintVerifier {
    pub baselines: HashMap<String, FingerprintBaseline>,
    pub probes: Vec<ProbeQuestion>,
    pub alert_threshold: f64,
    pub verification_history: Vec<VerificationResult>,
}

impl ModelFingerprintVerifier {
    pub fn new() -> Self {
        Self {
            baselines: HashMap::new(),
            probes: Vec::new(),
            alert_threshold: 0.7,
            verification_history: Vec::new(),
        }
    }

    pub fn add_probe(&mut self, probe: ProbeQuestion) {
        self.probes.push(probe);
    }

    pub fn register_baseline(&mut self, model_name: &str, responses: HashMap<String, String>) {
        let hashes: HashMap<String, u64> = responses.iter()
            .map(|(k, v)| (k.clone(), simple_hash(v)))
            .collect();
        self.baselines.insert(model_name.to_string(), FingerprintBaseline {
            model_name: model_name.to_string(),
            probe_responses: responses,
            response_hashes: hashes,
            registered_at: now_ms(),
        });
    }

    pub fn verify(&mut self, model_name: &str, current_responses: &HashMap<String, String>) -> VerificationResult {
        let baseline = self.baselines.get(model_name);
        let mut anomalies = Vec::new();
        let mut matches = 0;
        let mut total = 0;

        if let Some(bl) = baseline {
            for (probe_id, expected) in &bl.response_hashes {
                total += 1;
                if let Some(actual) = current_responses.get(probe_id) {
                    let actual_hash = simple_hash(actual);
                    if actual_hash == *expected {
                        matches += 1;
                    } else {
                        anomalies.push(format!("Probe {} mismatch", probe_id));
                    }
                } else {
                    anomalies.push(format!("Probe {} missing", probe_id));
                }
            }
        }

        let match_rate = if total > 0 { matches as f64 / total as f64 } else { 0.0 };
        let result = VerificationResult {
            model_name: model_name.to_string(),
            is_genuine: match_rate >= self.alert_threshold,
            match_rate,
            anomalies,
            verified_at: now_ms(),
        };
        self.verification_history.push(result.clone());
        result
    }

    pub fn detect_switch(&self, model_name: &str) -> bool {
        self.verification_history.iter()
            .filter(|r| r.model_name == model_name)
            .last()
            .map(|r| !r.is_genuine)
            .unwrap_or(false)
    }

    pub fn alert(&self, result: &VerificationResult) -> Option<String> {
        if !result.is_genuine {
            Some(format!("ALERT: Model {} fingerprint mismatch! Match rate: {:.1}%", result.model_name, result.match_rate * 100.0))
        } else {
            None
        }
    }
}

fn simple_hash(s: &str) -> u64 {
    s.bytes().fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64))
}

fn now_ms() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_verify() {
        let mut verifier = ModelFingerprintVerifier::new();
        let mut baseline = HashMap::new();
        baseline.insert("p1".into(), "answer_a".into());
        verifier.register_baseline("gpt-4", baseline);

        let mut current = HashMap::new();
        current.insert("p1".into(), "answer_a".into());
        let result = verifier.verify("gpt-4", &current);
        assert!(result.is_genuine);
        assert_eq!(result.match_rate, 1.0);
    }

    #[test]
    fn test_detect_switch() {
        let mut verifier = ModelFingerprintVerifier::new();
        let mut baseline = HashMap::new();
        baseline.insert("p1".into(), "correct".into());
        verifier.register_baseline("model", baseline);

        let mut wrong = HashMap::new();
        wrong.insert("p1".into(), "wrong_answer".into());
        verifier.verify("model", &wrong);
        assert!(verifier.detect_switch("model"));
    }

    #[test]
    fn test_alert() {
        let mut verifier = ModelFingerprintVerifier::new();
        let result = VerificationResult {
            model_name: "test".into(), is_genuine: false, match_rate: 0.3,
            anomalies: vec!["mismatch".into()], verified_at: 0,
        };
        assert!(verifier.alert(&result).is_some());
    }
}
