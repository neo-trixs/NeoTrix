use serde::{Deserialize, Serialize};

use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ReasoningMode {
    Analytical,
    Creative,
    Exploratory,
    Recovery,
    Execution,
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ModeTransition {
    Upgrade,
    Degrade,
    Maintain,
    Switch,
}

#[derive(Debug, Clone)]
pub struct ReasoningModeEntry {
    pub mode: ReasoningMode,
    pub signature_vector: Vec<u8>,
    pub success_rate: f64,
    pub invocation_count: u64,
    pub last_used: u64,
    pub creation_step: u64,
}

#[derive(Debug, Clone)]
pub struct EmergentReasoningConfig {
    pub min_emergence_frequency: usize,
    pub mode_decay_rate: f64,
    pub max_modes: usize,
    pub emergence_threshold: f64,
}

impl Default for EmergentReasoningConfig {
    fn default() -> Self {
        Self {
            min_emergence_frequency: 3,
            mode_decay_rate: 0.1,
            max_modes: 12,
            emergence_threshold: 0.6,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StrategyScore {
    pub mode: ReasoningMode,
    pub score: f64,
    pub confidence: f64,
}

fn mode_to_idx(mode: &ReasoningMode) -> usize {
    match mode {
        ReasoningMode::Analytical => 0,
        ReasoningMode::Creative => 1,
        ReasoningMode::Exploratory => 2,
        ReasoningMode::Recovery => 3,
        ReasoningMode::Execution => 4,
        ReasoningMode::Default => 5,
    }
}

fn idx_to_mode(idx: usize) -> ReasoningMode {
    match idx {
        0 => ReasoningMode::Analytical,
        1 => ReasoningMode::Creative,
        2 => ReasoningMode::Exploratory,
        3 => ReasoningMode::Recovery,
        4 => ReasoningMode::Execution,
        _ => ReasoningMode::Default,
    }
}

pub struct ReasoningNavigator {
    pub q_values: [f64; 6],
    pub learning_rate: f64,
    pub exploration_rate: f64,
    pub exploration_decay: f64,
    pub mode_performance: [Vec<f64>; 6],
    pub max_history: usize,
    pub decisions: u64,
}

impl ReasoningNavigator {
    pub fn new() -> Self {
        Self {
            q_values: [0.5; 6],
            learning_rate: 0.1,
            exploration_rate: 0.3,
            exploration_decay: 0.995,
            mode_performance: [
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
            ],
            max_history: 20,
            decisions: 0,
        }
    }

    pub fn select_strategy(&mut self) -> ReasoningMode {
        self.decisions += 1;

        if rand::random::<f64>() < self.exploration_rate {
            let idx = (rand::random::<f64>() * 6.0) as usize % 6;
            return idx_to_mode(idx);
        }

        let best_idx = self
            .q_values
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(idx, _)| idx)
            .unwrap_or(0);

        idx_to_mode(best_idx)
    }

    pub fn update_from_outcome(&mut self, mode: ReasoningMode, success: bool, quality: f64) {
        let idx = mode_to_idx(&mode);
        let reward = if success { quality } else { -quality * 0.5 };

        self.mode_performance[idx].push(reward);
        if self.mode_performance[idx].len() > self.max_history {
            self.mode_performance[idx].remove(0);
        }

        let current_q = self.q_values[idx];
        let td_error = reward - current_q;
        self.q_values[idx] = current_q + self.learning_rate * td_error;

        self.exploration_rate *= self.exploration_decay;
        self.exploration_rate = self.exploration_rate.max(0.01);
    }

    pub fn best_strategy(&self) -> ReasoningMode {
        let best_idx = self
            .q_values
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(idx, _)| idx)
            .unwrap_or(0);
        idx_to_mode(best_idx)
    }

    pub fn strategy_scores(&self) -> Vec<StrategyScore> {
        (0..6)
            .map(|i| {
                let mode = idx_to_mode(i);
                let perf = &self.mode_performance[i];
                let confidence = if perf.len() < 3 {
                    0.3
                } else {
                    let mean = perf.iter().sum::<f64>() / perf.len() as f64;
                    let variance =
                        perf.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / perf.len() as f64;
                    1.0 - (variance.sqrt()).clamp(0.0, 1.0) * 0.5
                };
                StrategyScore {
                    mode,
                    score: self.q_values[i],
                    confidence,
                }
            })
            .collect()
    }

    pub fn reset(&mut self) {
        self.q_values = [0.5; 6];
        self.exploration_rate = 0.3;
        self.mode_performance = [
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        ];
        self.decisions = 0;
    }
}

pub struct EmergentReasoningMode {
    modes: Vec<ReasoningModeEntry>,
    current_mode: ReasoningMode,
    config: EmergentReasoningConfig,
    transition_history: Vec<(ReasoningMode, ReasoningMode, ModeTransition, u64)>,
    step: u64,
    pub navigator: ReasoningNavigator,
}

impl EmergentReasoningMode {
    pub fn new(config: EmergentReasoningConfig) -> Self {
        let now = 0;
        Self {
            modes: vec![ReasoningModeEntry {
                mode: ReasoningMode::Default,
                signature_vector: vec![0u8; 4096],
                success_rate: 0.5,
                invocation_count: 0,
                last_used: now,
                creation_step: now,
            }],
            current_mode: ReasoningMode::Default,
            config,
            transition_history: Vec::new(),
            step: 0,
            navigator: ReasoningNavigator::new(),
        }
    }

    fn transition(&mut self, from: ReasoningMode, to: ReasoningMode) -> ModeTransition {
        let rank = |m: ReasoningMode| -> usize {
            match m {
                ReasoningMode::Default => 0,
                ReasoningMode::Recovery => 1,
                ReasoningMode::Execution => 2,
                ReasoningMode::Analytical => 3,
                ReasoningMode::Creative => 4,
                ReasoningMode::Exploratory => 5,
            }
        };
        let from_r = rank(from);
        let to_r = rank(to);
        if to_r > from_r + 1 {
            ModeTransition::Upgrade
        } else if from_r > to_r + 1 {
            ModeTransition::Degrade
        } else if from == to {
            ModeTransition::Maintain
        } else {
            ModeTransition::Switch
        }
    }

    pub fn detect_mode(&mut self, context_vector: &[u8]) -> ReasoningMode {
        self.step += 1;
        let mut best_sim = 0.0f64;
        let mut best_mode = self.current_mode;

        for entry in &self.modes {
            let sim = QuantizedVSA::similarity(context_vector, &entry.signature_vector);
            if sim > best_sim {
                best_sim = sim;
                best_mode = entry.mode;
            }
        }

        if best_sim > self.config.emergence_threshold && best_mode != self.current_mode {
            let transition = self.transition(self.current_mode, best_mode);
            self.transition_history
                .push((self.current_mode, best_mode, transition, self.step));
            self.current_mode = best_mode;
        }

        best_mode
    }

    pub fn evolve_mode(&mut self, mode: ReasoningMode, success: bool) {
        if let Some(entry) = self.modes.iter_mut().find(|e| e.mode == mode) {
            let n = entry.invocation_count;
            entry.success_rate =
                (entry.success_rate * n as f64 + success as u64 as f64) / (n + 1) as f64;
            entry.invocation_count = n + 1;
            entry.last_used = self.step;
        }
    }

    pub fn select_reasoning_strategy(&mut self) -> ReasoningMode {
        let selected = self.navigator.select_strategy();
        self.current_mode = selected;
        selected
    }

    pub fn update_navigator(&mut self, mode: ReasoningMode, success: bool, quality: f64) {
        self.navigator.update_from_outcome(mode, success, quality);
    }

    /// Apply a DGM-H self-improvement adjustment.
    /// Lowers emergence threshold (makes mode switching easier),
    /// increases exploration and learning rate in the reasoning navigator.
    pub fn dgmh_adjust(&mut self, gain: f64) {
        self.config.emergence_threshold =
            (self.config.emergence_threshold - gain * 0.1).clamp(0.1, 0.9);
        self.navigator.exploration_rate =
            (self.navigator.exploration_rate + gain * 0.05).clamp(0.01, 0.5);
        self.navigator.learning_rate =
            (self.navigator.learning_rate + gain * 0.02).clamp(0.01, 0.5);
    }

    pub fn distill_patterns(&mut self) {
        let targets: Vec<ReasoningMode> = self
            .modes
            .iter()
            .filter(|e| e.invocation_count as usize >= self.config.min_emergence_frequency)
            .map(|e| e.mode)
            .collect();
        for mode in targets {
            let similar: Vec<&[u8]> = self
                .modes
                .iter()
                .filter(|e| {
                    e.mode == mode
                        && e.invocation_count as usize >= self.config.min_emergence_frequency
                })
                .map(|e| e.signature_vector.as_slice())
                .collect();
            if !similar.is_empty() {
                let centroid = QuantizedVSA::majority_bundle(&similar);
                if let Some(ref mut e) = self.modes.iter_mut().find(|e| e.mode == mode) {
                    e.signature_vector = centroid;
                }
            }
        }
    }

    pub fn generalize_modes(&mut self, threshold: f64) {
        let mut to_remove = Vec::new();
        let n = self.modes.len();
        for i in 0..n {
            for j in (i + 1)..n {
                if to_remove.contains(&j) || to_remove.contains(&i) {
                    continue;
                }
                let sim = QuantizedVSA::similarity(
                    &self.modes[i].signature_vector,
                    &self.modes[j].signature_vector,
                );
                if sim > threshold {
                    let keep = if self.modes[i].success_rate >= self.modes[j].success_rate {
                        i
                    } else {
                        j
                    };
                    let remove = if keep == i { j } else { i };
                    to_remove.push(remove);
                }
            }
        }
        to_remove.sort_unstable();
        to_remove.dedup();
        for idx in to_remove.into_iter().rev() {
            if self.modes[idx].mode != ReasoningMode::Default {
                self.modes.remove(idx);
            }
        }
    }

    pub fn prune_infrequent_modes(&mut self) {
        self.modes.retain(|entry| {
            entry.mode == ReasoningMode::Default
                || entry.invocation_count as usize >= self.config.min_emergence_frequency
        });
    }

    pub fn current_mode(&self) -> ReasoningMode {
        self.current_mode
    }

    pub fn mode_history(&self) -> &[(ReasoningMode, ReasoningMode, ModeTransition, u64)] {
        &self.transition_history
    }

    pub fn mode_success_rate(&self, mode: ReasoningMode) -> f64 {
        self.modes
            .iter()
            .find(|e| e.mode == mode)
            .map(|e| e.success_rate)
            .unwrap_or(0.0)
    }

    pub fn stats(&self) -> (usize, f64, ReasoningMode) {
        let count = self.modes.len();
        let avg = if count > 0 {
            self.modes.iter().map(|e| e.success_rate).sum::<f64>() / count as f64
        } else {
            0.0
        };
        (count, avg, self.current_mode)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_sig(fill: u8) -> Vec<u8> {
        vec![fill; 4096]
    }

    #[test]
    fn test_initial_state() {
        let config = EmergentReasoningConfig::default();
        let erm = EmergentReasoningMode::new(config);
        let (count, avg, mode) = erm.stats();
        assert_eq!(count, 1);
        assert!(avg.abs() < f64::EPSILON || (avg - 0.5).abs() < 1e-6);
        assert_eq!(mode, ReasoningMode::Default);
        assert_eq!(erm.current_mode(), ReasoningMode::Default);
    }

    #[test]
    fn test_detect_mode_known() {
        let mut erm = EmergentReasoningMode::new(EmergentReasoningConfig {
            emergence_threshold: 0.5,
            ..Default::default()
        });
        let sig = make_sig(0);
        erm.modes.push(ReasoningModeEntry {
            mode: ReasoningMode::Analytical,
            signature_vector: make_sig(0),
            success_rate: 0.5,
            invocation_count: 0,
            last_used: 0,
            creation_step: 0,
        });
        let detected = erm.detect_mode(&sig);
        assert_eq!(detected, ReasoningMode::Analytical);
    }

    #[test]
    fn test_evolve_success() {
        let mut erm = EmergentReasoningMode::new(EmergentReasoningConfig::default());
        erm.modes.push(ReasoningModeEntry {
            mode: ReasoningMode::Analytical,
            signature_vector: make_sig(0),
            success_rate: 0.0,
            invocation_count: 0,
            last_used: 0,
            creation_step: 0,
        });
        for _ in 0..5 {
            erm.evolve_mode(ReasoningMode::Analytical, true);
        }
        let rate = erm.mode_success_rate(ReasoningMode::Analytical);
        assert!((rate - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_evolve_failure() {
        let mut erm = EmergentReasoningMode::new(EmergentReasoningConfig::default());
        erm.modes.push(ReasoningModeEntry {
            mode: ReasoningMode::Analytical,
            signature_vector: make_sig(0),
            success_rate: 1.0,
            invocation_count: 1,
            last_used: 0,
            creation_step: 0,
        });
        erm.evolve_mode(ReasoningMode::Analytical, false);
        let rate = erm.mode_success_rate(ReasoningMode::Analytical);
        assert!(rate < 1.0);
    }

    #[test]
    fn test_distill_patterns() {
        let mut erm = EmergentReasoningMode::new(EmergentReasoningConfig {
            min_emergence_frequency: 2,
            ..Default::default()
        });
        for _ in 0..3 {
            erm.modes.push(ReasoningModeEntry {
                mode: ReasoningMode::Analytical,
                signature_vector: make_sig(1),
                success_rate: 0.8,
                invocation_count: 3,
                last_used: 1,
                creation_step: 0,
            });
        }
        erm.distill_patterns();
    }

    #[test]
    fn test_generalize() {
        let mut erm = EmergentReasoningMode::new(EmergentReasoningConfig::default());
        erm.modes.push(ReasoningModeEntry {
            mode: ReasoningMode::Analytical,
            signature_vector: make_sig(1),
            success_rate: 0.9,
            invocation_count: 5,
            last_used: 1,
            creation_step: 0,
        });
        erm.modes.push(ReasoningModeEntry {
            mode: ReasoningMode::Exploratory,
            signature_vector: make_sig(1),
            success_rate: 0.3,
            invocation_count: 2,
            last_used: 1,
            creation_step: 0,
        });
        erm.generalize_modes(0.9);
    }

    #[test]
    fn test_prune_infrequent() {
        let mut erm = EmergentReasoningMode::new(EmergentReasoningConfig {
            min_emergence_frequency: 5,
            ..Default::default()
        });
        erm.modes.push(ReasoningModeEntry {
            mode: ReasoningMode::Analytical,
            signature_vector: make_sig(1),
            success_rate: 0.5,
            invocation_count: 2,
            last_used: 1,
            creation_step: 0,
        });
        erm.modes.push(ReasoningModeEntry {
            mode: ReasoningMode::Creative,
            signature_vector: make_sig(0),
            success_rate: 0.5,
            invocation_count: 10,
            last_used: 2,
            creation_step: 0,
        });
        erm.prune_infrequent_modes();
        assert!(erm.modes.iter().any(|e| e.mode == ReasoningMode::Default));
        assert!(!erm
            .modes
            .iter()
            .any(|e| e.mode == ReasoningMode::Analytical));
        assert!(erm.modes.iter().any(|e| e.mode == ReasoningMode::Creative));
    }

    #[test]
    fn test_mode_history() {
        let mut erm = EmergentReasoningMode::new(EmergentReasoningConfig {
            emergence_threshold: 0.0,
            ..Default::default()
        });
        let sig = make_sig(0);
        erm.modes.push(ReasoningModeEntry {
            mode: ReasoningMode::Analytical,
            signature_vector: sig.clone(),
            success_rate: 0.5,
            invocation_count: 0,
            last_used: 0,
            creation_step: 0,
        });
        erm.detect_mode(&sig);
        let hist = erm.mode_history();
        assert!(!hist.is_empty());
    }

    #[test]
    fn test_navigator_initial_q_values() {
        let nav = ReasoningNavigator::new();
        for q in &nav.q_values {
            assert!((q - 0.5).abs() < 1e-6);
        }
    }

    #[test]
    fn test_navigator_selects_best_mode() {
        let mut nav = ReasoningNavigator::new();
        nav.q_values[0] = 0.9;
        nav.q_values[1] = 0.1;
        nav.exploration_rate = 0.0;
        let best = nav.best_strategy();
        assert_eq!(best, ReasoningMode::Analytical);
    }

    #[test]
    fn test_navigator_updates_q_values() {
        let mut nav = ReasoningNavigator::new();
        nav.exploration_rate = 0.0;
        let mode = ReasoningMode::Analytical;
        let before = nav.q_values[0];
        nav.update_from_outcome(mode, true, 1.0);
        let after = nav.q_values[0];
        assert!(after > before, "success should increase Q-value");
        let before2 = nav.q_values[0];
        nav.update_from_outcome(ReasoningMode::Creative, false, 1.0);
        let after2 = nav.q_values[1];
        assert!(after2 < before2, "failure should decrease Q-value");
    }

    #[test]
    fn test_navigator_exploration_decays() {
        let mut nav = ReasoningNavigator::new();
        let initial_eps = nav.exploration_rate;
        for _ in 0..100 {
            nav.update_from_outcome(ReasoningMode::Default, true, 0.5);
        }
        assert!(
            nav.exploration_rate < initial_eps,
            "exploration should decay"
        );
        assert!(nav.exploration_rate >= 0.01, "exploration floor is 0.01");
    }

    #[test]
    fn test_navigator_strategy_scores_returns_all() {
        let nav = ReasoningNavigator::new();
        let scores = nav.strategy_scores();
        assert_eq!(scores.len(), 6);
        for s in &scores {
            assert!((s.score - 0.5).abs() < 1e-6);
        }
    }

    #[test]
    fn test_select_reasoning_strategy_integration() {
        let config = EmergentReasoningConfig::default();
        let mut erm = EmergentReasoningMode::new(config);
        erm.navigator.exploration_rate = 0.0;
        erm.navigator.q_values = [0.1, 0.9, 0.1, 0.1, 0.1, 0.1];
        let selected = erm.select_reasoning_strategy();
        assert_eq!(selected, ReasoningMode::Creative);
        assert_eq!(erm.current_mode(), ReasoningMode::Creative);
    }

    #[test]
    fn test_update_navigator_integration() {
        let config = EmergentReasoningConfig::default();
        let mut erm = EmergentReasoningMode::new(config);
        let before = erm.navigator.q_values[0];
        erm.update_navigator(ReasoningMode::Default, true, 0.8);
        let after = erm.navigator.q_values[5];
        assert!(after > before);
    }

    #[test]
    fn test_mode_to_idx_roundtrip() {
        let modes = [
            ReasoningMode::Analytical,
            ReasoningMode::Creative,
            ReasoningMode::Exploratory,
            ReasoningMode::Recovery,
            ReasoningMode::Execution,
            ReasoningMode::Default,
        ];
        for m in &modes {
            let idx = mode_to_idx(m);
            let back = idx_to_mode(idx);
            assert_eq!(*m, back);
        }
    }
}
