//! Evidence Ledger - 证据账本模块
//!
//! Claw核心: 会话账本 + 反幻觉闸门 + 技能按需加载

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// 证据条目
#[derive(Debug, Clone)]
pub struct EvidenceEntry {
    pub id: String,
    pub timestamp: u64,
    pub evidence_type: EvidenceType,
    pub content: String,
    pub source: String,
    pub confidence: f64,
    pub verified: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceType {
    Observation,
    Inference,
    Claim,
    Refutation,
    External,
}

/// 会话账本
pub struct SessionLedger {
    session_id: String,
    entries: Vec<EvidenceEntry>,
    claims: Vec<Claim>,
    claim_gates: Vec<ClaimGate>,
}

#[derive(Debug, Clone)]
pub struct Claim {
    pub id: String,
    pub statement: String,
    pub evidence_ids: Vec<String>,
    pub confidence: f64,
    pub status: ClaimStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClaimStatus {
    Pending,
    Verified,
    Refuted,
    Uncertain,
}

#[derive(Debug, Clone)]
pub struct ClaimGate {
    pub claim_id: String,
    pub gate_type: GateType,
    pub passed: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateType {
    EvidenceSupport,
    CrossReference,
    SourceReliability,
    LogicalConsistency,
}

impl SessionLedger {
    pub fn new(session_id: &str) -> Self {
        Self {
            session_id: session_id.to_string(),
            entries: Vec::new(),
            claims: Vec::new(),
            claim_gates: Vec::new(),
        }
    }

    /// 添加证据
    pub fn add_evidence(&mut self, evidence_type: EvidenceType, content: &str, source: &str, confidence: f64) -> String {
        let id = format!("evidence_{}", self.entries.len());
        let entry = EvidenceEntry {
            id: id.clone(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            evidence_type,
            content: content.to_string(),
            source: source.to_string(),
            confidence,
            verified: false,
        };
        self.entries.push(entry);
        id
    }

    /// 添加声明
    pub fn add_claim(&mut self, statement: &str, evidence_ids: Vec<String>, confidence: f64) -> String {
        let id = format!("claim_{}", self.claims.len());
        let claim = Claim {
            id: id.clone(),
            statement: statement.to_string(),
            evidence_ids,
            confidence,
            status: ClaimStatus::Pending,
        };
        self.claims.push(claim);
        id
    }

    /// 验证声明 (反幻觉闸门)
    pub fn verify_claim(&mut self, claim_id: &str) -> bool {
        if let Some(claim) = self.claims.iter_mut().find(|c| c.id == claim_id) {
            // 检查证据支持
            let supporting_evidence: Vec<&EvidenceEntry> = self.entries.iter()
                .filter(|e| claim.evidence_ids.contains(&e.id))
                .collect();

            let total_confidence: f64 = supporting_evidence.iter()
                .map(|e| e.confidence)
                .sum();

            let avg_confidence = if supporting_evidence.is_empty() {
                0.0
            } else {
                total_confidence / supporting_evidence.len() as f64
            };

            // 通过闸门检查
            let gates_passed = avg_confidence > 0.6 && supporting_evidence.len() >= 2;

            if gates_passed {
                claim.status = ClaimStatus::Verified;
                true
            } else {
                claim.status = ClaimStatus::Refuted;
                false
            }
        } else {
            false
        }
    }

    /// 检测停滞
    pub fn detect_stall(&self) -> bool {
        if self.entries.len() < 2 {
            return false;
        }

        let recent_entries = &self.entries[self.entries.len().saturating_sub(5)..];
        let unique_sources: std::collections::HashSet<&str> = recent_entries.iter()
            .map(|e| e.source.as_str())
            .collect();

        // 如果最近5条证据来自同一来源，可能停滞
        unique_sources.len() <= 1
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> HashMap<String, String> {
        let mut stats = HashMap::new();
        stats.insert("session_id".to_string(), self.session_id.clone());
        stats.insert("total_entries".to_string(), self.entries.len().to_string());
        stats.insert("total_claims".to_string(), self.claims.len().to_string());
        stats.insert("verified_claims".to_string(),
            self.claims.iter().filter(|c| c.status == ClaimStatus::Verified).count().to_string());
        stats.insert("refuted_claims".to_string(),
            self.claims.iter().filter(|c| c.status == ClaimStatus::Refuted).count().to_string());
        stats
    }
}

impl Default for SessionLedger {
    fn default() -> Self {
        Self::new("default")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_evidence() {
        let mut ledger = SessionLedger::new("test");
        let id = ledger.add_evidence(EvidenceType::Observation, "test content", "test_source", 0.8);
        assert!(!id.is_empty());
        assert_eq!(ledger.entries.len(), 1);
    }

    #[test]
    fn test_add_claim() {
        let mut ledger = SessionLedger::new("test");
        let id = ledger.add_claim("test statement", vec![], 0.7);
        assert!(!id.is_empty());
        assert_eq!(ledger.claims.len(), 1);
    }

    #[test]
    fn test_detect_stall() {
        let mut ledger = SessionLedger::new("test");
        // 添加多条来自同一来源的证据
        for _ in 0..5 {
            ledger.add_evidence(EvidenceType::Observation, "content", "same_source", 0.8);
        }
        assert!(ledger.detect_stall());
    }
}
