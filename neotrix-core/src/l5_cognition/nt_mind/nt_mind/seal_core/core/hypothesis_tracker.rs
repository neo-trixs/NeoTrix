//! Hypothesis Tracker — 假说追踪器
//! 追踪和验证研究假说，支持证据积累和置信度更新

use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HypothesisStatus {
    Proposed,
    Testing,
    Supported,
    Refuted,
    Abandoned,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hypothesis {
    pub id: String,
    pub claim: String,
    pub status: HypothesisStatus,
    pub confidence: f64,
    pub evidence_for: Vec<String>,
    pub evidence_against: Vec<String>,
    pub created_at: u64,
    pub updated_at: u64,
}

pub struct HypothesisTracker {
    hypotheses: Vec<Hypothesis>,
}

impl HypothesisTracker {
    pub fn new() -> Self {
        Self { hypotheses: Vec::new() }
    }

    pub fn propose(&mut self, claim: &str) -> String {
        let id = format!("hypo_{}", self.hypotheses.len());
        let now = Self::now();
        self.hypotheses.push(Hypothesis {
            id: id.clone(),
            claim: claim.to_string(),
            status: HypothesisStatus::Proposed,
            confidence: 0.5,
            evidence_for: Vec::new(),
            evidence_against: Vec::new(),
            created_at: now,
            updated_at: now,
        });
        id
    }

    pub fn add_evidence(&mut self, id: &str, supporting: bool, evidence: &str) {
        if let Some(h) = self.hypotheses.iter_mut().find(|h| h.id == id) {
            if supporting {
                h.evidence_for.push(evidence.to_string());
                h.confidence = (h.confidence + 0.1).min(1.0);
            } else {
                h.evidence_against.push(evidence.to_string());
                h.confidence = (h.confidence - 0.1).max(0.0);
            }
            h.updated_at = Self::now();
            
            // Auto-update status based on confidence
            if h.confidence > 0.8 && h.evidence_for.len() >= 3 {
                h.status = HypothesisStatus::Supported;
            } else if h.confidence < 0.2 {
                h.status = HypothesisStatus::Refuted;
            } else if !h.evidence_for.is_empty() || !h.evidence_against.is_empty() {
                h.status = HypothesisStatus::Testing;
            }
        }
    }

    pub fn get(&self, id: &str) -> Option<&Hypothesis> {
        self.hypotheses.iter().find(|h| h.id == id)
    }

    pub fn active(&self) -> Vec<&Hypothesis> {
        self.hypotheses.iter()
            .filter(|h| h.status == HypothesisStatus::Proposed || h.status == HypothesisStatus::Testing)
            .collect()
    }

    pub fn supported(&self) -> Vec<&Hypothesis> {
        self.hypotheses.iter()
            .filter(|h| h.status == HypothesisStatus::Supported)
            .collect()
    }

    pub fn len(&self) -> usize { self.hypotheses.len() }
    pub fn is_empty(&self) -> bool { self.hypotheses.is_empty() }

    fn now() -> u64 {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
    }
}

impl Default for HypothesisTracker {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracker_basic() {
        let mut t = HypothesisTracker::new();
        let id = t.propose("Rust is faster than Python");
        assert_eq!(t.len(), 1);
        
        t.add_evidence(&id, true, "Benchmark shows 10x speedup");
        t.add_evidence(&id, true, "Real-world test confirms");
        t.add_evidence(&id, true, "Memory usage 5x lower");
        
        let h = t.get(&id).unwrap();
        assert_eq!(h.status, HypothesisStatus::Supported);
        assert!(h.confidence > 0.7);
    }
}
