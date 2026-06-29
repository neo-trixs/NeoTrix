pub mod discovery;
pub mod goal;
pub mod group_evolution;
pub mod pipeline_graph;
pub mod state;
pub mod uat_gate;
pub mod verifier;

pub use discovery::*;
pub use goal::*;
pub use group_evolution::*;
pub use pipeline_graph::*;
pub use state::*;
pub use uat_gate::*;
pub use verifier::*;

use std::collections::HashMap;

use crate::core::nt_core_experience::HypothesisTreeConfig;

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum LoopPhase {
    Observe,
    Ideate,
    Assign,
    Execute,
    Verify,
    Persist,
    Decide,
}

impl LoopPhase {
    pub fn next(&self) -> Self {
        match self {
            LoopPhase::Observe => LoopPhase::Ideate,
            LoopPhase::Ideate => LoopPhase::Assign,
            LoopPhase::Assign => LoopPhase::Execute,
            LoopPhase::Execute => LoopPhase::Verify,
            LoopPhase::Verify => LoopPhase::Persist,
            LoopPhase::Persist => LoopPhase::Decide,
            LoopPhase::Decide => LoopPhase::Observe,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            LoopPhase::Observe => "gathering observations and gaps",
            LoopPhase::Ideate => "generating hypothesis candidates",
            LoopPhase::Assign => "prioritizing tasks",
            LoopPhase::Execute => "running pipeline",
            LoopPhase::Verify => "checking output",
            LoopPhase::Persist => "saving state",
            LoopPhase::Decide => "planning next cycle",
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LoopEngine {
    pub phase: LoopPhase,
    pub cycle: u64,
    pub goal: Option<LoopGoal>,
    pub state: LoopState,
    pub handler_registry: HashMap<String, bool>,
    pub verifier: LoopVerifier,
    pub discovery: HandlerDiscovery,
    pub group_pool: GroupExperiencePool,
    pub arbor_enabled: bool,
    pub hypothesis_tree_config: HypothesisTreeConfig,
    pub hypothesis_trees_created: u64,
    pub hypothesis_trees_verified: u64,
    pub hypothesis_trees_pruned: u64,
}

impl LoopEngine {
    pub fn new() -> Self {
        Self {
            phase: LoopPhase::Observe,
            cycle: 0,
            goal: None,
            state: LoopState::load(),
            handler_registry: HashMap::new(),
            verifier: LoopVerifier::new(),
            discovery: HandlerDiscovery::new(),
            group_pool: GroupExperiencePool::default(),
            arbor_enabled: true,
            hypothesis_tree_config: HypothesisTreeConfig::default(),
            hypothesis_trees_created: 0,
            hypothesis_trees_verified: 0,
            hypothesis_trees_pruned: 0,
        }
    }

    pub fn tick(&mut self) -> LoopPhase {
        self.cycle += 1;
        self.phase = self.phase.next();
        if self.cycle % 6 == 0 {
            self.state.save();
        }
        self.phase
    }

    pub fn current_label(&self) -> &'static str {
        self.phase.label()
    }

    pub fn stats(&self) -> LoopStats {
        let registered = self.handler_registry.len();
        let covered = self.handler_registry.values().filter(|&&v| v).count();
        let pool_stats = self.group_pool.stats();
        LoopStats {
            cycle: self.cycle,
            phase: self.phase,
            handlers_registered: registered,
            handlers_called: covered,
            coverage_pct: if registered > 0 {
                (covered as f64 / registered as f64) * 100.0
            } else {
                0.0
            },
            verifier_score: self.verifier.last_score,
            goal_active: self.goal.is_some(),
            pool_size: pool_stats.pool_size,
            transfer_events: pool_stats.transfer_events,
            diversity: pool_stats.diversity,
            arbor_enabled: self.arbor_enabled,
            hypothesis_trees_created: self.hypothesis_trees_created,
            hypothesis_trees_verified: self.hypothesis_trees_verified,
            hypothesis_trees_pruned: self.hypothesis_trees_pruned,
        }
    }

    pub fn record_experience(
        &mut self,
        branch: &str,
        context: &str,
        action: &str,
        outcome: &str,
        success: bool,
        utility: f64,
    ) -> u64 {
        self.group_pool.add_experience(
            branch, context, action, outcome, success, utility, self.cycle,
        )
    }

    pub fn gea_tick(&mut self) -> Vec<ExperienceRecord> {
        let current = self.current_label();
        let context = format!("phase:{}_cycle:{}", current, self.cycle);
        self.record_experience("loop_engine", &context, "tick", "cycle_advance", true, 0.5);
        self.group_pool
            .cross_seed("loop_engine", current, self.cycle)
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct LoopStats {
    pub cycle: u64,
    pub phase: LoopPhase,
    pub handlers_registered: usize,
    pub handlers_called: usize,
    pub coverage_pct: f64,
    pub verifier_score: f64,
    pub goal_active: bool,
    pub pool_size: usize,
    pub transfer_events: u64,
    pub diversity: f64,
    pub arbor_enabled: bool,
    pub hypothesis_trees_created: u64,
    pub hypothesis_trees_verified: u64,
    pub hypothesis_trees_pruned: u64,
}

impl Default for LoopEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loop_engine_new() {
        let e = LoopEngine::new();
        assert_eq!(e.phase, LoopPhase::Observe);
        assert_eq!(e.cycle, 0);
        assert!(e.goal.is_none());
    }

    #[test]
    fn test_loop_engine_tick_cycles_phase() {
        let mut e = LoopEngine::new();
        assert_eq!(e.tick(), LoopPhase::Ideate);
        assert_eq!(e.cycle, 1);
        assert_eq!(e.tick(), LoopPhase::Assign);
        assert_eq!(e.tick(), LoopPhase::Execute);
        assert_eq!(e.tick(), LoopPhase::Verify);
        assert_eq!(e.tick(), LoopPhase::Persist);
        assert_eq!(e.tick(), LoopPhase::Decide);
        assert_eq!(e.tick(), LoopPhase::Observe); // wraps around
        assert_eq!(e.cycle, 8);
    }

    #[test]
    fn test_loop_engine_phase_label() {
        assert_eq!(
            LoopPhase::Observe.label(),
            "gathering observations and gaps"
        );
        assert_eq!(
            LoopPhase::Ideate.label(),
            "generating hypothesis candidates"
        );
        assert_eq!(LoopPhase::Execute.label(), "running pipeline");
        assert_eq!(LoopPhase::Decide.label(), "planning next cycle");
    }

    #[test]
    fn test_loop_engine_stats_no_goal() {
        let e = LoopEngine::new();
        let s = e.stats();
        assert_eq!(s.cycle, 0);
        assert!(!s.goal_active);
    }

    #[test]
    fn test_loop_engine_stats_with_goal() {
        let mut e = LoopEngine::new();
        e.goal = Some(LoopGoal::new("test", 0.8, 0.7, 10));
        let s = e.stats();
        assert!(s.goal_active);
    }

    #[test]
    fn test_loop_phase_next_full_cycle() {
        let phases = [
            LoopPhase::Observe,
            LoopPhase::Ideate,
            LoopPhase::Assign,
            LoopPhase::Execute,
            LoopPhase::Verify,
            LoopPhase::Persist,
            LoopPhase::Decide,
        ];
        for i in 0..6 {
            assert_eq!(phases[i].next(), phases[i + 1]);
        }
        assert_eq!(LoopPhase::Decide.next(), LoopPhase::Observe);
    }

    #[test]
    fn test_loop_engine_handler_registry_tracking() {
        let mut e = LoopEngine::new();
        e.handler_registry.insert("test_handler".to_string(), true);
        let s = e.stats();
        assert_eq!(s.handlers_registered, 1);
        assert_eq!(s.handlers_called, 1);
        assert!((s.coverage_pct - 100.0).abs() < 0.01);
    }
}
