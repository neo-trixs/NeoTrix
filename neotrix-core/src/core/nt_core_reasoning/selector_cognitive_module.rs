use super::strategy_selector::{SelectorStats, SelfHealingSelector, StrategyConfig};
use crate::core::nt_core_consciousness::cognitive_module_registry::{CognitiveModule, ModulePhase};

#[derive(Debug)]
pub struct SelectorCognitiveModule {
    selector: SelfHealingSelector,
    tick_count: u64,
}

impl SelectorCognitiveModule {
    pub fn new(config: StrategyConfig) -> Self {
        Self {
            selector: SelfHealingSelector::new(config),
            tick_count: 0,
        }
    }

    pub fn selector(&self) -> &SelfHealingSelector {
        &self.selector
    }

    pub fn selector_mut(&mut self) -> &mut SelfHealingSelector {
        &mut self.selector
    }

    pub fn stats(&self) -> SelectorStats {
        self.selector.stats()
    }
}

impl CognitiveModule for SelectorCognitiveModule {
    fn name(&self) -> &'static str {
        "strategy_selector"
    }

    fn phase(&self) -> ModulePhase {
        ModulePhase::PreRefinery
    }

    fn tick(&mut self) -> bool {
        self.tick_count += 1;
        let _ = self.selector.select_strategy("cognitive");
        true
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
        let module = SelectorCognitiveModule::new(StrategyConfig::default());
        assert_eq!(module.name(), "strategy_selector");
        assert_eq!(module.phase(), ModulePhase::PreRefinery);
        assert!(module.is_crash_safe());
    }

    #[test]
    fn test_tick_selects_strategy() {
        let mut module = SelectorCognitiveModule::new(StrategyConfig::default());
        let changed = module.tick();
        assert!(changed);
        let stats = module.selector.stats();
        assert!(stats.total_steps > 0);
    }

    #[test]
    fn test_multiple_ticks_accumulate_steps() {
        let mut module = SelectorCognitiveModule::new(StrategyConfig::default());
        for _ in 0..10 {
            module.tick();
        }
        let stats = module.selector.stats();
        assert_eq!(stats.total_steps, 10);
    }

    #[test]
    fn test_register_in_registry() {
        let mut registry = ModuleRegistry::new();
        registry.register(Box::new(SelectorCognitiveModule::new(StrategyConfig::default())));
        assert_eq!(registry.count(), 1);
        let count = registry.run_phase(ModulePhase::PreRefinery);
        assert_eq!(count, 1);
        assert_eq!(registry.healthy_count(), 1);
    }

    #[test]
    fn test_selector_access() {
        let module = SelectorCognitiveModule::new(StrategyConfig::default());
        let stats = module.selector.stats();
        assert_eq!(stats.total_steps, 0);
    }

    #[test]
    fn test_current_strategy_after_tick() {
        let mut module = SelectorCognitiveModule::new(StrategyConfig::default());
        module.tick();
        let stats = module.selector.stats();
        assert!(stats.current_strategy.name().len() > 0);
    }
}
