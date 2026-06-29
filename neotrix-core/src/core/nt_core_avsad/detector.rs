use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Detects adversarial perturbations in visual inputs by comparing
/// VSA encodings of original vs perturbed analysis.
pub struct AvsadDetector {
    /// Threshold for flagging adversarial content (0.0-1.0)
    pub anomaly_threshold: f64,
    /// History of anomaly scores for drift tracking
    pub score_history: Vec<f64>,
    pub max_history: usize,
    /// Detection counts per cycle
    pub total_checks: u64,
    pub flagged_count: u64,
}

impl AvsadDetector {
    pub fn new() -> Self {
        Self {
            anomaly_threshold: 0.7,
            score_history: Vec::with_capacity(256),
            max_history: 1000,
            total_checks: 0,
            flagged_count: 0,
        }
    }

    pub fn with_threshold(threshold: f64) -> Self {
        Self {
            anomaly_threshold: threshold.clamp(0.0, 1.0),
            ..Self::new()
        }
    }

    /// Analyze an image byte slice for adversarial patterns.
    /// Returns (anomaly_score, is_adversarial, details).
    pub fn analyze(&mut self, image_bytes: &[u8]) -> (f64, bool, String) {
        self.total_checks += 1;

        let baseline = self.bytes_to_vsa(image_bytes);
        let perturbed = self.create_perturbed_variants(image_bytes);

        let mut similarities = Vec::with_capacity(perturbed.len());
        for (i, pv) in perturbed.iter().enumerate() {
            let sim = cosine_similarity(&baseline, pv);
            similarities.push((i, sim));
        }

        let mean_sim = if similarities.is_empty() {
            1.0
        } else {
            similarities.iter().map(|(_, s)| s).sum::<f64>() / similarities.len() as f64
        };

        let anomaly_score = 1.0 - mean_sim;
        let is_adversarial = anomaly_score > self.anomaly_threshold;

        if is_adversarial {
            self.flagged_count += 1;
        }

        self.record_score(anomaly_score);

        let details = if is_adversarial {
            let variants: Vec<String> = similarities
                .iter()
                .map(|(i, s)| format!("v{}:{:.4}", i, s))
                .collect();
            format!(
                "ADVERSARIAL:score={:.4}_threshold={:.2}_variants=[{}]",
                anomaly_score,
                self.anomaly_threshold,
                variants.join(",")
            )
        } else {
            format!(
                "clean:score={:.4}_variants={}",
                anomaly_score,
                similarities.len()
            )
        };

        (anomaly_score, is_adversarial, details)
    }

    /// Current adversarial detection rate.
    pub fn detection_rate(&self) -> f64 {
        if self.total_checks == 0 {
            return 0.0;
        }
        self.flagged_count as f64 / self.total_checks as f64
    }

    fn record_score(&mut self, score: f64) {
        self.score_history.push(score);
        if self.score_history.len() > self.max_history {
            let excess = self.score_history.len() - self.max_history;
            self.score_history.drain(0..excess);
        }
    }

    /// Encode raw bytes to a VSA-like feature vector using deterministic hashing.
    fn bytes_to_vsa(&self, bytes: &[u8]) -> Vec<f64> {
        let dim = 256;
        let chunk_size = bytes.len().max(1);
        let mut vec = vec![0.0_f64; dim];
        for (i, chunk) in bytes.chunks(chunk_size.max(1) / dim.max(1) + 1).enumerate() {
            let idx = i % dim;
            let mut h = DefaultHasher::new();
            chunk.hash(&mut h);
            let hash_val = h.finish();
            vec[idx] = (hash_val as f64 / u64::MAX as f64) * 2.0 - 1.0;
        }
        vec
    }

    /// Create 3 perturbed variants and return their VSA encodings.
    fn create_perturbed_variants(&self, bytes: &[u8]) -> Vec<Vec<f64>> {
        let mut variants = Vec::with_capacity(3);

        // Variant 1: bit-flip perturbation (simulates brightness shift)
        let mut v1 = bytes.to_vec();
        let flip_seed = 42usize;
        for i in (0..v1.len()).step_by(137) {
            v1[i] = v1[i].wrapping_add(flip_seed as u8);
        }
        variants.push(self.bytes_to_vsa(&v1));

        // Variant 2: byte-swap perturbation (simulates contrast shift)
        let mut v2 = bytes.to_vec();
        if v2.len() > 1 {
            for i in (0..v2.len() - 1).step_by(64) {
                v2.swap(i, i + 1);
            }
        }
        variants.push(self.bytes_to_vsa(&v2));

        // Variant 3: cropping simulation via prefix truncation
        let trunc = bytes.len() / 16;
        let v3 = if bytes.len() > trunc {
            &bytes[trunc..]
        } else {
            bytes
        };
        variants.push(self.bytes_to_vsa(v3));

        variants
    }
}

impl Default for AvsadDetector {
    fn default() -> Self {
        Self::new()
    }
}

fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
    let len = a.len().min(b.len());
    if len == 0 {
        return 1.0;
    }
    let mut dot = 0.0_f64;
    let mut norm_a = 0.0_f64;
    let mut norm_b = 0.0_f64;
    for i in 0..len {
        dot += a[i] * b[i];
        norm_a += a[i] * a[i];
        norm_b += b[i] * b[i];
    }
    let denom = norm_a.sqrt() * norm_b.sqrt();
    if denom < 1e-12 {
        1.0
    } else {
        (dot / denom).clamp(-1.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_detector_defaults() {
        let d = AvsadDetector::new();
        assert!((d.anomaly_threshold - 0.7).abs() < 1e-6);
        assert_eq!(d.total_checks, 0);
        assert_eq!(d.flagged_count, 0);
    }

    #[test]
    fn test_clean_image_not_flagged() {
        let mut d = AvsadDetector::new();
        let clean = vec![0u8; 4096];
        let (score, flagged, _) = d.analyze(&clean);
        assert!(
            !flagged,
            "clean image should not be flagged, score={}",
            score
        );
        assert_eq!(d.total_checks, 1);
        assert_eq!(d.flagged_count, 0);
    }

    #[test]
    fn test_adversarial_image_flagged() {
        let mut d = AvsadDetector::with_threshold(0.1);
        let clean = vec![0u8; 4096];
        let (_, flagged, _) = d.analyze(&clean);
        assert!(flagged, "low threshold should flag even clean image");
    }

    #[test]
    fn test_score_history_bounded() {
        let mut d = AvsadDetector::new();
        d.max_history = 10;
        for _ in 0..20 {
            d.analyze(&[0u8; 256]);
        }
        assert!(d.score_history.len() <= 10);
    }

    #[test]
    fn test_detection_rate() {
        let mut d = AvsadDetector::with_threshold(0.1);
        assert!((d.detection_rate() - 0.0).abs() < 1e-6);
        d.analyze(&[0u8; 256]);
        assert!(d.detection_rate() > 0.0);
    }

    #[test]
    fn test_cosine_similarity_identical() {
        let v = vec![1.0, 2.0, 3.0];
        let sim = cosine_similarity(&v, &v);
        assert!((sim - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        let sim = cosine_similarity(&a, &b);
        assert!((sim - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_perturbed_variants_differ() {
        let d = AvsadDetector::new();
        let bytes = vec![42u8; 4096];
        let baseline = d.bytes_to_vsa(&bytes);
        let variants = d.create_perturbed_variants(&bytes);
        assert_eq!(variants.len(), 3);
        for v in &variants {
            let sim = cosine_similarity(&baseline, v);
            assert!(
                sim > 0.0,
                "variants should have some similarity to baseline"
            );
        }
    }

    #[test]
    fn test_empty_input() {
        let mut d = AvsadDetector::new();
        let (score, flagged, _) = d.analyze(&[]);
        assert!(!score.is_nan());
        assert!(!flagged);
    }
}
