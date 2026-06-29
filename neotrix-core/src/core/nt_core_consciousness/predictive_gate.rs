#[derive(Debug, Clone)]
pub struct PredictiveGate {
    pub threshold: f64,
    pub adaptation_rate: f64,
    pub min_threshold: f64,
    pub max_threshold: f64,
    pub recent_surprisals: Vec<f64>,
    pub window_size: usize,
}

impl PredictiveGate {
    pub fn new(threshold: f64) -> Self {
        Self {
            threshold,
            adaptation_rate: 0.05,
            min_threshold: 0.1,
            max_threshold: 10.0,
            recent_surprisals: Vec::new(),
            window_size: 100,
        }
    }

    pub fn surprisal(&self, prediction_error: f64, expected_uncertainty: f64) -> f64 {
        if expected_uncertainty <= 0.0 {
            if prediction_error > 0.0 {
                10.0
            } else {
                0.0
            }
        } else {
            (prediction_error / expected_uncertainty).abs()
        }
    }

    pub fn should_write(&self, prediction_error: f64, expected_uncertainty: f64) -> bool {
        let s = self.surprisal(prediction_error, expected_uncertainty);
        s > self.threshold
    }

    pub fn record_and_adapt(&mut self, prediction_error: f64, expected_uncertainty: f64) -> bool {
        let s = self.surprisal(prediction_error, expected_uncertainty);
        self.recent_surprisals.push(s);
        if self.recent_surprisals.len() > self.window_size {
            self.recent_surprisals.remove(0);
        }
        let mean: f64 =
            self.recent_surprisals.iter().sum::<f64>() / self.recent_surprisals.len().max(1) as f64;
        let target = mean * 1.5;
        self.threshold += (target - self.threshold) * self.adaptation_rate;
        self.threshold = self.threshold.clamp(self.min_threshold, self.max_threshold);
        s > self.threshold
    }

    pub fn gate_probability(&self, prediction_error: f64, expected_uncertainty: f64) -> f64 {
        let s = self.surprisal(prediction_error, expected_uncertainty);
        1.0 / (1.0 + (-(s - self.threshold)).exp())
    }

    pub fn reset(&mut self) {
        self.recent_surprisals.clear();
        self.threshold = 1.0;
    }
}

pub struct PredictiveGateConfig {
    pub base_threshold: f64,
    pub adaptation_rate: f64,
    pub use_msv_integration: bool,
}

impl Default for PredictiveGateConfig {
    fn default() -> Self {
        Self {
            base_threshold: 1.0,
            adaptation_rate: 0.05,
            use_msv_integration: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_low_error_no_write() {
        let gate = PredictiveGate::new(2.0);
        assert!(!gate.should_write(0.1, 1.0));
    }

    #[test]
    fn test_high_error_writes() {
        let gate = PredictiveGate::new(1.0);
        assert!(gate.should_write(5.0, 1.0));
    }

    #[test]
    fn test_surprisal_zero_uncertainty() {
        let gate = PredictiveGate::new(1.0);
        assert_eq!(gate.surprisal(0.0, 0.0), 0.0);
        assert_eq!(gate.surprisal(5.0, 0.0), 10.0);
    }

    #[test]
    fn test_gate_probability_sigmoid() {
        let gate = PredictiveGate::new(2.0);
        let p_low = gate.gate_probability(0.1, 1.0);
        let p_high = gate.gate_probability(5.0, 1.0);
        assert!(p_low < 0.5, "low error should have low prob, got {}", p_low);
        assert!(
            p_high > 0.8,
            "high error should have high prob, got {}",
            p_high
        );
    }

    #[test]
    fn test_adaptation_raises_threshold_on_high_surprisal() {
        let mut gate = PredictiveGate::new(1.0);
        for _ in 0..50 {
            gate.record_and_adapt(5.0, 1.0);
        }
        assert!(
            gate.threshold > 1.0,
            "threshold should rise with high surprisal"
        );
    }

    #[test]
    fn test_adaptation_lowers_threshold_on_low_surprisal() {
        let mut gate = PredictiveGate::new(5.0);
        for _ in 0..50 {
            gate.record_and_adapt(0.1, 1.0);
        }
        assert!(
            gate.threshold < 5.0,
            "threshold should fall with low surprisal"
        );
    }

    #[test]
    fn test_threshold_clamping() {
        let mut gate = PredictiveGate::new(1.0);
        for _ in 0..500 {
            gate.record_and_adapt(100.0, 1.0);
        }
        assert!(gate.threshold <= gate.max_threshold);
        for _ in 0..500 {
            gate.record_and_adapt(0.0, 1.0);
        }
        assert!(gate.threshold >= gate.min_threshold);
    }

    #[test]
    fn test_reset() {
        let mut gate = PredictiveGate::new(1.0);
        gate.record_and_adapt(5.0, 1.0);
        gate.reset();
        assert_eq!(gate.threshold, 1.0);
        assert!(gate.recent_surprisals.is_empty());
    }

    #[test]
    fn test_borderline_error() {
        let gate = PredictiveGate::new(2.0);
        let write = gate.should_write(2.0, 1.0);
        let p = gate.gate_probability(2.0, 1.0);
        assert!(!write || p > 0.4);
    }

    #[test]
    fn test_window_sliding() {
        let mut gate = PredictiveGate::new(1.0);
        for _ in 0..200 {
            gate.record_and_adapt(1.0, 1.0);
        }
        assert!(gate.recent_surprisals.len() <= gate.window_size);
    }
}
