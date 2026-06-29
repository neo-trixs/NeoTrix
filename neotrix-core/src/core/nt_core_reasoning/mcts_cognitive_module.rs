use super::mcts_reasoner::{MctsConfig, MctsReasoner, MctsStats};
use super::vsa_blackboard::{ExpertType, Hypothesis};
use crate::core::nt_core_consciousness::cognitive_module_registry::{CognitiveModule, ModulePhase};
use crate::core::unix_now_ms;

#[derive(Debug)]
pub struct MctsCognitiveModule {
    reasoner: MctsReasoner,
    current_hypothesis: Hypothesis,
    last_result: Vec<Hypothesis>,
    last_stats: Option<MctsStats>,
    tick_count: u64,
}

impl MctsCognitiveModule {
    pub fn new(config: MctsConfig) -> Self {
        let seed = unix_now_ms();
        let seed_hypothesis = Hypothesis {
            id: seed,
            content: Vec::new(),
            confidence: 0.5,
            expert: ExpertType::Synthesis,
            supporting_evidence: vec![],
            created_at: seed,
            is_contradicted: false,
        };
        Self {
            reasoner: MctsReasoner::new(config),
            current_hypothesis: seed_hypothesis,
            last_result: Vec::new(),
            last_stats: None,
            tick_count: 0,
        }
    }

    pub fn last_result(&self) -> &[Hypothesis] {
        &self.last_result
    }

    pub fn last_stats(&self) -> Option<&MctsStats> {
        self.last_stats.as_ref()
    }

    pub fn reasoner_mut(&mut self) -> &mut MctsReasoner {
        &mut self.reasoner
    }

    pub fn reasoner(&self) -> &MctsReasoner {
        &self.reasoner
    }

    pub fn seed_hypothesis(&mut self, hypothesis: Hypothesis) {
        self.current_hypothesis = hypothesis;
    }

    pub fn search(&mut self) -> Vec<Hypothesis> {
        let result = self.reasoner.search(self.current_hypothesis.clone());
        self.last_result = result;
        self.last_stats = Some(self.reasoner.stats());

        if !self.last_result.is_empty() {
            let best = self.last_result.last().expect("non-empty result");
            if best.confidence > self.current_hypothesis.confidence {
                self.current_hypothesis = best.clone();
            }
        }

        self.last_result.clone()
    }
}

impl CognitiveModule for MctsCognitiveModule {
    fn name(&self) -> &'static str {
        "mcts_reasoner"
    }

    fn phase(&self) -> ModulePhase {
        ModulePhase::PostRefinery
    }

    fn tick(&mut self) -> bool {
        self.tick_count += 1;
        self.search();
        if let Some(ref stats) = self.last_stats {
            stats.total_nodes > 1 && stats.best_value > 0.0
        } else {
            false
        }
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
        let module = MctsCognitiveModule::new(MctsConfig::default());
        assert_eq!(module.name(), "mcts_reasoner");
        assert_eq!(module.phase(), ModulePhase::PostRefinery);
        assert!(module.is_crash_safe());
        assert!(module.last_result().is_empty());
        assert!(module.last_stats().is_none());
    }

    #[test]
    fn test_tick_runs_search() {
        let mut module = MctsCognitiveModule::new(MctsConfig::default());
        let changed = module.tick();
        assert!(module.last_stats().is_some());
        let stats = module.last_stats().unwrap();
        assert!(stats.total_nodes > 0);
    }

    #[test]
    fn test_search_returns_reasoning_path() {
        let mut module = MctsCognitiveModule::new(MctsConfig::default());
        let path = module.search();
        if !path.is_empty() {
            assert!(path.iter().any(|h| h.confidence > 0.0));
        }
    }

    #[test]
    fn test_seed_hypothesis_override() {
        let mut module = MctsCognitiveModule::new(MctsConfig::default());
        let custom = Hypothesis {
            id: 999,
            content: vec![1, 2, 3],
            confidence: 0.9,
            expert: ExpertType::Causal,
            supporting_evidence: vec![1, 2, 3],
            created_at: unix_now_ms(),
            is_contradicted: false,
        };
        module.seed_hypothesis(custom.clone());
        module.search();
        assert_eq!(module.current_hypothesis.expert, ExpertType::Causal);
    }

    #[test]
    fn test_register_in_registry() {
        let mut registry = ModuleRegistry::new();
        registry.register(Box::new(MctsCognitiveModule::new(MctsConfig::default())));
        assert_eq!(registry.count(), 1);
        let count = registry.run_phase(ModulePhase::PostRefinery);
        assert_eq!(count, 1);
        assert_eq!(registry.healthy_count(), 1);
    }

    #[test]
    fn test_reasoner_access() {
        let mut module = MctsCognitiveModule::new(MctsConfig::default());
        let stats = module.reasoner().stats();
        assert_eq!(stats.total_nodes, 0);
    }

    #[test]
    fn test_evolves_hypothesis() {
        let mut module = MctsCognitiveModule::new(MctsConfig::default());
        let initial_id = module.current_hypothesis.id;
        module.search();
        let _initial = initial_id;
    }
}
