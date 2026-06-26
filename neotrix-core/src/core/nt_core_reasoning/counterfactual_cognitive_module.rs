use super::counterfactual_simulator::{
    CounterfactualConfig, CounterfactualSimulator, CounterfactualStats, CounterfactualType,
};
use crate::core::nt_core_consciousness::cognitive_module_registry::{CognitiveModule, ModulePhase};

#[derive(Debug)]
pub struct CounterfactualCognitiveModule {
    simulator: CounterfactualSimulator,
    tick_count: u64,
}

impl CounterfactualCognitiveModule {
    pub fn new(config: CounterfactualConfig) -> Self {
        Self {
            simulator: CounterfactualSimulator::new(config),
            tick_count: 0,
        }
    }

    pub fn simulator(&self) -> &CounterfactualSimulator {
        &self.simulator
    }

    pub fn simulator_mut(&mut self) -> &mut CounterfactualSimulator {
        &mut self.simulator
    }

    pub fn stats(&self) -> CounterfactualStats {
        self.simulator.stats()
    }
}

impl CognitiveModule for CounterfactualCognitiveModule {
    fn name(&self) -> &'static str {
        "counterfactual_simulator"
    }

    fn phase(&self) -> ModulePhase {
        ModulePhase::PostDualPath
    }

    fn tick(&mut self) -> bool {
        self.tick_count += 1;
        let factual = format!("tick_{}_state", self.tick_count).into_bytes();
        let ids = self
            .simulator
            .generate_scenarios(&factual, CounterfactualType::InputPerturbation, 2);
        for id in &ids {
            self.simulator.simulate_scenario(*id);
        }
        let stats = self.simulator.stats();
        stats.total_scenarios > 0
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
        let module =
            CounterfactualCognitiveModule::new(CounterfactualConfig::default());
        assert_eq!(module.name(), "counterfactual_simulator");
        assert_eq!(module.phase(), ModulePhase::PostDualPath);
        assert!(module.is_crash_safe());
    }

    #[test]
    fn test_tick_generates_scenarios() {
        let mut module =
            CounterfactualCognitiveModule::new(CounterfactualConfig::default());
        let changed = module.tick();
        assert!(changed);
        let stats = module.simulator.stats();
        assert!(stats.total_scenarios > 0);
    }

    #[test]
    fn test_multiple_ticks_accumulate() {
        let mut module =
            CounterfactualCognitiveModule::new(CounterfactualConfig::default());
        for _ in 0..5 {
            module.tick();
        }
        let stats = module.simulator.stats();
        assert!(stats.total_scenarios >= 5);
    }

    #[test]
    fn test_register_in_registry() {
        let mut registry = ModuleRegistry::new();
        registry.register(Box::new(CounterfactualCognitiveModule::new(
            CounterfactualConfig::default(),
        )));
        assert_eq!(registry.count(), 1);
        let count = registry.run_phase(ModulePhase::PostDualPath);
        assert_eq!(count, 1);
        assert_eq!(registry.healthy_count(), 1);
    }

    #[test]
    fn test_simulator_access() {
        let module =
            CounterfactualCognitiveModule::new(CounterfactualConfig::default());
        let stats = module.simulator.stats();
        assert_eq!(stats.total_scenarios, 0);
    }

    #[test]
    fn test_scenario_generation_count() {
        let mut module =
            CounterfactualCognitiveModule::new(CounterfactualConfig::default());
        let factual = b"test_state".to_vec();
        let ids = module
            .simulator
            .generate_scenarios(&factual, CounterfactualType::InputPerturbation, 3);
        assert!(ids.len() <= 3);
    }
}
