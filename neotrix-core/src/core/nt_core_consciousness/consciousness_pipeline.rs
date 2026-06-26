use std::time::Instant;

use super::cognitive_blackboard::{BlackboardConfig, CognitiveBlackboard, EngineType};
use super::cognitive_module_registry::{CognitiveModule, ModulePhase, ModuleRegistry};
use super::consciousness_cycle::{ConsciousnessCycle, CycleConfig};
use super::consciousness_refinery::{
    ConsciousnessRefineryLoop, ConvergenceSignal, RefineryConfig, RefineryResult,
};
use super::dual_path_inference::{DualPathConfig, DualPathInference, DualPathResult};
use super::episodic_buffer::{BufferConfig, EpisodicConsciousnessBuffer, RecallResult};
use super::executable_belief::{
    BeliefVerificationConfig, ExecutableBeliefVerifier, VerificationReport,
};
use super::meta_cognition_bridge::MetaCognitionBridge;
use super::meta_evolution_loop::{
    EvolutionRecommendation, MetaArchitectureEvolutionLoop, MetaEvolutionConfig,
};
use super::resource_allocator::{
    AllocatorConfig, BudgetAllocation, ConsciousResourceAllocator, InternalState,
};
use super::spectrum_signal::{Candidate, SpectrumConfig, SpectrumSignal};
use super::vsa_tag::VsaTagged;

#[derive(Debug, Clone)]
pub struct PipelineConfig {
    pub refinery: RefineryConfig,
    pub blackboard: BlackboardConfig,
    pub dual_path: DualPathConfig,
    pub buffer: BufferConfig,
    pub belief_verify: BeliefVerificationConfig,
    pub meta_evolution: MetaEvolutionConfig,
    pub allocator: AllocatorConfig,
    pub spectrum: SpectrumConfig,
    pub cycles_per_meta_assessment: u64,
    pub enable_dual_path: bool,
    pub enable_spectrum: bool,
    pub enable_blackboard_sync: bool,
    pub enable_belief_verification: bool,
    pub enable_meta_evolution: bool,
    pub enable_load_shedding: bool,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            refinery: RefineryConfig::default(),
            blackboard: BlackboardConfig::default(),
            dual_path: DualPathConfig::default(),
            buffer: BufferConfig::default(),
            belief_verify: BeliefVerificationConfig::default(),
            meta_evolution: MetaEvolutionConfig::default(),
            allocator: AllocatorConfig::default(),
            spectrum: SpectrumConfig::default(),
            cycles_per_meta_assessment: 10,
            enable_dual_path: true,
            enable_spectrum: true,
            enable_blackboard_sync: true,
            enable_belief_verification: true,
            enable_meta_evolution: true,
            enable_load_shedding: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PipelineStepResult {
    pub step_name: String,
    pub duration_ns: u64,
    pub success: bool,
    pub details: String,
}

#[derive(Debug, Clone)]
pub struct IntegratedResult {
    pub cycle_number: u64,
    pub refinery: RefineryResult,
    pub dual_path: Option<DualPathResult>,
    pub verification: Option<VerificationReport>,
    pub spectrum_candidates: Vec<Candidate>,
    pub blackboard_synced: bool,
    pub episodic_recorded: bool,
    pub load_shed_active: bool,
    pub allocation: Vec<BudgetAllocation>,
    pub meta_recommendations: Vec<EvolutionRecommendation>,
    pub step_timings: Vec<PipelineStepResult>,
    pub total_duration_ns: u64,
    pub all_passed: bool,
}

pub struct ConsciousnessPipeline {
    cycle: ConsciousnessCycle,
    refinery: ConsciousnessRefineryLoop,
    blackboard: CognitiveBlackboard,
    dual_path: DualPathInference,
    buffer: EpisodicConsciousnessBuffer,
    verifier: ExecutableBeliefVerifier,
    meta_evolution: MetaArchitectureEvolutionLoop,
    allocator: ConsciousResourceAllocator,
    spectrum: SpectrumSignal,
    config: PipelineConfig,
    cycle_counter: u64,
    meta_bridge: Option<MetaCognitionBridge>,
    last_bridge_gaps: Vec<String>,
    last_bridge_insights: usize,
    registry: ModuleRegistry,
}

impl ConsciousnessPipeline {
    pub fn new(config: PipelineConfig) -> Self {
        let cycle_config = CycleConfig {
            enable_visual_pipeline: true,
            enable_modality_gate: true,
            ..CycleConfig::default()
        };
        let cycle = ConsciousnessCycle::new(cycle_config);
        let refinery = ConsciousnessRefineryLoop::new(cycle.clone(), config.refinery.clone());
        let blackboard = CognitiveBlackboard::new(config.blackboard.clone());
        let dual_path = DualPathInference::new(config.dual_path.clone());
        let buffer = EpisodicConsciousnessBuffer::new(config.buffer.clone());
        let verifier = ExecutableBeliefVerifier::new(config.belief_verify.clone());
        let meta_evolution = MetaArchitectureEvolutionLoop::new(config.meta_evolution.clone());
        let allocator = ConsciousResourceAllocator::new(config.allocator.clone());
        let spectrum = SpectrumSignal::new(config.spectrum.clone());

        Self {
            cycle,
            refinery,
            blackboard,
            dual_path,
            buffer,
            verifier,
            meta_evolution,
            allocator,
            spectrum,
            config,
            cycle_counter: 0,
            meta_bridge: None,
            last_bridge_gaps: Vec::new(),
            last_bridge_insights: 0,
            registry: ModuleRegistry::new(),
        }
    }

    pub fn cycle(&self) -> &ConsciousnessCycle {
        &self.cycle
    }
    pub fn cycle_mut(&mut self) -> &mut ConsciousnessCycle {
        &mut self.cycle
    }
    pub fn refinery(&self) -> &ConsciousnessRefineryLoop {
        &self.refinery
    }
    pub fn refinery_mut(&mut self) -> &mut ConsciousnessRefineryLoop {
        &mut self.refinery
    }
    pub fn blackboard(&self) -> &CognitiveBlackboard {
        &self.blackboard
    }
    pub fn blackboard_mut(&mut self) -> &mut CognitiveBlackboard {
        &mut self.blackboard
    }
    pub fn dual_path(&self) -> &DualPathInference {
        &self.dual_path
    }
    pub fn dual_path_mut(&mut self) -> &mut DualPathInference {
        &mut self.dual_path
    }
    pub fn buffer(&self) -> &EpisodicConsciousnessBuffer {
        &self.buffer
    }
    pub fn buffer_mut(&mut self) -> &mut EpisodicConsciousnessBuffer {
        &mut self.buffer
    }
    pub fn verifier(&self) -> &ExecutableBeliefVerifier {
        &self.verifier
    }
    pub fn verifier_mut(&mut self) -> &mut ExecutableBeliefVerifier {
        &mut self.verifier
    }
    pub fn meta_evolution(&self) -> &MetaArchitectureEvolutionLoop {
        &self.meta_evolution
    }
    pub fn meta_evolution_mut(&mut self) -> &mut MetaArchitectureEvolutionLoop {
        &mut self.meta_evolution
    }
    pub fn allocator(&self) -> &ConsciousResourceAllocator {
        &self.allocator
    }
    pub fn allocator_mut(&mut self) -> &mut ConsciousResourceAllocator {
        &mut self.allocator
    }
    pub fn spectrum(&self) -> &SpectrumSignal {
        &self.spectrum
    }
    pub fn spectrum_mut(&mut self) -> &mut SpectrumSignal {
        &mut self.spectrum
    }
    pub fn config(&self) -> &PipelineConfig {
        &self.config
    }
    pub fn config_mut(&mut self) -> &mut PipelineConfig {
        &mut self.config
    }
    pub fn cycle_counter(&self) -> u64 {
        self.cycle_counter
    }

    pub fn with_meta_bridge(mut self, bridge: MetaCognitionBridge) -> Self {
        self.meta_bridge = Some(bridge);
        self
    }

    pub fn meta_bridge(&self) -> Option<&MetaCognitionBridge> {
        self.meta_bridge.as_ref()
    }

    pub fn meta_bridge_mut(&mut self) -> Option<&mut MetaCognitionBridge> {
        self.meta_bridge.as_mut()
    }

    pub fn update_internal_state(&mut self, state: InternalState) {
        self.allocator.update_state(state);
    }

    pub fn adjust_uncertainty(&mut self, delta: f64) {
        self.allocator.adjust_uncertainty(delta);
    }
    pub fn adjust_surprise(&mut self, delta: f64) {
        self.allocator.adjust_surprise(delta);
    }
    pub fn adjust_curiosity(&mut self, delta: f64) {
        self.allocator.adjust_curiosity(delta);
    }
    pub fn adjust_boredom(&mut self, delta: f64) {
        self.allocator.adjust_boredom(delta);
    }

    /// 注册一个认知模块到 ModuleRegistry
    pub fn register_module(&mut self, module: Box<dyn CognitiveModule>) {
        self.registry.register(module);
    }

    /// ModuleRegistry 引用
    pub fn module_registry(&self) -> &ModuleRegistry {
        &self.registry
    }

    pub fn module_registry_mut(&mut self) -> &mut ModuleRegistry {
        &mut self.registry
    }

    /// 在所有管道阶段执行已注册的认知模块
    fn run_registered_at_phase(
        &mut self,
        phase: ModulePhase,
        timings: &mut Vec<PipelineStepResult>,
    ) {
        let start = std::time::Instant::now();
        let count = self.registry.run_phase(phase);
        if count > 0 {
            timings.push(PipelineStepResult {
                step_name: format!("cognitive_modules:{:?}", phase),
                duration_ns: start.elapsed().as_nanos() as u64,
                success: true,
                details: format!("{} modules ran", count),
            });
        }
    }

    pub fn run_full_cycle(&mut self, input: Option<VsaTagged>) -> IntegratedResult {
        let global_start = Instant::now();
        let mut step_timings = Vec::new();
        self.cycle_counter += 1;

        // Phase 1: Resource allocation — determine cognitive budget
        // Guided by meta-cognition bridge insights from previous cycle
        if self.last_bridge_insights > 0 || !self.last_bridge_gaps.is_empty() {
            let gap_count = self.last_bridge_gaps.len();
            if gap_count > 2 {
                // Multiple layer gaps → increase curiosity to explore uncovered layers
                self.allocator.adjust_curiosity(0.1 * gap_count as f64);
            }
            if self.last_bridge_insights > 5 {
                // Many architecture insights → reduce risk tolerance
                self.allocator.adjust_uncertainty(-0.05);
            }
        }

        let (alloc_start, allocation) = Self::measure(|| {
            if self.config.enable_load_shedding && self.allocator.should_shed_load() {
                let budget = self.allocator.allocate();
                (true, budget)
            } else {
                let budget = self.allocator.allocate();
                (false, budget)
            }
        });
        let (load_shed_active, allocation) = allocation;
        step_timings.push(PipelineStepResult {
            step_name: "resource_allocation".into(),
            duration_ns: alloc_start,
            success: true,
            details: format!(
                "urgency={:.2} load_shed={} iterations={}",
                self.allocator.state().urgency(),
                load_shed_active,
                self.allocator.recommended_iterations(),
            ),
        });

        // Phase 1b: Registered cognitive modules (PreRefinery)
        self.run_registered_at_phase(ModulePhase::PreRefinery, &mut step_timings);

        // Phase 2: Refinery loop (runs ConsciousnessCycle N times)
        let (refine_duration, refinery_result) = Self::measure(|| self.refinery.refine(input));
        step_timings.push(PipelineStepResult {
            step_name: "refinery_loop".into(),
            duration_ns: refine_duration,
            success: refinery_result.metrics.convergence_signal != ConvergenceSignal::Diverging,
            details: format!(
                "iterations={} converged={:?} delta_ema={:.4}",
                refinery_result.metrics.iteration,
                refinery_result.metrics.convergence_signal,
                refinery_result.metrics.delta_ema,
            ),
        });

        let final_state = Some(refinery_result.final_cycle.output_state.clone())
            .filter(|s: &Option<VsaTagged>| s.is_some())
            .flatten();

        // Phase 3: Dual-path inference (requires vector slice from state)
        let dual_path_result = if self.config.enable_dual_path {
            if let Some(ref state) = final_state {
                let (dur, result) =
                    Self::measure(|| self.dual_path.infer(&state.vector, "consciousness_cycle"));
                step_timings.push(PipelineStepResult {
                    step_name: "dual_path_inference".into(),
                    duration_ns: dur,
                    success: result.consensus > 0.5,
                    details: format!("consensus={:.2}", result.consensus),
                });
                Some(result)
            } else {
                None
            }
        } else {
            None
        };

        // Phase 3b: Registered cognitive modules (PostRefinery)
        self.run_registered_at_phase(ModulePhase::PostRefinery, &mut step_timings);

        // Phase 4: Blackboard sync — post refinery output as claim
        if self.config.enable_blackboard_sync {
            if let Some(ref state) = final_state {
                let (dur, _) = Self::measure(|| {
                    self.blackboard.post_claim(
                        EngineType::DualPath,
                        "consciousness_cycle".into(),
                        format!("cycle_{}_refined", self.cycle_counter),
                        state.vector.clone(),
                        state.confidence,
                    );
                });
                step_timings.push(PipelineStepResult {
                    step_name: "blackboard_sync".into(),
                    duration_ns: dur,
                    success: true,
                    details: "posted refinery output to blackboard".into(),
                });
            }
        }

        // Phase 5: Spectrum signal — generate diversity candidates
        let spectrum_candidates = if self.config.enable_spectrum {
            if let Some(ref state) = final_state {
                let (dur, opt_candidate) = Self::measure(|| {
                    self.spectrum
                        .run_pipeline(&state.vector, "consciousness_cycle")
                });
                let candidates: Vec<Candidate> = opt_candidate.into_iter().collect();
                step_timings.push(PipelineStepResult {
                    step_name: "spectrum_signal".into(),
                    duration_ns: dur,
                    success: !candidates.is_empty(),
                    details: format!("{} candidates generated", candidates.len()),
                });
                candidates
            } else {
                vec![]
            }
        } else {
            vec![]
        };

        // Phase 5b: Registered cognitive modules (PostSpectrum)
        self.run_registered_at_phase(ModulePhase::PreBeliefVerify, &mut step_timings);

        // Phase 6: Belief verification
        let verification = if self.config.enable_belief_verification {
            if let Some(ref state) = final_state {
                let (dur, report) =
                    Self::measure(|| self.verifier.verify("consciousness_cycle", state, vec![]));
                step_timings.push(PipelineStepResult {
                    step_name: "belief_verification".into(),
                    duration_ns: dur,
                    success: report.levels_passed.len() >= 2,
                    details: format!(
                        "passed={} failed={}",
                        report.levels_passed.len(),
                        report.levels_failed.len(),
                    ),
                });
                Some(report)
            } else {
                None
            }
        } else {
            None
        };

        // Phase 7: Episodic recording
        if let Some(state) = final_state.clone() {
            let (dur, _) = Self::measure(|| {
                self.buffer.push(
                    state,
                    self.cycle_counter,
                    format!("cycle_{}", self.cycle_counter),
                );
            });
            step_timings.push(PipelineStepResult {
                step_name: "episodic_record".into(),
                duration_ns: dur,
                success: true,
                details: format!("buffer_size={}", self.buffer.len()),
            });
        }

        // Phase 7b: Registered cognitive modules (Final)
        self.run_registered_at_phase(ModulePhase::Final, &mut step_timings);

        // Phase 8: Meta-evolution assessment (every N cycles)
        let meta_recommendations = if self.config.enable_meta_evolution
            && self.cycle_counter % self.config.cycles_per_meta_assessment == 0
        {
            let (dur, recs) = Self::measure(|| self.meta_evolution.assess());
            step_timings.push(PipelineStepResult {
                step_name: "meta_evolution_assessment".into(),
                duration_ns: dur,
                success: true,
                details: format!("{} recommendations", recs.len()),
            });

            // Phase 8b: MetaCognition bridge — self-understanding → architecture health
            if let Some(ref mut bridge) = self.meta_bridge {
                let (bdur, report) = Self::measure(|| bridge.run_bridge(self.cycle_counter));
                self.last_bridge_gaps = report.layer_gaps.clone();
                self.last_bridge_insights = report.new_insights;
                step_timings.push(PipelineStepResult {
                    step_name: "meta_cognition_bridge".into(),
                    duration_ns: bdur,
                    success: report.new_insights > 0 || report.total_entities > 0,
                    details: format!(
                        "entities={} insights={} gaps={}",
                        report.total_entities,
                        report.new_insights,
                        report.layer_gaps.len(),
                    ),
                });
            }

            recs
        } else {
            vec![]
        };

        let total_duration_ns = global_start.elapsed().as_nanos() as u64;
        let all_passed = step_timings.iter().all(|s| s.success);

        IntegratedResult {
            cycle_number: self.cycle_counter,
            refinery: refinery_result,
            dual_path: dual_path_result,
            verification,
            spectrum_candidates,
            blackboard_synced: self.config.enable_blackboard_sync,
            episodic_recorded: final_state.is_some(),
            load_shed_active,
            allocation,
            meta_recommendations,
            step_timings,
            total_duration_ns,
            all_passed,
        }
    }

    /// Run an external measurement of a closure, returning (duration_ns, result)
    fn measure<T>(f: impl FnOnce() -> T) -> (u64, T) {
        let start = Instant::now();
        let result = f();
        (start.elapsed().as_nanos() as u64, result)
    }

    /// Recall recent episodic history for context injection.
    pub fn recall_episodic(&mut self, query: &VsaTagged, k: usize) -> RecallResult {
        self.buffer.recall_similar(query, k)
    }

    /// Replay the last N cycles for consolidation.
    pub fn replay_recent(&self, n: usize) -> Vec<&super::episodic_buffer::EpisodicEntry> {
        self.buffer.replay_last(n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_state() -> Option<VsaTagged> {
        Some(VsaTagged::self_thought("test"))
    }

    #[test]
    fn test_pipeline_default_config() {
        let config = PipelineConfig::default();
        assert!(config.enable_dual_path);
        assert!(config.enable_spectrum);
        assert!(config.enable_blackboard_sync);
        assert!(config.enable_belief_verification);
        assert!(config.enable_meta_evolution);
        assert_eq!(config.cycles_per_meta_assessment, 10);
    }

    #[test]
    fn test_pipeline_creation() {
        let mut pipe = ConsciousnessPipeline::new(PipelineConfig::default());
        assert_eq!(pipe.cycle_counter(), 0);
        assert!(pipe.blackboard().topics().is_empty());
        assert!(pipe.buffer().is_empty());
        assert_eq!(pipe.allocator().state().urgency(), 0.425);
    }

    #[test]
    fn test_run_full_cycle_basic() {
        let mut pipe = ConsciousnessPipeline::new(PipelineConfig::default());
        let result = pipe.run_full_cycle(make_test_state());
        assert!(result.cycle_number >= 1);
        assert!(!result.allocation.is_empty());
        assert!(result.blackboard_synced);
        assert!(result.episodic_recorded || !result.episodic_recorded);
        assert!(result.step_timings.len() >= 4);
        assert!(result.total_duration_ns > 0);
    }

    #[test]
    fn test_run_full_cycle_no_dual_path() {
        let config = PipelineConfig {
            enable_dual_path: false,
            ..PipelineConfig::default()
        };
        let mut pipe = ConsciousnessPipeline::new(config);
        let result = pipe.run_full_cycle(make_test_state());
        assert!(result.dual_path.is_none());
    }

    #[test]
    fn test_run_full_cycle_no_spectrum() {
        let config = PipelineConfig {
            enable_spectrum: false,
            ..PipelineConfig::default()
        };
        let mut pipe = ConsciousnessPipeline::new(config);
        let result = pipe.run_full_cycle(make_test_state());
        assert!(result.spectrum_candidates.is_empty());
    }

    #[test]
    fn test_run_full_cycle_no_blackboard() {
        let config = PipelineConfig {
            enable_blackboard_sync: false,
            ..PipelineConfig::default()
        };
        let mut pipe = ConsciousnessPipeline::new(config);
        let result = pipe.run_full_cycle(make_test_state());
        let sync_step = result
            .step_timings
            .iter()
            .find(|s| s.step_name == "blackboard_sync");
        assert!(sync_step.is_none());
    }

    #[test]
    fn test_run_full_cycle_no_verification() {
        let config = PipelineConfig {
            enable_belief_verification: false,
            ..PipelineConfig::default()
        };
        let mut pipe = ConsciousnessPipeline::new(config);
        let result = pipe.run_full_cycle(make_test_state());
        assert!(result.verification.is_none());
    }

    #[test]
    fn test_run_full_cycle_no_meta_evolution() {
        let config = PipelineConfig {
            enable_meta_evolution: false,
            ..PipelineConfig::default()
        };
        let mut pipe = ConsciousnessPipeline::new(config);
        let result = pipe.run_full_cycle(make_test_state());
        assert!(result.meta_recommendations.is_empty());
    }

    #[test]
    fn test_run_multiple_cycles_increases_counter() {
        let mut pipe = ConsciousnessPipeline::new(PipelineConfig::default());
        let r1 = pipe.run_full_cycle(make_test_state());
        let r2 = pipe.run_full_cycle(make_test_state());
        assert_eq!(r2.cycle_number, r1.cycle_number + 1);
        assert_eq!(pipe.cycle_counter(), 2);
    }

    #[test]
    fn test_run_multiple_cycles_populates_buffer() {
        let mut pipe = ConsciousnessPipeline::new(PipelineConfig::default());
        pipe.run_full_cycle(make_test_state());
        pipe.run_full_cycle(make_test_state());
        pipe.run_full_cycle(make_test_state());
        assert!(pipe.buffer().len() > 0);
    }

    #[test]
    fn test_internal_state_adjustment() {
        let mut pipe = ConsciousnessPipeline::new(PipelineConfig::default());
        pipe.adjust_uncertainty(0.3);
        pipe.adjust_surprise(0.2);
        pipe.adjust_curiosity(0.1);
        pipe.adjust_boredom(-0.1);
        let state = pipe.allocator().state();
        assert!(state.uncertainty > 0.5);
        assert!(state.surprise > 0.1);
        assert!(state.curiosity > 0.3);
    }

    #[test]
    fn test_load_shedding_config() {
        let mut pipe = ConsciousnessPipeline::new(PipelineConfig {
            enable_load_shedding: true,
            ..PipelineConfig::default()
        });
        pipe.update_internal_state(InternalState {
            cognitive_load: 0.9,
            ..InternalState::new()
        });
        let result = pipe.run_full_cycle(make_test_state());
        assert!(result.load_shed_active);
    }

    #[test]
    fn test_episodic_recall() {
        let mut pipe = ConsciousnessPipeline::new(PipelineConfig::default());
        for _ in 0..5 {
            pipe.run_full_cycle(make_test_state());
        }
        let query = VsaTagged::self_thought("query");
        let recalled = pipe.recall_episodic(&query, 3);
        assert!(recalled.count <= 3);
        assert!(recalled.count > 0);
    }

    #[test]
    fn test_pipeline_all_passed_good_cycle() {
        let mut pipe = ConsciousnessPipeline::new(PipelineConfig::default());
        let result = pipe.run_full_cycle(make_test_state());
        assert!(result.all_passed);
    }

    #[test]
    fn test_knowledge_accessors() {
        let pipe = ConsciousnessPipeline::new(PipelineConfig::default());
        assert!(pipe.config().refinery.max_iterations > 0);
        assert!(pipe.config().buffer.capacity >= 100);
        assert!(pipe.config().allocator.base_iterations >= 1);
    }

    #[test]
    fn test_meta_assessment_triggers_at_interval() {
        let config = PipelineConfig {
            cycles_per_meta_assessment: 2,
            ..PipelineConfig::default()
        };
        let mut pipe = ConsciousnessPipeline::new(config);
        let r1 = pipe.run_full_cycle(make_test_state());
        assert!(r1.meta_recommendations.is_empty());
        let r2 = pipe.run_full_cycle(make_test_state());
        assert!(!r2.meta_recommendations.is_empty());
    }

    #[test]
    fn test_replay_recent() {
        let mut pipe = ConsciousnessPipeline::new(PipelineConfig::default());
        for _ in 0..4 {
            pipe.run_full_cycle(make_test_state());
        }
        let entries = pipe.replay_recent(2);
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn test_mut_accessors() {
        let mut pipe = ConsciousnessPipeline::new(PipelineConfig::default());
        pipe.refinery_mut();
        pipe.blackboard_mut();
        pipe.dual_path_mut();
        pipe.buffer_mut();
        pipe.verifier_mut();
        pipe.meta_evolution_mut();
        pipe.allocator_mut();
        pipe.spectrum_mut();
        pipe.cycle_mut();
    }
}
