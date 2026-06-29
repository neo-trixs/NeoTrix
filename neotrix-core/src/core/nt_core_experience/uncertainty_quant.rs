/// P1.6 Uncertainty Quantification
/// Each pipeline step outputs confidence intervals.
/// Uncertainty drives curiosity signals.

#[derive(Debug, Clone)]
pub struct ConfidenceInterval {
    pub mean: f64,
    pub lower: f64,
    pub upper: f64,
    pub width: f64,
}

impl ConfidenceInterval {
    pub fn new(mean: f64, std_err: f64) -> Self {
        let margin = 1.96 * std_err;
        ConfidenceInterval {
            mean,
            lower: mean - margin,
            upper: mean + margin,
            width: 2.0 * margin,
        }
    }

    pub fn uncertainty_signal(&self) -> f64 {
        (self.width / (self.mean.abs() + 1.0)).min(1.0)
    }
}

#[derive(Debug, Clone)]
pub struct StepConfidence {
    pub step_name: &'static str,
    pub confidence: f64,
    pub interval: ConfidenceInterval,
    pub sample_count: u64,
}

#[derive(Debug)]
pub struct UncertaintyAwareMonitor {
    pub step_confidence: Vec<StepConfidence>,
    pub global_uncertainty: f64,
    pub curiosity_signal: f64,
    pub last_combined_health: f64,
    pub enabled: bool,
}

impl UncertaintyAwareMonitor {
    pub fn new() -> Self {
        UncertaintyAwareMonitor {
            step_confidence: Vec::with_capacity(20),
            global_uncertainty: 0.0,
            curiosity_signal: 0.0,
            last_combined_health: 1.0,
            enabled: true,
        }
    }

    pub fn record_step(&mut self, name: &'static str, mean: f64, std_err: f64, n: u64) {
        let interval = ConfidenceInterval::new(mean, std_err);
        let confidence = 1.0 - interval.uncertainty_signal();

        if let Some(existing) = self
            .step_confidence
            .iter_mut()
            .find(|s| s.step_name == name)
        {
            existing.confidence = confidence;
            existing.interval = interval;
            existing.sample_count = n;
        } else {
            self.step_confidence.push(StepConfidence {
                step_name: name,
                confidence,
                interval,
                sample_count: n,
            });
        }

        self.update_global();
    }

    fn update_global(&mut self) {
        if self.step_confidence.is_empty() {
            self.global_uncertainty = 0.0;
            self.curiosity_signal = 0.0;
            return;
        }
        let avg_uncertainty: f64 = self
            .step_confidence
            .iter()
            .map(|s| 1.0 - s.confidence)
            .sum::<f64>()
            / self.step_confidence.len() as f64;
        self.global_uncertainty = avg_uncertainty;

        self.curiosity_signal = if avg_uncertainty > 0.8 {
            0.0
        } else if avg_uncertainty < 0.1 {
            0.2
        } else {
            1.0 - (avg_uncertainty - 0.5).abs() * 2.0
        };
    }

    pub fn uncertainty_report(&self) -> String {
        let mut s = format!(
            "Uncertainty Monitor | global={:.3} curiosity={:.3}\n",
            self.global_uncertainty, self.curiosity_signal
        );
        for step in &self.step_confidence {
            s.push_str(&format!(
                "  {:<30} conf={:.3} CI=[{:.3},{:.3}] n={}\n",
                step.step_name,
                step.confidence,
                step.interval.lower,
                step.interval.upper,
                step.sample_count
            ));
        }
        s
    }
}
