use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;

#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum FailureType {
    DecisionDeadlock,
    ConflictingGoals,
    InsufficientContext,
    ReasoningError,
    ResourceExhaustion,
    KnowledgeGap,
    RepeatedFailure,
}

#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum RepairMode {
    AdjustParameters,
    ChangeStrategy,
    FallbackBehavior,
    RequestHelp,
    AcquireKnowledge,
    ComposeSkills,
}

#[derive(Debug, Clone)]
pub struct FailurePattern {
    pub id: u64,
    pub symptom: Vec<u8>,
    pub failure_type: FailureType,
    pub frequency: u64,
    pub severity: f64,
    pub first_seen: u64,
    pub last_seen: u64,
    pub context_description: String,
}

#[derive(Debug, Clone)]
pub struct RepairPolicy {
    pub id: u64,
    pub target_pattern_id: u64,
    pub intervention: Vec<u8>,
    pub mode: RepairMode,
    pub confidence: f64,
    pub applied_count: u64,
    pub success_count: u64,
    pub description: String,
}

pub struct PolicyRepairEngine {
    failure_patterns: Vec<FailurePattern>,
    repair_policies: Vec<RepairPolicy>,
    next_id: u64,
    cycle: u64,
    max_patterns: usize,
    similarity_threshold: f64,
}

impl PolicyRepairEngine {
    pub fn new(max_patterns: usize) -> Self {
        Self {
            failure_patterns: Vec::with_capacity(max_patterns),
            repair_policies: Vec::new(),
            next_id: 1,
            cycle: 0,
            max_patterns,
            similarity_threshold: 0.7,
        }
    }

    pub fn detect_failure(
        &mut self,
        context_str: &str,
        failure_type: FailureType,
        severity: f64,
    ) -> u64 {
        self.cycle += 1;
        let symptom = QuantizedVSA::seeded_random(self.stable_hash(context_str), 4096);

        if let Some(existing) = self.find_similar_pattern(&symptom, self.similarity_threshold) {
            self.failure_patterns[existing].frequency += 1;
            self.failure_patterns[existing].severity =
                self.failure_patterns[existing].severity * 0.7 + severity * 0.3;
            self.failure_patterns[existing].last_seen = self.cycle;
            if self.failure_patterns[existing].frequency >= 3 {
                self.generate_repair(existing);
            }
            return self.failure_patterns[existing].id;
        }

        if self.failure_patterns.len() >= self.max_patterns {
            self.failure_patterns.sort_by(|a, b| {
                let sa = a.severity * (a.frequency as f64);
                let sb = b.severity * (b.frequency as f64);
                sa.partial_cmp(&sb).unwrap_or(std::cmp::Ordering::Equal)
            });
            self.failure_patterns.truncate(self.max_patterns / 2);
        }

        let id = self.next_id;
        self.next_id += 1;
        self.failure_patterns.push(FailurePattern {
            id,
            symptom,
            failure_type,
            frequency: 1,
            severity,
            first_seen: self.cycle,
            last_seen: self.cycle,
            context_description: context_str.to_string(),
        });
        id
    }

    fn find_similar_pattern(&self, symptom: &[u8], threshold: f64) -> Option<usize> {
        self.failure_patterns
            .iter()
            .enumerate()
            .filter(|(_, p)| QuantizedVSA::similarity(&p.symptom, symptom) >= threshold)
            .max_by(|a, b| {
                let sa = QuantizedVSA::similarity(&a.1.symptom, symptom);
                let sb = QuantizedVSA::similarity(&b.1.symptom, symptom);
                sa.partial_cmp(&sb).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(i, _)| i)
    }

    fn generate_repair(&mut self, pattern_idx: usize) -> u64 {
        let pattern = &self.failure_patterns[pattern_idx];
        if self
            .repair_policies
            .iter()
            .any(|r| r.target_pattern_id == pattern.id)
        {
            return 0;
        }

        let mode = match pattern.failure_type {
            FailureType::DecisionDeadlock => RepairMode::ChangeStrategy,
            FailureType::ConflictingGoals => RepairMode::AdjustParameters,
            FailureType::InsufficientContext => RepairMode::AcquireKnowledge,
            FailureType::ReasoningError => RepairMode::ChangeStrategy,
            FailureType::ResourceExhaustion => RepairMode::FallbackBehavior,
            FailureType::KnowledgeGap => RepairMode::AcquireKnowledge,
            FailureType::RepeatedFailure => RepairMode::ComposeSkills,
        };

        let intervention = QuantizedVSA::seeded_random(
            self.stable_hash(&format!("{:?}_{}", mode, pattern.context_description)),
            4096,
        );

        let id = self.next_id;
        self.next_id += 1;
        self.repair_policies.push(RepairPolicy {
            id,
            target_pattern_id: pattern.id,
            intervention,
            mode: mode.clone(),
            confidence: 0.4,
            applied_count: 0,
            success_count: 0,
            description: format!("{:?} for {:?}", mode, pattern.failure_type),
        });
        id
    }

    pub fn find_applicable_repairs(&self, context_vec: &[u8]) -> Vec<&RepairPolicy> {
        let mut scored: Vec<(f64, &RepairPolicy)> = Vec::new();
        for policy in &self.repair_policies {
            if let Some(pattern) = self
                .failure_patterns
                .iter()
                .find(|p| p.id == policy.target_pattern_id)
            {
                let sim = QuantizedVSA::similarity(context_vec, &pattern.symptom);
                let score = sim * policy.confidence;
                if score > 0.3 {
                    scored.push((score, policy));
                }
            }
        }
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored.into_iter().take(5).map(|(_, p)| p).collect()
    }

    pub fn apply_repair(&mut self, policy_id: u64, success: bool) {
        if let Some(policy) = self.repair_policies.iter_mut().find(|p| p.id == policy_id) {
            policy.applied_count += 1;
            if success {
                policy.success_count += 1;
            }
            policy.confidence = policy.success_count as f64 / policy.applied_count as f64;
        }
    }

    pub fn transfer_repair(
        &mut self,
        source_pattern_id: u64,
        target_context: &str,
        target_failure_type: FailureType,
    ) -> Option<u64> {
        let source_policy = self
            .repair_policies
            .iter()
            .find(|p| p.target_pattern_id == source_pattern_id)?;

        let target_symptom = QuantizedVSA::seeded_random(self.stable_hash(target_context), 4096);

        if let Some(pattern_idx) =
            self.find_similar_pattern(&target_symptom, self.similarity_threshold)
        {
            if self
                .repair_policies
                .iter()
                .any(|r| r.target_pattern_id == self.failure_patterns[pattern_idx].id)
            {
                return None;
            }
        }

        if self.failure_patterns.len() >= self.max_patterns {
            return None;
        }

        let pattern_id = self.next_id;
        self.next_id += 1;
        self.failure_patterns.push(FailurePattern {
            id: pattern_id,
            symptom: target_symptom,
            failure_type: target_failure_type,
            frequency: 1,
            severity: 0.5,
            first_seen: self.cycle,
            last_seen: self.cycle,
            context_description: target_context.to_string(),
        });

        let policy_id = self.next_id;
        self.next_id += 1;
        self.repair_policies.push(RepairPolicy {
            id: policy_id,
            target_pattern_id: pattern_id,
            intervention: source_policy.intervention.clone(),
            mode: source_policy.mode.clone(),
            confidence: source_policy.confidence * 0.7,
            applied_count: 0,
            success_count: 0,
            description: format!("transfer({})", source_policy.description),
        });
        Some(policy_id)
    }

    pub fn most_common_failures(&self, top_k: usize) -> Vec<&FailurePattern> {
        let mut sorted: Vec<&FailurePattern> = self.failure_patterns.iter().collect();
        sorted.sort_by(|a, b| {
            let sa = a.severity * a.frequency as f64;
            let sb = b.severity * b.frequency as f64;
            sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
        });
        sorted.truncate(top_k);
        sorted
    }

    pub fn pattern_count(&self) -> usize {
        self.failure_patterns.len()
    }

    pub fn policy_count(&self) -> usize {
        self.repair_policies.len()
    }

    pub fn best_policies(&self, top_k: usize) -> Vec<&RepairPolicy> {
        let mut sorted: Vec<&RepairPolicy> = self.repair_policies.iter().collect();
        sorted.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        sorted.truncate(top_k);
        sorted
    }

    fn stable_hash(&self, s: &str) -> u64 {
        let mut h: u64 = 0xdead_beef_cafe_babe_u64;
        for b in s.bytes() {
            h = h.wrapping_mul(0x9e3779b97f4a7c15u64);
            h ^= b as u64;
            h = h.rotate_left(23);
        }
        h
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_engine() -> PolicyRepairEngine {
        PolicyRepairEngine::new(50)
    }

    #[test]
    fn test_detect_failure_creates_pattern() {
        let mut eng = make_engine();
        let id = eng.detect_failure("deadlock in loop", FailureType::DecisionDeadlock, 0.8);
        assert!(id > 0);
        assert_eq!(eng.pattern_count(), 1);
        let p = &eng.failure_patterns[0];
        assert_eq!(p.failure_type, FailureType::DecisionDeadlock);
        assert_eq!(p.frequency, 1);
    }

    #[test]
    fn test_detect_failure_all_six_types() {
        let types = vec![
            FailureType::DecisionDeadlock,
            FailureType::ConflictingGoals,
            FailureType::InsufficientContext,
            FailureType::ReasoningError,
            FailureType::ResourceExhaustion,
            FailureType::KnowledgeGap,
        ];
        let mut eng = make_engine();
        for (i, ft) in types.iter().enumerate() {
            let ctx = format!("failure_type_{}", i);
            let id = eng.detect_failure(&ctx, ft.clone(), 0.5 + i as f64 * 0.05);
            assert!(id > 0);
        }
        assert_eq!(eng.pattern_count(), 6);
    }

    #[test]
    fn test_detect_failure_increments_frequency() {
        let mut eng = make_engine();
        eng.detect_failure("same_issue", FailureType::ReasoningError, 0.7);
        eng.detect_failure("same_issue", FailureType::ReasoningError, 0.7);
        assert_eq!(eng.failure_patterns[0].frequency, 2);
    }

    #[test]
    fn test_frequent_failure_generates_repair() {
        let mut eng = make_engine();
        for _ in 0..3 {
            eng.detect_failure("recurring", FailureType::DecisionDeadlock, 0.9);
        }
        assert_eq!(
            eng.policy_count(),
            1,
            "3rd occurrence should generate repair"
        );
        let policy = &eng.repair_policies[0];
        assert_eq!(policy.mode, RepairMode::ChangeStrategy);
    }

    #[test]
    fn test_repair_mode_mapping() {
        let mut eng = make_engine();
        for (ft, expected_mode) in vec![
            (FailureType::DecisionDeadlock, RepairMode::ChangeStrategy),
            (FailureType::ConflictingGoals, RepairMode::AdjustParameters),
            (
                FailureType::InsufficientContext,
                RepairMode::AcquireKnowledge,
            ),
            (FailureType::ReasoningError, RepairMode::ChangeStrategy),
            (
                FailureType::ResourceExhaustion,
                RepairMode::FallbackBehavior,
            ),
            (FailureType::KnowledgeGap, RepairMode::AcquireKnowledge),
            (FailureType::RepeatedFailure, RepairMode::ComposeSkills),
        ] {
            let ctx = format!("{:?}_test", ft);
            for _ in 0..3 {
                eng.detect_failure(&ctx, ft.clone(), 0.8);
            }
            let policy = eng.repair_policies.iter().find(|p| {
                let pattern = &eng.failure_patterns[0];
                let ft_match = pattern.failure_type == ft;
                ft_match && p.target_pattern_id == pattern.id
            });
            if let Some(p) = policy {
                assert_eq!(p.mode, expected_mode);
            }
        }
    }

    #[test]
    fn test_find_applicable_repairs() {
        let mut eng = make_engine();
        for _ in 0..3 {
            eng.detect_failure("test_ctx", FailureType::ReasoningError, 0.8);
        }
        let symptom = QuantizedVSA::seeded_random(eng.stable_hash("test_ctx"), 4096);
        let repairs = eng.find_applicable_repairs(&symptom);
        assert!(!repairs.is_empty());
    }

    #[test]
    fn test_apply_repair_increases_confidence() {
        let mut eng = make_engine();
        for _ in 0..3 {
            eng.detect_failure("ctx", FailureType::ReasoningError, 0.8);
        }
        let policy_id = eng.repair_policies[0].id;
        eng.apply_repair(policy_id, true);
        eng.apply_repair(policy_id, true);
        let policy = eng
            .repair_policies
            .iter()
            .find(|p| p.id == policy_id)
            .unwrap();
        assert_eq!(policy.applied_count, 2);
        assert_eq!(policy.success_count, 2);
        assert!((policy.confidence - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_apply_repair_updates_on_failure() {
        let mut eng = make_engine();
        for _ in 0..3 {
            eng.detect_failure("ctx", FailureType::ReasoningError, 0.8);
        }
        let policy_id = eng.repair_policies[0].id;
        eng.apply_repair(policy_id, true);
        eng.apply_repair(policy_id, false);
        let policy = eng
            .repair_policies
            .iter()
            .find(|p| p.id == policy_id)
            .unwrap();
        assert!((policy.confidence - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_transfer_repair_creates_policy() {
        let mut eng = make_engine();
        for _ in 0..3 {
            eng.detect_failure("source_ctx", FailureType::KnowledgeGap, 0.9);
        }
        let source_id = eng.failure_patterns[0].id;
        let transferred =
            eng.transfer_repair(source_id, "target_ctx", FailureType::InsufficientContext);
        assert!(transferred.is_some());
        assert_eq!(eng.pattern_count(), 2);
        assert_eq!(eng.policy_count(), 2);
        let new_policy = eng
            .repair_policies
            .iter()
            .find(|p| p.id == transferred.unwrap())
            .unwrap();
        assert_eq!(new_policy.mode, RepairMode::AcquireKnowledge);
    }

    #[test]
    fn test_transfer_repair_returns_none_for_nonexistent_source() {
        let mut eng = make_engine();
        let result = eng.transfer_repair(999, "target", FailureType::KnowledgeGap);
        assert!(result.is_none());
    }

    #[test]
    fn test_most_common_failures_orders_by_severity_and_frequency() {
        let mut eng = make_engine();
        eng.detect_failure("minor", FailureType::ReasoningError, 0.3);
        eng.detect_failure("major", FailureType::DecisionDeadlock, 0.9);
        eng.detect_failure("major", FailureType::DecisionDeadlock, 0.9);
        let common = eng.most_common_failures(5);
        assert!(
            common[0].severity * common[0].frequency as f64
                >= common[1].severity * common[1].frequency as f64
        );
    }

    #[test]
    fn test_best_policies_orders_by_confidence() {
        let mut eng = make_engine();
        for _ in 0..3 {
            eng.detect_failure("ft1", FailureType::ReasoningError, 0.8);
        }
        for _ in 0..3 {
            eng.detect_failure("ft2", FailureType::DecisionDeadlock, 0.8);
        }
        eng.apply_repair(eng.repair_policies[0].id, true);
        eng.apply_repair(eng.repair_policies[1].id, false);
        let best = eng.best_policies(5);
        assert!(best[0].confidence >= best[1].confidence);
    }

    #[test]
    fn test_pattern_count_zero_initial() {
        let eng = make_engine();
        assert_eq!(eng.pattern_count(), 0);
        assert_eq!(eng.policy_count(), 0);
    }
}
