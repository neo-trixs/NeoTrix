use super::parallel_hypothesis::{
    ParallelHypEvalStats, ParallelHypothesisConfig,
    ParallelHypothesisEvaluator,
};
use super::vsa_blackboard::VsaBlackboard;
use crate::core::nt_core_consciousness::cognitive_module_registry::{CognitiveModule, ModulePhase};

#[derive(Debug)]
pub struct ParallelHypothesisCognitiveModule {
    evaluator: ParallelHypothesisEvaluator,
    tick_count: u64,
}

impl ParallelHypothesisCognitiveModule {
    pub fn new(config: ParallelHypothesisConfig) -> Self {
        let blackboard = VsaBlackboard::new(64);
        Self {
            evaluator: ParallelHypothesisEvaluator::new(config, blackboard),
            tick_count: 0,
        }
    }

    pub fn evaluator(&self) -> &ParallelHypothesisEvaluator {
        &self.evaluator
    }

    pub fn evaluator_mut(&mut self) -> &mut ParallelHypothesisEvaluator {
        &mut self.evaluator
    }

    pub fn stats(&self) -> ParallelHypEvalStats {
        self.evaluator.stats()
    }
}

impl CognitiveModule for ParallelHypothesisCognitiveModule {
    fn name(&self) -> &'static str {
        "parallel_hypothesis"
    }

    fn phase(&self) -> ModulePhase {
        ModulePhase::PostRefinery
    }

    fn tick(&mut self) -> bool {
        self.tick_count += 1;
        self.evaluator
            .post_hypothesis(format!("tick_{}_hypothesis", self.tick_count), 0.5);
        self.evaluator.observe_evidence("routine_evidence", vec![self.tick_count], vec![]);
        self.evaluator.bayesian_update();
        let stats = self.evaluator.stats();
        stats.is_converged
    }

    fn is_crash_safe(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_consciousness::cognitive_module_registry::ModuleRegistry;

    #[test]
    fn test_module_creation() {
        let module = ParallelHypothesisCognitiveModule::new(ParallelHypothesisConfig::default());
        assert_eq!(module.name(), "parallel_hypothesis");
        assert_eq!(module.phase(), ModulePhase::PostRefinery);
        assert!(module.is_crash_safe());
    }

    #[test]
    fn test_tick_runs_evaluation() {
        let mut module = ParallelHypothesisCognitiveModule::new(ParallelHypothesisConfig::default());
        let changed = module.tick();
        let stats = module.evaluator.stats();
        assert!(stats.active_hypotheses > 0);
    }

    #[test]
    fn test_convergence_tracking() {
        let mut module = ParallelHypothesisCognitiveModule::new(ParallelHypothesisConfig::default());
        for _ in 0..10 {
            module.tick();
        }
        let stats = module.evaluator.stats();
        assert_eq!(stats.active_hypotheses, 1);
    }

    #[test]
    fn test_stats_reports_hypotheses() {
        let mut module = ParallelHypothesisCognitiveModule::new(ParallelHypothesisConfig::default());
        module.evaluator.post_hypothesis("alpha".into(), 0.7);
        module.evaluator.post_hypothesis("beta".into(), 0.3);
        let stats = module.stats();
        assert_eq!(stats.active_hypotheses, 2);
    }

    #[test]
    fn test_register_in_registry() {
        let mut registry = ModuleRegistry::new();
        registry.register(Box::new(ParallelHypothesisCognitiveModule::new(
            ParallelHypothesisConfig::default(),
        )));
        assert_eq!(registry.count(), 1);
        let count = registry.run_phase(ModulePhase::PostRefinery);
        assert_eq!(count, 1);
        assert_eq!(registry.healthy_count(), 1);
    }

    #[test]
    fn test_evaluator_access() {
        let module = ParallelHypothesisCognitiveModule::new(ParallelHypothesisConfig::default());
        let stats = module.evaluator.stats();
        assert_eq!(stats.active_hypotheses, 0);
    }
}
