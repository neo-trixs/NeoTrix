use super::error::ConsciousnessError;
use std::collections::VecDeque;
use std::sync::{Mutex, OnceLock};

/// How a claim was verified.
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum VerificationMethod {
    Unverified,
    SelfConsistency,
    DistractorNormalized,
    ExternalVerifier,
    LogicalDeduction,
}

/// A single atomic claim extracted from a response.
#[derive(Debug, Clone)]
pub struct AtomicClaim {
    pub text: String,
    pub confidence: f64,
    pub calibrated_confidence: f64,
    pub is_verified: bool,
    pub verification_method: VerificationMethod,
    pub distractors: Vec<String>,
}

/// A response decomposed into a bundle of atomic claims.
#[derive(Debug)]
pub struct ClaimBundle {
    pub response_id: String,
    pub claims: Vec<AtomicClaim>,
    pub overall_confidence: f64,
    pub min_confidence: f64,
    pub max_confidence: f64,
    pub calibration_error: f64,
    pub timestamp: std::time::Instant,
}

/// A calibration bin tracking claim-level confidence vs. correctness.
#[derive(Debug, Clone)]
pub struct CalibrationBin2 {
    pub lower: f64,
    pub upper: f64,
    pub count: usize,
    pub correct: usize,
    pub accuracy: f64,
}

impl CalibrationBin2 {
    fn new(lower: f64, upper: f64) -> Self {
        Self {
            lower,
            upper,
            count: 0,
            correct: 0,
            accuracy: 0.0,
        }
    }

    fn update(&mut self, correct: bool) {
        self.count += 1;
        if correct {
            self.correct += 1;
        }
        self.accuracy = if self.count > 0 {
            self.correct as f64 / self.count as f64
        } else {
            0.0
        };
    }
}

/// Claim-level confidence calibrator inspired by VibeThinker-3B's CLR and
/// DINCO (Distractor-Normalized Coherence, ICLR 2026).
#[derive(Debug)]
pub struct ClaimCalibrator {
    bins: Vec<CalibrationBin2>,
    claim_history: VecDeque<ClaimBundle>,
    total_claims: u64,
    total_correct: u64,
    running_ece: f64,
    distractor_count: usize,
    alpha: f64,
}

impl Default for ClaimCalibrator {
    fn default() -> Self {
        Self::new()
    }
}

impl ClaimCalibrator {
    /// 10 bins: [0, 0.1), [0.1, 0.2), ..., [0.9, 1.0]
    pub fn new() -> Self {
        let bins = (0..10)
            .map(|i| CalibrationBin2::new(i as f64 / 10.0, (i + 1) as f64 / 10.0))
            .collect();
        Self {
            bins,
            claim_history: VecDeque::with_capacity(500),
            total_claims: 0,
            total_correct: 0,
            running_ece: 0.0,
            distractor_count: 3,
            alpha: 0.1,
        }
    }

    /// Decompose a response string into atomic claims, assigning each a
    /// confidence from the provided slice (or 0.5 as default), and generating
    /// distractors for each claim.
    pub fn decompose_claims(
        &self,
        response_id: &str,
        response: &str,
        confidences: &[f64],
    ) -> ClaimBundle {
        let sentences = split_into_sentences(response);
        let claims: Vec<AtomicClaim> = sentences
            .into_iter()
            .enumerate()
            .map(|(i, text)| {
                let conf = confidences.get(i).copied().unwrap_or(0.5);
                let distractors = self.add_distractors(&text, self.distractor_count);
                AtomicClaim {
                    text,
                    confidence: conf,
                    calibrated_confidence: conf,
                    is_verified: false,
                    verification_method: VerificationMethod::Unverified,
                    distractors,
                }
            })
            .collect();

        let overall = if claims.is_empty() {
            0.0
        } else {
            claims.iter().map(|c| c.confidence).sum::<f64>() / claims.len() as f64
        };
        let min_c = claims
            .iter()
            .map(|c| c.confidence)
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0);
        let max_c = claims
            .iter()
            .map(|c| c.confidence)
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0);

        ClaimBundle {
            response_id: response_id.to_string(),
            claims,
            overall_confidence: overall,
            min_confidence: min_c,
            max_confidence: max_c,
            calibration_error: 0.0,
            timestamp: std::time::Instant::now(),
        }
    }

    /// Record a bundle's claims against ground-truth correctness labels,
    /// update calibration bins, and refresh the running ECE.
    pub fn record_bundle(&mut self, bundle: ClaimBundle, correct_claims: &[bool]) {
        let mut bundle_claim_count = 0u64;
        for (i, claim) in bundle.claims.iter().enumerate() {
            let correct = correct_claims.get(i).copied().unwrap_or(false);
            let clamped = claim.calibrated_confidence.clamp(0.0, 0.999);
            let bin_idx = (clamped * 10.0) as usize;
            let idx = bin_idx.min(9);
            self.bins[idx].update(correct);
            if correct {
                self.total_correct += 1;
            }
            self.total_claims += 1;
            bundle_claim_count += 1;
        }

        let current_ece = self.compute_ece();
        if self.total_claims == bundle_claim_count {
            self.running_ece = current_ece;
        } else {
            self.running_ece = (1.0 - self.alpha) * self.running_ece + self.alpha * current_ece;
        }

        self.claim_history.push_back(bundle);
        while self.claim_history.len() > 500 {
            self.claim_history.pop_front();
        }
    }

    /// Bin-based calibration: clamps to [0, 0.999], finds bin, returns bin
    /// accuracy if the bin has observations, otherwise returns raw confidence.
    pub fn calibrate_claim(&self, raw_confidence: f64) -> f64 {
        let clamped = raw_confidence.clamp(0.0, 0.999);
        let bin_idx = (clamped * 10.0) as usize;
        let idx = bin_idx.min(9);
        let bin = &self.bins[idx];
        if bin.count > 0 {
            bin.accuracy
        } else {
            clamped
        }
    }

    /// Return a new bundle where every claim has been recalibrated via
    /// `calibrate_claim`.
    pub fn calibrate_bundle(&self, bundle: &ClaimBundle) -> ClaimBundle {
        let calibrated_claims: Vec<AtomicClaim> = bundle
            .claims
            .iter()
            .map(|claim| {
                let cal_conf = self.calibrate_claim(claim.confidence);
                AtomicClaim {
                    text: claim.text.clone(),
                    confidence: claim.confidence,
                    calibrated_confidence: cal_conf,
                    is_verified: claim.is_verified,
                    verification_method: claim.verification_method,
                    distractors: claim.distractors.clone(),
                }
            })
            .collect();

        let overall = if calibrated_claims.is_empty() {
            0.0
        } else {
            calibrated_claims
                .iter()
                .map(|c| c.calibrated_confidence)
                .sum::<f64>()
                / calibrated_claims.len() as f64
        };
        let min_c = calibrated_claims
            .iter()
            .map(|c| c.calibrated_confidence)
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0);
        let max_c = calibrated_claims
            .iter()
            .map(|c| c.calibrated_confidence)
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0);
        let cal_error = if calibrated_claims.is_empty() {
            0.0
        } else {
            bundle
                .claims
                .iter()
                .zip(calibrated_claims.iter())
                .map(|(orig, cal)| (orig.confidence - cal.calibrated_confidence).abs())
                .sum::<f64>()
                / calibrated_claims.len() as f64
        };

        ClaimBundle {
            response_id: bundle.response_id.clone(),
            claims: calibrated_claims,
            overall_confidence: overall,
            min_confidence: min_c,
            max_confidence: max_c,
            calibration_error: cal_error,
            timestamp: std::time::Instant::now(),
        }
    }

    /// DINCO normalization: target_confidence / (target_confidence + mean(distractor_confidences)).
    /// Accounts for suggestibility bias by normalizing against self-generated alternatives.
    pub fn dinco_calibrate(&self, target_confidence: f64, distractor_confidences: &[f64]) -> f64 {
        if distractor_confidences.is_empty() {
            return target_confidence;
        }
        let mean_distractor =
            distractor_confidences.iter().sum::<f64>() / distractor_confidences.len() as f64;
        let denominator = target_confidence + mean_distractor;
        if denominator <= 0.0 {
            return 0.0;
        }
        target_confidence / denominator
    }

    /// Expected Calibration Error computed across all bins.
    pub fn ece(&self) -> f64 {
        if self.total_claims < 5 {
            return 0.0;
        }
        self.running_ece
    }

    /// Compute ECE from scratch (used internally to update running EMA).
    fn compute_ece(&self) -> f64 {
        if self.total_claims == 0 {
            return 0.0;
        }
        let total = self.total_claims as f64;
        self.bins
            .iter()
            .map(|b| {
                let center = (b.lower + b.upper) / 2.0;
                (center - b.accuracy).abs() * (b.count as f64 / total)
            })
            .sum()
    }

    /// Returns (bin_center, accuracy, gap) for each bin — suitable for
    /// plotting a reliability diagram.
    pub fn reliability_diagram(&self) -> Vec<(f64, f64, f64)> {
        self.bins
            .iter()
            .map(|b| {
                let center = (b.lower + b.upper) / 2.0;
                let gap = (center - b.accuracy).abs();
                (center, b.accuracy, gap)
            })
            .collect()
    }

    /// Generate simple distractors by negation, capitalization, and
    /// generalization.
    pub fn add_distractors(&self, claim: &str, count: usize) -> Vec<String> {
        let mut distractors = Vec::with_capacity(count);
        let claim_lower = claim.to_lowercase();

        if count > 0 {
            if !claim_lower.starts_with("not ") && !claim_lower.starts_with("no ") {
                distractors.push(format!("Not {}", claim));
            } else {
                distractors.push(format!("{} (disputed)", claim));
            }
        }

        if count > 1 {
            distractors.push(claim.to_uppercase());
        }

        if count > 2 {
            distractors.push(format!("Always {}", claim_lower));
        }

        while distractors.len() < count {
            distractors.push(format!("Maybe {}", claim));
        }

        distractors
    }

    /// Minimum calibrated confidence across all claims in the history.
    pub fn min_confidence_in_response(&self) -> f64 {
        self.claim_history
            .iter()
            .flat_map(|b| b.claims.iter())
            .map(|c| c.calibrated_confidence)
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0)
    }

    /// Spread (max - min) of calibrated confidences in a bundle.
    pub fn confidence_spread(&self, bundle: &ClaimBundle) -> f64 {
        if bundle.claims.is_empty() {
            return 0.0;
        }
        let min = bundle
            .claims
            .iter()
            .map(|c| c.calibrated_confidence)
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0);
        let max = bundle
            .claims
            .iter()
            .map(|c| c.calibrated_confidence)
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0);
        max - min
    }

    /// Generate a summary report from the calibrator's current state.
    pub fn report(&self) -> ClaimCalibratorReport {
        let total_claim_count: usize = self.claim_history.iter().map(|b| b.claims.len()).sum();
        let total_distractors: usize = self
            .claim_history
            .iter()
            .flat_map(|b| b.claims.iter())
            .map(|c| c.distractors.len())
            .sum();
        let avg_d = if total_claim_count > 0 {
            total_distractors as f64 / total_claim_count as f64
        } else {
            0.0
        };
        let overall_accuracy = if self.total_claims > 0 {
            self.total_correct as f64 / self.total_claims as f64
        } else {
            0.0
        };

        ClaimCalibratorReport {
            ece: self.ece(),
            total_claims_calibrated: self.total_claims,
            overall_accuracy,
            claim_count: total_claim_count,
            avg_distractors_per_claim: avg_d,
        }
    }
}

/// Summary report for the ClaimCalibrator.
#[derive(Debug, Clone)]
pub struct ClaimCalibratorReport {
    pub ece: f64,
    pub total_claims_calibrated: u64,
    pub overall_accuracy: f64,
    pub claim_count: usize,
    pub avg_distractors_per_claim: f64,
}

/// Singleton
static CLAIM_CALIBRATOR: OnceLock<Mutex<ClaimCalibrator>> = OnceLock::new();

pub fn global_claim_calibrator() -> &'static Mutex<ClaimCalibrator> {
    CLAIM_CALIBRATOR.get_or_init(|| Mutex::new(ClaimCalibrator::new()))
}

/// Convenience: calibrate a single claim's confidence using the global calibrator.
pub fn calibrate_claim_confidence(_claim_text: &str, raw_confidence: f64) -> f64 {
    let calibrator = global_claim_calibrator().lock().unwrap_or_else(|e| {
        log::error!(
            "{}",
            ConsciousnessError::Internal(format!("claim calibrator lock poisoned: {}", e))
        );
        e.into_inner()
    });
    calibrator.calibrate_claim(raw_confidence)
}

/// Convenience: decompose a response into claims, calibrate, return overall
/// reliability using the global calibrator.
pub fn calibrate_bundle_confidence(response_text: &str, per_sentence_confidences: &[f64]) -> f64 {
    let calibrator = global_claim_calibrator().lock().unwrap_or_else(|e| {
        log::error!(
            "{}",
            ConsciousnessError::Internal(format!("claim calibrator lock poisoned: {}", e))
        );
        e.into_inner()
    });
    let bundle = calibrator.decompose_claims("auto", response_text, per_sentence_confidences);
    let calibrated = calibrator.calibrate_bundle(&bundle);
    calibrated.overall_confidence
}

/// Split text into sentences on `.`, `!`, or `?` boundaries.
fn split_into_sentences(text: &str) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut current = String::new();
    for ch in text.chars() {
        current.push(ch);
        if ch == '.' || ch == '!' || ch == '?' {
            let trimmed = current.trim().to_string();
            if !trimmed.is_empty() && trimmed.len() > 1 {
                sentences.push(trimmed);
            }
            current = String::new();
        }
    }
    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() {
        sentences.push(trimmed);
    }
    sentences
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    fn test_decompose_claims() {
        let calibrator = ClaimCalibrator::new();
        let bundle = calibrator.decompose_claims("test1", "A is B. C is D.", &[]);
        assert_eq!(bundle.claims.len(), 2);
        assert_eq!(bundle.claims[0].text, "A is B.");
        assert_eq!(bundle.claims[1].text, "C is D.");
    }

    #[test]
    fn test_calibrate_claim_empty_bins_returns_raw() {
        let calibrator = ClaimCalibrator::new();
        let calibrated = calibrator.calibrate_claim(0.7);
        assert!((calibrated - 0.7).abs() < 0.001);
    }

    #[test]
    fn test_calibrate_claim_filled_bins_adjust() {
        let mut calibrator = ClaimCalibrator::new();
        // Fill bin [0.8, 0.9) with 8 correct out of 10
        for _ in 0..10 {
            let bundle = calibrator.decompose_claims("t", "Claim here.", &[0.85]);
            calibrator.record_bundle(bundle, &[true; 1]);
        }
        for _ in 0..2 {
            let bundle = calibrator.decompose_claims("t", "Claim here.", &[0.85]);
            calibrator.record_bundle(bundle, &[false; 1]);
        }
        // Now calibrate a claim in that bin — should be ~0.833
        let calibrated = calibrator.calibrate_claim(0.85);
        assert!((calibrated - 10.0 / 12.0).abs() < 0.01);
    }

    #[test]
    fn test_dinco_calibrate() {
        let calibrator = ClaimCalibrator::new();
        // target 0.9, distractors [0.8, 0.9] → mean = 0.85 → 0.9 / (0.9 + 0.85) ≈ 0.514
        let result = calibrator.dinco_calibrate(0.9, &[0.8, 0.9]);
        let expected = 0.9 / (0.9 + 0.85);
        assert!((result - expected).abs() < 0.001);
    }

    #[test]
    fn test_dinco_calibrate_empty_distractors() {
        let calibrator = ClaimCalibrator::new();
        let result = calibrator.dinco_calibrate(0.7, &[]);
        assert!((result - 0.7).abs() < 0.001);
    }

    #[test]
    fn test_add_distractors_negation() {
        let calibrator = ClaimCalibrator::new();
        let distractors = calibrator.add_distractors("the sky is blue", 3);
        assert!(distractors.len() >= 1);
        assert!(distractors[0].starts_with("Not "));
    }

    #[test]
    fn test_add_distractors_respects_count() {
        let calibrator = ClaimCalibrator::new();
        let distractors = calibrator.add_distractors("hello", 5);
        assert_eq!(distractors.len(), 5);
    }

    #[test]
    fn test_record_bundle_and_ece() {
        let mut calibrator = ClaimCalibrator::new();
        // All claims at 0.9 confidence and all correct
        for _ in 0..20 {
            let bundle = calibrator.decompose_claims("t", "Perfect claim.", &[0.9]);
            calibrator.record_bundle(bundle, &[true]);
        }
        assert_eq!(calibrator.total_claims, 20);
        assert_eq!(calibrator.total_correct, 20);
        assert!(calibrator.ece() < 0.15);
    }

    #[test]
    fn test_confidence_spread() {
        let calibrator = ClaimCalibrator::new();
        let bundle =
            calibrator.decompose_claims("t", "Low confidence. High confidence.", &[0.2, 0.9]);
        let spread = calibrator.confidence_spread(&bundle);
        assert!((spread - 0.7).abs() < 0.001);
    }

    #[test]
    fn test_calibrate_bundle() {
        let mut calibrator = ClaimCalibrator::new();
        // Calibrate with some data first
        for _ in 0..10 {
            let b = calibrator.decompose_claims("t", "Train.", &[0.3]);
            calibrator.record_bundle(b, &[true]);
        }
        let bundle =
            calibrator.decompose_claims("test2", "First claim. Second claim.", &[0.3, 0.9]);
        let calibrated = calibrator.calibrate_bundle(&bundle);
        assert_eq!(calibrated.claims.len(), 2);
        assert!(calibrated.calibration_error >= 0.0);
    }

    #[test]
    fn test_report() {
        let mut calibrator = ClaimCalibrator::new();
        let bundle = calibrator.decompose_claims("r", "A claim.", &[0.8]);
        calibrator.record_bundle(bundle, &[true]);
        let report = calibrator.report();
        assert_eq!(report.total_claims_calibrated, 1);
        assert!((report.overall_accuracy - 1.0).abs() < 0.001);
    }

    #[serial]
    #[test]
    fn test_global_calibrator() {
        let cal = global_claim_calibrator();
        let guard = cal.lock().unwrap_or_else(|e| e.into_inner());
        assert!(guard.total_claims == 0);
    }

    #[serial]
    #[test]
    fn test_calibrate_bundle_confidence_convenience() {
        let confidence = calibrate_bundle_confidence("This is true. So is this.", &[0.9, 0.8]);
        assert!(confidence >= 0.0 && confidence <= 1.0);
    }

    #[test]
    fn test_reliability_diagram() {
        let calibrator = ClaimCalibrator::new();
        let diagram = calibrator.reliability_diagram();
        assert_eq!(diagram.len(), 10);
        for (center, accuracy, gap) in &diagram {
            assert!(*center >= 0.0 && *center <= 1.0);
            assert!(*accuracy >= 0.0 && *accuracy <= 1.0);
            assert!(*gap >= 0.0 && *gap <= 1.0);
        }
    }

    #[test]
    fn test_min_confidence_in_response_empty() {
        let calibrator = ClaimCalibrator::new();
        assert!((calibrator.min_confidence_in_response() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_claim_bundle_timestamp() {
        let calibrator = ClaimCalibrator::new();
        let bundle = calibrator.decompose_claims("t", "Timestamp test.", &[0.5]);
        let elapsed = bundle.timestamp.elapsed();
        assert!(elapsed.as_secs() < 1);
    }

    #[test]
    fn test_split_empty_text() {
        let sentences = split_into_sentences("");
        assert!(sentences.is_empty());
    }

    #[test]
    fn test_verification_method_debug_clone_copy() {
        let v = VerificationMethod::Unverified;
        let _v2 = v;
        let _v3 = v.clone();
        let _debug = format!("{:?}", v);
    }
}
