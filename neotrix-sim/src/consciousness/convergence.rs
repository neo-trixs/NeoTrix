use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq)]
pub enum ConvergenceState {
    Exploring,
    Exploiting,
    Converged,
    Diverged,
}

pub struct ConvergenceDetector {
    window_size: usize,
    metrics: VecDeque<f64>,
    threshold_stable: f64,
    threshold_diverge: f64,
    min_samples: usize,
    state: ConvergenceState,
    state_since: u64,
}

impl ConvergenceDetector {
    pub fn new(window_size: usize, threshold_stable: f64, threshold_diverge: f64) -> Self {
        Self {
            window_size,
            metrics: VecDeque::new(),
            threshold_stable,
            threshold_diverge,
            min_samples: 10,
            state: ConvergenceState::Exploring,
            state_since: 0,
        }
    }

    pub fn default_new() -> Self {
        Self::new(50, 0.01, 0.1)
    }

    /// Record a metric value and update convergence state
    pub fn record(&mut self, value: f64, tick: u64) -> &ConvergenceState {
        self.metrics.push_back(value);
        if self.metrics.len() > self.window_size {
            self.metrics.pop_front();
        }

        if self.metrics.len() < self.min_samples {
            return &self.state;
        }

        let mean = self.metrics.iter().sum::<f64>() / self.metrics.len() as f64;
        let variance = self.metrics.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / self.metrics.len() as f64;
        let cv = variance.sqrt() / mean.abs().max(1e-10);  // coefficient of variation

        let trend = self.trend();

        let new_state = if cv < self.threshold_stable && trend.abs() < self.threshold_stable * 0.5 {
            ConvergenceState::Converged
        } else if cv > self.threshold_diverge || (trend.abs() > self.threshold_diverge && trend * self.metrics.front().unwrap_or(&0.0) < 0.0) {
            ConvergenceState::Diverged
        } else if cv < self.threshold_stable * 3.0 {
            ConvergenceState::Exploiting
        } else {
            ConvergenceState::Exploring
        };

        if new_state != self.state {
            self.state = new_state;
            self.state_since = tick;
        }

        &self.state
    }

    /// Linear regression trend of recent values
    fn trend(&self) -> f64 {
        let n = self.metrics.len() as f64;
        if n < 2.0 { return 0.0; }
        let sum_x: f64 = (0..self.metrics.len() as i64).map(|x| x as f64).sum();
        let sum_y: f64 = self.metrics.iter().sum();
        let sum_xy: f64 = self.metrics.iter().enumerate().map(|(i, &y)| i as f64 * y).sum();
        let sum_x2: f64 = (0..self.metrics.len() as i64).map(|x| (x as f64).powi(2)).sum();
        let denom = n * sum_x2 - sum_x * sum_x;
        if denom.abs() < 1e-10 { return 0.0; }
        (n * sum_xy - sum_x * sum_y) / denom
    }

    pub fn state(&self) -> &ConvergenceState { &self.state }
    pub fn state_duration(&self, current_tick: u64) -> u64 { current_tick.saturating_sub(self.state_since) }
    pub fn metric_count(&self) -> usize { self.metrics.len() }
    pub fn mean(&self) -> f64 {
        if self.metrics.is_empty() { return 0.0; }
        self.metrics.iter().sum::<f64>() / self.metrics.len() as f64
    }
    pub fn variance(&self) -> f64 {
        if self.metrics.len() < 2 { return 0.0; }
        let mean = self.mean();
        self.metrics.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / self.metrics.len() as f64
    }

    /// Force state (for testing / external override)
    pub fn set_state(&mut self, state: ConvergenceState, tick: u64) {
        self.state = state;
        self.state_since = tick;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_exploring() {
        let cd = ConvergenceDetector::default_new();
        assert_eq!(*cd.state(), ConvergenceState::Exploring);
    }

    #[test]
    fn converges_on_stable_values() {
        let mut cd = ConvergenceDetector::default_new();
        for i in 0..20 {
            cd.record(1.0, i);
        }
        assert_eq!(*cd.state(), ConvergenceState::Converged);
    }

    #[test]
    fn diverges_on_volatile() {
        let mut cd = ConvergenceDetector::default_new();
        for i in 0..20 {
            cd.record(if i % 2 == 0 { 1.0 } else { 100.0 }, i);
        }
        assert_eq!(*cd.state(), ConvergenceState::Diverged);
    }

    #[test]
    fn trend_positive() {
        let mut cd = ConvergenceDetector::default_new();
        for i in 0..20 {
            cd.record(i as f64, i);
        }
        assert!(cd.trend() > 0.0);
    }

    #[test]
    fn metric_count() {
        let mut cd = ConvergenceDetector::default_new();
        cd.record(1.0, 0);
        cd.record(2.0, 1);
        assert_eq!(cd.metric_count(), 2);
    }

    #[test]
    fn mean_calculation() {
        let mut cd = ConvergenceDetector::default_new();
        cd.record(2.0, 0);
        cd.record(4.0, 1);
        assert!((cd.mean() - 3.0).abs() < 0.001);
    }

    #[test]
    fn force_state() {
        let mut cd = ConvergenceDetector::default_new();
        cd.set_state(ConvergenceState::Converged, 0);
        assert_eq!(*cd.state(), ConvergenceState::Converged);
    }
}
