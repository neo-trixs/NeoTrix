#![forbid(unsafe_code)]

use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;

#[derive(Debug, Clone)]
pub struct HubnessDetector {
    pub k: usize,
    pub z_threshold: f64,
    pub window_size: usize,
}

impl Default for HubnessDetector {
    fn default() -> Self {
        Self {
            k: 10,
            z_threshold: 3.0,
            window_size: 100,
        }
    }
}

impl HubnessDetector {
    pub fn new(k: usize, z_threshold: f64, window_size: usize) -> Self {
        Self {
            k,
            z_threshold,
            window_size,
        }
    }

    pub fn compute_hubness_scores(&self, vectors: &[Vec<u8>], k: usize) -> Vec<f64> {
        let n = vectors.len();
        if n == 0 {
            return vec![];
        }
        let mut top_k_counts = vec![0usize; n];
        for i in 0..n {
            let mut sims: Vec<(usize, f64)> = vectors
                .iter()
                .enumerate()
                .filter(|(j, _)| *j != i)
                .map(|(j, v)| (j, QuantizedVSA::similarity(&vectors[i], v)))
                .collect();
            sims.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            for &(idx, _) in sims.iter().take(k.min(sims.len())) {
                top_k_counts[idx] += 1;
            }
        }
        let nf = (n - 1).max(1) as f64;
        top_k_counts.iter().map(|c| *c as f64 / nf).collect()
    }

    pub fn z_score_normalize(&self, scores: &[f64]) -> Vec<f64> {
        if scores.is_empty() {
            return vec![];
        }
        let mut sorted = scores.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let median = sorted[sorted.len() / 2];
        let mad: f64 = sorted
            .iter()
            .map(|v| (v - median).abs())
            .fold(0.0, |a, b| a + b)
            / sorted.len() as f64;
        if mad < 1e-10 {
            return scores.iter().map(|_| 0.0).collect();
        }
        let mad_scaled = mad * 1.4826;
        scores.iter().map(|v| (v - median) / mad_scaled).collect()
    }

    pub fn flag_hubs(&self, scores: &[f64], z_threshold: f64) -> Vec<usize> {
        scores
            .iter()
            .enumerate()
            .filter(|(_, s)| **s > z_threshold)
            .map(|(i, _)| i)
            .collect()
    }

    pub fn detect_poisoning(&self, current_scores: &[f64], window: usize) -> f64 {
        if current_scores.len() < 2 || window < 2 {
            return 0.0;
        }
        let mid = window / 2;
        let first_half: f64 = current_scores.iter().take(mid).sum::<f64>() / mid.max(1) as f64;
        let second_half: f64 = current_scores
            .iter()
            .skip(mid)
            .take(window - mid)
            .sum::<f64>()
            / (window - mid).max(1) as f64;
        second_half - first_half
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_vsa(seed: u8) -> Vec<u8> {
        let mut v = vec![0u8; 512];
        for i in 0..512 {
            v[i] = ((seed as u64)
                .wrapping_mul(2654435761)
                .wrapping_add(i as u64)
                % 2) as u8;
        }
        v
    }

    #[test]
    fn test_hubness_on_identical_vectors() {
        let detector = HubnessDetector::default();
        let v = make_vsa(1);
        let vectors = vec![v.clone(), v.clone(), v.clone(), v.clone()];
        let scores = detector.compute_hubness_scores(&vectors, 2);
        assert_eq!(scores.len(), 4);
        assert!(scores.iter().all(|s| *s > 0.0));
    }

    #[test]
    fn test_hubness_on_distinct_vectors() {
        let detector = HubnessDetector::default();
        let vectors: Vec<Vec<u8>> = (0..10).map(|i| make_vsa(i)).collect();
        let scores = detector.compute_hubness_scores(&vectors, 3);
        assert_eq!(scores.len(), 10);
    }

    #[test]
    fn test_empty_input() {
        let detector = HubnessDetector::default();
        let scores = detector.compute_hubness_scores(&[], 5);
        assert!(scores.is_empty());
    }

    #[test]
    fn test_z_score_normalize_uniform() {
        let detector = HubnessDetector::default();
        let scores = vec![0.5, 0.5, 0.5, 0.5];
        let normalized = detector.z_score_normalize(&scores);
        assert!(normalized.iter().all(|s| s.abs() < 1e-10));
    }

    #[test]
    fn test_z_score_normalize_identifies_outlier() {
        let detector = HubnessDetector::default();
        let scores = vec![0.1, 0.1, 0.1, 0.1, 0.9];
        let normalized = detector.z_score_normalize(&scores);
        assert!(normalized[4] > normalized[0]);
    }

    #[test]
    fn test_flag_hubs() {
        let detector = HubnessDetector::default();
        let scores = vec![0.5, 5.0, 0.5, 6.0];
        let hubs = detector.flag_hubs(&scores, 3.0);
        assert_eq!(hubs, vec![1, 3]);
    }

    #[test]
    fn test_detect_poisoning_increase() {
        let detector = HubnessDetector::default();
        let mut scores = vec![0.1; 20];
        for i in 10..20 {
            scores[i] = 0.5;
        }
        let change = detector.detect_poisoning(&scores, 20);
        assert!(change > 0.0);
    }

    #[test]
    fn test_detect_poisoning_no_change() {
        let detector = HubnessDetector::default();
        let scores = vec![0.3; 20];
        let change = detector.detect_poisoning(&scores, 20);
        assert!(change.abs() < 1e-10);
    }

    #[test]
    fn test_new_with_config() {
        let detector = HubnessDetector::new(5, 2.5, 50);
        assert_eq!(detector.k, 5);
        assert_eq!(detector.z_threshold, 2.5);
        assert_eq!(detector.window_size, 50);
    }
}
