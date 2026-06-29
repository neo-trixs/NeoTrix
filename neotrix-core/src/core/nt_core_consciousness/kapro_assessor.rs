/// KAPRO ActingDimensionAssessor — arXiv 2606.20661
///
/// Measures the Knowing→Acting gap in self-awareness:
/// - knowing_dimension: how well the system knows its own uncertainty (MetaAccuracy, ECE)
/// - acting_dimension: how well the system acts on that self-knowledge (behavioral adjustment)
/// - kapro_gap: |knowing - acting| — lower is better, gap > 0.3 signals dissociation
///
/// Reference: KAPRO framework (arXiv 2606.20661), Mirror benchmark (arXiv 2604.19809)
/// Maps to NeoTrix CV.1 (监控-控制分离) and CX.1 (Knowing≠Acting in Self-Awareness)

#[derive(Debug, Clone)]
pub struct ActingDimensionAssessor {
    /// Calibration-based knowing dimension (ECE → knowing score)
    pub knowing_dimension: f64,
    /// Behavioral adjustment acting dimension (how well system acts on knowledge)
    pub acting_dimension: f64,
    /// |knowing - acting| — the dissociation gap
    pub kapro_gap: f64,
    /// History of (knowing, acting) pairs for trend analysis
    pub history: Vec<(f64, f64)>,
    /// Maximum history length
    pub max_history: usize,
    /// EMA of kapro gap for trend detection
    pub ema_gap: f64,
    /// Number of assessments performed
    pub assessment_count: u64,
    /// Whether dissociation (gap > 0.3) has been detected
    pub dissociation_detected: bool,
}

#[derive(Debug, Clone)]
pub struct KaproReport {
    pub knowing_dimension: f64,
    pub acting_dimension: f64,
    pub kapro_gap: f64,
    pub dissociation_detected: bool,
    pub ema_gap: f64,
    pub gap_trend: String,
}

impl ActingDimensionAssessor {
    pub fn new() -> Self {
        Self {
            knowing_dimension: 0.5,
            acting_dimension: 0.5,
            kapro_gap: 0.0,
            history: Vec::with_capacity(100),
            max_history: 100,
            ema_gap: 0.0,
            assessment_count: 0,
            dissociation_detected: false,
        }
    }

    /// Assess the Knowing→Acting gap using calibration data and behavioral feedback.
    pub fn assess(&mut self, meta_d: f64, ece: f64, behavioral_adjustment: f64) -> KaproReport {
        self.assessment_count += 1;

        // Knowing dimension: calibration accuracy
        // meta_d > 1.0 = above-chance self-knowledge, ece < 0.15 = well-calibrated
        let knowing = ((meta_d - 0.5).max(0.0).min(2.0) / 2.0 + (1.0 - ece.clamp(0.0, 1.0))) / 2.0;

        // Acting dimension: behavioral adjustment
        // How much does the system actually adjust behavior based on self-knowledge?
        let acting = behavioral_adjustment.clamp(0.0, 1.0);

        let gap = (knowing - acting).abs();
        self.knowing_dimension = knowing;
        self.acting_dimension = acting;
        self.kapro_gap = gap;

        // EMA update
        if self.assessment_count == 1 {
            self.ema_gap = gap;
        } else {
            self.ema_gap = 0.3 * gap + 0.7 * self.ema_gap;
        }

        self.dissociation_detected = gap > 0.3;

        // History
        self.history.push((knowing, acting));
        while self.history.len() > self.max_history {
            self.history.remove(0);
        }

        // Trend
        let gap_trend = if self.history.len() >= 5 {
            let recent = self
                .history
                .iter()
                .rev()
                .take(5)
                .map(|(k, a)| (k - a).abs())
                .sum::<f64>()
                / 5.0;
            let older = self
                .history
                .iter()
                .rev()
                .skip(5)
                .take(5)
                .map(|(k, a)| (k - a).abs())
                .sum::<f64>()
                / 5.max(1) as f64;
            if recent > older + 0.05 {
                "worsening".to_string()
            } else if recent < older - 0.05 {
                "improving".to_string()
            } else {
                "stable".to_string()
            }
        } else {
            "insufficient_data".to_string()
        };

        KaproReport {
            knowing_dimension: knowing,
            acting_dimension: acting,
            kapro_gap: gap,
            dissociation_detected: self.dissociation_detected,
            ema_gap: self.ema_gap,
            gap_trend,
        }
    }

    /// Quick summary for meta_insights logging.
    pub fn summary(&self) -> String {
        format!(
            "kapro:knowing={:.2} acting={:.2} gap={:.2} ema={:.3} dissociation={}",
            self.knowing_dimension,
            self.acting_dimension,
            self.kapro_gap,
            self.ema_gap,
            self.dissociation_detected
        )
    }

    pub fn reset(&mut self) {
        self.knowing_dimension = 0.5;
        self.acting_dimension = 0.5;
        self.kapro_gap = 0.0;
        self.history.clear();
        self.ema_gap = 0.0;
        self.assessment_count = 0;
        self.dissociation_detected = false;
    }
}

impl Default for ActingDimensionAssessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_assessor() {
        let a = ActingDimensionAssessor::new();
        assert!((a.knowing_dimension - 0.5).abs() < 1e-6);
        assert!((a.acting_dimension - 0.5).abs() < 1e-6);
        assert!(!a.dissociation_detected);
    }

    #[test]
    fn test_good_calibration_good_acting() {
        let mut a = ActingDimensionAssessor::new();
        let report = a.assess(1.5, 0.05, 0.9);
        assert!(report.knowing_dimension > 0.7);
        assert!(report.acting_dimension > 0.8);
        assert!(report.kapro_gap < 0.3);
        assert!(!report.dissociation_detected);
    }

    #[test]
    fn test_poor_calibration_poor_acting_large_gap() {
        let mut a = ActingDimensionAssessor::new();
        let report = a.assess(0.6, 0.4, 0.2);
        assert!(report.knowing_dimension < 0.6);
        assert!(report.acting_dimension < 0.3);
        assert!(report.kapro_gap < 0.5);
    }

    #[test]
    fn test_dissociation_detection() {
        let mut a = ActingDimensionAssessor::new();
        let report = a.assess(2.0, 0.02, 0.1); // high knowing, low acting
        assert!(report.kapro_gap > 0.3);
        assert!(report.dissociation_detected);
    }

    #[test]
    fn test_multiple_assessments_trend() {
        let mut a = ActingDimensionAssessor::new();
        for _ in 0..10 {
            a.assess(1.0, 0.2, 0.5);
        }
        assert_eq!(a.assessment_count, 10);
        assert_eq!(a.history.len(), 10);
        assert!(a.summary().contains("kapro:"));
    }

    #[test]
    fn test_reset() {
        let mut a = ActingDimensionAssessor::new();
        a.assess(1.5, 0.05, 0.9);
        a.reset();
        assert!((a.knowing_dimension - 0.5).abs() < 1e-6);
        assert_eq!(a.assessment_count, 0);
    }
}
