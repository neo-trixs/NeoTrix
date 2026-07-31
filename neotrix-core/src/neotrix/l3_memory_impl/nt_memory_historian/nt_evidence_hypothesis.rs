#![deny(clippy::unwrap_used)]

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Hypothesis status in the verification lifecycle
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HypothesisStatus {
    Proposed,
    GatheringEvidence,
    Evaluating,
    Supported,
    Refuted,
    Inconclusive,
    Superseded,
}

impl HypothesisStatus {
    pub fn label(&self) -> &str {
        match self {
            HypothesisStatus::Proposed => "提出",
            HypothesisStatus::GatheringEvidence => "收集中",
            HypothesisStatus::Evaluating => "评估中",
            HypothesisStatus::Supported => "支持",
            HypothesisStatus::Refuted => "反驳",
            HypothesisStatus::Inconclusive => "无法定论",
            HypothesisStatus::Superseded => "被取代",
        }
    }
}

/// A formal hypothesis: a claim about reality that can be tested against evidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hypothesis {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: HypothesisStatus,
    pub prior_probability: f64,
    pub posterior_probability: f64,
    pub supporting_weight: f64,
    pub refuting_weight: f64,
    pub evidence_ids: Vec<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub tags: Vec<String>,
    pub parent_hypothesis: Option<String>,
}

impl Hypothesis {
    pub fn new(id: &str, title: &str, description: &str, prior: f64) -> Self {
        let ts = Utc::now().timestamp();
        Self {
            id: id.to_string(),
            title: title.to_string(),
            description: description.to_string(),
            status: HypothesisStatus::Proposed,
            prior_probability: prior.clamp(0.01, 0.99),
            posterior_probability: prior.clamp(0.01, 0.99),
            supporting_weight: 0.0,
            refuting_weight: 0.0,
            evidence_ids: Vec::new(),
            created_at: ts,
            updated_at: ts,
            tags: Vec::new(),
            parent_hypothesis: None,
        }
    }

    pub fn bayesian_update(&mut self, likelihood_given_h: f64, likelihood_given_not_h: f64) {
        let prior = self.posterior_probability;
        let odds = prior / (1.0 - prior + 1e-12);
        let bayes_factor = if likelihood_given_not_h > 0.0 {
            (likelihood_given_h + 1e-12) / (likelihood_given_not_h + 1e-12)
        } else {
            likelihood_given_h / 1e-12
        };
        let new_odds = odds * bayes_factor;
        self.posterior_probability = (new_odds / (1.0 + new_odds)).clamp(0.001, 0.999);
        self.updated_at = Utc::now().timestamp();
    }

    pub fn update_with_evidence(
        &mut self,
        evidence_confidence: f64,
        supports_hypothesis: bool,
        evidence_strength: f64,
    ) {
        let strength = evidence_confidence * evidence_strength;
        if supports_hypothesis {
            self.supporting_weight += strength;
            self.bayesian_update(0.7 * strength + 0.3, 0.3 * strength + 0.1);
        } else {
            self.refuting_weight += strength;
            self.bayesian_update(0.3 * strength + 0.1, 0.7 * strength + 0.3);
        }
        if self.posterior_probability > 0.85 {
            self.status = HypothesisStatus::Supported;
        } else if self.posterior_probability < 0.15 {
            self.status = HypothesisStatus::Refuted;
        } else if self.supporting_weight + self.refuting_weight > 5.0 {
            self.status = HypothesisStatus::Inconclusive;
        } else {
            self.status = HypothesisStatus::Evaluating;
        }
    }

    pub fn add_evidence(&mut self, evidence_id: &str) {
        if !self.evidence_ids.contains(&evidence_id.to_string()) {
            self.evidence_ids.push(evidence_id.to_string());
            self.updated_at = Utc::now().timestamp();
        }
    }

    pub fn bayes_factor_summary(&self) -> String {
        let log_bayes = (self.posterior_probability / (1.0 - self.posterior_probability + 1e-12))
            .ln()
            - (self.prior_probability / (1.0 - self.prior_probability + 1e-12)).ln();
        format!(
            "prior={:.3} posterior={:.3} logBF={:+.2} support={:.3} refute={:.3} status={}",
            self.prior_probability,
            self.posterior_probability,
            log_bayes,
            self.supporting_weight,
            self.refuting_weight,
            self.status.label(),
        )
    }
}

/// Subjective Logic opinion: belief, disbelief, uncertainty, base rate
/// ω = (b, d, u, a) where b + d + u = 1
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SubjectiveOpinion {
    pub belief: f64,
    pub disbelief: f64,
    pub uncertainty: f64,
    pub base_rate: f64,
}

impl SubjectiveOpinion {
    pub fn new(belief: f64, disbelief: f64, uncertainty: f64, base_rate: f64) -> Self {
        let total = belief + disbelief + uncertainty;
        if total > 0.0 {
            Self {
                belief: belief / total,
                disbelief: disbelief / total,
                uncertainty: uncertainty / total,
                base_rate,
            }
        } else {
            Self { belief: 0.0, disbelief: 0.0, uncertainty: 1.0, base_rate }
        }
    }

    pub fn projected_probability(&self) -> f64 {
        self.belief + self.base_rate * self.uncertainty
    }

    pub fn cumulative_fusion(a: &SubjectiveOpinion, b: &SubjectiveOpinion) -> SubjectiveOpinion {
        let belief = a.belief * b.uncertainty + a.uncertainty * b.belief;
        let disbelief = a.disbelief * b.uncertainty + a.uncertainty * b.disbelief;
        let uncertainty = a.uncertainty * b.uncertainty;
        let total = belief + disbelief + uncertainty;
        if total > 0.0 {
            SubjectiveOpinion {
                belief: belief / total,
                disbelief: disbelief / total,
                uncertainty: uncertainty / total,
                base_rate: (a.base_rate + b.base_rate) / 2.0,
            }
        } else {
            SubjectiveOpinion::new(0.0, 0.0, 1.0, 0.5)
        }
    }

    pub fn averaging_fusion(opinions: &[SubjectiveOpinion]) -> SubjectiveOpinion {
        if opinions.is_empty() {
            return SubjectiveOpinion::new(0.0, 0.0, 1.0, 0.5);
        }
        let mut b_sum = 0.0;
        let mut d_sum = 0.0;
        let mut u_sum = 0.0;
        let mut a_sum = 0.0;
        let n = opinions.len() as f64;
        for o in opinions {
            b_sum += o.belief;
            d_sum += o.disbelief;
            u_sum += o.uncertainty;
            a_sum += o.base_rate;
        }
        let total = b_sum + d_sum + u_sum;
        if total > 0.0 {
            SubjectiveOpinion {
                belief: b_sum / n,
                disbelief: d_sum / n,
                uncertainty: u_sum / n,
                base_rate: a_sum / n,
            }
        } else {
            SubjectiveOpinion::new(0.0, 0.0, 1.0, 0.5)
        }
    }
}

/// Weight of Evidence: log-odds contribution of a piece of evidence
pub fn weight_of_evidence(likelihood_h: f64, likelihood_not_h: f64) -> f64 {
    let lh = likelihood_h.max(1e-12);
    let ln = likelihood_not_h.max(1e-12);
    (lh / ln).ln()
}

/// Dempster-Shafer combination of two mass functions
pub fn dempster_shafer_combine(m1: &HashMap<String, f64>, m2: &HashMap<String, f64>) -> HashMap<String, f64> {
    let mut combined = HashMap::new();
    let mut conflict = 0.0;
    for (a, ma) in m1 {
        for (b, mb) in m2 {
            let product = ma * mb;
            if a == b {
                *combined.entry(a.clone()).or_insert(0.0) += product;
            } else {
                let intersection = a.chars().filter(|c| b.contains(*c)).collect::<String>();
                if !intersection.is_empty() {
                    *combined.entry(intersection).or_insert(0.0) += product;
                } else {
                    conflict += product;
                }
            }
        }
    }
    let norm = 1.0 - conflict;
    if norm > 1e-12 {
        for v in combined.values_mut() {
            *v /= norm;
        }
    }
    combined
}

/// Audit trail entry tracking every change to evidence or hypothesis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: String,
    pub timestamp: i64,
    pub action: String,
    pub entity_type: String,
    pub entity_id: String,
    pub field_changed: Option<String>,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub reason: Option<String>,
    pub actor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditTrail {
    pub entries: Vec<AuditEntry>,
    pub entity_versions: HashMap<String, u64>,
}

impl Default for AuditTrail {
    fn default() -> Self { Self::new() }
}

impl AuditTrail {
    pub fn new() -> Self {
        Self { entries: Vec::new(), entity_versions: HashMap::new() }
    }

    pub fn record(
        &mut self,
        entity_type: &str,
        entity_id: &str,
        action: &str,
        field_changed: Option<&str>,
        old_value: Option<&str>,
        new_value: Option<&str>,
        reason: Option<&str>,
        actor: &str,
    ) {
        let version = self.entity_versions
            .entry(format!("{}:{}", entity_type, entity_id))
            .or_insert(0);
        *version += 1;
        let entry = AuditEntry {
            id: format!("aud-{}-v{}", entity_id, version),
            timestamp: Utc::now().timestamp(),
            action: action.to_string(),
            entity_type: entity_type.to_string(),
            entity_id: entity_id.to_string(),
            field_changed: field_changed.map(String::from),
            old_value: old_value.map(String::from),
            new_value: new_value.map(String::from),
            reason: reason.map(String::from),
            actor: actor.to_string(),
        };
        self.entries.push(entry);
    }

    pub fn history(&self, entity_id: &str) -> Vec<&AuditEntry> {
        self.entries.iter().filter(|e| e.entity_id == entity_id).collect()
    }

    pub fn recent(&self, n: usize) -> Vec<&AuditEntry> {
        let mut sorted: Vec<&AuditEntry> = self.entries.iter().collect();
        sorted.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        sorted.into_iter().take(n).collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HypothesisNetwork {
    pub hypotheses: Vec<Hypothesis>,
    pub audit: AuditTrail,
    pub opinions: HashMap<String, SubjectiveOpinion>,
}

impl Default for HypothesisNetwork {
    fn default() -> Self { Self::new() }
}

impl HypothesisNetwork {
    pub fn new() -> Self {
        Self { hypotheses: Vec::new(), audit: AuditTrail::new(), opinions: HashMap::new() }
    }

    pub fn propose_hypothesis(&mut self, id: &str, title: &str, desc: &str, prior: f64) -> &mut Hypothesis {
        let h = Hypothesis::new(id, title, desc, prior);
        self.audit.record("hypothesis", id, "created", None, None, Some(&format!("prior={}", prior)), None, "system");
        self.hypotheses.push(h);
        self.hypotheses.last_mut().unwrap()
    }

    pub fn get_hypothesis(&self, id: &str) -> Option<&Hypothesis> {
        self.hypotheses.iter().find(|h| h.id == id)
    }

    pub fn get_hypothesis_mut(&mut self, id: &str) -> Option<&mut Hypothesis> {
        self.hypotheses.iter_mut().find(|h| h.id == id)
    }

    pub fn find_strongest_supported(&self) -> Option<&Hypothesis> {
        self.hypotheses.iter()
            .filter(|h| h.status == HypothesisStatus::Supported)
            .max_by(|a, b| a.posterior_probability.total_cmp(&b.posterior_probability))
    }

    pub fn find_strongest_refuted(&self) -> Option<&Hypothesis> {
        self.hypotheses.iter()
            .filter(|h| h.status == HypothesisStatus::Refuted)
            .min_by(|a, b| a.posterior_probability.total_cmp(&b.posterior_probability))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hypothesis_bayesian_update_support() {
        let mut h = Hypothesis::new("h1", "Test", "Test hypothesis", 0.5);
        h.bayesian_update(0.8, 0.2);
        assert!(h.posterior_probability > 0.5);
        assert!(h.posterior_probability < 0.95);
    }

    #[test]
    fn test_hypothesis_bayesian_update_refute() {
        let mut h = Hypothesis::new("h2", "Test", "Test hypothesis", 0.5);
        h.bayesian_update(0.2, 0.8);
        assert!(h.posterior_probability < 0.5);
    }

    #[test]
    fn test_hypothesis_strong_evidence_supported() {
        let mut h = Hypothesis::new("h3", "Test", "Test hypothesis", 0.3);
        for _ in 0..10 {
            h.update_with_evidence(0.9, true, 0.8);
        }
        assert_eq!(h.status, HypothesisStatus::Supported);
        assert!(h.posterior_probability > 0.85);
    }

    #[test]
    fn test_hypothesis_strong_refutation() {
        let mut h = Hypothesis::new("h4", "Test", "Test hypothesis", 0.7);
        for _ in 0..10 {
            h.update_with_evidence(0.9, false, 0.8);
        }
        assert_eq!(h.status, HypothesisStatus::Refuted);
        assert!(h.posterior_probability < 0.15);
    }

    #[test]
    fn test_subjective_opinion_fusion() {
        let a = SubjectiveOpinion::new(0.6, 0.2, 0.2, 0.5);
        let b = SubjectiveOpinion::new(0.7, 0.1, 0.2, 0.5);
        let fused = SubjectiveOpinion::cumulative_fusion(&a, &b);
        assert!(fused.uncertainty < a.uncertainty);
        assert!(fused.uncertainty < b.uncertainty);
        assert!((fused.projected_probability() - 0.5).abs() < 0.5);
    }

    #[test]
    fn test_weight_of_evidence_positive() {
        let woe = weight_of_evidence(0.9, 0.1);
        assert!(woe > 0.0);
    }

    #[test]
    fn test_weight_of_evidence_negative() {
        let woe = weight_of_evidence(0.1, 0.9);
        assert!(woe < 0.0);
    }

    #[test]
    fn test_dempster_shafer_basic() {
        let mut m1 = HashMap::new();
        m1.insert("A".into(), 0.6);
        m1.insert("B".into(), 0.4);
        let mut m2 = HashMap::new();
        m2.insert("A".into(), 0.7);
        m2.insert("C".into(), 0.3);
        let result = dempster_shafer_combine(&m1, &m2);
        assert!(result.contains_key("A"));
    }

    #[test]
    fn test_audit_trail_records_entries() {
        let mut audit = AuditTrail::new();
        audit.record("evidence", "ev1", "created", None, None, Some("initial"), None, "system");
        audit.record("evidence", "ev1", "updated", Some("confidence"), Some("0.5"), Some("0.8"), Some("calibration"), "system");
        assert_eq!(audit.entries.len(), 2);
        let history = audit.history("ev1");
        assert_eq!(history.len(), 2);
    }

    #[test]
    fn test_audit_trail_versioning() {
        let mut audit = AuditTrail::new();
        audit.record("hypothesis", "h1", "created", None, None, Some("initial"), None, "system");
        audit.record("hypothesis", "h1", "updated", Some("status"), Some("proposed"), Some("supported"), None, "system");
        let v = audit.entity_versions.get("hypothesis:h1").unwrap();
        assert_eq!(*v, 2);
    }

    #[test]
    fn test_hypothesis_network_create() {
        let mut net = HypothesisNetwork::new();
        net.propose_hypothesis("h1", "Origin of X", "X was created in 1000 BCE", 0.3);
        assert_eq!(net.hypotheses.len(), 1);
        let h = net.get_hypothesis("h1").unwrap();
        assert_eq!(h.status, HypothesisStatus::Proposed);
    }

    #[test]
    fn test_hypothesis_network_find_strongest() {
        let mut net = HypothesisNetwork::new();
        net.propose_hypothesis("h1", "Weak", "weak", 0.5);
        let h2 = net.propose_hypothesis("h2", "Strong", "strong", 0.8);
        for _ in 0..10 { h2.update_with_evidence(0.9, true, 0.9); }
        let strongest = net.find_strongest_supported();
        assert!(strongest.is_some());
        assert_eq!(strongest.unwrap().id, "h2");
    }

    #[test]
    fn test_subjective_opinion_averaging() {
        let opinions = vec![
            SubjectiveOpinion::new(0.8, 0.1, 0.1, 0.5),
            SubjectiveOpinion::new(0.6, 0.2, 0.2, 0.5),
            SubjectiveOpinion::new(0.7, 0.1, 0.2, 0.5),
        ];
        let avg = SubjectiveOpinion::averaging_fusion(&opinions);
        assert!((avg.belief - 0.7).abs() < 0.02);
    }

    #[test]
    fn test_hypothesis_mixed_evidence_inconclusive() {
        let mut h = Hypothesis::new("h5", "Mixed", "Equal support and refutation", 0.5);
        for _ in 0..5 { h.update_with_evidence(0.9, true, 1.0); }
        for _ in 0..5 { h.update_with_evidence(0.9, false, 1.0); }
        // total weight = 10 * 0.9*1.0 = 9.0 > 5.0, and posterior near 0.5
        assert_eq!(h.status, HypothesisStatus::Inconclusive);
    }

    #[test]
    fn test_hypothesis_superseded_by_parent() {
        let mut h = Hypothesis::new("h6", "Old", "Superseded hypothesis", 0.5);
        h.parent_hypothesis = Some("h7".into());
        // When replaced, we mark as superseded directly
        h.status = HypothesisStatus::Superseded;
        assert_eq!(h.status, HypothesisStatus::Superseded);
    }
}
