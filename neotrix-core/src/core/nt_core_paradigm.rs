//! 范式迁移检测器 — 跨域异常共振 (P4.3)。
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anomaly {
    pub domain: String,
    pub description: String,
    pub severity: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParadigmShiftHypothesis {
    pub description: String,
    pub cross_domain: bool,
    pub novelty: f64,
}

pub struct ParadigmShiftDetector {
    anomalies: Vec<Anomaly>,
    threshold: usize,
}

impl ParadigmShiftDetector {
    pub fn new(threshold: usize) -> Self { Self { anomalies: Vec::new(), threshold } }
    pub fn observe(&mut self, a: Anomaly) { self.anomalies.push(a); }
    pub fn detect(&self) -> Vec<ParadigmShiftHypothesis> {
        let domains: std::collections::HashSet<_> = self.anomalies.iter().map(|a| a.domain.clone()).collect();
        if domains.len() >= 2 && self.anomalies.len() >= self.threshold {
            vec![ParadigmShiftHypothesis {
                description: format!("cross-domain resonance: {} domains, {} anomalies", domains.len(), self.anomalies.len()),
                cross_domain: true,
                novelty: 0.8,
            }]
        } else { vec![] }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_detect() {
        let mut d = ParadigmShiftDetector::new(3);
        for domain in ["physics", "bio", "code"] {
            d.observe(Anomaly { domain: domain.into(), description: "x".into(), severity: 0.9 });
        }
        assert!(!d.detect().is_empty());
    }
}
