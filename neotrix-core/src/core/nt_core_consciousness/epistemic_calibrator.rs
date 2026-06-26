use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone)]
pub struct CalibrationRecord {
    pub domain: String,
    pub predicted_confidence: f64,
    pub actual_outcome: f64,
    pub timestamp: u64,
    pub calibrated_confidence: f64,
}

#[derive(Debug, Clone)]
pub struct KnowledgeBoundary {
    pub domain: String,
    pub confidence_threshold: f64,
    pub accuracy_estimate: f64,
    pub sample_size: u32,
}

#[derive(Debug, Clone)]
pub struct EpistemicCalibrator {
    pub records: VecDeque<CalibrationRecord>,
    pub boundaries: HashMap<String, KnowledgeBoundary>,
    pub max_records: usize,
    pub min_samples: u32,
    pub default_threshold: f64,
    pub learning_rate: f64,
    pub update_count: u64,
}

impl EpistemicCalibrator {
    pub fn new() -> Self {
        Self {
            records: VecDeque::new(),
            boundaries: HashMap::new(),
            max_records: 500,
            min_samples: 5,
            default_threshold: 0.3,
            learning_rate: 0.1,
            update_count: 0,
        }
    }

    pub fn calibrate_confidence(&mut self, raw_confidence: f64, domain: &str) -> f64 {
        let clamped = raw_confidence.clamp(0.1, 0.9);
        let boundary = self.boundaries.get(domain);
        match boundary {
            Some(b) if b.sample_size >= self.min_samples => {
                let diff = b.accuracy_estimate - b.confidence_threshold;
                let adjustment = diff * self.learning_rate;
                let adjusted = clamped + adjustment;
                adjusted.clamp(0.1, 0.9)
            }
            _ => clamped,
        }
    }

    pub fn knows_boundary(&self, domain: &str) -> Option<&KnowledgeBoundary> {
        self.boundaries.get(domain).and_then(|b| {
            if b.sample_size >= self.min_samples {
                Some(b)
            } else {
                None
            }
        })
    }

    pub fn record_outcome(&mut self, domain: &str, predicted: f64, actual: f64) {
        let calibrated = self.calibrate_confidence(predicted, domain);
        let record = CalibrationRecord {
            domain: domain.to_string(),
            predicted_confidence: predicted,
            actual_outcome: actual,
            timestamp: self.update_count,
            calibrated_confidence: calibrated,
        };
        if self.records.len() >= self.max_records {
            self.records.pop_front();
        }
        self.records.push_back(record);
        self.update_count += 1;

        let domain_records: Vec<&CalibrationRecord> =
            self.records.iter().filter(|r| r.domain == domain).collect();
        let n = domain_records.len() as u32;
        let accuracy: f64 = domain_records
            .iter()
            .map(|r| {
                let error = (r.predicted_confidence - r.actual_outcome).abs();
                1.0 - error
            })
            .sum::<f64>()
            / n as f64;
        let mean_predicted: f64 = domain_records
            .iter()
            .map(|r| r.predicted_confidence)
            .sum::<f64>()
            / n as f64;

        self.boundaries.insert(
            domain.to_string(),
            KnowledgeBoundary {
                domain: domain.to_string(),
                confidence_threshold: mean_predicted,
                accuracy_estimate: accuracy,
                sample_size: n,
            },
        );
    }

    pub fn is_outside_expertise(&self, domain: &str) -> bool {
        match self.boundaries.get(domain) {
            Some(b) => b.accuracy_estimate < self.default_threshold,
            None => true,
        }
    }

    pub fn domains_by_accuracy(&self) -> Vec<(String, f64)> {
        let mut result: Vec<(String, f64)> = self
            .boundaries
            .iter()
            .filter(|(_, b)| b.sample_size >= self.min_samples)
            .map(|(d, b)| (d.clone(), b.accuracy_estimate))
            .collect();
        result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        result
    }

    pub fn overall_calibration_error(&self) -> f64 {
        if self.records.is_empty() {
            return 0.0;
        }
        let sum: f64 = self
            .records
            .iter()
            .map(|r| (r.predicted_confidence - r.actual_outcome).abs())
            .sum();
        sum / self.records.len() as f64
    }

    pub fn best_domains(&self, top_n: usize) -> Vec<String> {
        self.domains_by_accuracy()
            .into_iter()
            .take(top_n)
            .map(|(d, _)| d)
            .collect()
    }

    pub fn weakest_domains(&self, bottom_n: usize) -> Vec<String> {
        let mut all: Vec<(String, f64)> = self
            .boundaries
            .iter()
            .filter(|(_, b)| b.sample_size >= self.min_samples)
            .map(|(d, b)| (d.clone(), b.accuracy_estimate))
            .collect();
        all.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        all.into_iter().take(bottom_n).map(|(d, _)| d).collect()
    }

    pub fn reset_domain(&mut self, domain: &str) {
        self.records.retain(|r| r.domain != domain);
        self.boundaries.remove(domain);
    }

    pub fn reset_all(&mut self) {
        self.records.clear();
        self.boundaries.clear();
        self.update_count = 0;
    }
}

impl Default for EpistemicCalibrator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_calibrator() -> EpistemicCalibrator {
        EpistemicCalibrator::new()
    }

    #[test]
    fn test_new_calibrator_defaults() {
        let ec = make_calibrator();
        assert_eq!(ec.max_records, 500);
        assert_eq!(ec.min_samples, 5);
        assert!((ec.default_threshold - 0.3).abs() < f64::EPSILON);
        assert!((ec.learning_rate - 0.1).abs() < f64::EPSILON);
        assert_eq!(ec.update_count, 0);
        assert!(ec.records.is_empty());
        assert!(ec.boundaries.is_empty());
    }

    #[test]
    fn test_calibrate_unknown_domain_clamps_confidence() {
        let mut ec = make_calibrator();
        let adjusted = ec.calibrate_confidence(0.99, "unseen");
        assert!(adjusted <= 0.9);
    }

    #[test]
    fn test_calibrate_unknown_domain_min() {
        let mut ec = make_calibrator();
        let adjusted = ec.calibrate_confidence(0.01, "unseen");
        assert!(adjusted >= 0.1);
    }

    #[test]
    fn test_record_outcome_increases_sample_count() {
        let mut ec = make_calibrator();
        for _ in 0..7 {
            ec.record_outcome("math", 0.8, 0.7);
        }
        let b = ec.boundaries.get("math").unwrap();
        assert_eq!(b.sample_size, 7);
    }

    #[test]
    fn test_calibrate_reduces_overconfidence() {
        let mut ec = make_calibrator();
        for _ in 0..5 {
            ec.record_outcome("physics", 0.9, 0.3);
        }
        let adjusted = ec.calibrate_confidence(0.9, "physics");
        assert!(adjusted < 0.9);
    }

    #[test]
    fn test_calibrate_increases_underconfidence() {
        let mut ec = make_calibrator();
        for _ in 0..5 {
            ec.record_outcome("chemistry", 0.3, 0.9);
        }
        let adjusted = ec.calibrate_confidence(0.3, "chemistry");
        assert!(adjusted > 0.3);
    }

    #[test]
    fn test_knows_boundary_not_enough_samples() {
        let mut ec = make_calibrator();
        ec.record_outcome("biology", 0.5, 0.6);
        assert!(ec.knows_boundary("biology").is_none());
    }

    #[test]
    fn test_knows_boundary_enough_samples() {
        let mut ec = make_calibrator();
        for _ in 0..5 {
            ec.record_outcome("biology", 0.5, 0.6);
        }
        assert!(ec.knows_boundary("biology").is_some());
    }

    #[test]
    fn test_is_outside_expertise_low_accuracy() {
        let mut ec = make_calibrator();
        for _ in 0..5 {
            ec.record_outcome("history", 0.8, 0.1);
        }
        assert!(ec.is_outside_expertise("history"));
    }

    #[test]
    fn test_is_outside_expertise_few_samples() {
        let ec = make_calibrator();
        assert!(ec.is_outside_expertise("unknown"));
    }

    #[test]
    fn test_domains_by_accuracy_sorted() {
        let mut ec = make_calibrator();
        for _ in 0..5 {
            ec.record_outcome("good", 0.9, 0.9);
            ec.record_outcome("bad", 0.9, 0.1);
            ec.record_outcome("mid", 0.5, 0.5);
        }
        let sorted = ec.domains_by_accuracy();
        assert!(sorted.len() >= 3);
        assert_eq!(sorted[0].0, "good");
        assert_eq!(sorted[2].0, "bad");
    }

    #[test]
    fn test_overall_calibration_error() {
        let mut ec = make_calibrator();
        ec.record_outcome("a", 0.8, 0.7);
        ec.record_outcome("b", 0.9, 0.5);
        let ece = ec.overall_calibration_error();
        let expected = (0.1_f64 + 0.4_f64) / 2.0;
        assert!((ece - expected).abs() < 1e-10);
    }

    #[test]
    fn test_best_and_weakest_domains() {
        let mut ec = make_calibrator();
        for _ in 0..5 {
            ec.record_outcome("strong", 0.9, 0.95);
            ec.record_outcome("weak", 0.9, 0.05);
            ec.record_outcome("medium", 0.5, 0.45);
        }
        let best = ec.best_domains(2);
        assert!(best.contains(&"strong".to_string()));
        let worst = ec.weakest_domains(2);
        assert!(worst.contains(&"weak".to_string()));
    }

    #[test]
    fn test_reset_all() {
        let mut ec = make_calibrator();
        for _ in 0..5 {
            ec.record_outcome("math", 0.8, 0.7);
        }
        assert_eq!(ec.records.len(), 5);
        ec.reset_all();
        assert!(ec.records.is_empty());
        assert!(ec.boundaries.is_empty());
        assert_eq!(ec.update_count, 0);
    }
}
