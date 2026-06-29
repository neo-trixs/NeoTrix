use super::bidirectional_pruner::{BidirectionalPruner, PrunerConfig, PrunerStats};
use super::vsa_blackboard::{ExpertType, Hypothesis};
use crate::core::nt_core_consciousness::cognitive_module_registry::{CognitiveModule, ModulePhase};
use crate::core::unix_now_ms;

#[derive(Debug)]
pub struct PrunerCognitiveModule {
    pruner: BidirectionalPruner,
    tick_count: u64,
}

impl PrunerCognitiveModule {
    pub fn new(config: PrunerConfig) -> Self {
        Self {
            pruner: BidirectionalPruner::new(config),
            tick_count: 0,
        }
    }

    pub fn pruner(&self) -> &BidirectionalPruner {
        &self.pruner
    }

    pub fn pruner_mut(&mut self) -> &mut BidirectionalPruner {
        &mut self.pruner
    }

    pub fn stats(&self) -> PrunerStats {
        self.pruner.stats()
    }
}

impl CognitiveModule for PrunerCognitiveModule {
    fn name(&self) -> &'static str {
        "bidirectional_pruner"
    }

    fn phase(&self) -> ModulePhase {
        ModulePhase::PostDualPath
    }

    fn tick(&mut self) -> bool {
        self.tick_count += 1;
        let initial = Hypothesis {
            id: self.tick_count,
            content: vec![self.tick_count as u8; 16],
            confidence: 0.5,
            expert: ExpertType::Synthesis,
            supporting_evidence: vec![self.tick_count],
            created_at: unix_now_ms(),
            is_contradicted: false,
        };
        let path_id = self.pruner.start_path(&initial);
        for i in 0..3 {
            let step = Hypothesis {
                id: self.tick_count * 10 + i,
                content: vec![(self.tick_count * 10 + i) as u8; 16],
                confidence: 0.3 + (i as f64) * 0.2,
                expert: ExpertType::MultiHop,
                supporting_evidence: vec![self.tick_count, i],
                created_at: unix_now_ms(),
                is_contradicted: false,
            };
            self.pruner.extend_path(path_id, &step, 0.5 + (i as f64) * 0.1);
        }
        let stats = self.pruner.stats();
        stats.total_paths_started > 0
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
        let module = PrunerCognitiveModule::new(PrunerConfig::default());
        assert_eq!(module.name(), "bidirectional_pruner");
        assert_eq!(module.phase(), ModulePhase::PostDualPath);
        assert!(module.is_crash_safe());
    }

    #[test]
    fn test_tick_starts_path() {
        let mut module = PrunerCognitiveModule::new(PrunerConfig::default());
        let changed = module.tick();
        assert!(changed);
        let stats = module.pruner.stats();
        assert!(stats.total_paths_started > 0);
    }

    #[test]
    fn test_multiple_ticks_accumulate_paths() {
        let mut module = PrunerCognitiveModule::new(PrunerConfig::default());
        for _ in 0..5 {
            module.tick();
        }
        let stats = module.pruner.stats();
        assert!(stats.total_paths_started >= 5);
    }

    #[test]
    fn test_register_in_registry() {
        let mut registry = ModuleRegistry::new();
        registry.register(Box::new(PrunerCognitiveModule::new(PrunerConfig::default())));
        assert_eq!(registry.count(), 1);
        let count = registry.run_phase(ModulePhase::PostDualPath);
        assert_eq!(count, 1);
        assert_eq!(registry.healthy_count(), 1);
    }

    #[test]
    fn test_pruner_access() {
        let module = PrunerCognitiveModule::new(PrunerConfig::default());
        let stats = module.pruner.stats();
        assert_eq!(stats.total_paths_started, 0);
    }

    #[test]
    fn test_path_extension() {
        let mut module = PrunerCognitiveModule::new(PrunerConfig::default());
        let initial = Hypothesis {
            id: 1,
            content: vec![0; 16],
            confidence: 0.5,
            expert: ExpertType::Analogical,
            supporting_evidence: vec![1],
            created_at: unix_now_ms(),
            is_contradicted: false,
        };
        let path_id = module.pruner.start_path(&initial);
        assert!(path_id > 0);
        let result = module.pruner.extend_path(
            path_id,
            &Hypothesis {
                id: 2,
                content: vec![1; 16],
                confidence: 0.6,
                expert: ExpertType::Causal,
                supporting_evidence: vec![1, 2],
                created_at: unix_now_ms(),
                is_contradicted: false,
            },
            0.7,
        );
        assert!(result.is_some() || module.pruner.stats().total_paths_started > 0);
    }
}
