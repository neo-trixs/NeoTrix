use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SafetyLevel {
    Safe,
    Questionable,
    Unsafe,
    Unprovable,
}

#[derive(Debug, Clone)]
pub struct ModificationProposal {
    pub id: u64,
    pub target: String,
    pub change_description: String,
    pub change_vector: Vec<u8>,
    pub expected_impact: f64,
    pub preconditions: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ProofStep {
    pub step: u64,
    pub rule_applied: String,
    pub derived_fact: String,
    pub confidence: f64,
}

#[derive(Debug, Clone)]
pub struct SafetyVerificationResult {
    pub proposal_id: u64,
    pub safety: SafetyLevel,
    pub proof_steps: Vec<ProofStep>,
    pub confidence: f64,
    pub reasoning: String,
}

#[derive(Debug, Clone)]
pub struct SelfModificationProof {
    pub proposal: ModificationProposal,
    pub verification: SafetyVerificationResult,
    pub applied: bool,
    pub actual_impact: Option<f64>,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct ProofSearchConfig {
    pub max_proof_depth: usize,
    pub confidence_threshold: f64,
    pub max_proposals_in_flight: usize,
    pub history_capacity: usize,
}

impl Default for ProofSearchConfig {
    fn default() -> Self {
        Self {
            max_proof_depth: 5,
            confidence_threshold: 0.7,
            max_proposals_in_flight: 10,
            history_capacity: 100,
        }
    }
}

pub struct ProofSearchSelfModification {
    config: ProofSearchConfig,
    proposals: Vec<ModificationProposal>,
    proof_history: VecDeque<SelfModificationProof>,
    next_id: u64,
    step: u64,
}

const CRITICAL_TARGETS: &[&str] = &[
    "proof_search",
    "authority",
    "constitution",
    "master_equation",
    "consciousness_core",
];

impl ProofSearchSelfModification {
    pub fn new(config: ProofSearchConfig) -> Self {
        Self {
            config,
            proposals: Vec::new(),
            proof_history: VecDeque::new(),
            next_id: 0,
            step: 0,
        }
    }

    pub fn propose_modification(
        &mut self,
        target: &str,
        description: &str,
        change_vector: Vec<u8>,
        expected_impact: f64,
        preconditions: Vec<String>,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let proposal = ModificationProposal {
            id,
            target: target.to_string(),
            change_description: description.to_string(),
            change_vector,
            expected_impact,
            preconditions,
        };
        self.proposals.push(proposal);
        id
    }

    pub fn verify(&self, proposal_id: u64) -> Option<SafetyVerificationResult> {
        let proposal = self.proposals.iter().find(|p| p.id == proposal_id)?;
        self.run_verification(proposal)
    }

    fn run_verification(
        &self,
        proposal: &ModificationProposal,
    ) -> Option<SafetyVerificationResult> {
        let mut steps = Vec::new();
        let mut pass = 0u32;
        let total = 5u32;
        let mut reasoning = String::new();

        // integrity_check: change_vector non-zero
        {
            let is_nonzero = proposal.change_vector.iter().any(|&b| b != 0);
            steps.push(ProofStep {
                step: self.step,
                rule_applied: "integrity_check".to_string(),
                derived_fact: if is_nonzero {
                    "change_vector is non-zero".to_string()
                } else {
                    "change_vector is zero".to_string()
                },
                confidence: 1.0,
            });
            if is_nonzero {
                pass += 1;
                reasoning.push_str("integrity_check: pass. ");
            } else {
                reasoning.push_str("integrity_check: FAIL — zero change vector. ");
            }
        }

        // impact_bounded: |expected_impact| <= 1.0
        {
            let bounded = proposal.expected_impact.abs() <= 1.0;
            steps.push(ProofStep {
                step: self.step,
                rule_applied: "impact_bounded".to_string(),
                derived_fact: if bounded {
                    format!("impact {} within [-1.0, 1.0]", proposal.expected_impact)
                } else {
                    format!("impact {} exceeds [-1.0, 1.0]", proposal.expected_impact)
                },
                confidence: 1.0,
            });
            if bounded {
                pass += 1;
                reasoning.push_str("impact_bounded: pass. ");
            } else {
                reasoning.push_str("impact_bounded: FAIL — impact out of bounds. ");
            }
        }

        // change_small: hamming weight < dim/4
        {
            let dim = proposal.change_vector.len();
            let weight = proposal.change_vector.iter().filter(|&&b| b != 0).count();
            let small = weight < dim / 4;
            steps.push(ProofStep {
                step: self.step,
                rule_applied: "change_small".to_string(),
                derived_fact: if small {
                    format!("hamming weight {} < dim/4 ({})", weight, dim / 4)
                } else {
                    format!("hamming weight {} >= dim/4 ({})", weight, dim / 4)
                },
                confidence: 1.0,
            });
            if small {
                pass += 1;
                reasoning.push_str("change_small: pass. ");
            } else {
                reasoning.push_str("change_small: FAIL — too many bits flipped. ");
            }
        }

        // self_referential_safe: target != "proof_search"
        {
            let not_self = proposal.target != "proof_search";
            steps.push(ProofStep {
                step: self.step,
                rule_applied: "self_referential_safe".to_string(),
                derived_fact: if not_self {
                    format!("target '{}' is not proof_search", proposal.target)
                } else {
                    "target is proof_search — self-modification forbidden".to_string()
                },
                confidence: 1.0,
            });
            if not_self {
                pass += 1;
                reasoning.push_str("self_referential_safe: pass. ");
            } else {
                reasoning.push_str("self_referential_safe: FAIL — cannot modify self. ");
            }
        }

        // not_critical: target not in hardcoded critical list
        {
            let not_critical = !CRITICAL_TARGETS.contains(&proposal.target.as_str());
            steps.push(ProofStep {
                step: self.step,
                rule_applied: "not_critical".to_string(),
                derived_fact: if not_critical {
                    format!("target '{}' is not in critical list", proposal.target)
                } else {
                    format!(
                        "target '{}' is in critical list — modification forbidden",
                        proposal.target
                    )
                },
                confidence: 1.0,
            });
            if not_critical {
                pass += 1;
                reasoning.push_str("not_critical: pass. ");
            } else {
                reasoning.push_str("not_critical: FAIL — critical target. ");
            }
        }

        let confidence = pass as f64 / total as f64;
        let safety = if pass == total {
            SafetyLevel::Safe
        } else if pass >= 2 {
            SafetyLevel::Questionable
        } else if pass == 0 {
            SafetyLevel::Unprovable
        } else {
            SafetyLevel::Unsafe
        };

        Some(SafetyVerificationResult {
            proposal_id: proposal.id,
            safety,
            proof_steps: steps,
            confidence,
            reasoning: reasoning.trim().to_string(),
        })
    }

    pub fn apply_modification(&mut self, proposal_id: u64, actual_impact: Option<f64>) -> bool {
        let pos = self.proposals.iter().position(|p| p.id == proposal_id);
        match pos {
            None => return false,
            Some(idx) => {
                let proposal = self.proposals[idx].clone();
                let verification = match self.run_verification(&proposal) {
                    Some(v) => v,
                    None => return false,
                };

                let allowed = match verification.safety {
                    SafetyLevel::Safe => true,
                    SafetyLevel::Questionable => {
                        verification.confidence > self.config.confidence_threshold
                    }
                    _ => false,
                };

                if !allowed {
                    return false;
                }

                let proof = SelfModificationProof {
                    proposal,
                    verification,
                    applied: true,
                    actual_impact,
                    timestamp: self.step,
                };

                if self.proof_history.len() >= self.config.history_capacity {
                    self.proof_history.pop_front();
                }
                self.proof_history.push_back(proof);
                self.step += 1;
                true
            }
        }
    }

    pub fn learn_from_outcome(&mut self, proof_id: u64, actual_impact: f64) {
        if let Some(proof) = self.proof_history.iter_mut().find(|p| {
            let vid = p.verification.proposal_id;
            vid == proof_id
        }) {
            proof.actual_impact = Some(actual_impact);
        }
    }

    pub fn safe_modifications(&self) -> Vec<&SelfModificationProof> {
        self.proof_history
            .iter()
            .filter(|p| p.applied && p.verification.safety == SafetyLevel::Safe)
            .collect()
    }

    pub fn rejection_rate(&self) -> f64 {
        if self.proposals.is_empty() {
            return 0.0;
        }
        let verified: Vec<&ModificationProposal> = self
            .proposals
            .iter()
            .filter(|p| {
                self.run_verification(p).map_or(false, |v| {
                    matches!(v.safety, SafetyLevel::Unsafe | SafetyLevel::Unprovable)
                })
            })
            .collect();
        verified.len() as f64 / self.proposals.len() as f64
    }

    pub fn stats(&self) -> (usize, usize, f64, f64) {
        let total = self.proposals.len();
        let applied = self.proof_history.iter().filter(|p| p.applied).count();
        let rate = self.rejection_rate();
        let avg_conf = if self.proof_history.is_empty() {
            0.0
        } else {
            let sum: f64 = self
                .proof_history
                .iter()
                .map(|p| p.verification.confidence)
                .sum();
            sum / self.proof_history.len() as f64
        };
        (total, applied, rate, avg_conf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn safe_proposal() -> (Vec<u8>, Vec<String>) {
        let vec = vec![1u8; 128];
        let pre = vec![
            "integrity_check".to_string(),
            "impact_bounded".to_string(),
            "change_small".to_string(),
        ];
        (vec, pre)
    }

    #[test]
    fn test_initial_state() {
        let cfg = ProofSearchConfig::default();
        let pss = ProofSearchSelfModification::new(cfg);
        assert_eq!(pss.proposals.len(), 0);
        assert_eq!(pss.proof_history.len(), 0);
    }

    #[test]
    fn test_propose_modification() {
        let cfg = ProofSearchConfig::default();
        let mut pss = ProofSearchSelfModification::new(cfg);
        let (vec, pre) = safe_proposal();
        let id = pss.propose_modification("test_target", "test desc", vec, 0.5, pre);
        assert_eq!(id, 0);
    }

    #[test]
    fn test_verify_safe_modification() {
        let cfg = ProofSearchConfig::default();
        let pss = ProofSearchSelfModification::new(cfg);
        let proposal = ModificationProposal {
            id: 1,
            target: "some_module".to_string(),
            change_description: "safe change".to_string(),
            change_vector: vec![1u8; 128],
            expected_impact: 0.3,
            preconditions: vec![],
        };
        let result = pss.run_verification(&proposal).unwrap();
        assert_eq!(result.safety, SafetyLevel::Safe);
        assert!((result.confidence - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_verify_unsafe_modification() {
        let cfg = ProofSearchConfig::default();
        let pss = ProofSearchSelfModification::new(cfg);
        let proposal = ModificationProposal {
            id: 2,
            target: "proof_search".to_string(),
            change_description: "self modify".to_string(),
            change_vector: vec![1u8; 128],
            expected_impact: 0.3,
            preconditions: vec![],
        };
        let result = pss.run_verification(&proposal).unwrap();
        assert_eq!(result.safety, SafetyLevel::Unsafe);
        assert!(result.confidence < 1.0);
    }

    #[test]
    fn test_verify_empty_change() {
        let cfg = ProofSearchConfig::default();
        let pss = ProofSearchSelfModification::new(cfg);
        let proposal = ModificationProposal {
            id: 3,
            target: "some_module".to_string(),
            change_description: "empty change".to_string(),
            change_vector: vec![0u8; 128],
            expected_impact: 0.3,
            preconditions: vec![],
        };
        let result = pss.run_verification(&proposal).unwrap();
        assert_eq!(result.safety, SafetyLevel::Unsafe);
    }

    #[test]
    fn test_apply_safe_modification() {
        let mut cfg = ProofSearchConfig::default();
        cfg.confidence_threshold = 0.7;
        let mut pss = ProofSearchSelfModification::new(cfg);
        let (vec, pre) = safe_proposal();
        let id = pss.propose_modification("some_module", "safe change", vec, 0.3, pre);
        let applied = pss.apply_modification(id, Some(0.25));
        assert!(applied);
    }

    #[test]
    fn test_apply_unsafe_rejected() {
        let cfg = ProofSearchConfig::default();
        let mut pss = ProofSearchSelfModification::new(cfg);
        let (vec, pre) = safe_proposal();
        let id = pss.propose_modification("proof_search", "self mod", vec, 0.3, pre);
        let applied = pss.apply_modification(id, None);
        assert!(!applied);
    }

    #[test]
    fn test_rejection_rate() {
        let mut pss = ProofSearchSelfModification::new(ProofSearchConfig::default());
        let (vec1, pre1) = safe_proposal();
        pss.propose_modification("target_a", "safe", vec1, 0.3, pre1);
        let (vec2, pre2) = safe_proposal();
        pss.propose_modification("proof_search", "bad", vec2, 0.3, pre2);
        let (vec3, pre3) = safe_proposal();
        pss.propose_modification("target_c", "safe", vec3, 0.3, pre3);
        let (vec4, pre4) = safe_proposal();
        pss.propose_modification("authority", "critical", vec4, 0.3, pre4);
        let rate = pss.rejection_rate();
        assert!((rate - 0.5).abs() < 1e-6, "expected 0.5 got {}", rate);
    }

    #[test]
    fn test_stats() {
        let mut cfg = ProofSearchConfig::default();
        cfg.confidence_threshold = 0.7;
        let mut pss = ProofSearchSelfModification::new(cfg);
        let (v1, p1) = safe_proposal();
        let id = pss.propose_modification("some_module", "change", v1, 0.3, p1);
        pss.apply_modification(id, Some(0.25));
        let (total, applied, _rate, avg_conf) = pss.stats();
        assert_eq!(total, 1);
        assert_eq!(applied, 1);
        assert!(avg_conf > 0.0);
    }
}
