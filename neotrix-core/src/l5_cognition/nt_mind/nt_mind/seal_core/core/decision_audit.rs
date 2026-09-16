//! Decision Audit Trail — 决策审计轨迹
//! 不可变决策历史，支持回溯和可解释性

use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionRecord {
    pub decision_id: u64,
    pub decision_type: String,
    pub context: String,
    pub options: Vec<String>,
    pub chosen: String,
    pub reasoning: String,
    pub confidence: f64,
    pub outcome: Option<String>,
    pub timestamp: u64,
}

pub struct DecisionAudit {
    records: Vec<DecisionRecord>,
    next_id: u64,
}

impl DecisionAudit {
    pub fn new() -> Self {
        Self { records: Vec::new(), next_id: 0 }
    }

    pub fn record(&mut self, decision_type: &str, context: &str, options: Vec<String>, chosen: &str, reasoning: &str, confidence: f64) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.records.push(DecisionRecord {
            decision_id: id,
            decision_type: decision_type.to_string(),
            context: context.to_string(),
            options,
            chosen: chosen.to_string(),
            reasoning: reasoning.to_string(),
            confidence,
            outcome: None,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
        });
        id
    }

    pub fn record_outcome(&mut self, id: u64, outcome: &str) {
        if let Some(record) = self.records.iter_mut().find(|r| r.decision_id == id) {
            record.outcome = Some(outcome.to_string());
        }
    }

    pub fn get(&self, id: u64) -> Option<&DecisionRecord> {
        self.records.iter().find(|r| r.decision_id == id)
    }

    pub fn by_type(&self, decision_type: &str) -> Vec<&DecisionRecord> {
        self.records.iter().filter(|r| r.decision_type == decision_type).collect()
    }

    pub fn recent(&self, n: usize) -> Vec<&DecisionRecord> {
        self.records.iter().rev().take(n).collect()
    }

    pub fn explain(&self, id: u64) -> Option<String> {
        let record = self.get(id)?;
        Some(format!(
            "Decision #{}: {} | Context: {} | Chosen: {} | Reasoning: {} | Confidence: {:.1}%",
            record.decision_id, record.decision_type, record.context, record.chosen, record.reasoning, record.confidence * 100.0
        ))
    }

    pub fn len(&self) -> usize { self.records.len() }
    pub fn is_empty(&self) -> bool { self.records.is_empty() }
}

impl Default for DecisionAudit {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_basic() {
        let mut audit = DecisionAudit::new();
        let id = audit.record("routing", "select model", vec!["fast".into(), "slow".into()], "fast", "Low latency needed", 0.8);
        
        assert_eq!(audit.len(), 1);
        assert!(audit.explain(id).is_some());
        
        audit.record_outcome(id, "success");
        let record = audit.get(id).unwrap();
        assert_eq!(record.outcome.as_deref(), Some("success"));
    }
}
