#![forbid(unsafe_code)]

use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CalibrationTarget {
    Confidence,
    Speed,
    Thoroughness,
    Creativity,
    Conservatism,
}

#[derive(Debug, Clone)]
pub struct CalibrationSignal {
    pub target: CalibrationTarget,
    pub observed_value: f64,
    pub expected_value: f64,
    pub gap: f64,
    pub weight: f64,
}

impl CalibrationSignal {
    pub fn new(target: CalibrationTarget, observed: f64, expected: f64, weight: f64) -> Self {
        Self {
            target,
            observed_value: observed.max(0.0).min(1.0),
            expected_value: expected.max(0.0).min(1.0),
            gap: (observed - expected).max(-1.0).min(1.0),
            weight: weight.max(0.0).min(1.0),
        }
    }

    pub fn adjustment(&self) -> f64 {
        -self.gap * self.weight * 0.1
    }
}

#[derive(Debug, Clone)]
pub struct CalibrationState {
    pub target: CalibrationTarget,
    pub current_value: f64,
    pub target_value: f64,
    pub history: VecDeque<f64>,
}

impl CalibrationState {
    pub fn new(target: CalibrationTarget, initial: f64) -> Self {
        let mut history = VecDeque::with_capacity(100);
        history.push_back(initial);
        Self {
            target,
            current_value: initial,
            target_value: initial,
            history,
        }
    }

    pub fn apply_adjustment(&mut self, delta: f64) {
        self.current_value = (self.current_value + delta).max(0.0).min(1.0);
        if self.history.len() >= 100 {
            self.history.pop_front();
        }
        self.history.push_back(self.current_value);
    }

    pub fn is_calibrated(&self, tolerance: f64) -> bool {
        (self.current_value - self.target_value).abs() <= tolerance
    }

    pub fn trend(&self) -> f64 {
        let n = self.history.len();
        if n < 5 {
            return 0.0;
        }
        let recent: Vec<&f64> = self.history.iter().rev().take(10).collect();
        if recent.len() < 2 {
            return 0.0;
        }
        recent[0] - recent[recent.len() - 1]
    }
}

#[derive(Debug, Clone)]
pub struct MetaCalibratorConfig {
    pub tolerance: f64,
    pub adaptation_rate: f64,
    pub max_history: usize,
}

impl Default for MetaCalibratorConfig {
    fn default() -> Self {
        Self {
            tolerance: 0.1,
            adaptation_rate: 0.05,
            max_history: 100,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CalibrationReport {
    pub calibrated_count: u32,
    pub total_targets: u32,
    pub avg_gap: f64,
    pub biggest_gap: (CalibrationTarget, f64),
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct MetaCalibrator {
    pub config: MetaCalibratorConfig,
    states: HashMap<CalibrationTarget, CalibrationState>,
    signals: VecDeque<CalibrationSignal>,
}

impl MetaCalibrator {
    pub fn new(config: MetaCalibratorConfig) -> Self {
        let mut states = HashMap::new();
        for target in [
            CalibrationTarget::Confidence,
            CalibrationTarget::Speed,
            CalibrationTarget::Thoroughness,
            CalibrationTarget::Creativity,
            CalibrationTarget::Conservatism,
        ] {
            let initial = match target {
                CalibrationTarget::Confidence => 0.7,
                CalibrationTarget::Speed => 0.5,
                CalibrationTarget::Thoroughness => 0.6,
                CalibrationTarget::Creativity => 0.5,
                CalibrationTarget::Conservatism => 0.5,
            };
            states.insert(target, CalibrationState::new(target, initial));
        }
        Self {
            config,
            states,
            signals: VecDeque::with_capacity(200),
        }
    }

    pub fn ingest_signal(&mut self, signal: CalibrationSignal) {
        if self.signals.len() >= 200 {
            self.signals.pop_front();
        }
        self.signals.push_back(signal.clone());
        if let Some(state) = self.states.get_mut(&signal.target) {
            let adj = signal.adjustment() * self.config.adaptation_rate;
            state.apply_adjustment(adj);
        }
    }

    pub fn get_value(&self, target: CalibrationTarget) -> Option<f64> {
        self.states.get(&target).map(|s| s.current_value)
    }

    pub fn set_target(&mut self, target: CalibrationTarget, value: f64) {
        if let Some(state) = self.states.get_mut(&target) {
            state.target_value = value.max(0.0).min(1.0);
        }
    }

    pub fn is_calibrated(&self, target: CalibrationTarget) -> bool {
        self.states
            .get(&target)
            .map(|s| s.is_calibrated(self.config.tolerance))
            .unwrap_or(false)
    }

    pub fn report(&self) -> CalibrationReport {
        let mut calibrated_count = 0u32;
        let mut gap_sum = 0.0f64;
        let mut biggest_gap = (CalibrationTarget::Confidence, 0.0f64);
        let mut recommendations = Vec::new();

        for state in self.states.values() {
            let gap = (state.current_value - state.target_value).abs();
            gap_sum += gap;
            if gap <= self.config.tolerance {
                calibrated_count += 1;
            } else {
                let dir = if state.current_value < state.target_value {
                    "increase"
                } else {
                    "decrease"
                };
                recommendations.push(format!(
                    "{:?}: {:?} by {:.2}",
                    state.target, dir, gap
                ));
            }
            if gap > biggest_gap.1 {
                biggest_gap = (state.target, gap);
            }
        }

        let total = self.states.len() as u32;
        CalibrationReport {
            calibrated_count,
            total_targets: total,
            avg_gap: if total == 0 { 0.0 } else { gap_sum / total as f64 },
            biggest_gap,
            recommendations,
        }
    }
}

impl Default for MetaCalibrator {
    fn default() -> Self {
        Self::new(MetaCalibratorConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_adjustment_direction() {
        let signal = CalibrationSignal::new(CalibrationTarget::Confidence, 0.3, 0.7, 1.0);
        let adj = signal.adjustment();
        assert!(adj > 0.0);
    }

    #[test]
    fn test_ingest_moves_value() {
        let mut cal = MetaCalibrator::default();
        let initial = cal.get_value(CalibrationTarget::Confidence).unwrap();
        for _ in 0..10 {
            cal.ingest_signal(CalibrationSignal::new(
                CalibrationTarget::Confidence, 0.3, 0.7, 1.0,
            ));
        }
        let after = cal.get_value(CalibrationTarget::Confidence).unwrap();
        assert!((after - initial).abs() > 0.001);
    }

    #[test]
    fn test_set_target_changes_calibration() {
        let mut cal = MetaCalibrator::default();
        assert_eq!(cal.get_value(CalibrationTarget::Creativity), Some(0.5));
        assert!(cal.is_calibrated(CalibrationTarget::Creativity));
        cal.set_target(CalibrationTarget::Creativity, 0.8);
        let gap = (cal.get_value(CalibrationTarget::Creativity).unwrap() - 0.8).abs();
        assert!(gap > 0.0, "setting target to 0.8 should create a gap from current 0.5");
    }

    #[test]
    fn test_report_counts() {
        let cal = MetaCalibrator::default();
        let report = cal.report();
        assert_eq!(report.total_targets, 5);
    }

    #[test]
    fn test_trend_after_multiple_updates() {
        let mut state = CalibrationState::new(CalibrationTarget::Confidence, 0.5);
        for _i in 0..15 {
            state.apply_adjustment(0.05);
        }
        let trend = state.trend();
        assert!(trend > 0.0);
    }
}
