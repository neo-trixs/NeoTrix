use crate::core::nt_core_knowledge::evidence::{EvidenceManager, EvidenceRecord, EvidenceState};
use crate::core::nt_core_knowledge::types::KnowledgeEntry;
// inking --

pub type ClaimId = u64;
pub type EvidenceRef = u64;

#[derive(Debug, Clone)]
pub struct Claim {
    pub id: ClaimId,
    pub text: String,
    pub evidence_refs: Vec<EvidenceRef>,
    pub verification_status: VerificationStatus,
    pub confidence: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerificationStatus {
    Unverified,
    Verified,
    Failed,
    Inconclusive,
}

impl VerificationStatus {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Unverified => "unverified",
            Self::Verified => "verified",
            Self::Failed => "failed",
            Self::Inconclusive => "inconclusive",
        }
    }
}

#[derive(Debug, Clone)]
pub struct EvidenceVerificationResult {
    pub claim_id: ClaimId,
    pub status: VerificationStatus,
    pub matched_evidence: Vec<EvidenceRef>,
    pub confidence: f64,
    pub details: String,
}

// ── Evidence Inspector ──

const MAX_CLAIMS: usize = 1000;

pub struct EvidenceInspector {
    pub claims: Vec<Claim>,
    pub evidence_manager: EvidenceManager,
    next_claim_id: u64,
}

impl EvidenceInspector {
    pub fn new(evidence_manager: EvidenceManager) -> Self {
        Self {
            claims: Vec::new(),
            evidence_manager,
            next_claim_id: 1,
        }
    }

    pub fn claim_count(&self) -> usize {
        self.claims.len()
    }

    pub fn evidence_count(&self) -> usize {
        self.evidence_manager.evidence_count()
    }

    /// Record a claim and link it to existing evidence.
    pub fn record_claim(&mut self, text: &str, evidence_ids: &[u64]) -> ClaimId {
        let id = self.next_claim_id;
        self.next_claim_id += 1;
        let claim = Claim {
            id,
            text: text.to_string(),
            evidence_refs: evidence_ids.to_vec(),
            verification_status: VerificationStatus::Unverified,
            confidence: 0.3,
        };
        self.claims.push(claim);
        if self.claims.len() > MAX_CLAIMS {
            let overflow = self.claims.len() - MAX_CLAIMS;
            self.claims.drain(0..overflow);
        }
        id
    }

    /// Verify a claim by checking if linked evidence supports it.
    pub fn verify_claim(&self, claim_id: ClaimId) -> EvidenceVerificationResult {
        let claim = match self.claims.iter().find(|c| c.id == claim_id) {
            Some(c) => c,
            None => {
                return EvidenceVerificationResult {
                    claim_id,
                    status: VerificationStatus::Inconclusive,
                    matched_evidence: Vec::new(),
                    confidence: 0.0,
                    details: "claim not found".to_string(),
                };
            }
        };
        if claim.evidence_refs.is_empty() {
            return EvidenceVerificationResult {
                claim_id,
                status: VerificationStatus::Inconclusive,
                matched_evidence: Vec::new(),
                confidence: 0.0,
                details: "no evidence linked to claim".to_string(),
            };
        }
        let evidence_records: Vec<&EvidenceRecord> = claim
            .evidence_refs
            .iter()
            .filter_map(|id| self.evidence_manager.get(*id))
            .collect();
        if evidence_records.is_empty() {
            return EvidenceVerificationResult {
                claim_id,
                status: VerificationStatus::Failed,
                matched_evidence: Vec::new(),
                confidence: 0.0,
                details: "linked evidence not found in manager".to_string(),
            };
        }
        let avg_confidence = evidence_records.iter().map(|e| e.confidence).sum::<f64>()
            / evidence_records.len() as f64;
        let all_validated = evidence_records
            .iter()
            .all(|e| e.state == EvidenceState::Validated);
        let any_disputed = evidence_records
            .iter()
            .any(|e| e.state == EvidenceState::Disputed);
        let status = if any_disputed {
            VerificationStatus::Failed
        } else if all_validated {
            VerificationStatus::Verified
        } else if avg_confidence > 0.5 {
            VerificationStatus::Verified
        } else {
            VerificationStatus::Inconclusive
        };
        EvidenceVerificationResult {
            claim_id,
            status,
            matched_evidence: claim.evidence_refs.clone(),
            confidence: avg_confidence,
            details: format!(
                "claim '{}': {} evidence records, avg confidence {:.2}",
                claim.text,
                evidence_records.len(),
                avg_confidence
            ),
        }
    }

    /// Verify all unverified claims.
    pub fn verify_all(&self) -> Vec<EvidenceVerificationResult> {
        self.claims
            .iter()
            .filter(|c| c.verification_status == VerificationStatus::Unverified)
            .map(|c| self.verify_claim(c.id))
            .collect()
    }

    /// Verify a KnowledgeEntry by looking up its linked evidence records.
    /// Produces a combined verification status based on evidence state and confidence.
    pub fn verify_entry(&self, entry: &KnowledgeEntry) -> EvidenceVerificationResult {
        let evidence_ids = &entry.evidence_ids;
        if evidence_ids.is_empty() {
            return EvidenceVerificationResult {
                claim_id: 0,
                status: VerificationStatus::Inconclusive,
                matched_evidence: Vec::new(),
                confidence: 0.0,
                details: format!("entry '{}' has no linked evidence", entry.title),
            };
        }
        let evidence_records: Vec<&EvidenceRecord> = evidence_ids
            .iter()
            .filter_map(|id| self.evidence_manager.get(*id))
            .collect();
        if evidence_records.is_empty() {
            return EvidenceVerificationResult {
                claim_id: 0,
                status: VerificationStatus::Failed,
                matched_evidence: Vec::new(),
                confidence: 0.0,
                details: format!(
                    "entry '{}': linked evidence IDs {:?} not found in manager",
                    entry.title, evidence_ids
                ),
            };
        }
        let avg_confidence = evidence_records.iter().map(|e| e.confidence).sum::<f64>()
            / evidence_records.len() as f64;
        let all_validated = evidence_records
            .iter()
            .all(|e| e.state == EvidenceState::Validated);
        let any_disputed = evidence_records
            .iter()
            .any(|e| e.state == EvidenceState::Disputed);
        let status = if any_disputed {
            VerificationStatus::Failed
        } else if all_validated {
            VerificationStatus::Verified
        } else if avg_confidence > 0.5 {
            VerificationStatus::Verified
        } else {
            VerificationStatus::Inconclusive
        };
        EvidenceVerificationResult {
            claim_id: 0,
            status,
            matched_evidence: evidence_ids.clone(),
            confidence: avg_confidence,
            details: format!(
                "entry '{}': {} evidence records, avg confidence {:.2}, status {}",
                entry.title,
                evidence_records.len(),
                avg_confidence,
                status.name()
            ),
        }
    }

    /// Generate an audit trail report.
    pub fn audit_report(&self) -> String {
        let mut report = String::new();
        report.push_str("=== Evidence Audit Report ===\n");
        report.push_str(&format!("Claims: {}\n", self.claims.len()));
        report.push_str(&format!(
            "Evidence: {}\n",
            self.evidence_manager.evidence_count()
        ));
        for claim in &self.claims {
            let result = self.verify_claim(claim.id);
            report.push_str(&format!(
                "  Claim #{} [{}]: '{}' confidence={:.2}\n",
                claim.id,
                result.status.name(),
                claim.text,
                result.confidence
            ));
            for ev_id in &claim.evidence_refs {
                if let Some(ev) = self.evidence_manager.get(*ev_id) {
                    report.push_str(&format!(
                        "    Evidence #{}: '{}' [{}]\n",
                        ev.id,
                        ev.assertion,
                        ev.state.name()
                    ));
                }
            }
        }
        report
    }
}

// ── Verifiability Gate ──

#[derive(Debug, Clone)]
pub struct VerifiabilityGate {
    pub min_confidence: f64,
    pub require_all_verified: bool,
}

impl Default for VerifiabilityGate {
    fn default() -> Self {
        Self {
            min_confidence: 0.6,
            require_all_verified: false,
        }
    }
}

impl VerifiabilityGate {
    pub fn new(min_confidence: f64) -> Self {
        Self {
            min_confidence,
            require_all_verified: false,
        }
    }

    /// Check whether a claim passes the verifiability gate.
    pub fn check(&self, result: &EvidenceVerificationResult) -> bool {
        if self.require_all_verified && result.status != VerificationStatus::Verified {
            return false;
        }
        result.confidence >= self.min_confidence
    }

    /// Filter a list of verification results, returning only those that pass.
    pub fn filter<'a>(
        &self,
        results: &'a [EvidenceVerificationResult],
    ) -> Vec<&'a EvidenceVerificationResult> {
        results.iter().filter(|r| self.check(r)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_knowledge::evidence::EvidenceRecord;
    use crate::core::nt_core_knowledge::KnowledgeSourceType;
    use serial_test::serial;

    fn make_inspector() -> EvidenceInspector {
        let mut em = EvidenceManager::new(100);
        em.add_evidence("https://example.com", "example", "test assertion 1");
        em.add_evidence("https://test.com", "test", "test assertion 2");
        EvidenceInspector::new(em)
    }

    fn make_validated_inspector() -> EvidenceInspector {
        let mut em = EvidenceManager::new(100);
        let id1 = em.add_evidence("https://example.com", "example", "validated claim");
        if let Some(rec) = em.get_mut(id1) {
            rec.verify();
        }
        EvidenceInspector::new(em)
    }

    #[serial]
    #[test]
    fn test_record_claim() {
        let mut inspector = make_inspector();
        let id = inspector.record_claim("claim 1", &[1]);
        assert_eq!(id, 1);
        assert_eq!(inspector.claim_count(), 1);
    }

    #[test]
    fn test_verify_claim_with_evidence() {
        let inspector = make_inspector();
        let mut i2 = inspector;
        i2.record_claim("test claim", &[1]);
        let result = i2.verify_claim(1);
        assert_eq!(result.claim_id, 1);
        assert!(!result.matched_evidence.is_empty());
    }

    #[test]
    fn test_verify_claim_no_evidence() {
        let inspector = make_inspector();
        let mut i2 = inspector;
        i2.record_claim("orphan claim", &[]);
        let result = i2.verify_claim(1);
        assert_eq!(result.status, VerificationStatus::Inconclusive);
    }

    #[test]
    fn test_verify_claim_not_found() {
        let inspector = make_inspector();
        let result = inspector.verify_claim(999);
        assert_eq!(result.status, VerificationStatus::Inconclusive);
    }

    #[test]
    fn test_verify_all() {
        let mut inspector = make_validated_inspector();
        inspector.record_claim("validated claim", &[1]);
        let results = inspector.verify_all();
        assert!(!results.is_empty());
        assert!(results
            .iter()
            .any(|r| r.status == VerificationStatus::Verified));
    }

    #[test]
    fn test_audit_report() {
        let mut inspector = make_inspector();
        inspector.record_claim("test claim", &[1]);
        let report = inspector.audit_report();
        assert!(report.contains("Evidence Audit Report"));
        assert!(report.contains("test claim"));
    }

    #[test]
    fn test_verifiability_gate() {
        let gate = VerifiabilityGate::new(0.6);
        let result = EvidenceVerificationResult {
            claim_id: 1,
            status: VerificationStatus::Verified,
            matched_evidence: vec![1],
            confidence: 0.8,
            details: "ok".into(),
        };
        assert!(gate.check(&result));
    }

    #[test]
    fn test_verifiability_gate_low_confidence() {
        let gate = VerifiabilityGate::new(0.6);
        let result = EvidenceVerificationResult {
            claim_id: 1,
            status: VerificationStatus::Unverified,
            matched_evidence: vec![],
            confidence: 0.3,
            details: "low".into(),
        };
        assert!(!gate.check(&result));
    }

    #[test]
    fn test_verifiability_gate_require_all() {
        let gate = VerifiabilityGate {
            min_confidence: 0.5,
            require_all_verified: true,
        };
        let verified = EvidenceVerificationResult {
            claim_id: 1,
            status: VerificationStatus::Verified,
            matched_evidence: vec![1],
            confidence: 0.7,
            details: "ok".into(),
        };
        let failed = EvidenceVerificationResult {
            claim_id: 2,
            status: VerificationStatus::Failed,
            matched_evidence: vec![],
            confidence: 0.1,
            details: "fail".into(),
        };
        assert!(gate.check(&verified));
        assert!(!gate.check(&failed));
    }

    #[test]
    fn test_verifiability_gate_filter() {
        let gate = VerifiabilityGate::new(0.5);
        let results = vec![
            EvidenceVerificationResult {
                claim_id: 1,
                status: VerificationStatus::Verified,
                matched_evidence: vec![],
                confidence: 0.9,
                details: "".into(),
            },
            EvidenceVerificationResult {
                claim_id: 2,
                status: VerificationStatus::Failed,
                matched_evidence: vec![],
                confidence: 0.1,
                details: "".into(),
            },
        ];
        let filtered = gate.filter(&results);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].claim_id, 1);
    }

    #[test]
    fn test_verify_entry_no_evidence() {
        let inspector = make_inspector();
        let entry = KnowledgeEntry::new(
            "Test Entry",
            "body",
            KnowledgeSourceType::WebPage,
            "https://example.com",
        );
        let result = inspector.verify_entry(&entry);
        assert_eq!(result.status, VerificationStatus::Inconclusive);
        assert!(result.details.contains("no linked evidence"));
    }

    #[test]
    fn test_verify_entry_with_evidence() {
        let mut em = EvidenceManager::new(100);
        let eid = em.add_evidence("https://example.com", "example", "validated claim");
        if let Some(rec) = em.get_mut(eid) {
            rec.verify();
        }
        let inspector = EvidenceInspector::new(em);
        let mut entry = KnowledgeEntry::new(
            "Validated Entry",
            "body",
            KnowledgeSourceType::WebPage,
            "https://example.com",
        );
        entry.evidence_ids.push(eid);
        let result = inspector.verify_entry(&entry);
        assert_eq!(result.status, VerificationStatus::Verified);
        assert!(result.confidence > 0.5);
    }

    #[test]
    fn test_verify_entry_disputed() {
        let mut em = EvidenceManager::new(100);
        let eid = em.add_evidence("https://example.com", "example", "disputed claim");
        if let Some(rec) = em.get_mut(eid) {
            rec.verify();
            rec.dispute();
        }
        let inspector = EvidenceInspector::new(em);
        let mut entry = KnowledgeEntry::new(
            "Disputed Entry",
            "body",
            KnowledgeSourceType::WebPage,
            "https://example.com",
        );
        entry.evidence_ids.push(eid);
        let result = inspector.verify_entry(&entry);
        assert_eq!(result.status, VerificationStatus::Failed);
    }

    #[test]
    fn test_verify_entry_evidence_not_found() {
        let inspector = make_inspector();
        let mut entry = KnowledgeEntry::new(
            "Orphan Entry",
            "body",
            KnowledgeSourceType::WebPage,
            "https://example.com",
        );
        entry.evidence_ids.push(999);
        let result = inspector.verify_entry(&entry);
        assert_eq!(result.status, VerificationStatus::Failed);
        assert!(result.details.contains("not found"));
    }
}
