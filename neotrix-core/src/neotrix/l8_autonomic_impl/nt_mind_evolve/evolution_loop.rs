use serde::{Deserialize, Serialize};

use super::benchmark::{BenchmarkGate, BenchmarkSuite, EvolutionStrategy, MutationScope};
use super::egl::{EglStatus, EglTracker};
use super::trait_store::TraitStore;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionConfig {
    pub benchmark_suite: BenchmarkSuite,
    pub gate: BenchmarkGate,
    pub egl_window: usize,
    pub check_interval_cycles: u64,
    pub max_cycles: u64,
}

impl Default for EvolutionConfig {
    fn default() -> Self {
        Self {
            benchmark_suite: BenchmarkSuite::new("default", "0.1.0", Vec::new()),
            gate: BenchmarkGate::default(),
            egl_window: 10,
            check_interval_cycles: 1,
            max_cycles: 20,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleResult {
    pub iteration: u64,
    pub pass_rate: f64,
    pub scope: MutationScope,
    pub strategy: EvolutionStrategy,
    pub egl_status: EglStatus,
    pub accepted: bool,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct EvolutionLoop {
    pub config: EvolutionConfig,
    pub egl_tracker: EglTracker,
    pub trait_store: TraitStore,
    pub iteration: u64,
}

impl EvolutionLoop {
    pub fn new(config: EvolutionConfig) -> Self {
        Self {
            egl_tracker: EglTracker::new(config.egl_window, -0.05),
            trait_store: TraitStore::new(),
            iteration: 0,
            config,
        }
    }

    pub fn run_cycle(&mut self, benchmark_scores: &[(String, f64)]) -> CycleResult {
        self.iteration += 1;

        let pass_rate = self.calculate_pass_rate(benchmark_scores);
        let scope = MutationScope::from_pass_rate(pass_rate);
        let strategy = EvolutionStrategy::select(scope, pass_rate, self.iteration);
        let egl_status = self.egl_tracker.track(pass_rate);

        let mut accepted = !egl_status.is_regressing();
        let mut suggestions = Vec::new();

        if egl_status.is_regressing() {
            accepted = false;
            suggestions.push(format!(
                "EGL regression detected: pass_rate {:.2} below avg {:.2}",
                egl_status.current(),
                egl_status.avg()
            ));
        }

        if self.iteration >= self.config.max_cycles {
            suggestions.push(format!(
                "Max cycles ({}) reached, evolution stopping",
                self.config.max_cycles
            ));
        }

        CycleResult {
            iteration: self.iteration,
            pass_rate,
            scope,
            strategy,
            egl_status,
            accepted,
            suggestions,
        }
    }

    pub fn is_converged(&self, egl_threshold: f64) -> bool {
        if self.iteration < 2 {
            return false;
        }
        self.egl_tracker.is_converged(self.config.egl_window, egl_threshold)
    }

    fn calculate_pass_rate(&self, scores: &[(String, f64)]) -> f64 {
        if scores.is_empty() {
            return 0.0;
        }
        let sum: f64 = scores.iter().map(|(_, score)| score).sum();
        sum / scores.len() as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evolution_config_default() {
        let config = EvolutionConfig::default();
        assert_eq!(config.max_cycles, 20);
        assert!((config.gate.required_pass_rate - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_evolution_loop_new() {
        let config = EvolutionConfig::default();
        let loop_ = EvolutionLoop::new(config);
        assert_eq!(loop_.iteration, 0);
        assert!(loop_.trait_store.is_empty());
    }

    #[test]
    fn test_cycle_result_structure() {
        let config = EvolutionConfig::default();
        let mut loop_ = EvolutionLoop::new(config);

        let scores = vec![
            ("task1".into(), 0.8),
            ("task2".into(), 0.6),
            ("task3".into(), 0.7),
        ];

        let result = loop_.run_cycle(&scores);
        assert_eq!(result.iteration, 1);
        assert!((result.pass_rate - 0.7).abs() < 1e-6);
        assert_eq!(result.scope, MutationScope::Targeted);
        assert_eq!(result.strategy, EvolutionStrategy::GuidedSynthesis);
        assert!(result.accepted);
    }

    #[test]
    fn test_cycle_result_empty_scores() {
        let config = EvolutionConfig::default();
        let mut loop_ = EvolutionLoop::new(config);
        let result = loop_.run_cycle(&[]);
        assert_eq!(result.pass_rate, 0.0);
        assert_eq!(result.scope, MutationScope::Comprehensive);
        assert_eq!(result.strategy, EvolutionStrategy::AdaptiveEvolve);
    }

    #[test]
    fn test_cycle_result_regression_detected() {
        let config = EvolutionConfig::default();
        let mut loop_ = EvolutionLoop::new(config);

        loop_.run_cycle(&[("t1".into(), 0.9)]);
        loop_.run_cycle(&[("t1".into(), 0.8)]);
        let result = loop_.run_cycle(&[("t1".into(), 0.3)]);

        assert!(result.egl_status.is_regressing());
        assert!(!result.accepted);
        assert!(!result.suggestions.is_empty());
    }

    #[test]
    fn test_cycle_result_improvement_detected() {
        let config = EvolutionConfig::default();
        let mut loop_ = EvolutionLoop::new(config);

        loop_.run_cycle(&[("t1".into(), 0.3)]);
        let result = loop_.run_cycle(&[("t1".into(), 0.7)]);

        assert!(result.egl_status.is_improving());
        assert!(result.accepted);
    }

    #[test]
    fn test_is_converged() {
        let config = EvolutionConfig { egl_window: 3, ..EvolutionConfig::default() };
        let mut loop_ = EvolutionLoop::new(config);

        assert!(!loop_.is_converged(0.01));

        for _ in 0..5 {
            loop_.run_cycle(&[("t1".into(), 0.75)]);
        }
        assert!(loop_.is_converged(0.01));
    }

    #[test]
    fn test_evolution_strategy_in_cycle() {
        let config = EvolutionConfig::default();
        let mut loop_ = EvolutionLoop::new(config);

        let result = loop_.run_cycle(&[("t1".into(), 0.2)]);
        assert_eq!(result.strategy, EvolutionStrategy::AdaptiveEvolve);

        let result = loop_.run_cycle(&[("t1".into(), 0.5)]);
        assert_eq!(result.strategy, EvolutionStrategy::GuidedSynthesis);

        let mut loop2_ = EvolutionLoop::new(EvolutionConfig::default());
        for _ in 0..5 {
            loop2_.run_cycle(&[("t1".into(), 0.5)]);
        }
        assert_eq!(loop2_.iteration, 5);
    }
}
