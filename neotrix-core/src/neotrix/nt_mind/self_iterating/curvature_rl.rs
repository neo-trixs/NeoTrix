/// Curvature-aware reinforcement learning for meta-parameter adaptation.
///
/// Uses second-order signal curvature (rather than raw reward) to detect
/// plateaus, oscillations, and gradient direction changes, then adapts
/// learning rate and exploration rate accordingly.

/// The current curvature regime detected from reward history.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurvatureRegime {
    /// Stable positive gradient — continue with current params
    Stable,
    /// Plateau detected — reward improvement slowing, increase exploration
    Plateau,
    /// Oscillation detected — reward bouncing, reduce learning rate
    Oscillating,
    /// Sharp drop — negative curvature spike, clamp learning rate
    Crashing,
    /// Cold start — insufficient data
    Cold,
}

impl CurvatureRegime {
    pub fn lr_multiplier(&self) -> f64 {
        match self {
            CurvatureRegime::Stable => 1.0,
            CurvatureRegime::Plateau => 1.5,
            CurvatureRegime::Oscillating => 0.5,
            CurvatureRegime::Crashing => 0.1,
            CurvatureRegime::Cold => 1.0,
        }
    }

    pub fn explore_bonus(&self) -> f64 {
        match self {
            CurvatureRegime::Stable => 0.0,
            CurvatureRegime::Plateau => 0.3,
            CurvatureRegime::Oscillating => 0.1,
            CurvatureRegime::Crashing => 0.0,
            CurvatureRegime::Cold => 0.5,
        }
    }
}

/// Tracks reward curvature using finite-difference second derivative.
#[derive(Debug, Clone)]
pub struct CurvatureTracker {
    pub window: Vec<f64>,
    pub max_window: usize,
    pub first_derivatives: Vec<f64>,
    pub second_derivatives: Vec<f64>,
    pub last_regime: CurvatureRegime,
    pub regime_history: Vec<CurvatureRegime>,
    /// Separate window for N_total tracking
    pub negentropy_window: Vec<f64>,
    pub negentropy_first_derivatives: Vec<f64>,
    pub negentropy_second_derivatives: Vec<f64>,
    pub last_negentropy_regime: CurvatureRegime,
    pub negentropy_regime_history: Vec<CurvatureRegime>,
}

impl CurvatureTracker {
    pub fn new(max_window: usize) -> Self {
        Self {
            window: Vec::with_capacity(max_window),
            max_window,
            first_derivatives: Vec::new(),
            second_derivatives: Vec::new(),
            last_regime: CurvatureRegime::Cold,
            regime_history: Vec::new(),
            negentropy_window: Vec::with_capacity(max_window),
            negentropy_first_derivatives: Vec::new(),
            negentropy_second_derivatives: Vec::new(),
            last_negentropy_regime: CurvatureRegime::Cold,
            negentropy_regime_history: Vec::new(),
        }
    }

    /// Record a new reward value and update curvature estimates.
    pub fn record(&mut self, reward: f64) {
        self.window.push(reward);
        if self.window.len() > self.max_window {
            self.window.remove(0);
        }
        self.update_derivatives();
        self.last_regime = self.classify_regime();
        self.regime_history.push(self.last_regime);
        if self.regime_history.len() > 100 {
            self.regime_history.remove(0);
        }
    }

    /// Record N_total value into the negentropy window and update curvature.
    pub fn record_negentropy(&mut self, n_total: f64) {
        self.negentropy_window.push(n_total);
        if self.negentropy_window.len() > self.max_window {
            self.negentropy_window.remove(0);
        }
        self.update_negentropy_derivatives();
        self.last_negentropy_regime = self.classify_negentropy_regime();
        self.negentropy_regime_history
            .push(self.last_negentropy_regime);
        if self.negentropy_regime_history.len() > 100 {
            self.negentropy_regime_history.remove(0);
        }
    }

    fn update_negentropy_derivatives(&mut self) {
        if self.negentropy_window.len() < 3 {
            return;
        }
        let n = self.negentropy_window.len();
        let first: Vec<f64> = (1..n)
            .map(|i| self.negentropy_window[i] - self.negentropy_window[i - 1])
            .collect();
        let second: Vec<f64> = (1..first.len()).map(|i| first[i] - first[i - 1]).collect();
        self.negentropy_first_derivatives = first;
        self.negentropy_second_derivatives = second;
    }

    pub fn classify_negentropy_regime(&self) -> CurvatureRegime {
        if self.negentropy_window.len() < 5 {
            return CurvatureRegime::Cold;
        }

        let second = &self.negentropy_second_derivatives;
        if second.len() < 3 {
            return CurvatureRegime::Cold;
        }

        let recent: Vec<f64> = second.iter().rev().take(5).copied().collect();
        if recent.is_empty() {
            return CurvatureRegime::Cold;
        }

        let mean = recent.iter().sum::<f64>() / recent.len() as f64;
        let variance = recent.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / recent.len() as f64;
        let std_dev = variance.sqrt();

        let first = &self.negentropy_first_derivatives;
        let recent_first: Vec<f64> = first.iter().rev().take(5).copied().collect();
        let first_mean = recent_first.iter().sum::<f64>() / recent_first.len() as f64;

        // Crashing: N_total dropping sharply
        if first_mean < -0.05 && mean < 0.0 {
            return CurvatureRegime::Crashing;
        }

        // Oscillating: high variance in N_total curvature
        if std_dev > 0.15 {
            return CurvatureRegime::Oscillating;
        }

        // Plateau: near-zero N_total change
        if first_mean.abs() < 0.005 && mean.abs() < 0.005 {
            return CurvatureRegime::Plateau;
        }

        CurvatureRegime::Stable
    }

    /// Compute first and second derivatives via finite differences.
    fn update_derivatives(&mut self) {
        if self.window.len() < 3 {
            return;
        }
        let n = self.window.len();
        let first: Vec<f64> = (1..n)
            .map(|i| self.window[i] - self.window[i - 1])
            .collect();
        let second: Vec<f64> = (1..first.len()).map(|i| first[i] - first[i - 1]).collect();
        self.first_derivatives = first;
        self.second_derivatives = second;
    }

    /// Classify the current regime based on second derivative statistics.
    pub fn classify_regime(&self) -> CurvatureRegime {
        if self.window.len() < 5 {
            return CurvatureRegime::Cold;
        }

        let second = &self.second_derivatives;
        if second.len() < 3 {
            return CurvatureRegime::Cold;
        }

        let recent: Vec<f64> = second.iter().rev().take(5).copied().collect();
        if recent.is_empty() {
            return CurvatureRegime::Cold;
        }

        let mean = recent.iter().sum::<f64>() / recent.len() as f64;
        let variance = recent.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / recent.len() as f64;
        let std_dev = variance.sqrt();

        let first = &self.first_derivatives;
        let recent_first: Vec<f64> = first.iter().rev().take(5).copied().collect();
        let first_mean = recent_first.iter().sum::<f64>() / recent_first.len() as f64;

        // Crashing: large negative first derivative + negative second derivative
        if first_mean < -0.1 && mean < 0.0 {
            return CurvatureRegime::Crashing;
        }

        // Oscillating: high variance in second derivative
        if std_dev > 0.2 {
            return CurvatureRegime::Oscillating;
        }

        // Plateau: near-zero first and second derivatives
        if first_mean.abs() < 0.01 && mean.abs() < 0.01 {
            return CurvatureRegime::Plateau;
        }

        CurvatureRegime::Stable
    }

    /// Get the recent oscillation frequency (0-1 scale).
    pub fn oscillation_frequency(&self) -> f64 {
        if self.regime_history.len() < 10 {
            return 0.0;
        }
        let recent: Vec<_> = self.regime_history.iter().rev().take(20).collect();
        let osc_count = recent
            .iter()
            .filter(|&&r| *r == CurvatureRegime::Oscillating)
            .count();
        osc_count as f64 / recent.len() as f64
    }

    /// Suggested learning rate multiplier based on current regime.
    pub fn lr_multiplier(&self) -> f64 {
        self.last_regime.lr_multiplier()
    }

    /// Suggested exploration bonus based on current regime.
    pub fn explore_bonus(&self) -> f64 {
        self.last_regime.explore_bonus()
    }

    pub fn negentropy_regime(&self) -> CurvatureRegime {
        self.last_negentropy_regime
    }

    pub fn negentropy_lr_multiplier(&self) -> f64 {
        self.last_negentropy_regime.lr_multiplier()
    }

    pub fn negentropy_explore_bonus(&self) -> f64 {
        self.last_negentropy_regime.explore_bonus()
    }

    pub fn negentropy_oscillation_frequency(&self) -> f64 {
        if self.negentropy_regime_history.len() < 10 {
            return 0.0;
        }
        let recent: Vec<_> = self
            .negentropy_regime_history
            .iter()
            .rev()
            .take(20)
            .collect();
        let osc_count = recent
            .iter()
            .filter(|&&r| *r == CurvatureRegime::Oscillating)
            .count();
        osc_count as f64 / recent.len() as f64
    }

    /// Number of consecutive steps in the same regime.
    pub fn regime_streak(&self) -> usize {
        let mut streak = 0;
        for r in self.regime_history.iter().rev() {
            if *r == self.last_regime {
                streak += 1;
            } else {
                break;
            }
        }
        streak
    }
}

/// Adapts meta-parameters based on curvature signals.
#[derive(Debug, Clone)]
pub struct CurvaturePolicy {
    pub base_lr: f64,
    pub min_lr: f64,
    pub max_lr: f64,
    pub base_explore: f64,
    pub tracker: CurvatureTracker,
}

impl CurvaturePolicy {
    pub fn new(base_lr: f64, min_lr: f64, max_lr: f64, base_explore: f64, window: usize) -> Self {
        Self {
            base_lr,
            min_lr,
            max_lr,
            base_explore,
            tracker: CurvatureTracker::new(window),
        }
    }

    /// Feed a reward signal and get adapted learning rate.
    pub fn adapt_lr(&mut self, reward: f64) -> f64 {
        self.tracker.record(reward);
        let mult = self.tracker.lr_multiplier();
        (self.base_lr * mult).clamp(self.min_lr, self.max_lr)
    }

    /// Feed a reward signal and get adapted exploration rate.
    pub fn adapt_explore(&mut self, reward: f64) -> f64 {
        self.tracker.record(reward);
        self.base_explore + self.tracker.explore_bonus()
    }

    /// Feed N_total signal and get learning rate adapted to negentropy curvature.
    pub fn adapt_lr_to_negentropy(&mut self, n_total: f64) -> f64 {
        self.tracker.record_negentropy(n_total);
        let mult = self.tracker.negentropy_lr_multiplier();
        (self.base_lr * mult).clamp(self.min_lr, self.max_lr)
    }

    /// Feed N_total signal and get exploration rate adapted to negentropy curvature.
    pub fn adapt_explore_to_negentropy(&mut self, n_total: f64) -> f64 {
        self.tracker.record_negentropy(n_total);
        self.base_explore + self.tracker.negentropy_explore_bonus()
    }

    pub fn negentropy_regime(&self) -> CurvatureRegime {
        self.tracker.negentropy_regime()
    }

    pub fn regime(&self) -> CurvatureRegime {
        self.tracker.last_regime
    }

    pub fn stats(&self) -> CurvatureStats {
        CurvatureStats {
            regime: self.tracker.last_regime,
            lr_multiplier: self.tracker.lr_multiplier(),
            explore_bonus: self.tracker.explore_bonus(),
            oscillation_freq: self.tracker.oscillation_frequency(),
            regime_streak: self.tracker.regime_streak(),
            data_points: self.tracker.window.len(),
            negentropy_regime: self.tracker.negentropy_regime(),
            negentropy_lr_multiplier: self.tracker.negentropy_lr_multiplier(),
            negentropy_explore_bonus: self.tracker.negentropy_explore_bonus(),
            negentropy_oscillation_freq: self.tracker.negentropy_oscillation_frequency(),
            negentropy_data_points: self.tracker.negentropy_window.len(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CurvatureStats {
    pub regime: CurvatureRegime,
    pub lr_multiplier: f64,
    pub explore_bonus: f64,
    pub oscillation_freq: f64,
    pub regime_streak: usize,
    pub data_points: usize,
    pub negentropy_regime: CurvatureRegime,
    pub negentropy_lr_multiplier: f64,
    pub negentropy_explore_bonus: f64,
    pub negentropy_oscillation_freq: f64,
    pub negentropy_data_points: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cold_start() {
        let mut tracker = CurvatureTracker::new(100);
        assert_eq!(tracker.classify_regime(), CurvatureRegime::Cold);
        tracker.record(0.5);
        assert_eq!(tracker.classify_regime(), CurvatureRegime::Cold);
        tracker.record(0.6);
        assert_eq!(tracker.classify_regime(), CurvatureRegime::Cold);
    }

    #[test]
    fn test_stable_regime() {
        let mut tracker = CurvatureTracker::new(100);
        for i in 0..20 {
            tracker.record(0.1 * i as f64);
        }
        assert_eq!(tracker.classify_regime(), CurvatureRegime::Stable);
    }

    #[test]
    fn test_plateau_detected() {
        let mut tracker = CurvatureTracker::new(100);
        for _ in 0..20 {
            tracker.record(0.5);
        }
        assert_eq!(tracker.classify_regime(), CurvatureRegime::Plateau);
    }

    #[test]
    fn test_oscillation_detected() {
        let mut tracker = CurvatureTracker::new(100);
        for i in 0..30 {
            tracker.record(if i % 2 == 0 { 1.0 } else { 0.0 });
        }
        // After 30 points, oscillations should be detected
        assert_eq!(tracker.classify_regime(), CurvatureRegime::Oscillating);
    }

    #[test]
    fn test_crashing_detected() {
        let mut tracker = CurvatureTracker::new(100);
        for i in 0..20 {
            tracker.record(1.0 - 0.1 * i as f64);
        }
        assert_eq!(tracker.classify_regime(), CurvatureRegime::Crashing);
    }

    #[test]
    fn test_lr_multiplier_per_regime() {
        assert_eq!(CurvatureRegime::Stable.lr_multiplier(), 1.0);
        assert_eq!(CurvatureRegime::Plateau.lr_multiplier(), 1.5);
        assert_eq!(CurvatureRegime::Oscillating.lr_multiplier(), 0.5);
        assert_eq!(CurvatureRegime::Crashing.lr_multiplier(), 0.1);
    }

    #[test]
    fn test_explore_bonus_per_regime() {
        assert_eq!(CurvatureRegime::Stable.explore_bonus(), 0.0);
        assert_eq!(CurvatureRegime::Plateau.explore_bonus(), 0.3);
        assert_eq!(CurvatureRegime::Cold.explore_bonus(), 0.5);
    }

    #[test]
    fn test_policy_adapt_lr() {
        let mut policy = CurvaturePolicy::new(0.01, 0.001, 0.1, 0.1, 100);
        let lr = policy.adapt_lr(1.0);
        assert!(lr >= 0.001 && lr <= 0.1);
    }

    #[test]
    fn test_policy_adapt_lr_stable() {
        let mut policy = CurvaturePolicy::new(0.01, 0.001, 0.1, 0.1, 100);
        let mut lr = 0.0;
        for i in 0..20 {
            lr = policy.adapt_lr(0.1 * i as f64);
        }
        assert!(
            (lr - 0.01).abs() < 0.001,
            "stable regime should keep base lr: got {}",
            lr
        );
    }

    #[test]
    fn test_policy_adapt_lr_oscillating() {
        let mut policy = CurvaturePolicy::new(0.01, 0.001, 0.1, 0.1, 100);
        let mut lr = 0.01;
        for i in 0..30 {
            lr = policy.adapt_lr(if i % 2 == 0 { 1.0 } else { 0.0 });
        }
        assert!(lr < 0.01, "oscillation should reduce lr: got {}", lr);
    }

    #[test]
    fn test_curvature_stats_report() {
        let mut policy = CurvaturePolicy::new(0.01, 0.001, 0.1, 0.1, 100);
        policy.adapt_lr(0.5);
        let stats = policy.stats();
        assert_eq!(stats.data_points, 1);
    }

    #[test]
    fn test_regime_streak() {
        let mut tracker = CurvatureTracker::new(100);
        for _ in 0..10 {
            tracker.record(0.5);
        }
        let streak = tracker.regime_streak();
        assert!(streak >= 5);
    }

    #[test]
    fn test_oscillation_frequency() {
        let mut tracker = CurvatureTracker::new(100);
        for i in 0..30 {
            tracker.record(if i % 2 == 0 { 1.0 } else { 0.0 });
        }
        let freq = tracker.oscillation_frequency();
        assert!(freq > 0.0, "oscillations should produce non-zero frequency");
    }
}
