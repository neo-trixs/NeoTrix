use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CognitiveProcess {
    Gather,
    Gate,
    Propose,
    Compete,
    Refine,
    DualReason,
    Verify,
    Act,
    Record,
    MetaMonitor,
    Sleep,
    BlackboardSync,
    EvolutionAssess,
}

impl CognitiveProcess {
    pub fn name(&self) -> &'static str {
        match self {
            CognitiveProcess::Gather => "gather",
            CognitiveProcess::Gate => "gate",
            CognitiveProcess::Propose => "propose",
            CognitiveProcess::Compete => "compete",
            CognitiveProcess::Refine => "refine",
            CognitiveProcess::DualReason => "dual_reason",
            CognitiveProcess::Verify => "verify",
            CognitiveProcess::Act => "act",
            CognitiveProcess::Record => "record",
            CognitiveProcess::MetaMonitor => "meta_monitor",
            CognitiveProcess::Sleep => "sleep",
            CognitiveProcess::BlackboardSync => "blackboard_sync",
            CognitiveProcess::EvolutionAssess => "evolution_assess",
        }
    }
    pub fn default_budget(&self) -> f64 {
        match self {
            CognitiveProcess::Refine => 0.20,
            CognitiveProcess::DualReason => 0.15,
            CognitiveProcess::Compete => 0.15,
            CognitiveProcess::Gather => 0.10,
            CognitiveProcess::MetaMonitor => 0.10,
            CognitiveProcess::Verify => 0.08,
            CognitiveProcess::BlackboardSync => 0.07,
            CognitiveProcess::EvolutionAssess => 0.05,
            _ => 0.05,
        }
    }
}

#[derive(Debug, Clone)]
pub struct InternalState {
    pub uncertainty: f64,
    pub surprise: f64,
    pub curiosity: f64,
    pub boredom: f64,
    pub cognitive_load: f64,
    pub confidence: f64,
}

impl InternalState {
    pub fn new() -> Self {
        Self {
            uncertainty: 0.5,
            surprise: 0.0,
            curiosity: 0.3,
            boredom: 0.0,
            cognitive_load: 0.3,
            confidence: 0.5,
        }
    }

    /// Overall urgency: how much does the consciousness need to process?
    pub fn urgency(&self) -> f64 {
        (self.uncertainty * 0.3
            + self.surprise * 0.25
            + self.curiosity * 0.2
            + (1.0 - self.boredom) * 0.15
            + (1.0 - self.confidence) * 0.1)
            .clamp(0.0, 1.0)
    }

    /// Which process needs attention most right now?
    pub fn attention_signal(&self) -> &'static str {
        if self.surprise > 0.7 {
            "gather_new_info"
        } else if self.uncertainty > 0.6 {
            "reduce_uncertainty"
        } else if self.curiosity > 0.6 {
            "explore"
        } else if self.boredom > 0.6 {
            "seek_novelty"
        } else if self.cognitive_load > 0.7 {
            "offload_or_simplify"
        } else {
            "routine_processing"
        }
    }
}

#[derive(Debug, Clone)]
pub struct BudgetAllocation {
    pub process: CognitiveProcess,
    pub allocated_budget: f64,
    pub priority: u8,
    pub rationale: String,
}

#[derive(Debug, Clone)]
pub struct AllocatorConfig {
    pub enable_dynamic_budget: bool,
    pub base_iterations: usize,
    pub max_iterations: usize,
    pub uncertainty_weight: f64,
    pub surprise_weight: f64,
    pub load_shedding_threshold: f64,
}

impl Default for AllocatorConfig {
    fn default() -> Self {
        Self {
            enable_dynamic_budget: true,
            base_iterations: 3,
            max_iterations: 10,
            uncertainty_weight: 0.4,
            surprise_weight: 0.3,
            load_shedding_threshold: 0.8,
        }
    }
}

pub struct ConsciousResourceAllocator {
    config: AllocatorConfig,
    state: InternalState,
    allocation_history: Vec<(InternalState, Vec<BudgetAllocation>)>,
}

impl ConsciousResourceAllocator {
    pub fn new(config: AllocatorConfig) -> Self {
        Self {
            config,
            state: InternalState::new(),
            allocation_history: Vec::new(),
        }
    }

    pub fn config(&self) -> &AllocatorConfig {
        &self.config
    }
    pub fn config_mut(&mut self) -> &mut AllocatorConfig {
        &mut self.config
    }
    pub fn state(&self) -> &InternalState {
        &self.state
    }

    pub fn update_state(&mut self, new: InternalState) {
        self.state = new;
    }

    pub fn adjust_uncertainty(&mut self, delta: f64) {
        self.state.uncertainty = (self.state.uncertainty + delta).clamp(0.0, 1.0);
    }
    pub fn adjust_surprise(&mut self, delta: f64) {
        self.state.surprise = (self.state.surprise + delta).clamp(0.0, 1.0);
    }
    pub fn adjust_curiosity(&mut self, delta: f64) {
        self.state.curiosity = (self.state.curiosity + delta).clamp(0.0, 1.0);
    }
    pub fn adjust_boredom(&mut self, delta: f64) {
        self.state.boredom = (self.state.boredom + delta).clamp(0.0, 1.0);
    }

    /// Allocate cognitive budget across processes based on internal state.
    pub fn allocate(&mut self) -> Vec<BudgetAllocation> {
        let all_processes = CognitiveProcess::all();
        let urgencies = self.compute_process_urgencies();
        let total: f64 = urgencies.values().sum();
        let mut allocations = Vec::new();

        for process in &all_processes {
            let default = process.default_budget();
            let urgency = urgencies.get(process).copied().unwrap_or(default);
            let raw = if self.config.enable_dynamic_budget {
                default * (1.0 + urgency * 2.0)
            } else {
                default
            };
            let normalized = raw / total.max(0.01);
            let priority = if urgency > 0.7 {
                0
            } else if urgency > 0.4 {
                1
            } else {
                2
            };

            allocations.push(BudgetAllocation {
                process: *process,
                allocated_budget: normalized.clamp(0.01, 0.5),
                priority,
                rationale: format!("urgency={:.2} default={:.2}", urgency, default),
            });
        }

        allocations.sort_by_key(|a| a.priority);
        self.allocation_history
            .push((self.state.clone(), allocations.clone()));
        if self.allocation_history.len() > 50 {
            self.allocation_history.remove(0);
        }

        allocations
    }

    /// How many refinement iterations should the refinery loop use?
    pub fn recommended_iterations(&self) -> usize {
        let base = self.config.base_iterations;
        if !self.config.enable_dynamic_budget {
            return base;
        }
        let extra = (self.state.uncertainty * 3.0 + self.state.surprise * 2.0) as usize;
        (base + extra).min(self.config.max_iterations)
    }

    /// Should we shed load (reduce processing due to overload)?
    pub fn should_shed_load(&self) -> bool {
        self.state.cognitive_load > self.config.load_shedding_threshold
    }

    fn compute_process_urgencies(&self) -> HashMap<CognitiveProcess, f64> {
        let mut urgencies = HashMap::new();
        let s = &self.state;

        urgencies.insert(
            CognitiveProcess::Gather,
            s.surprise * 0.8 + s.curiosity * 0.2,
        );
        urgencies.insert(
            CognitiveProcess::Refine,
            s.uncertainty * 0.7 + (1.0 - s.confidence) * 0.3,
        );
        urgencies.insert(
            CognitiveProcess::DualReason,
            s.uncertainty * 0.6 + s.curiosity * 0.4,
        );
        urgencies.insert(
            CognitiveProcess::Verify,
            (1.0 - s.confidence) * 0.8 + s.uncertainty * 0.2,
        );
        urgencies.insert(
            CognitiveProcess::MetaMonitor,
            s.cognitive_load * 0.6 + s.uncertainty * 0.4,
        );
        urgencies.insert(
            CognitiveProcess::EvolutionAssess,
            s.boredom * 0.7 + s.curiosity * 0.3,
        );
        urgencies.insert(CognitiveProcess::Compete, s.uncertainty * 0.5);
        urgencies.insert(
            CognitiveProcess::BlackboardSync,
            s.uncertainty * 0.4 + s.surprise * 0.3,
        );
        urgencies.insert(CognitiveProcess::Gate, s.surprise * 0.5);
        urgencies.insert(CognitiveProcess::Propose, s.curiosity * 0.4);
        urgencies.insert(CognitiveProcess::Act, 1.0 - s.uncertainty);
        urgencies.insert(CognitiveProcess::Record, 0.3);
        urgencies.insert(CognitiveProcess::Sleep, s.boredom * 0.3);

        urgencies
    }
}

impl CognitiveProcess {
    fn all() -> Vec<CognitiveProcess> {
        vec![
            CognitiveProcess::Gather,
            CognitiveProcess::Gate,
            CognitiveProcess::Propose,
            CognitiveProcess::Compete,
            CognitiveProcess::Refine,
            CognitiveProcess::DualReason,
            CognitiveProcess::Verify,
            CognitiveProcess::Act,
            CognitiveProcess::Record,
            CognitiveProcess::MetaMonitor,
            CognitiveProcess::Sleep,
            CognitiveProcess::BlackboardSync,
            CognitiveProcess::EvolutionAssess,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AllocatorConfig::default();
        assert!(config.enable_dynamic_budget);
        assert_eq!(config.base_iterations, 3);
    }

    #[test]
    fn test_internal_state_defaults() {
        let state = InternalState::new();
        assert!((state.uncertainty - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_urgency_high_uncertainty() {
        let mut state = InternalState::new();
        state.uncertainty = 0.9;
        assert!(state.urgency() > 0.5);
    }

    #[test]
    fn test_allocate_returns_budgets() {
        let mut alloc = ConsciousResourceAllocator::new(AllocatorConfig::default());
        let budgets = alloc.allocate();
        assert!(!budgets.is_empty());
        assert!(budgets.iter().all(|b| b.allocated_budget > 0.0));
    }

    #[test]
    fn test_recommended_iterations() {
        let alloc = ConsciousResourceAllocator::new(AllocatorConfig::default());
        let iters = alloc.recommended_iterations();
        assert!(iters >= alloc.config().base_iterations);
    }

    #[test]
    fn test_surprise_affects_allocation() {
        let mut alloc = ConsciousResourceAllocator::new(AllocatorConfig::default());
        alloc.adjust_surprise(0.7);
        let budgets = alloc.allocate();
        let gather = budgets
            .iter()
            .find(|b| b.process == CognitiveProcess::Gather);
        assert!(gather.is_some());
    }

    #[test]
    fn test_attention_signal() {
        let mut state = InternalState::new();
        state.surprise = 0.9;
        assert_eq!(state.attention_signal(), "gather_new_info");
        state.surprise = 0.0;
        state.uncertainty = 0.8;
        assert_eq!(state.attention_signal(), "reduce_uncertainty");
        state.uncertainty = 0.0;
        state.boredom = 0.8;
        assert_eq!(state.attention_signal(), "seek_novelty");
    }

    #[test]
    fn test_load_shedding() {
        let mut alloc = ConsciousResourceAllocator::new(AllocatorConfig {
            load_shedding_threshold: 0.7,
            ..Default::default()
        });
        alloc.state.cognitive_load = 0.9;
        assert!(alloc.should_shed_load());
    }

    #[test]
    fn test_dynamic_budget_off() {
        let mut alloc = ConsciousResourceAllocator::new(AllocatorConfig {
            enable_dynamic_budget: false,
            ..Default::default()
        });
        alloc.adjust_surprise(0.9);
        let budgets = alloc.allocate();
        let total: f64 = budgets.iter().map(|b| b.allocated_budget).sum();
        assert!((total - 1.0).abs() < 0.1);
    }
}
