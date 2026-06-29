use super::process_reward_model::{PrmConfig, PrmStats, ProcessRewardModel, ReasoningStep, StepType};
use crate::core::nt_core_consciousness::cognitive_module_registry::{CognitiveModule, ModulePhase};

#[derive(Debug)]
pub struct PrmCognitiveModule {
    model: ProcessRewardModel,
    tick_count: u64,
}

impl PrmCognitiveModule {
    pub fn new(config: PrmConfig) -> Self {
        Self {
            model: ProcessRewardModel::new(config),
            tick_count: 0,
        }
    }

    pub fn model(&self) -> &ProcessRewardModel {
        &self.model
    }

    pub fn model_mut(&mut self) -> &mut ProcessRewardModel {
        &mut self.model
    }

    pub fn stats(&self) -> PrmStats {
        self.model.stats()
    }
}

impl CognitiveModule for PrmCognitiveModule {
    fn name(&self) -> &'static str {
        "process_reward_model"
    }

    fn phase(&self) -> ModulePhase {
        ModulePhase::PostRefinery
    }

    fn tick(&mut self) -> bool {
        self.tick_count += 1;
        let step = ReasoningStep {
            step_id: self.tick_count,
            hypothesis_id: None,
            step_type: StepType::Infer,
            content: format!("cognitive_step_{}", self.tick_count),
            pre_reward: 0.0,
            post_reward: 0.0,
            process_score: 0.5,
            outcome_score: 0.5,
        };
        let id = self.model.add_step(step);
        if id > 0 {
            self.model.evaluate_step(id);
        }
        let stats = self.model.stats();
        stats.total_steps > 0
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
        let module = PrmCognitiveModule::new(PrmConfig::default());
        assert_eq!(module.name(), "process_reward_model");
        assert_eq!(module.phase(), ModulePhase::PostRefinery);
        assert!(module.is_crash_safe());
    }

    #[test]
    fn test_tick_adds_step() {
        let mut module = PrmCognitiveModule::new(PrmConfig::default());
        let changed = module.tick();
        assert!(changed);
        let stats = module.model.stats();
        assert!(stats.total_steps > 0);
    }

    #[test]
    fn test_multiple_ticks_accumulate_steps() {
        let mut module = PrmCognitiveModule::new(PrmConfig::default());
        for _ in 0..5 {
            module.tick();
        }
        let stats = module.model.stats();
        assert_eq!(stats.total_steps, 5);
    }

    #[test]
    fn test_register_in_registry() {
        let mut registry = ModuleRegistry::new();
        registry.register(Box::new(PrmCognitiveModule::new(PrmConfig::default())));
        assert_eq!(registry.count(), 1);
        let count = registry.run_phase(ModulePhase::PostRefinery);
        assert_eq!(count, 1);
        assert_eq!(registry.healthy_count(), 1);
    }

    #[test]
    fn test_model_access() {
        let module = PrmCognitiveModule::new(PrmConfig::default());
        let stats = module.model.stats();
        assert_eq!(stats.total_steps, 0);
    }

    #[test]
    fn test_step_evaluation() {
        let mut module = PrmCognitiveModule::new(PrmConfig::default());
        let id = module.model.add_step(ReasoningStep {
            step_id: 1,
            step_type: StepType::Verify,
            content: "test_eval".into(),
            pre_reward: 0.0,
            post_reward: 0.0,
            process_score: 0.5,
            outcome_score: 0.8,
            hypothesis_id: None,
        });
        assert!(id > 0);
        let reward = module.model.evaluate_step(id);
        assert!(reward > 0.0);
    }
}
