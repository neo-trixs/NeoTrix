use crate::core::l7_capability::registry::{
    Capability, MaturityLevel,
};
use crate::core::l7_capability::SkillBank;

#[derive(Debug, Clone)]
pub struct MaturityFeedback {
    pub prm_score: f64,
    pub success: bool,
    pub latency_ms: f64,
    pub diversity: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvolveResult {
    Promoted(MaturityLevel),
    Stable,
    Stagnated,
}

pub struct MaturityEngine {
    pub promote_candidate_threshold: u64,
    pub promote_reviewed_threshold: u64,
    pub promote_validated_threshold: u64,
    pub promote_groundtruth_threshold: u64,
    pub min_prm_candidate: f64,
    pub min_prm_reviewed: f64,
    pub min_prm_validated: f64,
    pub min_prm_groundtruth: f64,
    pub min_diversity: f64,
}

impl Default for MaturityEngine {
    fn default() -> Self {
        Self {
            promote_candidate_threshold: 3,
            promote_reviewed_threshold: 10,
            promote_validated_threshold: 100,
            promote_groundtruth_threshold: 1000,
            min_prm_candidate: 0.6,
            min_prm_reviewed: 0.7,
            min_prm_validated: 0.85,
            min_prm_groundtruth: 0.95,
            min_diversity: 0.3,
        }
    }
}

impl MaturityEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn evaluate(
        &self,
        cap: &Capability,
        _feedback: &MaturityFeedback,
    ) -> EvolveResult {
        let stats = &cap.stats;
        let m = cap.maturity;

        match m {
            MaturityLevel::Primitive => {
                if stats.call_count >= 1 {
                    return EvolveResult::Promoted(MaturityLevel::Candidate);
                }
            }
            MaturityLevel::Candidate => {
                if stats.call_count >= self.promote_candidate_threshold
                    && stats.avg_prm_score >= self.min_prm_candidate
                {
                    return EvolveResult::Promoted(MaturityLevel::Reviewed);
                }
            }
            MaturityLevel::Reviewed => {
                if stats.call_count >= self.promote_reviewed_threshold
                    && stats.avg_prm_score >= self.min_prm_reviewed
                    && stats.diversity_score >= self.min_diversity
                {
                    return EvolveResult::Promoted(MaturityLevel::Validated);
                }
            }
            MaturityLevel::Validated => {
                if stats.call_count >= self.promote_validated_threshold
                    && stats.avg_prm_score >= self.min_prm_validated
                    && stats.diversity_score >= self.min_diversity * 2.0
                {
                    return EvolveResult::Promoted(MaturityLevel::GroundTruth);
                }
            }
            MaturityLevel::GroundTruth => {
                if stats.call_count >= self.promote_groundtruth_threshold
                    && stats.avg_prm_score >= self.min_prm_groundtruth
                    && stats.diversity_score >= self.min_diversity * 3.0
                {
                    return EvolveResult::Promoted(MaturityLevel::Transcendent);
                }
            }
            MaturityLevel::Transcendent => {
                return EvolveResult::Stable;
            }
        }

        if stats.failure_count > stats.success_count * 2 && stats.call_count > 5 {
            EvolveResult::Stagnated
        } else {
            EvolveResult::Stable
        }
    }

    pub fn apply_feedback(
        &mut self,
        cap: &mut Capability,
        feedback: &MaturityFeedback,
    ) -> EvolveResult {
        cap.stats.record_call(feedback.success, feedback.latency_ms, feedback.prm_score);
        let result = self.evaluate(cap, feedback);
        if let EvolveResult::Promoted(new_level) = result {
            cap.maturity = new_level;
        }
        result
    }

    /// Skill-enhanced evaluate: leverages SkillBank to inform maturity decisions.
    /// If the capability has matching skill records, the success rate from skills
    /// is blended into the maturity score, accelerating promotions for well-skilled caps.
    pub fn evaluate_with_skill_bank(
        &self,
        cap: &Capability,
        feedback: &MaturityFeedback,
        skill_bank: &SkillBank,
    ) -> EvolveResult {
        let domain = &cap.name;
        let domain_skills = skill_bank.list(domain);
        if domain_skills.is_empty() {
            return self.evaluate(cap, feedback);
        }
        let avg_skill_rate: f64 = domain_skills
            .iter()
            .map(|s| s.metrics.success_rate)
            .sum::<f64>()
            / domain_skills.len() as f64;
        let skill_boost = avg_skill_rate * 0.2;
        let mut boosted = feedback.clone();
        boosted.prm_score = (boosted.prm_score + skill_boost).max(0.0).min(1.0);
        self.evaluate(cap, &boosted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::l7_capability::registry::{
        capability_id_from_name, CapabilityCost, CapabilityKind,
        CapabilityStats, CapabilityVector, CapabilityTier, CapabilityRuntime,
        CapabilityStability, DomainCategory,
    };

    fn test_cap(maturity: MaturityLevel) -> Capability {
        Capability {
            id: capability_id_from_name("mature_test"),
            name: "mature_test".to_string(),
            tags: vec!["test".to_string()],
            kind: CapabilityKind::Cognitive,
            maturity,
            vector: CapabilityVector::default(),
            e8_triggers: vec![],
            context_requirements: vec![],
            cost: CapabilityCost::default(),
            stats: CapabilityStats::default(),
            version: "0.1.0".to_string(),
            layer: 4,
            tier: CapabilityTier::Core, runtime: CapabilityRuntime::Local,
            stability: CapabilityStability::Production,
            fallback_chain: vec![], provider: None,
            domain: DomainCategory::General,
            input_schema: None, output_schema: None,
            resource_cpu: 1.0, resource_ram_mb: 64.0, resource_vram_mb: 0.0,
            dependencies: vec![],
        }
    }

    #[test]
    fn test_primitive_to_candidate() {
        let engine = MaturityEngine::default();
        let mut cap = test_cap(MaturityLevel::Primitive);
        cap.stats.call_count = 1;
        let feedback = MaturityFeedback {
            prm_score: 0.5, success: true, latency_ms: 100.0, diversity: 0.0,
        };
        let result = engine.evaluate(&cap, &feedback);
        assert_eq!(result, EvolveResult::Promoted(MaturityLevel::Candidate));
    }

    #[test]
    fn test_candidate_to_reviewed() {
        let engine = MaturityEngine::default();
        let mut cap = test_cap(MaturityLevel::Candidate);
        for _ in 0..5 {
            cap.stats.record_call(true, 100.0, 0.7);
        }
        let feedback = MaturityFeedback {
            prm_score: 0.7, success: true, latency_ms: 100.0, diversity: 0.4,
        };
        let result = engine.evaluate(&cap, &feedback);
        assert_eq!(result, EvolveResult::Promoted(MaturityLevel::Reviewed));
    }

    #[test]
    fn test_not_enough_calls() {
        let engine = MaturityEngine::default();
        let mut cap = test_cap(MaturityLevel::Candidate);
        cap.stats.record_call(true, 100.0, 0.7);
        let feedback = MaturityFeedback {
            prm_score: 0.7, success: true, latency_ms: 100.0, diversity: 0.0,
        };
        let result = engine.evaluate(&cap, &feedback);
        assert_eq!(result, EvolveResult::Stable);
    }

    #[test]
    fn test_stagnated() {
        let engine = MaturityEngine::default();
        let mut cap = test_cap(MaturityLevel::Reviewed);
        for _ in 0..10 {
            cap.stats.record_call(false, 100.0, 0.3);
        }
        let feedback = MaturityFeedback {
            prm_score: 0.3, success: false, latency_ms: 100.0, diversity: 0.0,
        };
        let result = engine.evaluate(&cap, &feedback);
        assert_eq!(result, EvolveResult::Stagnated);
    }

    #[test]
    fn test_transcendent_stable() {
        let engine = MaturityEngine::default();
        let cap = test_cap(MaturityLevel::Transcendent);
        let feedback = MaturityFeedback {
            prm_score: 1.0, success: true, latency_ms: 10.0, diversity: 1.0,
        };
        let result = engine.evaluate(&cap, &feedback);
        assert_eq!(result, EvolveResult::Stable);
    }

    #[test]
    fn test_apply_feedback_updates_stats_and_maturity() {
        let mut engine = MaturityEngine::default();
        let mut cap = test_cap(MaturityLevel::Primitive);
        cap.stats.call_count = 1;
        let feedback = MaturityFeedback {
            prm_score: 0.8, success: true, latency_ms: 50.0, diversity: 0.5,
        };
        engine.apply_feedback(&mut cap, &feedback);
        assert_eq!(cap.maturity, MaturityLevel::Candidate);
        assert_eq!(cap.stats.call_count, 2);
        assert_eq!(cap.stats.success_count, 1);
    }
}
