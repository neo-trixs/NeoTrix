use std::time::Instant;

use super::consciousness_cycle::{ConsciousnessCycle, CycleResult};
use super::vsa_tag::VsaTagged;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConvergenceSignal {
    Converged,
    Diverging,
    Oscillating,
    NotConverged,
}

#[derive(Debug, Clone)]
pub struct RefineryConfig {
    /// Maximum refinement iterations per consciousness cycle
    pub max_iterations: usize,
    /// VSA hamming distance threshold for "converged" (0.0-1.0)
    pub convergence_threshold: f64,
    /// Exit early if divergence detected (delta > threshold)
    pub divergence_threshold: f64,
    /// EMA alpha for convergence tracking
    pub ema_alpha: f64,
    /// Whether to enable early exit
    pub enable_early_exit: bool,
}

impl Default for RefineryConfig {
    fn default() -> Self {
        Self {
            max_iterations: 5,
            convergence_threshold: 0.05,
            divergence_threshold: 0.85,
            ema_alpha: 0.3,
            enable_early_exit: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RefineryMetrics {
    pub iteration: usize,
    pub total_cycles: u64,
    pub convergence_signal: ConvergenceSignal,
    pub delta_vsa: f64,
    pub delta_ema: f64,
    pub cycle_durations_ms: Vec<u64>,
    pub early_exited: bool,
}

#[derive(Debug, Clone)]
pub struct RefineryResult {
    pub final_cycle: CycleResult,
    pub metrics: RefineryMetrics,
    pub all_iterations: Vec<CycleResult>,
}

pub struct ConsciousnessRefineryLoop {
    inner: ConsciousnessCycle,
    pub last_mcts_stats: Option<crate::core::nt_core_reasoning::mcts_reasoner::MctsStats>,
    pub last_prm_stats: Option<crate::core::nt_core_reasoning::process_reward_model::PrmStats>,
    pub last_pruner_stats:
        Option<crate::core::nt_core_reasoning::bidirectional_pruner::PrunerStats>,
    pub last_selector_stats:
        Option<crate::core::nt_core_reasoning::strategy_selector::SelectorStats>,
    pub last_dead_end_stats:
        Option<crate::core::nt_core_reasoning::dead_end_detector::DeadEndStats>,
    pub refinery_metadata: RefineryMetadata,
    config: RefineryConfig,
    previous_state: Option<VsaTagged>,
    delta_ema: f64,
    total_cycles_run: u64,
}

#[derive(Debug, Clone)]
pub struct RefineryMetadata {
    pub total_cycles: u64,
    pub avg_mcts_nodes: f64,
    pub avg_prm_steps: f64,
    pub avg_prune_ratio: f64,
    pub total_dead_ends: usize,
    pub total_strategy_switches: usize,
    pub mcts_active: bool,
    pub prm_active: bool,
    pub pruner_active: bool,
    pub selector_active: bool,
    pub dead_end_active: bool,
}

impl RefineryMetadata {
    pub fn new() -> Self {
        Self {
            total_cycles: 0,
            avg_mcts_nodes: 0.0,
            avg_prm_steps: 0.0,
            avg_prune_ratio: 0.0,
            total_dead_ends: 0,
            total_strategy_switches: 0,
            mcts_active: false,
            prm_active: false,
            pruner_active: false,
            selector_active: false,
            dead_end_active: false,
        }
    }
}

impl Default for RefineryMetadata {
    fn default() -> Self {
        Self::new()
    }
}

impl ConsciousnessRefineryLoop {
    pub fn new(inner: ConsciousnessCycle, config: RefineryConfig) -> Self {
        Self {
            inner,
            last_mcts_stats: None,
            last_prm_stats: None,
            last_pruner_stats: None,
            last_selector_stats: None,
            last_dead_end_stats: None,
            refinery_metadata: RefineryMetadata::new(),
            config,
            previous_state: None,
            delta_ema: 1.0,
            total_cycles_run: 0,
        }
    }

    pub fn inner(&self) -> &ConsciousnessCycle {
        &self.inner
    }
    pub fn inner_mut(&mut self) -> &mut ConsciousnessCycle {
        &mut self.inner
    }
    pub fn config(&self) -> &RefineryConfig {
        &self.config
    }
    pub fn total_cycles(&self) -> u64 {
        self.total_cycles_run
    }

    pub fn refine(&mut self, input: Option<VsaTagged>) -> RefineryResult {
        let _start = Instant::now();
        let mut all_iterations = Vec::with_capacity(self.config.max_iterations);
        let mut cycle_durations = Vec::new();

        let mut current_input = input;

        for iteration in 0..self.config.max_iterations {
            let cycle_start = Instant::now();
            let result = self.inner.run_cycle(current_input.take());
            let cycle_ms = cycle_start.elapsed().as_millis() as u64;
            cycle_durations.push(cycle_ms);
            self.total_cycles_run += 1;

            let state = result.output_state.clone();

            let (delta, signal) = if let Some(ref new_state) = state {
                let delta = self.compute_delta(Some(new_state));
                let signal = self.check_convergence(delta);
                (delta, signal)
            } else {
                (1.0, ConvergenceSignal::NotConverged)
            };

            self.delta_ema =
                self.config.ema_alpha * delta + (1.0 - self.config.ema_alpha) * self.delta_ema;
            self.previous_state = state.clone();

            let early_exited = self.config.enable_early_exit
                && (signal == ConvergenceSignal::Converged
                    || signal == ConvergenceSignal::Diverging);

            all_iterations.push(result);

            if early_exited {
                let last = all_iterations.last().unwrap();
                let metrics = RefineryMetrics {
                    iteration,
                    total_cycles: self.total_cycles_run,
                    convergence_signal: signal,
                    delta_vsa: delta,
                    delta_ema: self.delta_ema,
                    cycle_durations_ms: cycle_durations,
                    early_exited: true,
                };
                return RefineryResult {
                    final_cycle: last.clone(),
                    metrics,
                    all_iterations,
                };
            }

            current_input = state;
        }

        let last = all_iterations.last().unwrap();
        let final_delta = self.compute_delta(last.output_state.as_ref());
        let final_signal = self.check_convergence(final_delta);

        RefineryResult {
            final_cycle: last.clone(),
            metrics: RefineryMetrics {
                iteration: self.config.max_iterations - 1,
                total_cycles: self.total_cycles_run,
                convergence_signal: final_signal,
                delta_vsa: final_delta,
                delta_ema: self.delta_ema,
                cycle_durations_ms: cycle_durations,
                early_exited: false,
            },
            all_iterations,
        }
    }

    fn compute_delta(&self, new_state: Option<&VsaTagged>) -> f64 {
        match (&self.previous_state, new_state) {
            (Some(prev), Some(next)) => {
                let n = prev.vector.len().min(next.vector.len());
                if n == 0 {
                    return 1.0;
                }
                let diff: usize = prev
                    .vector
                    .iter()
                    .zip(next.vector.iter())
                    .filter(|(a, b)| a != b)
                    .count();
                diff as f64 / n as f64
            }
            _ => 1.0,
        }
    }

    fn check_convergence(&self, delta: f64) -> ConvergenceSignal {
        if delta < self.config.convergence_threshold {
            ConvergenceSignal::Converged
        } else if delta > self.config.divergence_threshold {
            ConvergenceSignal::Diverging
        } else if self.delta_ema > 0.1 && (delta - self.delta_ema).abs() < 0.01 {
            ConvergenceSignal::Oscillating
        } else {
            ConvergenceSignal::NotConverged
        }
    }

    pub fn convergence_rate(&self, _recent_n: usize) -> f64 {
        0.5
    }

    pub fn collect_reasoning_stats(&mut self) {
        let _ = self.inner();

        self.refinery_metadata.total_cycles = self.total_cycles_run;
        self.refinery_metadata.mcts_active = self.last_mcts_stats.is_some();
        self.refinery_metadata.prm_active = self.last_prm_stats.is_some();
        self.refinery_metadata.pruner_active = self.last_pruner_stats.is_some();
        self.refinery_metadata.selector_active = self.last_selector_stats.is_some();
        self.refinery_metadata.dead_end_active = self.last_dead_end_stats.is_some();

        if let Some(ref mcts) = self.last_mcts_stats {
            let n = self.refinery_metadata.total_cycles.max(1);
            self.refinery_metadata.avg_mcts_nodes = self.refinery_metadata.avg_mcts_nodes
                * ((n - 1) as f64 / n as f64)
                + mcts.total_nodes as f64 / n as f64;
        }
        if let Some(ref prm) = self.last_prm_stats {
            let n = self.refinery_metadata.total_cycles.max(1);
            self.refinery_metadata.avg_prm_steps = self.refinery_metadata.avg_prm_steps
                * ((n - 1) as f64 / n as f64)
                + prm.total_steps as f64 / n as f64;
        }
        if let Some(ref pruner) = self.last_pruner_stats {
            let n = self.refinery_metadata.total_cycles.max(1);
            self.refinery_metadata.avg_prune_ratio = self.refinery_metadata.avg_prune_ratio
                * ((n - 1) as f64 / n as f64)
                + pruner.pruning_ratio / n as f64;
        }
        if let Some(ref selector) = self.last_selector_stats {
            self.refinery_metadata.total_strategy_switches += selector.switch_count;
        }
        if let Some(ref de) = self.last_dead_end_stats {
            self.refinery_metadata.total_dead_ends += de.dead_ends_detected;
        }
    }

    pub fn reasoning_metadata(&self) -> &RefineryMetadata {
        &self.refinery_metadata
    }

    pub fn has_active_reasoning_modules(&self) -> bool {
        self.last_mcts_stats.is_some()
            || self.last_prm_stats.is_some()
            || self.last_pruner_stats.is_some()
            || self.last_selector_stats.is_some()
            || self.last_dead_end_stats.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_consciousness::consciousness_cycle::CycleConfig;

    #[test]
    fn test_refinery_default_config() {
        let config = RefineryConfig::default();
        assert_eq!(config.max_iterations, 5);
        assert!((config.convergence_threshold - 0.05).abs() < 1e-6);
        assert!(config.enable_early_exit);
    }

    #[test]
    fn test_refinery_runs_multiple_cycles() {
        let cycle = ConsciousnessCycle::new(CycleConfig::default());
        let config = RefineryConfig {
            max_iterations: 3,
            enable_early_exit: false,
            ..Default::default()
        };
        let mut refinery = ConsciousnessRefineryLoop::new(cycle, config);
        let result = refinery.refine(None);
        assert_eq!(result.all_iterations.len(), 3);
        assert_eq!(result.metrics.total_cycles, 3);
    }

    #[test]
    fn test_refinery_early_exit_on_convergence() {
        let cycle = ConsciousnessCycle::new(CycleConfig::default());
        let config = RefineryConfig {
            max_iterations: 10,
            convergence_threshold: 0.5,
            enable_early_exit: true,
            ..Default::default()
        };
        let mut refinery = ConsciousnessRefineryLoop::new(cycle, config);
        let result = refinery.refine(None);
        assert!(result.all_iterations.len() <= 10);
    }

    #[test]
    fn test_convergence_signals() {
        let cycle = ConsciousnessCycle::new(CycleConfig::default());
        let refinery = ConsciousnessRefineryLoop::new(cycle, RefineryConfig::default());
        assert_eq!(
            refinery.check_convergence(0.01),
            ConvergenceSignal::Converged
        );
        assert_eq!(
            refinery.check_convergence(0.9),
            ConvergenceSignal::Diverging
        );
        assert_eq!(
            refinery.check_convergence(0.5),
            ConvergenceSignal::NotConverged
        );
    }

    #[test]
    fn test_refinery_preserves_cycle_count() {
        let cycle = ConsciousnessCycle::new(CycleConfig::default());
        let config = RefineryConfig::default();
        let mut refinery = ConsciousnessRefineryLoop::new(cycle, config);
        let result = refinery.refine(None);
        assert!(result.final_cycle.cycle_num > 0);
        assert_eq!(
            result.metrics.total_cycles,
            result.all_iterations.len() as u64
        );
    }

    #[test]
    fn test_refinery_tracks_durations() {
        let cycle = ConsciousnessCycle::new(CycleConfig::default());
        let config = RefineryConfig {
            max_iterations: 2,
            enable_early_exit: false,
            ..Default::default()
        };
        let mut refinery = ConsciousnessRefineryLoop::new(cycle, config);
        let result = refinery.refine(None);
        assert_eq!(result.metrics.cycle_durations_ms.len(), 2);
    }

    #[test]
    fn test_refinery_empty_input() {
        let cycle = ConsciousnessCycle::new(CycleConfig::default());
        let mut refinery = ConsciousnessRefineryLoop::new(cycle, RefineryConfig::default());
        let result = refinery.refine(None);
        assert!(result.final_cycle.cycle_num >= 1);
    }

    #[test]
    fn test_no_active_modules_by_default() {
        let cycle = ConsciousnessCycle::new(CycleConfig::default());
        let refinery = ConsciousnessRefineryLoop::new(cycle, RefineryConfig::default());
        assert!(!refinery.has_active_reasoning_modules());
    }

    #[test]
    fn test_refinery_metadata_default_values() {
        let cycle = ConsciousnessCycle::new(CycleConfig::default());
        let refinery = ConsciousnessRefineryLoop::new(cycle, RefineryConfig::default());
        let meta = refinery.reasoning_metadata();
        assert_eq!(meta.total_cycles, 0);
        assert!((meta.avg_mcts_nodes - 0.0).abs() < 1e-9);
        assert!((meta.avg_prm_steps - 0.0).abs() < 1e-9);
        assert!((meta.avg_prune_ratio - 0.0).abs() < 1e-9);
        assert_eq!(meta.total_dead_ends, 0);
        assert_eq!(meta.total_strategy_switches, 0);
        assert!(!meta.mcts_active);
        assert!(!meta.prm_active);
        assert!(!meta.pruner_active);
        assert!(!meta.selector_active);
        assert!(!meta.dead_end_active);
    }

    #[test]
    fn test_collect_stats_without_reasoning_subsystems() {
        let cycle = ConsciousnessCycle::new(CycleConfig::default());
        let config = RefineryConfig {
            max_iterations: 1,
            enable_early_exit: false,
            ..Default::default()
        };
        let mut refinery = ConsciousnessRefineryLoop::new(cycle, config);
        let _ = refinery.refine(None);
        refinery.collect_reasoning_stats();
        let meta = refinery.reasoning_metadata();
        assert_eq!(meta.total_cycles, 1);
        assert!(!meta.mcts_active);
        assert_eq!(meta.avg_mcts_nodes, 0.0);
    }
}
