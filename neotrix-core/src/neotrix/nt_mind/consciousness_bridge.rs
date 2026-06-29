use super::self_iterating::SelfIteratingBrain;
use crate::core::nt_core_gwt::module_def::{SpecialistModule, SpecialistType};
use crate::core::nt_core_hcube::QuantizedVSA;
use crate::core::nt_core_self::AttentionDomain;
use crate::neotrix::nt_expert_routing::workspace::GlobalWorkspace;
use crate::neotrix::nt_expert_routing::TaskType;
use crate::neotrix::nt_memory_kb::KnowledgeBase;

const SEAL_POLL_INTERVAL: u64 = 5;

/// Map AttentionDomain → SpecialistType.
/// Each attention head domain maps to a corresponding GWT specialist.
/// AnomalyDetector is left unmapped — it is driven by the resonance cycle.
pub fn attention_to_specialist(domain: &AttentionDomain) -> SpecialistType {
    use AttentionDomain::*;
    match domain {
        PatternMatch => SpecialistType::PatternMatcher,
        Code => SpecialistType::CodeAnalyzer,
        Semantic => SpecialistType::KnowledgeIntegrator,
        Temporal => SpecialistType::MetaCognitionAnalyst,
        Planning => SpecialistType::Planner,
        SelfReflection => SpecialistType::ReflectionEngine,
        ToolUse => SpecialistType::KnowledgeRetriever,
        GoalAlignment => SpecialistType::GoalPrioritizer,
        RiskAssessment => SpecialistType::RiskAssessor,
        Creativity => SpecialistType::CreativityGenerator,
        Reasoning => SpecialistType::PatternMatcher,
        Memory => SpecialistType::KnowledgeIntegrator,
        Social => SpecialistType::KnowledgeIntegrator,
        Emotional => SpecialistType::CreativityGenerator,
    }
}

/// Map AttentionDomain → deterministic 512-bit VSA fingerprint.
/// Enables VSA-based similarity comparisons between attention domains.
pub fn specialist_vsa(domain: &AttentionDomain) -> [u8; 64] {
    let tag: &[u8] = match domain {
        AttentionDomain::PatternMatch => b"pattern_match",
        AttentionDomain::Code => b"code",
        AttentionDomain::Semantic => b"semantic",
        AttentionDomain::Temporal => b"temporal",
        AttentionDomain::Planning => b"planning",
        AttentionDomain::SelfReflection => b"self_reflection",
        AttentionDomain::ToolUse => b"tool_use",
        AttentionDomain::GoalAlignment => b"goal_alignment",
        AttentionDomain::RiskAssessment => b"risk_assessment",
        AttentionDomain::Creativity => b"creativity",
        AttentionDomain::Reasoning => b"reasoning",
        AttentionDomain::Memory => b"memory",
        AttentionDomain::Social => b"social",
        AttentionDomain::Emotional => b"emotional",
    };
    let seed = tag.iter().fold(0u64, |acc: u64, &b| {
        acc.wrapping_mul(31).wrapping_add(b as u64)
    });
    let vec = QuantizedVSA::seeded_random(seed, 64);
    vec.try_into()
        .expect("seeded_random(seed, 64) must return 64 bytes")
}

pub struct ConsciousnessBridge {
    pub poll_interval: u64,
    pub iterations_since_last_poll: u64,
    pub last_broadcast: Option<String>,
    pub kb: Option<KnowledgeBase>,
}

impl ConsciousnessBridge {
    pub fn new() -> Self {
        Self {
            poll_interval: SEAL_POLL_INTERVAL,
            iterations_since_last_poll: 0,
            last_broadcast: None,
            kb: None,
        }
    }

    pub fn attach_kb(&mut self, kb: KnowledgeBase) {
        self.kb = Some(kb);
    }

    /// Extract task context from brain, register as a GWT specialist module
    pub fn from_seal(brain: &SelfIteratingBrain, gwt: &mut GlobalWorkspace) {
        let task_type = brain.task_scratch.current_task_type;
        let task_name = format!("seal_task_{:?}", task_type);

        let specialist_type = match task_type {
            TaskType::General => SpecialistType::KnowledgeIntegrator,
            TaskType::Design | TaskType::UIDesign => SpecialistType::CreativityGenerator,
            TaskType::CodeAnalysis => SpecialistType::CodeAnalyzer,
            TaskType::CodeGeneration => SpecialistType::Planner,
            TaskType::CodeReview => SpecialistType::CodeAnalyzer,
            TaskType::Security => SpecialistType::RiskAssessor,
            TaskType::Planning => SpecialistType::GoalPrioritizer,
            TaskType::Reflection => SpecialistType::ReflectionEngine,
            TaskType::Research | TaskType::Learning => SpecialistType::KnowledgeRetriever,
            TaskType::Debugging => SpecialistType::MetaCognitionAnalyst,
        };

        let mut module = SpecialistModule::new(specialist_type, task_name);
        module.activate(brain.brain.evaluate_capability(task_type));
        gwt.register(module);
    }

    /// Apply GWT broadcast result back to brain (adjust capability vector)
    pub fn to_seal(gwt: &GlobalWorkspace, brain: &mut SelfIteratingBrain) {
        let active = gwt.active_specialists();
        if active.is_empty() {
            return;
        }

        let winner_module = active.iter().max_by(|a, b| {
            a.activation
                .partial_cmp(&b.activation)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        if let Some(winner) = winner_module {
            let boost = (winner.activation * 0.02).min(0.05);
            match winner.specialist_type {
                SpecialistType::CreativityGenerator | SpecialistType::KnowledgeIntegrator => {
                    let current = brain.brain.capability.creativity();
                    brain
                        .brain
                        .capability
                        .set_creativity((current + boost).min(1.0));
                }
                SpecialistType::CodeAnalyzer | SpecialistType::Planner => {
                    let current = brain.brain.capability.analysis();
                    brain
                        .brain
                        .capability
                        .set_analysis((current + boost).min(1.0));
                }
                SpecialistType::RiskAssessor | SpecialistType::GoalPrioritizer => {
                    let current = brain.brain.capability.quality_gates();
                    brain
                        .brain
                        .capability
                        .set_quality_gates((current + boost).min(1.0));
                }
                SpecialistType::ReflectionEngine => {
                    let current = brain.brain.capability.inference_depth();
                    brain
                        .brain
                        .capability
                        .set_inference_depth((current + boost).min(1.0));
                }
                SpecialistType::KnowledgeRetriever => {
                    let current = brain.brain.capability.domain_specificity();
                    brain
                        .brain
                        .capability
                        .set_domain_specificity((current + boost).min(1.0));
                }
                _ => {}
            }
            brain.brain.capability.normalize();
        }
    }

    /// 从 KB 查询广播内容的关联知识并注入 GWT
    fn inject_kb_knowledge(&self, gwt: &mut GlobalWorkspace) {
        let content = match &gwt.active_content {
            Some(c) => c.clone(),
            None => return,
        };
        if let Some(ref kb) = self.kb {
            if let Ok(results) = kb.search(&content, 3) {
                for r in &results {
                    gwt.broadcast(&format!("KB: {} (score: {:.2})", r.node.title, r.score));
                }
            }
            // E8 模式推荐: 从 KB 获取当前上下文的知识建议
            let e8_terms = [
                "abstract",
                "concrete",
                "analytical",
                "generative",
                "deep",
                "fast",
            ];
            for term in &e8_terms {
                if content.contains(term) {
                    if let Ok(results) = kb.recommend_for_e8_mode(term, 2) {
                        for r in &results {
                            gwt.broadcast(&format!("E8: {} → {}", term, r.node.title));
                        }
                    }
                }
            }
        }
    }

    /// 记录当前意识状态到 KB
    fn log_consciousness_snapshot(&self, gwt: &GlobalWorkspace) {
        if let Some(ref kb) = self.kb {
            let active_count = gwt.active_specialists().len();
            let content = gwt.active_content.as_deref().unwrap_or("");
            let details = format!("GWT broadcast: {}, specialists: {}", content, active_count);
            let _ = kb.record_consciousness_snapshot(
                0.0,
                0.0,
                active_count > 0,
                "bridge_cycle",
                &details,
            );
        }
    }

    /// Check if it's time to poll GWT, and apply bridge if so
    pub fn maybe_poll(&mut self, brain: &mut SelfIteratingBrain, gwt: &mut GlobalWorkspace) {
        self.iterations_since_last_poll += 1;
        if self.iterations_since_last_poll >= self.poll_interval {
            Self::from_seal(brain, gwt);
            Self::to_seal(gwt, brain);
            self.last_broadcast = gwt.active_content.clone();
            self.inject_kb_knowledge(gwt);
            self.log_consciousness_snapshot(gwt);
            gwt.decay_all(0.3);
            self.iterations_since_last_poll = 0;
        }
    }

    /// Run a full bridge cycle: brain → GWT → brain → KB
    pub fn bridge_cycle(&self, brain: &mut SelfIteratingBrain, gwt: &mut GlobalWorkspace) {
        Self::from_seal(brain, gwt);
        Self::to_seal(gwt, brain);
        let msg = format!(
            "consciousness bridge: task={:?}, active_specialists={}",
            brain.task_scratch.current_task_type,
            gwt.active_specialists().len(),
        );
        gwt.broadcast(&msg);
        self.inject_kb_knowledge(gwt);
        self.log_consciousness_snapshot(gwt);
        gwt.decay_all(0.3);
    }
}

impl Default for ConsciousnessBridge {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_attention_domain_mapping_all_10() {
        let domains = AttentionDomain::all();
        assert_eq!(domains.len(), 10);
        let mapped: HashSet<SpecialistType> =
            domains.iter().map(|d| attention_to_specialist(d)).collect();
        // Every domain maps to a unique specialist type (no collisions)
        assert_eq!(
            mapped.len(),
            10,
            "each domain must map to a unique specialist"
        );
        // AnomalyDetector should NOT be in the mapped set
        assert!(!mapped.contains(&SpecialistType::AnomalyDetector));
    }

    #[test]
    fn test_consciousness_bridge_new() {
        let cb = ConsciousnessBridge::new();
        assert_eq!(cb.poll_interval, 5);
        assert_eq!(cb.iterations_since_last_poll, 0);
        assert!(cb.last_broadcast.is_none());
    }

    #[test]
    fn test_from_seal_registers_module() {
        let brain = SelfIteratingBrain::new();
        let mut gwt = GlobalWorkspace::new(0.4);
        ConsciousnessBridge::from_seal(&brain, &mut gwt);
        assert_eq!(gwt.active_specialists().len(), 1);
    }

    #[test]
    fn test_to_seal_does_not_panic_on_empty_gwt() {
        let gwt = GlobalWorkspace::new(0.4);
        let mut brain = SelfIteratingBrain::new();
        ConsciousnessBridge::to_seal(&gwt, &mut brain);
        let cap = brain.brain.capability.clone();
        assert!(cap.arr().iter().any(|&v| v >= 0.0));
    }

    #[test]
    fn test_maybe_poll_after_threshold() {
        let mut brain = SelfIteratingBrain::new();
        let mut gwt = GlobalWorkspace::new(0.4);
        let mut cb = ConsciousnessBridge::new();
        cb.poll_interval = 2;

        assert_eq!(cb.iterations_since_last_poll, 0);
        cb.maybe_poll(&mut brain, &mut gwt);
        assert_eq!(
            cb.iterations_since_last_poll, 1,
            "first poll should increment to 1 (below poll_interval=2)"
        );
        cb.maybe_poll(&mut brain, &mut gwt);
        assert_eq!(
            cb.iterations_since_last_poll, 0,
            "second poll should trigger at interval=2 and reset"
        );
        assert!(cb.last_broadcast.is_some() || gwt.active_specialists().is_empty());
    }

    #[test]
    fn test_bridge_cycle_updates_broadcast() {
        let mut brain = SelfIteratingBrain::new();
        let mut gwt = GlobalWorkspace::new(0.4);
        let cb = ConsciousnessBridge::new();
        let history_before = gwt.broadcast_history.len();
        cb.bridge_cycle(&mut brain, &mut gwt);
        assert!(gwt.broadcast_history.len() > history_before);
    }
}
