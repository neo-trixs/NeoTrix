#![forbid(unsafe_code)]

use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};

/// Type of exploration action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ExplorationType {
    Search,
    Reason,
    Synthesize,
    Experiment,
}

/// An exploration action emitted by the explorer.
#[derive(Debug, Clone)]
pub struct ExplorationAction {
    pub action_type: ExplorationType,
    pub query: String,
    pub timestamp: u64,
}

/// Configuration for the NativeEvolutionExplorer.
#[derive(Debug, Clone)]
pub struct ExplorerConfig {
    /// Cognitive load below which exploration is triggered.
    pub load_threshold: f64,
    /// Minimum knowledge gain to consider an exploration worthwhile.
    pub min_knowledge_gain: f64,
    /// Cycles to wait between explorations.
    pub cooldown_cycles: u64,
    /// Maximum number of trajectories to retain.
    pub max_trajectories: usize,
}

impl Default for ExplorerConfig {
    fn default() -> Self {
        Self {
            load_threshold: 0.3,
            min_knowledge_gain: 0.01,
            cooldown_cycles: 10,
            max_trajectories: 100,
        }
    }
}

/// A recorded exploration trajectory.
#[derive(Debug, Clone)]
pub struct ExplorationTrajectory {
    pub action_type: ExplorationType,
    pub query: String,
    pub knowledge_gain: f64,
    pub token_cost: f64,
    pub score: f64,
    pub timestamp: u64,
}

/// Aggregate statistics for the explorer.
#[derive(Debug, Clone)]
pub struct ExplorerStats {
    pub total_explorations: u64,
    pub total_knowledge_gain: f64,
    pub total_token_cost: f64,
    pub mean_score: f64,
    pub max_score: f64,
    pub min_score: f64,
    pub active_trajectories: usize,
}

/// Qwen3-30B-style reward-free spontaneous evolution explorer.
///
/// At low cognitive load (< load_threshold), the explorer automatically enters
/// exploration mode: it selects an exploration action type based on attractor
/// state entropy (proxy for knowledge gap). Intrinsic curiosity is driven by
/// N_total curvature via exploration_score = knowledge_gain / token_cost,
/// with no external reward signal.
pub struct NativeEvolutionExplorer {
    config: ExplorerConfig,
    curiosity_signal: f64,
    knowledge_gain_history: VecDeque<f64>,
    exploration_trajectories: VecDeque<ExplorationTrajectory>,
    cycle: u64,
    cycles_since_last: u64,
}

impl NativeEvolutionExplorer {
    pub fn new() -> Self {
        Self::with_config(ExplorerConfig::default())
    }

    pub fn with_config(config: ExplorerConfig) -> Self {
        Self {
            curiosity_signal: 0.0,
            knowledge_gain_history: VecDeque::with_capacity(config.max_trajectories),
            exploration_trajectories: VecDeque::with_capacity(config.max_trajectories),
            cycle: 0,
            cycles_since_last: 0,
            config,
        }
    }

    /// Tick the explorer. Returns an exploration action when cognitive load is
    /// below threshold and cooldown has elapsed.
    pub fn tick(
        &mut self,
        cognitive_load: f64,
        attractor_state: &[u8],
    ) -> Option<ExplorationAction> {
        self.cycle += 1;
        self.cycles_since_last += 1;

        if cognitive_load >= self.config.load_threshold {
            return None;
        }
        if self.cycles_since_last < self.config.cooldown_cycles {
            return None;
        }

        let action = self.suggest_action(attractor_state);
        self.cycles_since_last = 0;
        Some(action)
    }

    /// Suggest an exploration action based on attractor state entropy.
    ///
    /// High entropy → more uncertainty → Search/Reason (gather information).
    /// Low entropy → existing structure → Synthesize/Experiment (test/combine).
    /// Empty state → complete uncertainty → Search.
    fn suggest_action(&self, attractor_state: &[u8]) -> ExplorationAction {
        let entropy = state_entropy(attractor_state);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let action_type = if attractor_state.is_empty() || entropy > 0.8 {
            ExplorationType::Search
        } else if entropy > 0.6 {
            ExplorationType::Reason
        } else if entropy > 0.3 {
            ExplorationType::Synthesize
        } else {
            ExplorationType::Experiment
        };

        let query = format!(
            "explore:type={:?} entropy={:.2} cycle={}",
            action_type, entropy, self.cycle
        );

        ExplorationAction {
            action_type,
            query,
            timestamp: now,
        }
    }

    /// Record the outcome of an exploration action.
    pub fn record_outcome(
        &mut self,
        action: &ExplorationAction,
        knowledge_gain: f64,
        token_cost: f64,
    ) {
        let score = if token_cost > 0.0 {
            knowledge_gain / token_cost
        } else {
            0.0
        };

        let trajectory = ExplorationTrajectory {
            action_type: action.action_type,
            query: action.query.clone(),
            knowledge_gain,
            token_cost,
            score,
            timestamp: action.timestamp,
        };

        self.exploration_trajectories.push_back(trajectory);
        while self.exploration_trajectories.len() > self.config.max_trajectories {
            self.exploration_trajectories.pop_front();
        }

        self.knowledge_gain_history.push_back(score);
        while self.knowledge_gain_history.len() > self.config.max_trajectories {
            self.knowledge_gain_history.pop_front();
        }

        self.curiosity_signal = self
            .knowledge_gain_history
            .iter()
            .copied()
            .filter(|&s| s > 0.0)
            .sum::<f64>()
            / (self.knowledge_gain_history.len() as f64).max(1.0);
    }

    /// Current exploration efficiency score: mean knowledge_gain / token_cost
    /// over recent history.
    pub fn exploration_score(&self) -> f64 {
        let n = self.knowledge_gain_history.len();
        if n == 0 {
            return 0.0;
        }
        self.knowledge_gain_history.iter().sum::<f64>() / n as f64
    }

    pub fn stats(&self) -> ExplorerStats {
        let total_explorations = self.exploration_trajectories.len() as u64;
        let total_knowledge_gain: f64 = self
            .exploration_trajectories
            .iter()
            .map(|t| t.knowledge_gain)
            .sum();
        let total_token_cost: f64 = self
            .exploration_trajectories
            .iter()
            .map(|t| t.token_cost)
            .sum();

        let (mean_score, max_score, min_score) = if self.exploration_trajectories.is_empty() {
            (0.0, 0.0, 0.0)
        } else {
            let scores: Vec<f64> = self
                .exploration_trajectories
                .iter()
                .map(|t| t.score)
                .collect();
            let mean = scores.iter().sum::<f64>() / scores.len() as f64;
            let max = scores.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            let min = scores.iter().copied().fold(f64::INFINITY, f64::min);
            (mean, max, min)
        };

        ExplorerStats {
            total_explorations,
            total_knowledge_gain,
            total_token_cost,
            mean_score,
            max_score,
            min_score,
            active_trajectories: self.exploration_trajectories.len(),
        }
    }
}

impl Default for NativeEvolutionExplorer {
    fn default() -> Self {
        Self::new()
    }
}

/// Compute a normalized byte-level Shannon entropy proxy from attractor state.
///
/// Returns a value in [0.0, 1.0], where 1.0 corresponds to maximum uncertainty
/// (all 256 byte values equally likely) and 0.0 corresponds to a single
/// repeated byte. Empty state returns 1.0 (complete uncertainty).
fn state_entropy(state: &[u8]) -> f64 {
    if state.is_empty() {
        return 1.0;
    }
    let mut counts = [0u64; 256];
    for &b in state {
        counts[b as usize] += 1;
    }
    let len = state.len() as f64;
    let entropy: f64 = counts
        .iter()
        .filter(|&&c| c > 0)
        .map(|&c| {
            let p = c as f64 / len;
            -p * p.log2()
        })
        .sum();
    (entropy / 8.0).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_explorer() -> NativeEvolutionExplorer {
        NativeEvolutionExplorer::new()
    }

    #[test]
    fn test_new_defaults() {
        let e = default_explorer();
        assert_eq!(e.cycle, 0);
        assert_eq!(e.cycles_since_last, 0);
        assert!(e.knowledge_gain_history.is_empty());
        assert!(e.exploration_trajectories.is_empty());
        assert!((e.config.load_threshold - 0.3).abs() < 1e-9);
    }

    #[test]
    fn test_low_load_triggers_exploration() {
        let mut e = default_explorer();
        e.cycle = 10;
        e.cycles_since_last = 10;
        let action = e.tick(0.1, &[0u8; 32]);
        assert!(action.is_some(), "Low load should trigger exploration");
    }

    #[test]
    fn test_high_load_suppresses_exploration() {
        let mut e = default_explorer();
        e.cycle = 10;
        e.cycles_since_last = 10;
        let action = e.tick(0.9, &[0u8; 32]);
        assert!(action.is_none(), "High load should suppress exploration");
    }

    #[test]
    fn test_cooldown_respected() {
        let mut e = default_explorer();
        e.cycle = 20;
        e.cycles_since_last = 1;
        let action = e.tick(0.1, &[0u8; 32]);
        assert!(action.is_none(), "Cooldown should prevent exploration");
    }

    #[test]
    fn test_exploration_resets_cooldown() {
        let mut e = default_explorer();
        e.cycle = 10;
        e.cycles_since_last = 10;
        let _ = e.tick(0.1, &[0u8; 32]);
        assert_eq!(e.cycles_since_last, 0);
        let action = e.tick(0.1, &[0u8; 32]);
        assert!(action.is_none(), "Cooldown should reset after exploration");
    }

    #[test]
    fn test_record_outcome_tracks_metrics() {
        let mut e = default_explorer();
        let action = ExplorationAction {
            action_type: ExplorationType::Search,
            query: "test".into(),
            timestamp: 1000,
        };
        e.record_outcome(&action, 0.5, 2.0);
        let stats = e.stats();
        assert_eq!(stats.total_explorations, 1);
        assert!((stats.total_knowledge_gain - 0.5).abs() < 1e-9);
        assert!((stats.total_token_cost - 2.0).abs() < 1e-9);
    }

    #[test]
    fn test_exploration_score_computed() {
        let mut e = default_explorer();
        assert!((e.exploration_score() - 0.0).abs() < 1e-9);

        let action = ExplorationAction {
            action_type: ExplorationType::Reason,
            query: "t1".into(),
            timestamp: 1,
        };
        e.record_outcome(&action, 0.4, 2.0);
        assert!((e.exploration_score() - 0.2).abs() < 1e-9);

        let action2 = ExplorationAction {
            action_type: ExplorationType::Search,
            query: "t2".into(),
            timestamp: 2,
        };
        e.record_outcome(&action2, 0.6, 2.0);
        assert!((e.exploration_score() - 0.25).abs() < 1e-9);
    }

    #[test]
    fn test_stats_accuracy() {
        let mut e = default_explorer();

        let a1 = ExplorationAction {
            action_type: ExplorationType::Search,
            query: "a".into(),
            timestamp: 1,
        };
        e.record_outcome(&a1, 1.0, 4.0);
        let a2 = ExplorationAction {
            action_type: ExplorationType::Synthesize,
            query: "b".into(),
            timestamp: 2,
        };
        e.record_outcome(&a2, 2.0, 4.0);

        let s = e.stats();
        assert_eq!(s.total_explorations, 2);
        assert!((s.total_knowledge_gain - 3.0).abs() < 1e-9);
        assert!((s.total_token_cost - 8.0).abs() < 1e-9);
        assert!((s.mean_score - 0.375).abs() < 1e-9);
        assert!((s.max_score - 0.5).abs() < 1e-9);
        assert!((s.min_score - 0.25).abs() < 1e-9);
    }

    #[test]
    fn test_trajectory_limit_enforced() {
        let config = ExplorerConfig {
            max_trajectories: 5,
            ..Default::default()
        };
        let mut e = NativeEvolutionExplorer::with_config(config);

        for i in 0..20 {
            let action = ExplorationAction {
                action_type: ExplorationType::Experiment,
                query: format!("t{}", i),
                timestamp: i as u64,
            };
            e.record_outcome(&action, 0.1, 1.0);
        }

        assert_eq!(e.exploration_trajectories.len(), 5);
        assert_eq!(e.knowledge_gain_history.len(), 5);
        let first = e.exploration_trajectories.front().unwrap();
        assert_eq!(first.query, "t15");
        let last = e
            .exploration_trajectories
            .back()
            .expect("5 trajectories were just inserted");
        assert_eq!(last.query, "t19");
    }

    #[test]
    fn test_empty_state_triggers_search() {
        let mut e = default_explorer();
        e.cycle = 10;
        e.cycles_since_last = 10;
        let action = e.tick(0.1, &[]).unwrap();
        assert_eq!(action.action_type, ExplorationType::Search);
    }

    #[test]
    fn test_uniform_state_triggers_experiment() {
        let mut e = default_explorer();
        e.cycle = 20;
        e.cycles_since_last = 10;
        let uniform = vec![42u8; 64];
        let action = e.tick(0.1, &uniform).unwrap();
        assert_eq!(action.action_type, ExplorationType::Experiment);
    }

    #[test]
    fn test_curiosity_signal_updates() {
        let mut e = default_explorer();
        assert!((e.curiosity_signal - 0.0).abs() < 1e-9);

        let a = ExplorationAction {
            action_type: ExplorationType::Search,
            query: "x".into(),
            timestamp: 1,
        };
        e.record_outcome(&a, 0.5, 2.0);
        assert!((e.curiosity_signal - 0.25).abs() < 1e-9);
    }

    #[test]
    fn test_stats_empty() {
        let e = default_explorer();
        let s = e.stats();
        assert_eq!(s.total_explorations, 0);
        assert_eq!(s.active_trajectories, 0);
        assert!((s.mean_score - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_with_config_overrides() {
        let config = ExplorerConfig {
            load_threshold: 0.5,
            cooldown_cycles: 3,
            ..Default::default()
        };
        let e = NativeEvolutionExplorer::with_config(config);
        assert!((e.config.load_threshold - 0.5).abs() < 1e-9);
        assert_eq!(e.config.cooldown_cycles, 3);
    }

    #[test]
    fn test_default_config_values() {
        let c = ExplorerConfig::default();
        assert!((c.load_threshold - 0.3).abs() < 1e-9);
        assert!((c.min_knowledge_gain - 0.01).abs() < 1e-9);
        assert_eq!(c.cooldown_cycles, 10);
        assert_eq!(c.max_trajectories, 100);
    }

    #[test]
    fn test_cycle_increments_on_tick() {
        let mut e = default_explorer();
        assert_eq!(e.cycle, 0);
        let _ = e.tick(0.1, &[0u8; 16]);
        assert_eq!(e.cycle, 1);
        let _ = e.tick(0.9, &[0u8; 16]);
        assert_eq!(e.cycle, 2);
    }

    #[test]
    fn test_state_entropy_boundaries() {
        assert!((state_entropy(&[]) - 1.0).abs() < 1e-9);
        assert!((state_entropy(&[0u8; 64]) - 0.0).abs() < 1e-9);
        let all_unique: Vec<u8> = (0..64).collect();
        let e = state_entropy(&all_unique);
        assert!(e > 0.0, "entropy should be > 0 for varied data, got {}", e);
    }

    #[test]
    fn test_exploration_score_zero_cost() {
        let mut e = default_explorer();
        let a = ExplorationAction {
            action_type: ExplorationType::Search,
            query: "z".into(),
            timestamp: 0,
        };
        e.record_outcome(&a, 0.5, 0.0);
        let s = e.stats();
        assert!(
            (s.mean_score - 0.0).abs() < 1e-9,
            "zero cost should yield 0 score"
        );
    }
}
