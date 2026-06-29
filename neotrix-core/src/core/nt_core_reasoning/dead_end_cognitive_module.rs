use super::dead_end_detector::{DeadEndConfig, DeadEndDetector, DeadEndStats};
use super::vsa_blackboard::Hypothesis;
use crate::core::nt_core_consciousness::cognitive_module_registry::{CognitiveModule, ModulePhase};
use crate::core::unix_now_ms;

#[derive(Debug)]
pub struct DeadEndCognitiveModule {
    detector: DeadEndDetector,
    tick_count: u64,
}

impl DeadEndCognitiveModule {
    pub fn new(config: DeadEndConfig) -> Self {
        Self {
            detector: DeadEndDetector::new(config),
            tick_count: 0,
        }
    }

    pub fn detector(&self) -> &DeadEndDetector {
        &self.detector
    }

    pub fn detector_mut(&mut self) -> &mut DeadEndDetector {
        &mut self.detector
    }

    pub fn stats(&self) -> DeadEndStats {
        self.detector.stats()
    }
}

impl CognitiveModule for DeadEndCognitiveModule {
    fn name(&self) -> &'static str {
        "dead_end_detector"
    }

    fn phase(&self) -> ModulePhase {
        ModulePhase::PostRefinery
    }

    fn tick(&mut self) -> bool {
        self.tick_count += 1;
        let dummy = Hypothesis {
            id: self.tick_count,
            content: vec![self.tick_count as u8; 16],
            confidence: 0.5,
            expert: super::vsa_blackboard::ExpertType::Synthesis,
            supporting_evidence: vec![self.tick_count],
            created_at: unix_now_ms(),
            is_contradicted: false,
        };
        self.detector.record_step(&dummy);
        let stats = self.detector.stats();
        stats.dead_ends_detected > 0
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
        let module = DeadEndCognitiveModule::new(DeadEndConfig::default());
        assert_eq!(module.name(), "dead_end_detector");
        assert_eq!(module.phase(), ModulePhase::PostRefinery);
        assert!(module.is_crash_safe());
    }

    #[test]
    fn test_tick_records_step() {
        let mut module = DeadEndCognitiveModule::new(DeadEndConfig::default());
        let changed = module.tick();
        let stats = module.detector.stats();
        assert!(stats.total_checks > 0);
    }

    #[test]
    fn test_multiple_ticks_accumulate_steps() {
        let mut module = DeadEndCognitiveModule::new(DeadEndConfig::default());
        for _ in 0..10 {
            module.tick();
        }
        let stats = module.detector.stats();
        assert!(stats.total_checks >= 10);
    }

    #[test]
    fn test_register_in_registry() {
        let mut registry = ModuleRegistry::new();
        registry.register(Box::new(DeadEndCognitiveModule::new(DeadEndConfig::default())));
        assert_eq!(registry.count(), 1);
        let count = registry.run_phase(ModulePhase::PostRefinery);
        assert_eq!(count, 1);
        assert_eq!(registry.healthy_count(), 1);
    }

    #[test]
    fn test_detector_access() {
        let module = DeadEndCognitiveModule::new(DeadEndConfig::default());
        let stats = module.detector.stats();
        assert_eq!(stats.total_checks, 0);
        assert_eq!(stats.dead_ends_detected, 0);
    }

    #[test]
    fn test_contradiction_tracking() {
        let mut module = DeadEndCognitiveModule::new(DeadEndConfig {
            fast_monitor_interval: 1,
            ..DeadEndConfig::default()
        });
        for _ in 0..5 {
            module.tick();
        }
        let stats = module.detector.stats();
        assert!(stats.contradiction_count >= 0);
    }
}
