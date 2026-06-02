use crate::core::nt_core_self::{
    SiliconSelfModel, SiliconSelfState, ThinkingTrace, ThinkingStep, StrategyKind,
    AttentionDomain, ReflectionGrade, SiliconArchive, IntrinsicMotivation,
    MotivationState, SelfReferentialMonitor,
    CognitiveEvaluator, CognitiveHealthReport, CrystalRegistry,
};
use crate::core::nt_core_gwt::workspace::GlobalWorkspace;
use super::hypercube_attention_bridge::{AttentionHypercubeBridge, AttentionRecallItem};
use crate::core::nt_core_hcube::cube::KnowledgeHyperCube;
use crate::core::nt_core_gwt::module_def::SpecialistModule;
use super::consciousness_bridge::attention_to_specialist;

pub struct ThinkingBridge {
    pub silicon: SiliconSelfModel,
    pub project_root: String,
    pub self_repair_count: usize,
    pub consecutive_poor_traces: usize,
    pub archive: SiliconArchive,
    pub intrinsic: IntrinsicMotivation,
    pub monitor: SelfReferentialMonitor,
    pub evaluator: CognitiveEvaluator,
    pub hypercube_attention: AttentionHypercubeBridge,
    pub skill_registry: CrystalRegistry,
    pub skill_auto_extract: bool,
}

impl ThinkingBridge {
    pub fn new(project_root: &str) -> Self {
        Self {
            silicon: SiliconSelfModel::new(),
            project_root: project_root.to_string(),
            self_repair_count: 0,
            consecutive_poor_traces: 0,
            archive: SiliconArchive::new(),
            intrinsic: IntrinsicMotivation::new(),
            monitor: SelfReferentialMonitor::new(),
            evaluator: CognitiveEvaluator::new(),
            hypercube_attention: AttentionHypercubeBridge::new(),
            skill_registry: CrystalRegistry::new(),
            skill_auto_extract: true,
        }
    }

    pub fn run_reflection_cycle(&mut self) -> ThinkingCycleResult {
        self.silicon.observe("reflection cycle start");
        let trace_id = self.silicon.begin_thinking_trace("self-reflection");
        self.silicon.add_thinking_step(trace_id,
            ThinkingStep::new(1, "scan attention profile", StrategyKind::Direct)
                .with_domain(AttentionDomain::SelfReflection));
        let state = self.silicon.current_state();
        self.silicon.add_thinking_step(trace_id,
            ThinkingStep::new(2, &format!("analyze state: dominant={:?}", state.attention_profile.dominant), StrategyKind::Reflection)
                .with_domain(AttentionDomain::SelfReflection));
        let grade = if state.context_usage > 0.8 { 0.6 } else { 0.9 };
        self.silicon.complete_thinking_trace(trace_id, "reflection complete", grade);
        ThinkingCycleResult {
            iteration: self.silicon.iteration,
            state,
            trace: self.silicon.thinking_traces.last().cloned(),
            summary: self.silicon.stats(),
        }
    }
    pub fn observe_task(&mut self, task: &str) -> usize { self.silicon.observe(&format!("task:{}", task)) }
    pub fn observe_tool_use(&mut self, tool: &str, result: &str) -> (usize, usize) { self.silicon.observe_tool_call(tool, result) }
    pub fn status_summary(&self) -> String { self.silicon.stats() }
    pub fn attention_profile_summary(&self) -> String {
        let profile = self.silicon.attention_manager.profile();
        let active_heads: Vec<String> = self.silicon.attention_manager.active_heads()
            .iter()
            .map(|h| format!("{}:{:.2}", h.domain.label(), h.activation))
            .collect();
        format!(
            "AttentionProfile: dominant={:?}, active_heads=[{}]",
            profile.dominant,
            active_heads.join(", "),
        )
    }

    pub fn reset(&mut self) { self.silicon.reset_session(); }
    pub fn auto_snapshot(&mut self) -> Option<usize> {
        if self.silicon.iteration > 0 && self.silicon.iteration % 5 == 0 {
            let id = self.archive.snapshot(
                &format!("auto-iter-{}", self.silicon.iteration),
                &self.silicon,
            );
            Some(id)
        } else {
            None
        }
    }

    pub fn check_self_repair_needed(&mut self) -> Option<String> {
        let recent = self.silicon.recent_traces(3);
        if recent.len() < 3 {
            return None;
        }

        let poor_count = recent.iter().filter(|t| {
            matches!(t.grade, ReflectionGrade::Adequate | ReflectionGrade::Poor | ReflectionGrade::Failed)
        }).count();

        if poor_count < 3 {
            self.consecutive_poor_traces = 0;
            return None;
        }

        self.consecutive_poor_traces += 1;

        Some(format!(
            "3 consecutive poor reflection grades detected (traces: {:?})",
            recent.iter().map(|t| (t.id, t.grade.label())).collect::<Vec<_>>()
        ))
    }

    pub fn trigger_self_repair(&mut self) -> String {
        self.silicon.reset_session();
        self.silicon.strategy_registry.reset_stats();
        let repaired = self.silicon.strategy_registry.strategies.get_mut(&StrategyKind::ChainOfThought);
        if let Some(s) = repaired {
            s.effectiveness = 0.7;
        }
        self.self_repair_count += 1;
        self.consecutive_poor_traces = 0;
        format!(
            "self-repair #{}: session reset, strategy stats reset, ChainOfThought boosted to 0.7",
            self.self_repair_count
        )
    }

    pub fn compute_motivation(&mut self) -> MotivationState {
        self.intrinsic.compute(&self.silicon)
    }

    pub fn evaluate_cognitive_health(&mut self) -> &CognitiveHealthReport {
        self.evaluator.evaluate(&self.silicon);
        if let Some(report) = self.evaluator.latest_report().cloned() {
            self.monitor.record_plan("cognitive-eval", report.stability_score);
            if self.monitor.auto_tune_enabled {
                let _ = self.monitor.auto_tune(&self.silicon);
            }
        }
        self.evaluator.latest_report().expect("result")
    }

    pub fn run_full_evolution_cycle(&mut self) -> String {
        self.run_reflection_cycle();
        let mot = self.compute_motivation();
        let stability_score = {
            let health = self.evaluate_cognitive_health();
            health.stability_score
        };
        let iteration = self.silicon.iteration;
        let snapshot_count = self.archive.snapshots.len();
        let repair = if self.check_self_repair_needed().is_some() {
            self.trigger_self_repair()
        } else {
            String::new()
        };
        let archive_msg = if self.auto_snapshot().is_some() {
            "snapshot taken"
        } else {
            "no snapshot"
        };

        format!(
            "EvoCycle #{} | R_Int={:.3} | stability={:.2} | explore={} | repair={} | {} | archive={} snapshots",
            iteration,
            mot.intrinsic_reward,
            stability_score,
            if mot.should_explore { "YES" } else { "no" },
            if repair.is_empty() { "none" } else { &repair[..repair.len().min(20)] },
            archive_msg,
            snapshot_count,
        )
    }

    pub fn recall_from_attention(&self, hypercube: Option<&KnowledgeHyperCube>) -> Vec<AttentionRecallItem> {
        let attention = &self.silicon.attention_manager;
        match hypercube {
            Some(cube) => self.hypercube_attention.recall_from_attention(attention, cube),
            None => Vec::new(),
        }
    }

    /// Wire attention heads into GlobalWorkspace: map domains→SpecialistType, register, broadcast
    pub fn broadcast_attention_to_gwt(&self, gwt: &mut GlobalWorkspace) -> Vec<(AttentionDomain, f64)> {
        let active = self.silicon.attention_manager.active_heads();
        let mut results = Vec::new();
        let novelty = 1.0;
        let coherence = 1.0;

        for head in &active {
            let specialist_type = attention_to_specialist(&head.domain);
            let name = format!("attn_{}", head.domain.label());
            let mut module = SpecialistModule::new(specialist_type, name);
            module.activate(head.salience(novelty, coherence));
            gwt.register(module);
            results.push((head.domain, head.activation));
        }

        if !active.is_empty() {
            let profile = self.silicon.attention_manager.profile();
            gwt.broadcast(&format!(
                "attention_broadcast: dominant={:?}, active_heads={}, distribution={:?}",
                profile.dominant,
                active.len(),
                profile.distribution,
            ));
        }

        results
    }

    pub fn auto_crystallize(&mut self) -> String {
        let iteration = self.silicon.iteration;
        let traces: Vec<ThinkingTrace> = self.silicon.recent_traces(10).iter()
            .map(|t| (*t).clone())
            .collect();
        let extracted = SkillBridge::extract_from_recent(&mut self.skill_registry, &traces, iteration);
        let maintain = SkillBridge::auto_maintain_registry(&mut self.skill_registry);
        format!("crystallized: {} new, {}", extracted, maintain)
    }

    pub fn evolution_summary(&self) -> String {
        let mot = if self.intrinsic.reward_history.is_empty() {
            "IntrinsicMotivation: not computed".to_string()
        } else {
            self.intrinsic.summary()
        };
        let health = self.evaluator.latest_report()
            .map(|r| format!("Stability={:.2} flags={} repairs={}",
                r.stability_score, r.flags.len(), r.repair_suggestions.len()))
            .unwrap_or_else(|| "No evaluation".to_string());
        format!("{}\n{}\n{}", mot, health, self.monitor.summary())
    }
}

/// Bridge: ThinkingTrace → CrystalRegistry (skill crystallization).
/// Extracts reusable skills from high-grade thinking traces.
pub struct SkillBridge;

impl SkillBridge {
    pub fn new() -> Self { Self }

    pub fn extract_from_recent(registry: &mut CrystalRegistry, traces: &[ThinkingTrace], iteration: usize) -> usize {
        let before = registry.crystals.len();
        for trace in traces {
            if trace.grade.score() >= 0.75 {
                registry.extract_from_trace(trace, iteration);
            }
        }
        registry.crystals.len() - before
    }

    pub fn auto_maintain_registry(registry: &mut CrystalRegistry) -> String {
        let mut parts = Vec::new();
        let pruned = registry.prune_weak(0.3);
        if pruned > 0 { parts.push(format!("pruned {} weak", pruned)); }
        if registry.crystals.len() > registry.max_crystals {
            let auto = registry.auto_prune();
            if auto > 0 { parts.push(format!("auto-pruned {} over max({})", auto, registry.max_crystals)); }
        }
        if parts.is_empty() { "no maintenance needed".to_string() } else { parts.join(", ") }
    }

    pub fn summary(registry: &CrystalRegistry) -> String { registry.summary() }

    pub fn recommend(registry: &CrystalRegistry, strategy: StrategyKind, domain: AttentionDomain) -> Option<String> {
        registry.find_similar(strategy, domain).map(|c| format!("SkillCrystal #{}: {} (eff={:.3}, used={}x)", c.id, c.pattern, c.effectiveness, c.use_count))
    }
}

impl Default for SkillBridge {
    fn default() -> Self { Self::new() }
}

#[derive(Debug, Clone)]
pub struct ThinkingCycleResult {
    pub iteration: usize,
    pub state: SiliconSelfState,
    pub trace: Option<ThinkingTrace>,
    pub summary: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_gwt::workspace::GlobalWorkspace;


    #[test] fn test_bridge_new() { let bridge = ThinkingBridge::new("/tmp"); assert_eq!(bridge.silicon.iteration, 0); assert!(bridge.status_summary().contains("SiliconSelf")); }
    #[test] fn test_reflection_cycle() { let mut bridge = ThinkingBridge::new("/tmp"); let result = bridge.run_reflection_cycle(); assert!(result.iteration > 0); assert!(result.trace.is_some()); assert!(result.summary.contains("SiliconSelf")); }
    #[test] fn test_observe_task() { let mut bridge = ThinkingBridge::new("/tmp"); let id = bridge.observe_task("analyze architecture"); assert!(id > 0 || id == 0); assert!(bridge.silicon.context_window.len() > 0); }
    #[test] fn test_observe_tool_use() { let mut bridge = ThinkingBridge::new("/tmp"); let (call, result) = bridge.observe_tool_use("grep", "found 3 matches"); assert!(call < result); }
    #[test] fn test_attention_profile() { let mut bridge = ThinkingBridge::new("/tmp"); bridge.run_reflection_cycle(); let profile = bridge.attention_profile_summary(); assert!(profile.contains("AttentionProfile")); }
    #[test] fn test_reset() { let mut bridge = ThinkingBridge::new("/tmp"); bridge.run_reflection_cycle(); assert!(bridge.silicon.iteration > 0); bridge.reset(); assert_eq!(bridge.silicon.attention_manager.active_heads().len(), 0); }
    #[test] fn test_multiple_reflection_cycles() { let mut bridge = ThinkingBridge::new("/tmp"); let r1 = bridge.run_reflection_cycle(); let r2 = bridge.run_reflection_cycle(); assert!(r2.iteration > r1.iteration); }
    #[test] fn test_self_repair_not_needed_initial() { let mut bridge = ThinkingBridge::new("/tmp"); assert!(bridge.check_self_repair_needed().is_none()); assert_eq!(bridge.consecutive_poor_traces, 0); }
    #[test] fn test_self_repair_triggered() { let mut bridge = ThinkingBridge::new("/tmp"); for _ in 0..3 { let id = bridge.silicon.thinking_traces.len(); bridge.silicon.add_thinking_trace(ThinkingTrace::new(id, "poor cycle")); bridge.silicon.complete_thinking_trace(id, "bad result", 0.3); } assert!(bridge.check_self_repair_needed().is_some()); }
    #[test] fn test_self_repair_not_needed_good_traces() { let mut bridge = ThinkingBridge::new("/tmp"); for _ in 0..3 { let id = bridge.silicon.thinking_traces.len(); bridge.silicon.add_thinking_trace(ThinkingTrace::new(id, "good cycle")); bridge.silicon.complete_thinking_trace(id, "good result", 0.95); } assert!(bridge.check_self_repair_needed().is_none()); }
    #[test] fn test_trigger_self_repair_resets_state() { let mut bridge = ThinkingBridge::new("/tmp"); for _ in 0..3 { let id = bridge.silicon.thinking_traces.len(); bridge.silicon.add_thinking_trace(ThinkingTrace::new(id, "poor")); bridge.silicon.complete_thinking_trace(id, "bad", 0.3); } let _ = bridge.check_self_repair_needed(); let msg = bridge.trigger_self_repair(); assert!(msg.contains("self-repair")); }
    #[test] fn test_intrinsic_motivation_initial() { let mut bridge = ThinkingBridge::new("/tmp"); bridge.run_reflection_cycle(); let mot = bridge.compute_motivation(); assert!(mot.intrinsic_reward >= 0.0); }
    #[test] fn test_evolution_summary_contains_expected() { let mut bridge = ThinkingBridge::new("/tmp"); bridge.run_full_evolution_cycle(); let summary = bridge.evolution_summary(); assert!(summary.contains("IntrinsicMotivation") || summary.contains("Stability")); }
    #[test] fn test_broadcast_attention_no_active_heads() { let bridge = ThinkingBridge::new("/tmp"); let mut gwt = GlobalWorkspace::new(0.5); let results = bridge.broadcast_attention_to_gwt(&mut gwt); assert!(results.is_empty()); }
    #[test] fn test_broadcast_attention_with_stimulated_heads() { let mut bridge = ThinkingBridge::new("/tmp"); bridge.silicon.attention_manager.stimulate_domain(AttentionDomain::Code, 0.9); bridge.silicon.attention_manager.stimulate_domain(AttentionDomain::Planning, 0.8); bridge.silicon.attention_manager.stimulate_domain(AttentionDomain::SelfReflection, 0.7); bridge.silicon.attention_manager.global_threshold = 0.5; let mut gwt = GlobalWorkspace::new(0.3); let results = bridge.broadcast_attention_to_gwt(&mut gwt); assert_eq!(results.len(), 3); }
    #[test] fn test_broadcast_attention_salience_above_gwt_threshold() { let mut bridge = ThinkingBridge::new("/tmp"); bridge.silicon.attention_manager.stimulate_domain(AttentionDomain::ToolUse, 0.85); bridge.silicon.attention_manager.global_threshold = 0.3; let mut gwt = GlobalWorkspace::new(0.6); let results = bridge.broadcast_attention_to_gwt(&mut gwt); assert_eq!(results.len(), 1); }

    #[test] fn test_skill_extract_from_recent() { let mut reg = CrystalRegistry::new(); let mut good_trace = ThinkingTrace::new(0, "refactor api"); good_trace.grade = ReflectionGrade::Good; good_trace.steps.push(ThinkingStep::new(1, "analyze", StrategyKind::Reflection).with_domain(AttentionDomain::Code)); let mut poor_trace = ThinkingTrace::new(1, "failed attempt"); poor_trace.grade = ReflectionGrade::Poor; poor_trace.steps.push(ThinkingStep::new(1, "try", StrategyKind::Direct).with_domain(AttentionDomain::Code)); let extracted = SkillBridge::extract_from_recent(&mut reg, &[good_trace, poor_trace], 1); assert_eq!(extracted, 1); assert_eq!(reg.crystals.len(), 1); }
    #[test] fn test_skill_recommend_found() { let mut reg = CrystalRegistry::new(); let mut trace = ThinkingTrace::new(0, "debug"); trace.grade = ReflectionGrade::Excellent; trace.steps.push(ThinkingStep::new(1, "step", StrategyKind::ChainOfThought).with_domain(AttentionDomain::Code)); reg.extract_from_trace(&trace, 1); let rec = SkillBridge::recommend(&reg, StrategyKind::ChainOfThought, AttentionDomain::Code); assert!(rec.is_some()); assert!(rec.unwrap().contains("SkillCrystal")); }
    #[test] fn test_skill_recommend_not_found() { let reg = CrystalRegistry::new(); let rec = SkillBridge::recommend(&reg, StrategyKind::Direct, AttentionDomain::Code); assert!(rec.is_none()); }
    #[test] fn test_skill_auto_maintain_empty() { let mut reg = CrystalRegistry::new(); assert_eq!(SkillBridge::auto_maintain_registry(&mut reg), "no maintenance needed"); }
}
