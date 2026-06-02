use std::collections::VecDeque;

/// Entropy-based deadlock monitor for GWT resonance cycles.
///
/// Implements the "Evaluation" phase of the Discovery loop (Variation→Evaluation→Retention):
/// - Monitors Shannon entropy of the specialist activation distribution over a sliding window
/// - Detects when the system is "stuck" (entropy consistently below threshold → reasoning deadlock)
/// - Generates stochastic stimulus to re-introduce variation and break the deadlock
/// - Tracks deadlock statistics for downstream selective retention decisions
/// - After N failed stimulus attempts, signals that a SEAL rollback is needed
///
/// Reference: GWA (arXiv:2604.08206), §3.2 — entropy-based intrinsic drive to break deadlocks
#[derive(Debug, Clone)]
pub struct EntropyMonitor {
    /// Sliding window of recent entropy readings (most recent first).
    pub history: VecDeque<f64>,
    /// Maximum window size.
    pub window: usize,
    /// Entropy threshold for "stuck" detection (below = stuck).
    pub deadlock_threshold: f64,
    /// Minimum consecutive stuck cycles to declare deadlock.
    pub stuck_min_cycles: usize,
    /// Total deadlocks detected across the lifetime.
    pub deadlock_count: u64,
    /// Whether currently in a deadlock state.
    pub in_deadlock: bool,
    /// Stimulus magnitude (fraction of [0,1] range to perturb).
    pub stimulus_magnitude: f64,
    /// Strength of the last injected stimulus (-1.0 = no stimulus).
    pub last_stimulus_strength: f64,
    /// Running average entropy over the window.
    pub avg_entropy: f64,
    /// How many stimulus injections have been performed during the current deadlock.
    pub stimulus_attempts: u64,
    /// Max stimulus attempts before triggering rollback.
    pub max_stimulus_before_rollback: u64,
    /// Whether the stimulus helped (entropy increased after injection).
    pub stimulus_succeeded: bool,
}

impl Default for EntropyMonitor {
    fn default() -> Self {
        Self {
            history: VecDeque::with_capacity(10),
            window: 10,
            deadlock_threshold: 0.5,
            stuck_min_cycles: 5,
            deadlock_count: 0,
            in_deadlock: false,
            stimulus_magnitude: 0.15,
            last_stimulus_strength: -1.0,
            avg_entropy: 0.0,
            stimulus_attempts: 0,
            max_stimulus_before_rollback: 3,
            stimulus_succeeded: false,
        }
    }
}

impl EntropyMonitor {
    pub fn new(window: usize, threshold: f64, min_stuck: usize) -> Self {
        Self {
            history: VecDeque::with_capacity(window),
            window,
            deadlock_threshold: threshold,
            stuck_min_cycles: min_stuck,
            ..Default::default()
        }
    }

    /// Feed a new entropy reading and update deadlock state.
    pub fn feed(&mut self, entropy: f64) {
        let was_deadlocked = self.in_deadlock;

        self.history.push_front(entropy);
        while self.history.len() > self.window {
            self.history.pop_back();
        }

        self.avg_entropy = if self.history.is_empty() {
            0.0
        } else {
            self.history.iter().sum::<f64>() / self.history.len() as f64
        };

        let stuck_count = self.history.iter().take(self.stuck_min_cycles).filter(|&&e| e < self.deadlock_threshold).count();

        let previously_deadlocked = self.in_deadlock;
        self.in_deadlock = self.history.len() >= self.stuck_min_cycles && stuck_count >= self.stuck_min_cycles;

        if self.in_deadlock && !previously_deadlocked {
            self.deadlock_count += 1;
        }

        if was_deadlocked && !self.in_deadlock {
            self.stimulus_succeeded = true;
            self.stimulus_attempts = 0;
        }
    }

    /// Whether the system is currently in a reasoning deadlock.
    pub fn is_deadlocked(&self) -> bool {
        self.in_deadlock
    }

    /// Fraction of the current window that is stuck (entropy < threshold).
    pub fn stuck_ratio(&self) -> f64 {
        if self.history.is_empty() {
            return 0.0;
        }
        let stuck = self.history.iter().filter(|&&e| e < self.deadlock_threshold).count();
        stuck as f64 / self.history.len() as f64
    }

    /// Generate a random stimulus vector and inject it into raw saliences.
    /// This breaks the deadlock by perturbing the activation distribution.
    /// Returns the strength of injected stimulus.
    pub fn inject_stimulus(&mut self, raw_salience: &mut [f64]) -> f64 {
        self.stimulus_attempts += 1;
        self.stimulus_succeeded = false;
        let perturbation = self.stimulus_magnitude * (rand::random::<f64>() * 2.0 - 1.0);
        let target = rand::random::<usize>() % raw_salience.len();
        raw_salience[target] = (raw_salience[target] + perturbation).clamp(0.0, 1.0);
        self.last_stimulus_strength = perturbation.abs();
        self.last_stimulus_strength
    }

    /// Whether the deadlock is persistent and requires a SEAL rollback.
    /// True when deadlocked AND stimulus has been attempted max times without recovery.
    pub fn should_rollback(&self) -> bool {
        self.in_deadlock && self.stimulus_attempts >= self.max_stimulus_before_rollback
    }

    /// Intensity of the deadlock crisis (0.0 = none, 1.0 = critical).
    /// Combines stuck_ratio with stimulus exhaustion.
    pub fn crisis_level(&self) -> f64 {
        let base = self.stuck_ratio();
        if self.in_deadlock {
            let attempt_ratio = (self.stimulus_attempts as f64 / self.max_stimulus_before_rollback as f64).min(1.0);
            (base * 0.5 + attempt_ratio * 0.5).min(1.0)
        } else {
            base * 0.3
        }
    }

    /// Reset the monitor (e.g., after a successful cycle).
    pub fn reset(&mut self) {
        self.history.clear();
        self.in_deadlock = false;
        self.avg_entropy = 0.0;
        self.last_stimulus_strength = -1.0;
        self.stimulus_attempts = 0;
        self.stimulus_succeeded = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::resonance::MODULE_COUNT;

    #[test]
    fn test_entropy_monitor_default() {
        let m = EntropyMonitor::default();
        assert_eq!(m.window, 10);
        assert_eq!(m.deadlock_count, 0);
        assert!(!m.in_deadlock);
    }

    #[test]
    fn test_feed_updates_avg_entropy() {
        let mut m = EntropyMonitor::new(5, 0.5, 3);
        m.feed(1.0);
        m.feed(2.0);
        m.feed(3.0);
        assert!((m.avg_entropy - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_feed_respects_window() {
        let mut m = EntropyMonitor::new(3, 0.5, 2);
        m.feed(1.0);
        m.feed(2.0);
        m.feed(3.0);
        m.feed(4.0);
        assert_eq!(m.history.len(), 3);
        assert!((m.avg_entropy - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_stuck_ratio_after_low_entropy() {
        let mut m = EntropyMonitor::new(5, 1.0, 3);
        m.feed(0.1);
        m.feed(0.2);
        m.feed(0.3);
        assert!((m.stuck_ratio() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_stuck_ratio_partial() {
        let mut m = EntropyMonitor::new(4, 1.0, 3);
        m.feed(0.1);
        m.feed(0.9);
        m.feed(2.0);
        m.feed(3.0);
        let ratio = m.stuck_ratio();
        assert!((ratio - 0.5).abs() < 1e-6, "expected 0.5, got {}", ratio);
    }

    #[test]
    fn test_deadlock_detection_after_consecutive_stuck() {
        let mut m = EntropyMonitor::new(10, 0.5, 4);
        assert!(!m.is_deadlocked());
        for _ in 0..4 {
            m.feed(0.1);
        }
        assert!(m.is_deadlocked());
    }

    #[test]
    fn test_deadlock_not_triggered_below_min_stuck() {
        let mut m = EntropyMonitor::new(10, 0.5, 5);
        for _ in 0..3 {
            m.feed(0.1);
        }
        assert!(!m.is_deadlocked());
    }

    #[test]
    fn test_deadlock_toggle_on_high_entropy() {
        let mut m = EntropyMonitor::new(10, 0.5, 3);
        for _ in 0..3 {
            m.feed(0.1);
        }
        assert!(m.is_deadlocked());
        for _ in 0..3 {
            m.feed(1.0);
        }
        assert!(!m.is_deadlocked());
    }

    #[test]
    fn test_deadlock_count_increments_once() {
        let mut m = EntropyMonitor::new(10, 0.5, 3);
        assert_eq!(m.deadlock_count, 0);
        for _ in 0..3 {
            m.feed(0.1);
        }
        assert_eq!(m.deadlock_count, 1);
        for _ in 0..3 {
            m.feed(0.1);
        }
        assert_eq!(m.deadlock_count, 1);
    }

    #[test]
    fn test_inject_stimulus_modifies_salience() {
        let mut m = EntropyMonitor::new(10, 0.5, 3);
        let mut raw = [0.3; MODULE_COUNT];
        let before = raw;
        m.inject_stimulus(&mut raw);
        let changed = raw.iter().zip(before.iter()).any(|(a, b)| (*a - *b).abs() > 1e-9);
        assert!(changed, "stimulus should change at least one element");
    }

    #[test]
    fn test_inject_stimulus_clamps_to_range() {
        let mut m = EntropyMonitor::new(10, 0.5, 3);
        let mut raw = [0.99; MODULE_COUNT];
        for _ in 0..20 {
            m.inject_stimulus(&mut raw);
            for &v in &raw {
                assert!(v >= 0.0 && v <= 1.0, "value {} out of range", v);
            }
        }
    }

    #[test]
    fn test_inject_stimulus_tracks_last_strength() {
        let mut m = EntropyMonitor::new(10, 0.5, 3);
        let mut raw = [0.3; MODULE_COUNT];
        let strength = m.inject_stimulus(&mut raw);
        assert!((m.last_stimulus_strength - strength).abs() < 1e-9);
        assert!(strength > 0.0 && strength <= m.stimulus_magnitude);
    }

    #[test]
    fn test_reset_clears_state() {
        let mut m = EntropyMonitor::new(10, 0.5, 3);
        for _ in 0..5 {
            m.feed(0.1);
        }
        assert!(m.is_deadlocked());
        m.reset();
        assert!(!m.is_deadlocked());
        assert_eq!(m.history.len(), 0);
    }

    #[test]
    fn test_empty_history_stuck_ratio() {
        let m = EntropyMonitor::new(10, 0.5, 3);
        assert!((m.stuck_ratio() - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_entropy_monitor_new() {
        let m = EntropyMonitor::new(20, 0.3, 5);
        assert_eq!(m.window, 20);
        assert!((m.deadlock_threshold - 0.3).abs() < 1e-9);
        assert_eq!(m.stuck_min_cycles, 5);
    }

    #[test]
    fn test_stimulus_attempts_increment() {
        let mut m = EntropyMonitor::new(10, 0.5, 3);
        let mut raw = [0.3; MODULE_COUNT];
        assert_eq!(m.stimulus_attempts, 0);
        m.inject_stimulus(&mut raw);
        assert_eq!(m.stimulus_attempts, 1);
        m.inject_stimulus(&mut raw);
        assert_eq!(m.stimulus_attempts, 2);
    }

    #[test]
    fn test_should_rollback_after_max_stimulus() {
        let mut m = EntropyMonitor::new(10, 0.5, 3);
        m.max_stimulus_before_rollback = 2;
        assert!(!m.should_rollback());
        // deadlock first
        for _ in 0..3 { m.feed(0.1); }
        assert!(m.is_deadlocked());
        m.inject_stimulus(&mut [0.3; MODULE_COUNT]);
        assert!(!m.should_rollback());
        m.inject_stimulus(&mut [0.3; MODULE_COUNT]);
        assert!(m.should_rollback());
    }

    #[test]
    fn test_should_rollback_false_when_not_deadlocked() {
        let mut m = EntropyMonitor::new(10, 0.5, 3);
        m.inject_stimulus(&mut [0.3; MODULE_COUNT]);
        assert!(!m.should_rollback());
    }

    #[test]
    fn test_stimulus_succeeded_flag_on_recovery() {
        let mut m = EntropyMonitor::new(5, 0.5, 3);
        for _ in 0..3 { m.feed(0.1); }
        assert!(m.is_deadlocked());
        m.inject_stimulus(&mut [0.3; MODULE_COUNT]);
        assert!(!m.stimulus_succeeded);
        for _ in 0..3 { m.feed(1.0); }
        assert!(!m.is_deadlocked());
        assert!(m.stimulus_succeeded);
        assert_eq!(m.stimulus_attempts, 0);
    }

    #[test]
    fn test_crisis_level_no_deadlock() {
        let m = EntropyMonitor::new(10, 0.5, 3);
        assert!(m.crisis_level() < 0.01);
    }

    #[test]
    fn test_crisis_level_increases_with_attempts() {
        let mut m = EntropyMonitor::new(10, 0.5, 3);
        m.max_stimulus_before_rollback = 2;
        for _ in 0..3 { m.feed(0.1); }
        m.inject_stimulus(&mut [0.3; MODULE_COUNT]);
        let level1 = m.crisis_level();
        m.inject_stimulus(&mut [0.3; MODULE_COUNT]);
        let level2 = m.crisis_level();
        assert!(level2 >= level1, "crisis should increase: {:.4} vs {:.4}", level1, level2);
    }

    #[test]
    fn test_reset_clears_stimulus_state() {
        let mut m = EntropyMonitor::new(10, 0.5, 3);
        for _ in 0..3 { m.feed(0.1); }
        m.inject_stimulus(&mut [0.3; MODULE_COUNT]);
        m.reset();
        assert_eq!(m.stimulus_attempts, 0);
        assert!(!m.stimulus_succeeded);
    }
}
