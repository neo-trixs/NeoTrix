use super::vsa_tag::VsaTagged;
use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;

#[derive(Debug, Clone)]
pub struct ConformalSet {
    pub predictions: Vec<(usize, f64)>,
    pub threshold: f64,
    pub prediction_set: Vec<usize>,
    pub coverage: f64,
    pub empty: bool,
}

impl ConformalSet {
    fn new(predictions: Vec<(usize, f64)>, threshold: f64, coverage: f64) -> Self {
        let prediction_set: Vec<usize> = predictions
            .iter()
            .filter(|(_, score)| *score <= threshold)
            .map(|(idx, _)| *idx)
            .collect();
        let empty = prediction_set.is_empty();
        Self {
            predictions,
            threshold,
            prediction_set,
            coverage,
            empty,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConformalUQ {
    calibration_scores: Vec<f64>,
    coverage_target: f64,
    max_calibration_size: usize,
}

impl ConformalUQ {
    pub fn new(coverage_target: f64, max_cal_size: usize) -> Self {
        let target = coverage_target.clamp(0.0, 1.0);
        Self {
            calibration_scores: Vec::with_capacity(max_cal_size.min(1024)),
            coverage_target: target,
            max_calibration_size: max_cal_size,
        }
    }

    pub fn add_calibration(&mut self, scores: &[f64]) {
        self.calibration_scores.extend_from_slice(scores);
        if self.calibration_scores.len() > self.max_calibration_size {
            self.calibration_scores
                .sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            self.calibration_scores.truncate(self.max_calibration_size);
        }
    }

    pub fn calibrate(&self) -> f64 {
        let n = self.calibration_scores.len();
        if n == 0 {
            return 1.0;
        }
        let mut sorted = self.calibration_scores.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let q_index = ((n as f64 + 1.0) * self.coverage_target).ceil() as usize;
        let idx = q_index.min(n).saturating_sub(1);
        sorted[idx]
    }

    pub fn predict(&self, query: &[u8], candidates: &[&[u8]]) -> ConformalSet {
        let threshold = self.calibrate();
        let predictions: Vec<(usize, f64)> = candidates
            .iter()
            .enumerate()
            .map(|(i, c)| (i, Self::nonconformity_score(query, c)))
            .collect();
        ConformalSet::new(predictions, threshold, self.coverage_target)
    }

    pub fn predict_single(&self, query: &[u8], candidate: &[u8]) -> f64 {
        let n = self.calibration_scores.len();
        if n == 0 {
            return 0.5;
        }
        let nc = Self::nonconformity_score(query, candidate);
        let count_gte = self.calibration_scores.iter().filter(|&&s| s >= nc).count();
        (count_gte + 1) as f64 / (n + 1) as f64
    }

    pub fn is_reliable(p_value: f64) -> bool {
        p_value >= 0.2
    }

    pub fn estimate_confidence(&self, query: &[u8], candidates: &[&[u8]]) -> f64 {
        let cs = self.predict(query, candidates);
        if cs.empty {
            return 0.0;
        }
        if cs.prediction_set.len() == 1 {
            let idx = cs.prediction_set[0];
            let nc_score = cs
                .predictions
                .iter()
                .find(|(i, _)| *i == idx)
                .map(|(_, s)| *s)
                .unwrap_or(1.0);
            return 1.0 - nc_score;
        }
        1.0 / cs.prediction_set.len() as f64
    }

    pub fn calibration_size(&self) -> usize {
        self.calibration_scores.len()
    }

    pub fn update_target(&mut self, new_target: f64) {
        self.coverage_target = new_target.clamp(0.0, 1.0);
    }

    pub fn apply_to_tagged(&self, tagged: &mut VsaTagged, query: &[u8], candidates: &[&[u8]]) {
        tagged.confidence = self.estimate_confidence(query, candidates);
    }

    pub fn nonconformity_score(query: &[u8], candidate: &[u8]) -> f64 {
        1.0 - QuantizedVSA::similarity(query, candidate)
    }

    pub fn normalized_nonconformity(scores: &[f64]) -> Vec<f64> {
        if scores.is_empty() {
            return Vec::new();
        }
        let min = scores.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = scores.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        if (max - min).abs() < f64::EPSILON {
            return vec![0.5; scores.len()];
        }
        scores.iter().map(|&s| (s - min) / (max - min)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_consciousness::vsa_tag::{VsaOrigin, VsaSelfCategory};
    use crate::core::nt_core_hcube::vsa_quantized::VSA_DIM;

    fn binary_vector(value: u8) -> Vec<u8> {
        vec![value; VSA_DIM]
    }

    fn random_binary() -> Vec<u8> {
        QuantizedVSA::random_binary()
    }

    #[test]
    fn test_new_sets_coverage_target() {
        let uq = ConformalUQ::new(0.9, 100);
        assert!((uq.coverage_target - 0.9).abs() < 1e-9);
        assert_eq!(uq.max_calibration_size, 100);
        assert_eq!(uq.calibration_size(), 0);
    }

    #[test]
    fn test_new_clamps_coverage_target() {
        let uq = ConformalUQ::new(1.5, 100);
        assert!((uq.coverage_target - 1.0).abs() < 1e-9);
        let uq = ConformalUQ::new(-0.5, 100);
        assert!((uq.coverage_target - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_calibrate_with_simple_scores() {
        let mut uq = ConformalUQ::new(0.8, 100);
        uq.add_calibration(&[0.1, 0.2, 0.3, 0.4, 0.5]);
        let threshold = uq.calibrate();
        assert!((threshold - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_calibrate_empty_returns_one() {
        let uq = ConformalUQ::new(0.9, 100);
        let threshold = uq.calibrate();
        assert!((threshold - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_predict_returns_empty_for_far_query() {
        let mut uq = ConformalUQ::new(0.95, 1000);
        uq.add_calibration(&[0.1, 0.15, 0.2, 0.25, 0.3]);
        let q = binary_vector(1);
        let c0 = binary_vector(0);
        let c1 = binary_vector(0);
        let candidates = vec![c0.as_slice(), c1.as_slice()];
        let cs = uq.predict(&q, &candidates);
        assert!(cs.empty);
        assert!(cs.prediction_set.is_empty());
    }

    #[test]
    fn test_predict_returns_non_empty_for_close_query() {
        let mut uq = ConformalUQ::new(0.8, 1000);
        uq.add_calibration(&[0.1, 0.2, 0.3, 0.4, 0.5]);
        let q = binary_vector(1);
        let c0 = binary_vector(1);
        let c1 = binary_vector(0);
        let candidates = vec![c0.as_slice(), c1.as_slice()];
        let cs = uq.predict(&q, &candidates);
        assert!(!cs.empty);
        assert!(cs.prediction_set.contains(&0));
        assert!(!cs.prediction_set.contains(&1));
    }

    #[test]
    fn test_predict_single_identical_vectors() {
        let mut uq = ConformalUQ::new(0.9, 1000);
        uq.add_calibration(&[0.1, 0.2, 0.3, 0.4, 0.5]);
        let q = binary_vector(1);
        let c = binary_vector(1);
        let p = uq.predict_single(&q, &c);
        assert!((p - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_predict_single_opposite_vectors() {
        let mut uq = ConformalUQ::new(0.9, 1000);
        uq.add_calibration(&[0.1, 0.2, 0.3, 0.4, 0.5]);
        let q = binary_vector(1);
        let c = binary_vector(0);
        let p = uq.predict_single(&q, &c);
        assert!((p - 1.0 / 6.0).abs() < 1e-9);
    }

    #[test]
    fn test_is_reliable() {
        assert!(ConformalUQ::is_reliable(0.5));
        assert!(ConformalUQ::is_reliable(0.2));
        assert!(!ConformalUQ::is_reliable(0.19));
        assert!(!ConformalUQ::is_reliable(0.0));
    }

    #[test]
    fn test_estimate_confidence_empty_set() {
        let mut uq = ConformalUQ::new(0.99, 1000);
        uq.add_calibration(&[0.01, 0.01, 0.01, 0.01, 0.01]);
        let q = binary_vector(1);
        let c = binary_vector(0);
        let candidates = vec![c.as_slice()];
        let conf = uq.estimate_confidence(&q, &candidates);
        assert!((conf - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_estimate_confidence_perfect_match() {
        let mut uq = ConformalUQ::new(0.8, 1000);
        uq.add_calibration(&[0.1, 0.2, 0.3, 0.4, 0.5]);
        let q = binary_vector(1);
        let c = binary_vector(1);
        let candidates = vec![c.as_slice()];
        let conf = uq.estimate_confidence(&q, &candidates);
        assert!((conf - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_apply_to_tagged_modifies_confidence() {
        let mut uq = ConformalUQ::new(0.8, 1000);
        uq.add_calibration(&[0.1, 0.2, 0.3, 0.4, 0.5]);
        let q = binary_vector(1);
        let c = binary_vector(1);
        let candidates = vec![c.as_slice()];
        let mut tagged = VsaTagged::new(q.clone(), VsaOrigin::Self_(VsaSelfCategory::Thought));
        assert!((tagged.confidence - 1.0).abs() < 1e-9);
        uq.apply_to_tagged(&mut tagged, &q, &candidates);
        assert!((tagged.confidence - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_update_target_changes_behavior() {
        let mut uq = ConformalUQ::new(0.5, 1000);
        uq.add_calibration(&[0.1, 0.2, 0.3, 0.4, 0.5]);
        let q = binary_vector(1);
        let c = binary_vector(1);
        let candidates = vec![c.as_slice()];

        uq.update_target(0.9);
        assert!((uq.coverage_target - 0.9).abs() < 1e-9);

        let cs_wider = uq.predict(&q, &candidates);
        uq.update_target(0.5);
        let cs_tighter = uq.predict(&q, &candidates);
        assert!(cs_wider.prediction_set.len() >= cs_tighter.prediction_set.len());
    }

    #[test]
    fn test_add_calibration_does_not_exceed_max() {
        let mut uq = ConformalUQ::new(0.9, 10);
        uq.add_calibration(&[0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0]);
        assert_eq!(uq.calibration_size(), 10);
        uq.add_calibration(&[0.15, 0.25]);
        assert_eq!(uq.calibration_size(), 10);
    }

    #[test]
    fn test_nonconformity_score_is_symmetric() {
        let a = random_binary();
        let b = random_binary();
        let nc_ab = ConformalUQ::nonconformity_score(&a, &b);
        let nc_ba = ConformalUQ::nonconformity_score(&b, &a);
        assert!((nc_ab - nc_ba).abs() < 1e-12);
    }

    #[test]
    fn test_nonconformity_score_zero_for_identical() {
        let v = random_binary();
        let nc = ConformalUQ::nonconformity_score(&v, &v);
        assert!((nc - 0.0).abs() < 1e-12);
    }

    #[test]
    fn test_normalized_nonconformity() {
        let scores = vec![0.0, 0.5, 1.0];
        let norm = ConformalUQ::normalized_nonconformity(&scores);
        assert!((norm[0] - 0.0).abs() < 1e-9);
        assert!((norm[1] - 0.5).abs() < 1e-9);
        assert!((norm[2] - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_normalized_nonconformity_constant() {
        let scores = vec![0.5, 0.5, 0.5];
        let norm = ConformalUQ::normalized_nonconformity(&scores);
        assert!(norm.iter().all(|&x| (x - 0.5).abs() < 1e-9));
    }

    #[test]
    fn test_normalized_nonconformity_empty() {
        let norm = ConformalUQ::normalized_nonconformity(&[]);
        assert!(norm.is_empty());
    }

    #[test]
    fn test_calibrate_high_coverage() {
        let mut uq = ConformalUQ::new(0.95, 100);
        uq.add_calibration(&[0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0]);
        let threshold = uq.calibrate();
        assert!((threshold - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_estimate_confidence_multiple_in_set() {
        let mut uq = ConformalUQ::new(0.9, 1000);
        uq.add_calibration(&[0.5, 0.5, 0.5, 0.5, 0.5]);
        let q = binary_vector(1);
        let candidates = vec![q.as_slice(), q.as_slice()];
        let conf = uq.estimate_confidence(&q, &candidates);
        assert!((conf - 0.5).abs() < 1e-9);
    }
}
