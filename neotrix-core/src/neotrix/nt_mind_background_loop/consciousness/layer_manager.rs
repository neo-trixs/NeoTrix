use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayerTier {
    Hot,
    Warm,
    Cold,
}

#[derive(Debug, Clone)]
pub struct SubsystemState {
    pub name: String,
    pub tier: LayerTier,
    pub last_access_cycle: u64,
    pub load_cost: f64,
    pub is_resident: bool,
}

pub struct CognitiveLayerManager {
    pub subsystems: HashMap<String, SubsystemState>,
    pub cycle: u64,
    /// HOT → WARM demotion after this many idle cycles.
    /// Per Memory Tiering paper (clawrxiv:2603.00037): HOT is "aggressively pruned".
    pub hot_idle_threshold: u64,
    /// WARM → COLD demotion after this many idle cycles.
    /// Per paper: WARM "moves to COLD only when it becomes historical".
    pub warm_idle_threshold: u64,
    /// Load threshold above which COLD eviction begins (default 0.7)
    pub eviction_load_threshold: f64,
    /// Whether a compaction event has occurred since last tick
    pub compaction_pending: bool,
}

impl CognitiveLayerManager {
    pub fn new() -> Self {
        let mut manager = Self {
            subsystems: HashMap::new(),
            cycle: 0,
            hot_idle_threshold: 5,
            warm_idle_threshold: 15,
            eviction_load_threshold: 0.7,
            compaction_pending: false,
        };

        for name in &[
            "e8_reasoning",
            "gwt_workspace",
            "hypercube_vsa",
            "consciousness_core",
        ] {
            manager.register_subsystem(name, LayerTier::Hot, 10.0);
        }

        for name in &[
            "knowledge_base",
            "search_engine",
            "fusion_deliberator",
            "handler_profiler",
        ] {
            manager.register_subsystem(name, LayerTier::Warm, 25.0);
        }

        for name in &[
            "jepa_world_model",
            "vision_pipeline",
            "pdf_extractor",
            "spatial_scene",
            "physics_commonsense",
            "imagination_engine",
            "counterfactual_futures",
        ] {
            manager.register_subsystem(name, LayerTier::Cold, 50.0);
        }

        manager
    }

    pub fn register_subsystem(&mut self, name: &str, tier: LayerTier, load_cost: f64) {
        let is_resident = tier == LayerTier::Hot;
        self.subsystems.insert(
            name.to_string(),
            SubsystemState {
                name: name.to_string(),
                tier,
                last_access_cycle: self.cycle,
                load_cost,
                is_resident,
            },
        );
    }

    pub fn access_subsystem(&mut self, name: &str) {
        if let Some(state) = self.subsystems.get_mut(name) {
            state.last_access_cycle = self.cycle;
            if state.tier == LayerTier::Cold {
                state.tier = LayerTier::Warm;
                state.is_resident = true;
            }
        }
    }

    /// Notify the manager that a compaction event has occurred.
    /// On the next tick, COLD tiers will be re-evaluated for eviction
    /// regardless of load (matching the paper's post-compaction trigger).
    pub fn notify_compaction(&mut self) {
        self.compaction_pending = true;
    }

    pub fn tick(&mut self) {
        self.cycle += 1;
        let mut to_warm = Vec::new();
        let mut to_cold = Vec::new();

        for (name, state) in &self.subsystems {
            let idle = self.cycle - state.last_access_cycle;
            match state.tier {
                LayerTier::Hot if idle > self.hot_idle_threshold => to_warm.push(name.clone()),
                LayerTier::Warm if idle > self.warm_idle_threshold => to_cold.push(name.clone()),
                _ => {}
            }
        }

        for name in to_warm {
            if let Some(s) = self.subsystems.get_mut(&name) {
                s.tier = LayerTier::Warm;
            }
        }

        for name in to_cold {
            if let Some(s) = self.subsystems.get_mut(&name) {
                s.tier = LayerTier::Cold;
                s.is_resident = false;
            }
        }

        self.compaction_pending = false;
    }

    pub fn resident_count(&self) -> usize {
        self.subsystems.values().filter(|s| s.is_resident).count()
    }

    pub fn memory_saved(&self) -> f64 {
        self.subsystems
            .values()
            .filter(|s| !s.is_resident)
            .map(|s| s.load_cost)
            .sum()
    }

    pub fn should_evict_cold(&self, load: f64) -> Vec<String> {
        if load <= self.eviction_load_threshold && !self.compaction_pending {
            return Vec::new();
        }
        let mut cold: Vec<&SubsystemState> = self
            .subsystems
            .values()
            .filter(|s| s.tier == LayerTier::Cold && s.is_resident)
            .collect();
        cold.sort_by_key(|s| s.last_access_cycle);
        cold.into_iter().map(|s| s.name.clone()).collect()
    }
}

impl Default for CognitiveLayerManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_manager_has_all_subsystems() {
        let m = CognitiveLayerManager::new();
        assert_eq!(m.subsystems.len(), 15);
        assert_eq!(m.resident_count(), 4);
    }

    #[test]
    fn test_register_and_access_subsystem() {
        let mut m = CognitiveLayerManager::new();
        m.register_subsystem("test_mod", LayerTier::Cold, 30.0);
        assert!(!m.subsystems["test_mod"].is_resident);
        m.access_subsystem("test_mod");
        assert!(m.subsystems["test_mod"].is_resident);
        assert_eq!(m.subsystems["test_mod"].tier, LayerTier::Warm);
    }

    #[test]
    fn test_tier_demotion_hot_to_warm() {
        let mut m = CognitiveLayerManager::new();
        m.hot_idle_threshold = 3;
        for _ in 0..4 {
            m.tick();
        }
        assert_eq!(m.subsystems["e8_reasoning"].tier, LayerTier::Warm);
        assert!(m.subsystems["e8_reasoning"].is_resident);
    }

    #[test]
    fn test_tier_demotion_warm_to_cold() {
        let mut m = CognitiveLayerManager::new();
        m.hot_idle_threshold = 2;
        m.warm_idle_threshold = 4;
        for _ in 0..6 {
            m.tick();
        }
        assert_eq!(m.subsystems["e8_reasoning"].tier, LayerTier::Warm);
        for _ in 0..6 {
            m.tick();
        }
        assert_eq!(m.subsystems["e8_reasoning"].tier, LayerTier::Cold);
        assert!(!m.subsystems["e8_reasoning"].is_resident);
    }

    #[test]
    fn test_resident_count_after_demotion() {
        let mut m = CognitiveLayerManager::new();
        m.hot_idle_threshold = 2;
        m.warm_idle_threshold = 4;
        assert_eq!(m.resident_count(), 4);
        for _ in 0..6 {
            m.tick();
        }
        assert_eq!(m.resident_count(), 0);
    }

    #[test]
    fn test_memory_saved_increases_on_cold_eviction() {
        let mut m = CognitiveLayerManager::new();
        assert_eq!(m.memory_saved(), 0.0);
        m.hot_idle_threshold = 2;
        m.warm_idle_threshold = 4;
        for _ in 0..12 {
            m.tick();
        }
        let saved = m.memory_saved();
        assert!(saved > 0.0);
    }

    #[test]
    fn test_should_evict_cold_returns_subsystems_under_load() {
        let mut m = CognitiveLayerManager::new();
        m.hot_idle_threshold = 2;
        m.warm_idle_threshold = 4;
        for _ in 0..12 {
            m.tick();
        }
        let to_evict = m.should_evict_cold(0.8);
        assert!(!to_evict.is_empty());
        for name in &to_evict {
            assert_eq!(m.subsystems[name].tier, LayerTier::Cold);
        }
    }

    #[test]
    fn test_should_evict_cold_returns_empty_when_load_low() {
        let m = CognitiveLayerManager::new();
        let to_evict = m.should_evict_cold(0.3);
        assert!(to_evict.is_empty());
    }

    #[test]
    fn test_no_subsystems() {
        let mut m = CognitiveLayerManager {
            subsystems: HashMap::new(),
            cycle: 0,
            hot_idle_threshold: 10,
            warm_idle_threshold: 20,
            eviction_load_threshold: 0.7,
            compaction_pending: false,
        };
        assert_eq!(m.resident_count(), 0);
        assert_eq!(m.memory_saved(), 0.0);
        assert!(m.should_evict_cold(0.9).is_empty());
        m.tick();
        assert_eq!(m.cycle, 1);
    }

    #[test]
    fn test_access_after_demotion_promotes_from_cold() {
        let mut m = CognitiveLayerManager::new();
        m.hot_idle_threshold = 2;
        m.warm_idle_threshold = 4;
        for _ in 0..12 {
            m.tick();
        }
        assert_eq!(m.subsystems["e8_reasoning"].tier, LayerTier::Cold);
        assert!(!m.subsystems["e8_reasoning"].is_resident);
        m.access_subsystem("e8_reasoning");
        assert_eq!(m.subsystems["e8_reasoning"].tier, LayerTier::Warm);
        assert!(m.subsystems["e8_reasoning"].is_resident);
    }

    #[test]
    fn test_access_updates_last_access_cycle() {
        let mut m = CognitiveLayerManager::new();
        let before = m.subsystems["hypercube_vsa"].last_access_cycle;
        m.access_subsystem("hypercube_vsa");
        assert_eq!(m.subsystems["hypercube_vsa"].last_access_cycle, before);
        m.tick();
        m.access_subsystem("hypercube_vsa");
        assert_eq!(m.subsystems["hypercube_vsa"].last_access_cycle, m.cycle);
    }

    #[test]
    fn test_hot_subsystems_never_evicted_when_accessed() {
        let mut m = CognitiveLayerManager::new();
        m.hot_idle_threshold = 2;
        for _ in 0..10 {
            m.access_subsystem("consciousness_core");
            m.tick();
        }
        assert_eq!(m.subsystems["consciousness_core"].tier, LayerTier::Hot);
        assert!(m.subsystems["consciousness_core"].is_resident);
    }
}
