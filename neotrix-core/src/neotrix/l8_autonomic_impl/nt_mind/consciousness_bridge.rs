use std::sync::Arc;
use crate::core::nt_core_gwt::module_def::{SpecialistModule, SpecialistType};
use crate::core::nt_core_gwt::workspace::GlobalWorkspace;
use crate::core::nt_core_self::AttentionDomain;
use crate::neotrix::nt_memory_kb::KnowledgeBase;
use crate::neotrix::nt_world_model::TaskType;
use super::self_iterating::SelfIteratingBrain;

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
    }
}

pub struct ConsciousnessBridge {
    pub poll_interval: u64,
    pub iterations_since_last_poll: u64,
    pub last_broadcast: Option<String>,
    pub kb: Option<Arc<KnowledgeBase>>,
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

    pub fn attach_kb(&mut self, kb: Arc<KnowledgeBase>) {
        self.kb = Some(kb);
    }

    /// Extract task context from brain, register as a GWT specialist module
    pub fn from_seal(brain: &SelfIteratingBrain, gwt: &mut GlobalWorkspace) {
        let task_type = brain._current_task_type();
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

        let winner_module = active
            .iter()
            .max_by(|a, b| a.activation.partial_cmp(&b.activation).unwrap_or(std::cmp::Ordering::Equal));

        if let Some(winner) = winner_module {
            let boost = (winner.activation * 0.02).min(0.05);
            match winner.specialist_type {
                SpecialistType::CreativityGenerator | SpecialistType::KnowledgeIntegrator => {
                    let current = brain.brain.capability.creativity();
                    brain.brain.capability.set_creativity((current + boost).min(1.0));
                }
                SpecialistType::CodeAnalyzer | SpecialistType::Planner => {
                    let current = brain.brain.capability.analysis();
                    brain.brain.capability.set_analysis((current + boost).min(1.0));
                }
                SpecialistType::RiskAssessor | SpecialistType::GoalPrioritizer => {
                    let current = brain.brain.capability.quality_gates();
                    brain.brain.capability.set_quality_gates((current + boost).min(1.0));
                }
                SpecialistType::ReflectionEngine => {
                    let current = brain.brain.capability.inference_depth();
                    brain.brain.capability.set_inference_depth((current + boost).min(1.0));
                }
                SpecialistType::KnowledgeRetriever => {
                    let current = brain.brain.capability.domain_specificity();
                    brain.brain.capability.set_domain_specificity((current + boost).min(1.0));
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
            // Flat search (existing)
            if let Ok(results) = kb.search(&content, 3) {
                for r in &results {
                    gwt.broadcast(&format!("KB: {} (score: {:.2})", r.node.title, r.score));
                }
            }
            // Hierarchical search — adds cluster-aware deduplication + provenance
            if let Ok(hier_results) = kb.search_hierarchical(&content, 3) {
                for r in &hier_results {
                    let cluster_info = if r.from_aggregate {
                        format!(" [cluster: {}]", r.hierarchy_path.join(", "))
                    } else {
                        String::new()
                    };
                    gwt.broadcast(&format!(
                        "KB-H: {} (score: {:.2}, redun: {:.2}){}",
                        r.node.title, r.score, r.redundancy_score, cluster_info
                    ));
                }
            }
            // E8 模式推荐: 从 KB 获取当前上下文的知识建议
            let e8_terms = ["abstract", "concrete", "analytical", "generative", "deep", "fast"];
            for term in &e8_terms {
                if content.contains(term) {
                    if let Ok(results) = kb.recommend_for_e8_mode(term, 2) {
                        for r in &results {
                            gwt.broadcast(&format!("E8: {} → {}", term, r.node.title));
                        }
                    }
                }
            }
            // Specialist-specific knowledge injection
            let specialists: Vec<(String, SpecialistType)> = gwt.active_specialists()
                .into_iter().map(|m| (m.name.clone(), m.specialist_type)).collect();
            for (name, st) in &specialists {
                if let Ok(spec_results) = kb.query_by_specialist(st, 2) {
                    for r in &spec_results {
                        gwt.broadcast(&format!("SPEC {} → {}", name, r.node.title));
                    }
                }
            }
        }
    }

    /// 记录当前意识状态到 KB (使用真实 GWT 数据而非硬编码 0.0)
    fn log_consciousness_snapshot(&self, gwt: &GlobalWorkspace) {
        if let Some(ref kb) = self.kb {
            let active_count = gwt.active_specialists().len();
            let content = gwt.active_content.as_deref().unwrap_or("");
            let specialists: Vec<String> = gwt.active_specialists()
                .iter().map(|m| m.name.clone()).collect();
            // Compute phi from geometry_sync if available, else resonance entropy
            let phi = gwt.geometry_sync.as_ref()
                .map(|gs| gs.current_phi().iit_phi)
                .unwrap_or_else(|| {
                    gwt.last_resonance.as_ref().map(|r| {
                        (1.0 - r.entropy / 11.0_f64.max(1.0)).max(0.0).min(1.0)
                    }).unwrap_or(0.0)
                });
            let coherence = active_count as f64 / gwt.specialists.len().max(1) as f64
                * gwt.active_specialists().iter()
                    .map(|m| m.activation)
                    .sum::<f64>().max(0.0);
            let details = format!("GWT broadcast: {}, specialists: {:?} (phi={:.3}, coherence={:.3})",
                content, specialists, phi, coherence);
            let _ = kb.record_consciousness_snapshot(phi, coherence, active_count > 0, "bridge_cycle", &details);
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
            brain._current_task_type(),
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

impl crate::core::nt_core_self_test::SelfTest for ConsciousnessBridge {
    fn name(&self) -> &str { "consciousness_bridge" }
    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();
        if self.poll_interval == 0 {
            failures.push("consciousness_bridge: poll_interval must be > 0".into());
        }
        if failures.is_empty() { Ok(()) } else { Err(failures) }
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
        let mapped: HashSet<SpecialistType> = domains.iter()
            .map(|d| attention_to_specialist(d))
            .collect();
        // Every domain maps to a unique specialist type (no collisions)
        assert_eq!(mapped.len(), 10, "each domain must map to a unique specialist");
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
        assert_eq!(cb.iterations_since_last_poll, 1, "first poll should increment to 1 (below poll_interval=2)");
        cb.maybe_poll(&mut brain, &mut gwt);
        assert_eq!(cb.iterations_since_last_poll, 0, "second poll should trigger at interval=2 and reset");
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
