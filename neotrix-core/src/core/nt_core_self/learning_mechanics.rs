use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum ObservableType {
    WeightNorm,
    GradientNorm,
    RepresentationSimilarity,
    AttentionEntropy,
    LossValue,
    Accuracy,
    ActivationSparsity,
    WeightUpdateMagnitude,
    GradientNoiseScale,
}

impl ObservableType {
    pub fn name(&self) -> &'static str {
        match self {
            Self::WeightNorm => "WeightNorm",
            Self::GradientNorm => "GradientNorm",
            Self::RepresentationSimilarity => "RepresentationSimilarity",
            Self::AttentionEntropy => "AttentionEntropy",
            Self::LossValue => "LossValue",
            Self::Accuracy => "Accuracy",
            Self::ActivationSparsity => "ActivationSparsity",
            Self::WeightUpdateMagnitude => "WeightUpdateMagnitude",
            Self::GradientNoiseScale => "GradientNoiseScale",
        }
    }

    pub fn category(&self) -> &'static str {
        match self {
            Self::WeightNorm => "weights",
            Self::GradientNorm => "gradients",
            Self::RepresentationSimilarity => "representations",
            Self::AttentionEntropy => "attention",
            Self::LossValue => "loss",
            Self::Accuracy => "accuracy",
            Self::ActivationSparsity => "activations",
            Self::WeightUpdateMagnitude => "weights",
            Self::GradientNoiseScale => "gradients",
        }
    }
}

pub struct ObservableSample {
    pub timestamp: u64,
    pub value: f64,
    pub observable: ObservableType,
    pub metadata: String,
}

pub struct PhaseTransitionSignal {
    pub observable: ObservableType,
    pub transition_time: u64,
    pub magnitude: f64,
    pub direction: &'static str,
}

pub struct LearningMechanicsObservatory {
    observations: HashMap<ObservableType, Vec<ObservableSample>>,
    max_samples: usize,
    phase_boundary_threshold: f64,
}

impl LearningMechanicsObservatory {
    pub fn new() -> Self {
        Self {
            observations: HashMap::new(),
            max_samples: 10000,
            phase_boundary_threshold: 2.0,
        }
    }

    pub fn record(&mut self, observable: ObservableType, value: f64, metadata: &str) {
        let entry = self.observations.entry(observable.clone()).or_default();
        let timestamp = entry.last().map(|s| s.timestamp + 1).unwrap_or(0);
        entry.push(ObservableSample {
            timestamp,
            value,
            observable,
            metadata: metadata.to_string(),
        });
        if entry.len() > self.max_samples {
            entry.remove(0);
        }
    }

    pub fn get_series(&self, observable: &ObservableType) -> Vec<&ObservableSample> {
        self.observations
            .get(observable)
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }

    fn window_samples(&self, observable: &ObservableType, window: usize) -> Vec<&ObservableSample> {
        self.observations
            .get(observable)
            .map(|v| {
                let start = if v.len() > window {
                    v.len() - window
                } else {
                    0
                };
                v[start..].iter().collect()
            })
            .unwrap_or_default()
    }

    pub fn recent_mean(&self, observable: &ObservableType, window: usize) -> Option<f64> {
        let samples = self.window_samples(observable, window);
        if samples.is_empty() {
            return None;
        }
        let sum: f64 = samples.iter().map(|s| s.value).sum();
        Some(sum / samples.len() as f64)
    }

    pub fn recent_std(&self, observable: &ObservableType, window: usize) -> Option<f64> {
        let mean = self.recent_mean(observable, window)?;
        let samples = self.window_samples(observable, window);
        if samples.len() < 2 {
            return None;
        }
        let variance: f64 = samples
            .iter()
            .map(|s| (s.value - mean).powi(2))
            .sum::<f64>()
            / (samples.len() - 1) as f64;
        Some(variance.sqrt())
    }

    pub fn detect_phase_transitions(
        &self,
        observable: &ObservableType,
        window_size: usize,
    ) -> Vec<PhaseTransitionSignal> {
        let samples = match self.observations.get(observable) {
            Some(s) if s.len() >= window_size * 2 + 1 => s,
            _ => return vec![],
        };

        let mut results = Vec::new();

        for i in window_size..samples.len() - window_size {
            let before: Vec<f64> = samples[i - window_size..i]
                .iter()
                .map(|s| s.value)
                .collect();
            let after: Vec<f64> = samples[i..i + window_size]
                .iter()
                .map(|s| s.value)
                .collect();

            let before_mean = before.iter().sum::<f64>() / before.len() as f64;
            let after_mean = after.iter().sum::<f64>() / after.len() as f64;

            let combined: Vec<f64> = before.iter().chain(after.iter()).copied().collect();
            let combined_mean = combined.iter().sum::<f64>() / combined.len() as f64;
            let variance: f64 = combined
                .iter()
                .map(|x| (x - combined_mean).powi(2))
                .sum::<f64>()
                / (combined.len() - 1) as f64;
            let std = variance.sqrt();
            if std == 0.0 {
                continue;
            }

            let diff = (after_mean - before_mean).abs();
            if diff / std > self.phase_boundary_threshold {
                let direction = if after_mean > before_mean {
                    "increasing"
                } else {
                    "decreasing"
                };
                results.push(PhaseTransitionSignal {
                    observable: observable.clone(),
                    transition_time: samples[i].timestamp,
                    magnitude: diff / std,
                    direction,
                });
            }
        }

        results
    }

    pub fn correlation(
        &self,
        a: &ObservableType,
        b: &ObservableType,
        window: usize,
    ) -> Option<f64> {
        let a_samples = self.window_samples(a, window);
        let b_samples = self.window_samples(b, window);
        let n = a_samples.len().min(b_samples.len());
        if n < 2 {
            return None;
        }

        let a_vals: Vec<f64> = a_samples
            .iter()
            .rev()
            .take(n)
            .rev()
            .map(|s| s.value)
            .collect();
        let b_vals: Vec<f64> = b_samples
            .iter()
            .rev()
            .take(n)
            .rev()
            .map(|s| s.value)
            .collect();

        let a_mean = a_vals.iter().sum::<f64>() / n as f64;
        let b_mean = b_vals.iter().sum::<f64>() / n as f64;

        let mut num = 0.0;
        let mut den_a = 0.0;
        let mut den_b = 0.0;
        for i in 0..n {
            let da = a_vals[i] - a_mean;
            let db = b_vals[i] - b_mean;
            num += da * db;
            den_a += da * da;
            den_b += db * db;
        }

        let den = (den_a * den_b).sqrt();
        if den == 0.0 {
            None
        } else {
            Some(num / den)
        }
    }

    pub fn top_informative_observables(
        &self,
        target: &ObservableType,
        k: usize,
    ) -> Vec<(ObservableType, f64)> {
        let mut scores: Vec<(ObservableType, f64)> = self
            .observations
            .keys()
            .filter(|o| *o != target)
            .filter_map(|o| {
                self.correlation(target, o, 100)
                    .map(|c| (o.clone(), c.abs()))
            })
            .collect();

        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scores.truncate(k);
        scores
    }

    pub fn slope(&self, observable: &ObservableType, window: usize) -> Option<f64> {
        let samples = self.window_samples(observable, window);
        let n = samples.len();
        if n < 2 {
            return None;
        }

        let x_mean = (n - 1) as f64 / 2.0;
        let y_mean: f64 = samples.iter().map(|s| s.value).sum::<f64>() / n as f64;

        let mut num = 0.0;
        let mut den = 0.0;
        for (i, s) in samples.iter().enumerate() {
            let dx = i as f64 - x_mean;
            let dy = s.value - y_mean;
            num += dx * dy;
            den += dx * dx;
        }

        if den == 0.0 {
            None
        } else {
            Some(num / den)
        }
    }

    pub fn grokking_score(
        &self,
        loss_obs: &ObservableType,
        acc_obs: &ObservableType,
    ) -> Option<f64> {
        let loss_slope = self.slope(loss_obs, 200)?;
        let acc_slope = self.slope(acc_obs, 200)?;

        let mismatch = loss_slope.abs() + acc_slope.abs();
        if mismatch.abs() < 1e-12 {
            return Some(0.0);
        }

        let alignment = if loss_slope < 0.0 && acc_slope > 0.0 {
            1.0
        } else if loss_slope > 0.0 && acc_slope < 0.0 {
            -1.0
        } else {
            0.0
        };

        Some(alignment * mismatch)
    }
}

pub struct LearningMechanicsReport {
    pub phase_transitions: Vec<PhaseTransitionSignal>,
    pub active_correlations: Vec<(ObservableType, ObservableType, f64)>,
    pub grokking_score: f64,
    pub epochs_since_last_transition: u64,
}

impl LearningMechanicsReport {
    pub fn summary(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!("Learning Mechanics Report\n"));
        s.push_str(&format!("========================\n"));
        s.push_str(&format!(
            "Phase transitions detected: {}\n",
            self.phase_transitions.len()
        ));
        for pt in &self.phase_transitions {
            s.push_str(&format!(
                "  {} at t={}: mag={:.2}, dir={}\n",
                pt.observable.name(),
                pt.transition_time,
                pt.magnitude,
                pt.direction
            ));
        }
        s.push_str(&format!(
            "Active correlations: {}\n",
            self.active_correlations.len()
        ));
        for (a, b, c) in &self.active_correlations {
            s.push_str(&format!("  {} vs {}: r={:.3}\n", a.name(), b.name(), c));
        }
        s.push_str(&format!("Grokking score: {:.4}\n", self.grokking_score));
        s.push_str(&format!(
            "Epochs since last transition: {}\n",
            self.epochs_since_last_transition
        ));
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fill_observatory() -> LearningMechanicsObservatory {
        let mut obs = LearningMechanicsObservatory::new();
        for i in 0..500 {
            let v = if i < 100 {
                1.0
            } else if i < 300 {
                1.0 + (i as f64 - 100.0) * 0.01
            } else {
                3.0 + (i as f64 - 300.0) * 0.005
            };
            obs.record(ObservableType::LossValue, v, "train");
            obs.record(
                ObservableType::Accuracy,
                0.5 + (i as f64).ln() * 0.05,
                "train",
            );
            obs.record(
                ObservableType::WeightNorm,
                2.0 + (i as f64).sqrt() * 0.1,
                "train",
            );
        }
        obs
    }

    #[test]
    fn test_record_samples() {
        let mut obs = LearningMechanicsObservatory::new();
        obs.record(ObservableType::LossValue, 1.0, "train");
        obs.record(ObservableType::LossValue, 2.0, "train");
        let series = obs.get_series(&ObservableType::LossValue);
        assert_eq!(series.len(), 2);
        assert_eq!(series[0].value, 1.0);
        assert_eq!(series[1].value, 2.0);
        assert_eq!(series[0].timestamp, 0);
        assert_eq!(series[1].timestamp, 1);
    }

    #[test]
    fn test_recent_mean() {
        let mut obs = LearningMechanicsObservatory::new();
        for i in 0..10 {
            obs.record(ObservableType::LossValue, i as f64, "train");
        }
        let mean = obs.recent_mean(&ObservableType::LossValue, 5).unwrap();
        assert!((mean - 7.0).abs() < 1e-6);
    }

    #[test]
    fn test_recent_std() {
        let mut obs = LearningMechanicsObservatory::new();
        for i in 0..10 {
            obs.record(ObservableType::LossValue, i as f64, "train");
        }
        let std = obs.recent_std(&ObservableType::LossValue, 5).unwrap();
        assert!((std - 1.5811388300841898).abs() < 1e-6);
    }

    #[test]
    fn test_detect_phase_transition_with_step() {
        let mut obs = LearningMechanicsObservatory::new();
        for i in 0..300 {
            let v = if i < 100 { 1.0 } else { 5.0 };
            obs.record(ObservableType::LossValue, v, "train");
        }
        let transitions = obs.detect_phase_transitions(&ObservableType::LossValue, 20);
        assert!(
            !transitions.is_empty(),
            "should detect at least one transition"
        );
        for t in &transitions {
            assert_eq!(t.direction, "increasing");
        }
    }

    #[test]
    fn test_correlation_perfect_positive() {
        let mut obs = LearningMechanicsObservatory::new();
        for i in 0..100 {
            obs.record(ObservableType::WeightNorm, i as f64, "");
            obs.record(ObservableType::WeightUpdateMagnitude, i as f64, "");
        }
        let corr = obs
            .correlation(
                &ObservableType::WeightNorm,
                &ObservableType::WeightUpdateMagnitude,
                100,
            )
            .unwrap();
        assert!((corr - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_correlation_perfect_negative() {
        let mut obs = LearningMechanicsObservatory::new();
        for i in 0..100 {
            obs.record(ObservableType::WeightNorm, i as f64, "");
            obs.record(ObservableType::WeightUpdateMagnitude, 99.0 - i as f64, "");
        }
        let corr = obs
            .correlation(
                &ObservableType::WeightNorm,
                &ObservableType::WeightUpdateMagnitude,
                100,
            )
            .unwrap();
        assert!((corr - (-1.0)).abs() < 1e-6);
    }

    #[test]
    fn test_top_informative_observables() {
        let mut obs = LearningMechanicsObservatory::new();
        for i in 0..100 {
            obs.record(ObservableType::LossValue, i as f64, "");
            obs.record(ObservableType::Accuracy, i as f64, "");
            obs.record(ObservableType::WeightNorm, (99 - i) as f64, "");
            obs.record(ObservableType::GradientNorm, (i * i) as f64, "");
        }
        let top = obs.top_informative_observables(&ObservableType::LossValue, 2);
        assert_eq!(top.len(), 2);
        assert!((top[0].1 - 1.0).abs() < 1e-6 || (top[1].1 - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_slope_linear() {
        let mut obs = LearningMechanicsObservatory::new();
        for i in 0..50 {
            obs.record(ObservableType::LossValue, 1.0 + 0.5 * i as f64, "");
        }
        let slope = obs.slope(&ObservableType::LossValue, 50).unwrap();
        assert!((slope - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_slope_flat() {
        let mut obs = LearningMechanicsObservatory::new();
        for _ in 0..50 {
            obs.record(ObservableType::LossValue, 3.0, "");
        }
        let slope = obs.slope(&ObservableType::LossValue, 50).unwrap();
        assert!(slope.abs() < 1e-10);
    }

    #[test]
    fn test_grokking_score() {
        let mut obs = LearningMechanicsObservatory::new();
        for i in 0..300 {
            let loss = if i < 100 { 5.0 - i as f64 * 0.04 } else { 1.0 };
            let acc = if i < 100 { 0.3 + i as f64 * 0.006 } else { 0.9 };
            obs.record(ObservableType::LossValue, loss.max(0.1), "train");
            obs.record(ObservableType::Accuracy, acc.min(1.0), "train");
        }
        let score = obs.grokking_score(&ObservableType::LossValue, &ObservableType::Accuracy);
        assert!(score.is_some());
        assert!(score.unwrap() > 0.0);
    }

    #[test]
    fn test_empty_state() {
        let obs = LearningMechanicsObservatory::new();
        assert!(obs.recent_mean(&ObservableType::LossValue, 10).is_none());
        assert!(obs.recent_std(&ObservableType::LossValue, 10).is_none());
        assert!(obs
            .correlation(&ObservableType::LossValue, &ObservableType::Accuracy, 10)
            .is_none());
        assert!(obs.slope(&ObservableType::LossValue, 10).is_none());
        assert!(obs
            .grokking_score(&ObservableType::LossValue, &ObservableType::Accuracy)
            .is_none());
        assert!(obs
            .detect_phase_transitions(&ObservableType::LossValue, 10)
            .is_empty());
        assert!(obs.get_series(&ObservableType::LossValue).is_empty());
    }

    #[test]
    fn test_single_sample_edge_case() {
        let mut obs = LearningMechanicsObservatory::new();
        obs.record(ObservableType::LossValue, 1.0, "train");
        assert!(obs.recent_std(&ObservableType::LossValue, 10).is_none());
        assert!(obs.slope(&ObservableType::LossValue, 10).is_none());
        assert!(obs
            .correlation(&ObservableType::LossValue, &ObservableType::Accuracy, 10)
            .is_none());
        assert_eq!(obs.get_series(&ObservableType::LossValue).len(), 1);
    }

    #[test]
    fn test_max_samples_pruning() {
        let mut obs = LearningMechanicsObservatory::new();
        obs.max_samples = 5;
        for i in 0..10 {
            obs.record(ObservableType::LossValue, i as f64, "train");
        }
        let series = obs.get_series(&ObservableType::LossValue);
        assert_eq!(series.len(), 5);
        assert_eq!(series[0].value, 5.0);
        assert_eq!(series[4].value, 9.0);
    }

    #[test]
    fn test_phase_boundary_threshold_sensitivity() {
        let mut obs = LearningMechanicsObservatory::new();
        obs.phase_boundary_threshold = 10.0;
        for i in 0..300 {
            let v = if i < 100 { 1.0 } else { 5.0 };
            obs.record(ObservableType::LossValue, v, "train");
        }
        let transitions = obs.detect_phase_transitions(&ObservableType::LossValue, 20);
        assert_eq!(
            transitions.len(),
            0,
            "high threshold should suppress detection"
        );

        obs.phase_boundary_threshold = 1.0;
        let transitions = obs.detect_phase_transitions(&ObservableType::LossValue, 20);
        assert!(!transitions.is_empty(), "low threshold should detect");
    }

    #[test]
    fn test_report_summary() {
        let report = LearningMechanicsReport {
            phase_transitions: vec![PhaseTransitionSignal {
                observable: ObservableType::LossValue,
                transition_time: 150,
                magnitude: 3.2,
                direction: "decreasing",
            }],
            active_correlations: vec![(ObservableType::LossValue, ObservableType::Accuracy, -0.95)],
            grokking_score: 1.5,
            epochs_since_last_transition: 50,
        };
        let summary = report.summary();
        assert!(summary.contains("Phase transitions detected: 1"));
        assert!(summary.contains("LossValue"));
        assert!(summary.contains("Accuracy"));
        assert!(summary.contains("Grokking score: 1.5"));
    }

    #[test]
    fn test_observable_type_methods() {
        assert_eq!(ObservableType::WeightNorm.name(), "WeightNorm");
        assert_eq!(ObservableType::WeightNorm.category(), "weights");
        assert_eq!(ObservableType::GradientNorm.name(), "GradientNorm");
        assert_eq!(ObservableType::GradientNorm.category(), "gradients");
        assert_eq!(ObservableType::AttentionEntropy.name(), "AttentionEntropy");
        assert_eq!(ObservableType::AttentionEntropy.category(), "attention");
    }

    #[test]
    fn test_top_k_returns_k_or_less() {
        let obs = fill_observatory();
        let top = obs.top_informative_observables(&ObservableType::LossValue, 1);
        assert!(top.len() <= 1);
    }
}
